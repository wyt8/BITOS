// SPDX-License-Identifier: MPL-2.0

//! Handles trap.

mod trap;

use spin::Once;
pub use trap::{GeneralRegs, TrapFrame, UserContext};

use super::cpu::context::CpuExceptionInfo;
use crate::{cpu_local_cell, mm::MAX_USERSPACE_VADDR};

cpu_local_cell! {
    static IS_KERNEL_INTERRUPTED: bool = false;
}

/// Initialize interrupt handling on RISC-V.
pub unsafe fn init() {
    self::trap::init();
}

/// Returns true if this function is called within the context of an IRQ handler
/// and the IRQ occurs while the CPU is executing in the kernel mode.
/// Otherwise, it returns false.
pub fn is_kernel_interrupted() -> bool {
    IS_KERNEL_INTERRUPTED.load()
}

/// Handle traps (only from kernel).
#[no_mangle]
extern "C" fn trap_handler(f: &mut TrapFrame) {
    use riscv::register::scause::{Exception, Trap};

    match riscv::register::scause::read().cause() {
        Trap::Interrupt(_) => {
            IS_KERNEL_INTERRUPTED.store(true);
            todo!();
            IS_KERNEL_INTERRUPTED.store(false);
        }
        Trap::Exception(e) => {
            let stval = riscv::register::stval::read();
            match e {
                // Handle page fault
                Exception::StorePageFault
                | Exception::LoadPageFault
                | Exception::InstructionPageFault => {
                    // Check if the page fault is caused by user-space address
                    if let Some(handler) = USER_PAGE_FAULT_HANDLER.get() {
                        let page_fault_addr = stval;
                        if (0..MAX_USERSPACE_VADDR).contains(&(page_fault_addr as usize)) {
                            handler(&CpuExceptionInfo { code: e, page_fault_addr: page_fault_addr, error_code: 0, instruction: 0 })
                                .unwrap_or_else(|_| {
                                    panic!(
                                        "User page fault handler failed: addr: {page_fault_addr:#x}, err: {e:?}"
                                    );
                                });
                            return;
                        }
                    }
                }
                _ => {}
            }
            panic!(
                "Cannot handle kernel cpu exception: {e:?}. stval: {stval:#x}, trapframe: {f:#x?}.",
            );
        }
    }
}

#[expect(clippy::type_complexity)]
static USER_PAGE_FAULT_HANDLER: Once<fn(&CpuExceptionInfo) -> core::result::Result<(), ()>> =
    Once::new();

/// Injects a custom handler for page faults that occur in the kernel and
/// are caused by user-space address.
pub fn inject_user_page_fault_handler(
    handler: fn(info: &CpuExceptionInfo) -> core::result::Result<(), ()>,
) {
    USER_PAGE_FAULT_HANDLER.call_once(|| handler);
}
