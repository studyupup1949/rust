use core::arch::asm;

use crate::*;

#[inline(always)]
pub fn yield_call() {
    unsafe {
        asm!(
        "syscall",
        in("rax") KernelCallType::Yield as Sword,
        out("rcx") _,
        out("r11") _,
        options(nostack),
        );
    }
}
