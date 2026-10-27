//! ARM-state instruction interpreter for the extracted T9 module only.
//! Guest pointers never become host pointers. No syscall, device, JIT or native code path.
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
pub(super) const ROM: u32 = 0x880cc000;
pub(super) const RAM: u32 = 0x20000000;
pub(super) const DB: u32 = 0x10040000;
const STOP: u32 = 0x10000000;
pub(super) const REQUEST: u32 = 0x10000010;
pub(super) const LOAD: u32 = 0x10000020;
pub(super) type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug)]
pub(super) struct Arm {
    pub tables: Arc<[u8]>,
    pub ram: Vec<u8>,
    pub code: Arc<[u8]>,
    pub db: Arc<[u8]>,
    pub chinese: bool,
    r: [u32; 16],
    pc: u32,
    next: u32,
    n: bool,
    z: bool,
    c: bool,
    v: bool,
    pub steps: usize,
    budget: Arc<AtomicUsize>,
}
impl Arm {
    pub fn new(code: Arc<[u8]>, db: Arc<[u8]>, tables: Arc<[u8]>, chinese: bool) -> Self {
        Self {
            tables,
            ram: vec![0; 0x20000],
            code,
            db,
            chinese,
            r: [0; 16],
            pc: 0,
            next: 0,
            n: false,
            z: false,
            c: false,
            v: false,
            steps: 0,
            budget: Arc::new(AtomicUsize::new(3_000_000)),
        }
    }
    pub fn reset_budget(&mut self) {
        self.budget = Arc::new(AtomicUsize::new(3_000_000));
    }
    pub fn read(&self, a: u32, n: usize) -> Result<u32> {
        if !matches!(n, 1 | 2 | 4) {
            return Err("T9 invalid read width".into());
        }
        for (base, bytes) in [
            (0x884ac300, self.tables.as_ref()),
            (ROM, self.code.as_ref()),
            (DB, self.db.as_ref()),
            (RAM, self.ram.as_slice()),
        ] {
            if let Some(offset) = a.checked_sub(base).map(|x| x as usize)
                && let Some(end) = offset.checked_add(n)
                && let Some(slice) = bytes.get(offset..end)
            {
                return Ok(slice
                    .iter()
                    .enumerate()
                    .fold(0, |v, (i, b)| v | u32::from(*b) << (8 * i)));
            }
        }
        Err(format!("T9 read outside memory: {a:08x}+{n}"))
    }
    pub fn write(&mut self, a: u32, v: u32, n: usize) -> Result<()> {
        let offset = a.checked_sub(RAM).ok_or("T9 write outside RAM")? as usize;
        let end = offset.checked_add(n).ok_or("T9 write overflow")?;
        let out = self
            .ram
            .get_mut(offset..end)
            .ok_or("T9 write outside RAM")?;
        if n > 4 {
            return Err("T9 transfer too wide".into());
        }
        out.copy_from_slice(&v.to_le_bytes()[..n]);
        Ok(())
    }
    fn reg(&self, i: usize) -> u32 {
        if i == 15 {
            self.pc.wrapping_add(8)
        } else {
            self.r[i]
        }
    }
    fn put(&mut self, i: usize, v: u32) -> Result<()> {
        if i == 15 {
            if v & 3 != 0 {
                return Err(format!("T9 unaligned/Thumb branch {v:08x}"));
            }
            self.next = v;
        } else {
            self.r[i] = v;
        }
        Ok(())
    }
    fn cond(&self, c: u32) -> bool {
        match c {
            0 => self.z,
            1 => !self.z,
            2 => self.c,
            3 => !self.c,
            4 => self.n,
            5 => !self.n,
            6 => self.v,
            7 => !self.v,
            8 => self.c && !self.z,
            9 => !self.c || self.z,
            10 => self.n == self.v,
            11 => self.n != self.v,
            12 => !self.z && self.n == self.v,
            13 => self.z || self.n != self.v,
            14 => true,
            _ => false,
        }
    }
    fn nz(&mut self, v: u32) {
        self.n = v >> 31 != 0;
        self.z = v == 0;
    }
    fn shift(&self, x: u32, k: u32, n: u32, register: bool) -> (u32, bool) {
        if register && n == 0 {
            return (x, self.c);
        }
        match k {
            0 => match n {
                0 => (x, self.c),
                1..=31 => (x << n, x >> (32 - n) & 1 != 0),
                32 => (0, x & 1 != 0),
                _ => (0, false),
            },
            1 => {
                let n = if n == 0 { 32 } else { n };
                if n < 32 {
                    (x >> n, x >> (n - 1) & 1 != 0)
                } else {
                    (0, n == 32 && x >> 31 != 0)
                }
            }
            2 => {
                let n = if n == 0 { 32 } else { n.min(32) };
                (((x as i32) >> (n.min(31))) as u32, x >> (n - 1) & 1 != 0)
            }
            _ => {
                if n == 0 {
                    ((u32::from(self.c) << 31) | (x >> 1), x & 1 != 0)
                } else {
                    let v = x.rotate_right(n);
                    (v, v >> 31 != 0)
                }
            }
        }
    }
    fn operand(&self, w: u32) -> (u32, bool) {
        if w & (1 << 25) != 0 {
            let rot = (w >> 8 & 15) * 2;
            let x = (w & 255).rotate_right(rot);
            (x, if rot == 0 { self.c } else { x >> 31 != 0 })
        } else {
            let reg = w & 16 != 0;
            let n = if reg {
                self.reg((w >> 8 & 15) as usize) & 255
            } else {
                w >> 7 & 31
            };
            self.shift(self.reg((w & 15) as usize), w >> 5 & 3, n, reg)
        }
    }
    fn callback(&mut self) -> Result<bool> {
        match self.pc {
            REQUEST => {
                let request = self.read(self.r[1], 4)?;
                if !matches!(request, 1 | 8) {
                    return Err(format!("Unsupported T9 host request {request}"));
                }
                self.r[0] = 0;
            }
            LOAD => {
                if self.chinese {
                    let out = self.r[1];
                    for i in 0..192 {
                        self.write(out + i, self.read(DB + i, 1)?, 1)?;
                    }
                    for i in 0..66 {
                        let offset = self.read(DB + 192 + i * 4, 4)?;
                        if offset != 0 && offset as usize >= self.db.len() {
                            return Err("T9 bad LDB offset".into());
                        }
                        self.write(
                            out + 192 + i * 4,
                            if offset == 0 { 0 } else { DB + offset },
                            4,
                        )?;
                    }
                    self.r[0] = 0;
                } else {
                    let offset = self.r[1];
                    if offset as usize >= self.db.len() {
                        return Err("T9 bad LDB read".into());
                    }
                    let block = offset / 0x3c00 * 0x3c00;
                    self.write(self.r[2], DB + block, 4)?;
                    self.write(self.r[3], block, 4)?;
                    self.r[0] = (self.db.len() as u32 - block).min(0x3c00);
                }
            }
            0x881009cc => {
                let d = self.r[1];
                if d == 0 {
                    return Err("T9 division by zero".into());
                }
                let n = self.r[0];
                self.r[0] = n / d;
                self.r[1] = n % d;
            }
            // Verified kernel SDIV + MLS veneer (0x818f5a48), including ARM overflow.
            0x88100844 => {
                let (n, d) = (self.r[0] as i32, self.r[1] as i32);
                if d == 0 {
                    return Err("T9 division by zero".into());
                }
                self.r[0] = n.wrapping_div(d) as u32;
                self.r[1] = n.wrapping_rem(d) as u32;
            }
            _ => return Ok(false),
        }
        self.pc = self.r[14];
        Ok(true)
    }
    pub fn call(&mut self, pc: u32, args: &[u32]) -> Result<u32> {
        self.r = [0; 16];
        self.r[13] = RAM + self.ram.len() as u32 - 0x1000;
        self.r[14] = STOP;
        for (i, v) in args.iter().copied().enumerate() {
            if i < 4 {
                self.r[i] = v
            } else {
                self.write(self.r[13] + (i as u32 - 4) * 4, v, 4)?;
            }
        }
        self.pc = pc;
        self.steps = 0;
        let limit = self.budget.load(Ordering::Relaxed);
        while self.pc != STOP {
            self.steps += 1;
            if self.steps > limit {
                return Err("T9 instruction budget exceeded".into());
            }
            if self.callback()? {
                continue;
            }
            if self.pc < ROM || self.pc as u64 + 4 > ROM as u64 + self.code.len() as u64 {
                return Err(format!("T9 forbidden branch {:08x}", self.pc));
            }
            let w = self.read(self.pc, 4)?;
            self.next = self.pc.wrapping_add(4);
            if w >> 28 == 15 {
                return Err("T9 unsupported unconditional instruction".into());
            }
            if self.cond(w >> 28) {
                self.step(w)
                    .map_err(|e| format!("{e} at {:08x}", self.pc))?;
            }
            self.pc = self.next;
        }
        self.budget.fetch_sub(self.steps, Ordering::Relaxed);
        Ok(self.r[0])
    }
    fn step(&mut self, w: u32) -> Result<()> {
        let op = w & 0x0fffffff;
        let rn = (w >> 16 & 15) as usize;
        let rd = (w >> 12 & 15) as usize;
        if matches!(op & 0x0ffffff0, 0x012fff10 | 0x012fff30) {
            let target = self.reg((w & 15) as usize);
            if op & 0x20 != 0 {
                self.r[14] = self.pc + 4;
            }
            return self.put(15, target);
        }
        if op >> 25 == 5 {
            if w & (1 << 24) != 0 {
                self.r[14] = self.pc + 4;
            }
            let offset = ((w << 8) as i32 >> 6) as u32;
            return self.put(15, self.pc.wrapping_add(8).wrapping_add(offset));
        }
        if op & 0x0f8000f0 == 0x00800090 {
            let a = self.reg((w & 15) as usize);
            let b = self.reg((w >> 8 & 15) as usize);
            let mut v = if w & (1 << 22) != 0 {
                ((a as i32 as i64) * (b as i32 as i64)) as u64
            } else {
                a as u64 * b as u64
            };
            if w & (1 << 21) != 0 {
                v = v.wrapping_add(self.reg(rd) as u64 | ((self.reg(rn) as u64) << 32));
            }
            self.put(rd, v as u32)?;
            self.put(rn, (v >> 32) as u32)?;
            if w & (1 << 20) != 0 {
                self.n = v >> 63 != 0;
                self.z = v == 0;
            }
            return Ok(());
        }
        if op & 0x0fc000f0 == 0x00000090 {
            let mut v = self
                .reg((w & 15) as usize)
                .wrapping_mul(self.reg((w >> 8 & 15) as usize));
            if w & (1 << 21) != 0 {
                v = v.wrapping_add(self.reg(rd));
            }
            self.put(rn, v)?;
            if w & (1 << 20) != 0 {
                self.nz(v)
            }
            return Ok(());
        }
        if op & 0x0ff00090 == 0x01600080 {
            let a = (self.reg((w & 15) as usize) >> if w & 32 != 0 { 16 } else { 0 }) as i16 as i32;
            let b = (self.reg((w >> 8 & 15) as usize) >> if w & 64 != 0 { 16 } else { 0 }) as i16
                as i32;
            return self.put(rn, a.wrapping_mul(b) as u32);
        }
        if op & 0x0e000090 == 0x00000090 {
            let kind = w >> 5 & 3;
            let load = w & (1 << 20) != 0;
            let off = if w & (1 << 22) != 0 {
                (w >> 8 & 15) << 4 | (w & 15)
            } else {
                self.reg((w & 15) as usize)
            };
            let base = self.reg(rn);
            let end = if w & (1 << 23) != 0 {
                base.wrapping_add(off)
            } else {
                base.wrapping_sub(off)
            };
            let pre = w & (1 << 24) != 0;
            let a = if pre { end } else { base };
            if !load && matches!(kind, 2 | 3) {
                if rd >= 14 {
                    return Err("T9 invalid double transfer".into());
                }
                if kind == 2 {
                    self.put(rd, self.read(a, 4)?)?;
                    self.put(rd + 1, self.read(a.wrapping_add(4), 4)?)?;
                } else {
                    self.write(a, self.reg(rd), 4)?;
                    self.write(a.wrapping_add(4), self.reg(rd + 1), 4)?;
                }
            } else {
                let n = match (kind, load) {
                    (1, _) | (3, true) => 2,
                    (2, true) => 1,
                    _ => return Err("T9 unsupported transfer".into()),
                };
                if load {
                    let v = self.read(a, n)?;
                    let v = match kind {
                        2 => v as i8 as i32 as u32,
                        3 => v as i16 as i32 as u32,
                        _ => v,
                    };
                    self.put(rd, v)?;
                } else {
                    self.write(a, self.reg(rd), n)?;
                }
            }
            if !pre || w & (1 << 21) != 0 {
                self.put(rn, end)?;
            }
            return Ok(());
        }
        if op >> 25 == 4 {
            if w & (1 << 22) != 0 {
                return Err("T9 privileged transfer".into());
            }
            let count = (w & 65535).count_ones();
            let base = self.reg(rn);
            let up = w & (1 << 23) != 0;
            let pre = w & (1 << 24) != 0;
            let mut addr = if up {
                base.wrapping_add(if pre { 4 } else { 0 })
            } else {
                base.wrapping_sub(4 * count)
                    .wrapping_add(if pre { 0 } else { 4 })
            };
            for i in 0..16 {
                if w & (1 << i) != 0 {
                    if w & (1 << 20) != 0 {
                        self.put(i, self.read(addr, 4)?)?;
                    } else {
                        self.write(addr, self.reg(i), 4)?;
                    }
                    addr = addr.wrapping_add(4);
                }
            }
            if w & (1 << 21) != 0 {
                self.put(
                    rn,
                    if up {
                        base.wrapping_add(4 * count)
                    } else {
                        base.wrapping_sub(4 * count)
                    },
                )?;
            }
            return Ok(());
        }
        if op >> 26 == 1 {
            let off = if w & (1 << 25) != 0 {
                if w & 16 != 0 {
                    return Err("T9 invalid shifted transfer".into());
                }
                self.shift(self.reg((w & 15) as usize), w >> 5 & 3, w >> 7 & 31, false)
                    .0
            } else {
                w & 4095
            };
            let base = self.reg(rn);
            let end = if w & (1 << 23) != 0 {
                base.wrapping_add(off)
            } else {
                base.wrapping_sub(off)
            };
            let pre = w & (1 << 24) != 0;
            let a = if pre { end } else { base };
            let n = if w & (1 << 22) != 0 { 1 } else { 4 };
            if w & (1 << 20) != 0 {
                self.put(rd, self.read(a, n)?)?;
            } else {
                self.write(a, self.reg(rd), n)?;
            }
            if !pre || w & (1 << 21) != 0 {
                self.put(rn, end)?;
            }
            return Ok(());
        }
        if op >> 26 == 0 {
            let opcode = w >> 21 & 15;
            let flags = w & (1 << 20) != 0;
            let a = self.reg(rn);
            let (b, carry) = self.operand(w);
            if (8..=11).contains(&opcode) && !flags {
                return Err("T9 system opcode".into());
            }
            let mut arith = None;
            let value = match opcode {
                0 | 8 => a & b,
                1 | 9 => a ^ b,
                12 => a | b,
                13 => b,
                14 => a & !b,
                15 => !b,
                _ => {
                    let (x, y, cin) = match opcode {
                        2 | 10 => (a, !b, 1),
                        3 => (b, !a, 1),
                        4 | 11 => (a, b, 0),
                        5 => (a, b, u32::from(self.c)),
                        6 => (a, !b, u32::from(self.c)),
                        7 => (b, !a, u32::from(self.c)),
                        _ => unreachable!(),
                    };
                    let sum = x as u64 + y as u64 + cin as u64;
                    let v = sum as u32;
                    arith = Some((
                        sum > u32::MAX as u64,
                        (!(x ^ y) & (x ^ v) & 0x80000000) != 0,
                    ));
                    v
                }
            };
            if flags {
                self.nz(value);
                if let Some((c, v)) = arith {
                    self.c = c;
                    self.v = v;
                } else {
                    self.c = carry
                }
            }
            if !(8..=11).contains(&opcode) {
                self.put(rd, value)?;
            }
            return Ok(());
        }
        Err(format!("T9 unsupported opcode {w:08x}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn machine(words: &[u32]) -> Arm {
        Arm::new(
            words
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect::<Vec<_>>()
                .into(),
            Arc::from([]),
            Arc::from([]),
            false,
        )
    }
    #[test]
    fn arithmetic_flags_shifts_division_and_guest_bounds() {
        let mut a = machine(&[0xe0900001, 0xe12fff1e]); // ADDS r0,r0,r1; BX lr
        assert_eq!(a.call(ROM, &[u32::MAX, 1]).unwrap(), 0);
        assert!(a.c && a.z && !a.v && !a.n);
        assert_eq!(a.call(ROM, &[0x7fffffff, 1]).unwrap(), 0x80000000);
        assert!(a.v && a.n && !a.c && !a.z);
        a.c = true;
        assert_eq!(a.shift(0x80000001, 0, 32, true), (0, true));
        assert_eq!(a.shift(0x80000001, 1, 0, false), (0, true));
        assert_eq!(a.shift(0x80000001, 2, 40, true), (u32::MAX, true));
        assert_eq!(a.shift(2, 3, 0, false), (0x80000001, false));
        assert_eq!(a.shift(0x80000001, 3, 32, true), (0x80000001, true));
        assert_eq!(
            a.call(0x88100844, &[(-7i32) as u32, 3]).unwrap(),
            (-2i32) as u32
        );
        assert_eq!(a.r[1], u32::MAX);
        assert_eq!(
            a.call(0x88100844, &[i32::MIN as u32, u32::MAX]).unwrap(),
            i32::MIN as u32
        );
        assert!(a.call(0x881009cc, &[4, 0]).is_err());
        assert!(a.read(RAM - 1, 4).is_err());
        assert!(a.read(RAM, 8).is_err());
        assert!(a.write(ROM, 0, 4).is_err());
        assert!(a.call(RAM, &[]).is_err());
        let mut a = machine(&[0xeafffffe]);
        a.budget.store(16, Ordering::Relaxed);
        assert!(a.call(ROM, &[]).unwrap_err().contains("budget"));
    }
}
