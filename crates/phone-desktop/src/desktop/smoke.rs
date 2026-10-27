//! Presentation check using the same synthetic fixtures as the CPU audit.
use super::*;
impl App {
    pub(super) fn smoke_frame(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.as_ref().is_none_or(|g| g.presented == 0) {
            return;
        }
        let Some(name) = crate::frames::name(self.smoke_stage) else {
            eprintln!(
                "SMOKE PASS: {} presented frames",
                self.gpu.as_ref().unwrap().presented
            );
            self.smoke = None;
            event_loop.exit();
            return;
        };
        let dir = self.smoke.clone().unwrap();
        if let Err(e) = self.runtime.canvas.save(&dir.join(format!("{name}.png"))) {
            self.fail(event_loop, e.to_string());
            return;
        }
        let now = self.now();
        crate::frames::advance(&mut self.runtime, self.smoke_stage, now);
        if self.smoke_stage == 124 {
            self.flush();
        }
        self.smoke_stage += 1;
    }
}

pub(super) fn theme(runtime: &mut crate::Runtime) {
    runtime.shell.settings.accent = 4;
}
