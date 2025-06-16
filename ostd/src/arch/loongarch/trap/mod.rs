mod trap;
pub use trap::{GeneralRegs, TrapFrame, UserContext};

use crate::cpu::context::CpuExceptionInfo;

pub unsafe fn init() {
    // self::trap::init();
    // self::plic::init();
}

/// Injects a custom handler for page faults that occur in the kernel and
/// are caused by user-space address.
pub fn inject_user_page_fault_handler(
    handler: fn(info: &CpuExceptionInfo) -> core::result::Result<(), ()>,
) {
    // USER_PAGE_FAULT_HANDLER.call_once(|| handler);
}

/// Returns true if this function is called within the context of an IRQ handler
/// and the IRQ occurs while the CPU is executing in the kernel mode.
/// Otherwise, it returns false.
pub fn is_kernel_interrupted() -> bool {
    // IS_KERNEL_INTERRUPTED.load()
    false
}
