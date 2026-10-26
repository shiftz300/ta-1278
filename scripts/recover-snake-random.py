#!/usr/bin/env python3
"""Verify the Snake RNG call chain. Offline; no firmware execution."""
import hashlib,json,struct
from pathlib import Path
from firmware_disassembly import disassemble_arm
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800'
u=(SOURCE/'user.bin').read_bytes(); k=(SOURCE/'kernel.bin').read_bytes()
assert hashlib.sha256(u).hexdigest()=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
assert hashlib.sha256(k).hexdigest()=='b871d5a004c9251f3bb7cc58ea637d486f05d680efd3c39be83c38051f11e303'
assert u[0x100a4c:0x100a50]==bytes.fromhex('04f01fe5')
assert struct.unpack_from('<I',u,0x100a50)[0]==0x8165ab59
base=0x80e65e00
assert k[0x8165ab58-base:0x8165ab88-base].hex()=='0b49d1f800c04a685cf8043b1068184442f8040b8c4528bfa1f1dc0c8a4228bfa1f1dc02c1f800c04a6020f000407047'
assert struct.unpack_from('<I',k,0x8165ab88-base)[0]==0x826d4e04
out=ROOT/'docs/evidence'
listing=disassemble_arm(u,0x88000000,[(0x886902ce,0x886902fc)])+disassemble_arm(u,0x88000000,[(0x88100a4c,0x88100a54)],thumb=False)+disassemble_arm(k,base,[(0x8165ab58,0x8165ab88)],label="<kernel.bin ELF wrapper>")
(out/'snake-random.txt').write_text(listing)
result={'user_sha256':hashlib.sha256(u).hexdigest(),'kernel_sha256':hashlib.sha256(k).hexdigest(),'kernel_image_base':hex(base),'call_chain':['Snake two draws: 0x886902dc and 0x886902ea','ARM veneer 0x88100a4c -> Thumb 0x8165ab59','Kernel implementation 0x8165ab58..0x8165ab86'],'verified':'Two pointers read/write a 55-word ring; uint32 add, store full sum, increment/wrap pointers, return sum & 0x7fffffff. Food takes two draws, x%12 and y%14+2.','unknown':['Initial 55 words at RAM 0x826d4d28','Initial read/write pointer separation','Boot seeding and intervening calls by other apps'],'replica_policy':'Deterministic locally seeded table and pointer gap31. Ring update and two-draw coordinate mapping follow code; sequence parity is not claimed.'}
(out/'snake-random.json').write_text(json.dumps(result,indent=2)+'\n')
print('Verified Snake -> ARM veneer -> kernel 55-word generator; seed state remains unknown.')
