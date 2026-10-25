.syntax unified
.thumb
.text
.global entry
.thumb_func
entry:
    cmp.w r0, #0xa0000000
    bhs custom
    /* Replay exactly the eight replaced bytes, then original c564. */
    push.w {r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,lr}
    mov r8,r1
    mov r6,r3
    ldr.w pc, original
.balign 4
original: .word 0x8010c565
custom:
    b.w custom_body
.org 64
custom_body:
    /* Builder appends the relocation-free debug_read .text here. */
