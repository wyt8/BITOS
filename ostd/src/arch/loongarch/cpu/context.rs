// SPDX-License-Identifier: MPL-2.0

//! CPU execution context control.

use core::{arch::asm, fmt::Debug};

use loongArch64::register::estat::{self, Exception, Interrupt, Trap};

pub use crate::arch::trap::GeneralRegs as RawGeneralRegs;
use crate::{
    arch::{
        mm::tlb_flush_addr,
        trap::{TrapFrame, UserContext as RawUserContext},
    },
    user::{ReturnReason, UserContextApi, UserContextApiInternal},
};

/// CPU exception information.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CpuExceptionInfo {
    /// The type of the exception.
    pub code: Exception,
    /// The page fault address.
    pub page_fault_addr: usize,
    /// The error code associated with the exception.
    pub error_code: usize, // TODO
}

impl Default for CpuExceptionInfo {
    fn default() -> Self {
        CpuExceptionInfo {
            code: Exception::AddressNotAligned,
            page_fault_addr: 0,
            error_code: 0,
        }
    }
}

impl CpuExceptionInfo {
    /// Get corresponding CPU exception
    pub fn cpu_exception(&self) -> CpuException {
        self.code
    }
}

/// Cpu context, including both general-purpose registers and FPU state.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct UserContext {
    user_context: RawUserContext,
    trap: Trap,
    fpu_state: FpuState,
    cpu_exception_info: CpuExceptionInfo,
}

impl Default for UserContext {
    fn default() -> Self {
        UserContext {
            user_context: RawUserContext::default(),
            trap: Trap::Unknown,
            fpu_state: FpuState,
            cpu_exception_info: CpuExceptionInfo::default(),
        }
    }
}

impl UserContext {
    /// Returns a reference to the general registers.
    pub fn general_regs(&self) -> &RawGeneralRegs {
        &self.user_context.general
    }

    /// Returns a mutable reference to the general registers
    pub fn general_regs_mut(&mut self) -> &mut RawGeneralRegs {
        &mut self.user_context.general
    }

    /// Returns the trap information.
    pub fn trap_information(&self) -> &CpuExceptionInfo {
        &self.cpu_exception_info
    }

    /// Returns a reference to the FPU state.
    pub fn fpu_state(&self) -> &FpuState {
        &self.fpu_state
    }

    /// Returns a mutable reference to the FPU state.
    pub fn fpu_state_mut(&mut self) -> &mut FpuState {
        &mut self.fpu_state
    }

    /// Sets thread-local storage pointer.
    pub fn set_tls_pointer(&mut self, tls: usize) {
        self.set_tp(tls)
    }

    /// Gets thread-local storage pointer.
    pub fn tls_pointer(&self) -> usize {
        self.tp()
    }

    /// Activates thread-local storage pointer on the current CPU.
    pub fn activate_tls_pointer(&self) {
        // No-op
    }

    /// Enable floating-point unit
    pub fn enable_fpu(&mut self) {
        self.user_context.euen = 0x1;
    }
}

impl UserContextApiInternal for UserContext {
    fn execute<F>(&mut self, mut has_kernel_event: F) -> ReturnReason
    where
        F: FnMut() -> bool,
    {
        let ret = loop {
            self.user_context.run();

            let cause = loongArch64::register::estat::read().cause();
            let badv = loongArch64::register::badv::read().raw();
            let badi = loongArch64::register::badi::read().raw();
            let era = loongArch64::register::era::read().raw();

            match cause {
                Trap::Exception(exception) => match exception {
                    Exception::Syscall => {
                        self.user_context.era += 4;
                        break ReturnReason::UserSyscall;
                    }
                    Exception::LoadPageFault
                    | Exception::StorePageFault
                    | Exception::FetchPageFault
                    | Exception::PageModifyFault
                    | Exception::PageNonReadableFault
                    | Exception::PageNonExecutableFault
                    | Exception::PagePrivilegeIllegal => {
                        // Handle page fault
                        // Disable the badv in TLB.
                        tlb_flush_addr(badv);
                        self.cpu_exception_info = CpuExceptionInfo {
                            code: exception,
                            page_fault_addr: badv,
                            error_code: 0, // TODO: Set error code if needed
                        };
                        break ReturnReason::UserException;
                    }
                    Exception::FetchInstructionAddressError
                    | Exception::MemoryAccessAddressError
                    | Exception::AddressNotAligned
                    | Exception::BoundsCheckFault
                    | Exception::Breakpoint
                    | Exception::InstructionNotExist
                    | Exception::InstructionPrivilegeIllegal => {
                        // Handle other exceptions
                        self.cpu_exception_info = CpuExceptionInfo {
                            code: exception,
                            page_fault_addr: 0,
                            error_code: 0, // TODO: Set error code if needed
                        };
                        log::warn!(
                            "Exception {exception:?} occurred, badv: {badv:#x?}, badi: {badi:#x?}, era: {era:#x?}"
                        );
                        break ReturnReason::UserException;
                    }
                    Exception::FloatingPointUnavailable => {
                        log::warn!(
                            "Floating point unit is not available, badv: {badv:#x?}, badi: {badi:#x?}, era: {era:#x?}"
                        );
                        // Enable FPU
                        self.enable_fpu();
                    }
                    Exception::TLBRFill => unreachable!(),
                },
                Trap::Interrupt(interrupt) => match interrupt {
                    Interrupt::SWI0 => todo!(),
                    Interrupt::SWI1 => todo!(),
                    Interrupt::HWI0 => todo!(),
                    Interrupt::HWI1 => todo!(),
                    Interrupt::HWI2 => todo!(),
                    Interrupt::HWI3 => todo!(),
                    Interrupt::HWI4 => todo!(),
                    Interrupt::HWI5 => todo!(),
                    Interrupt::HWI6 => todo!(),
                    Interrupt::HWI7 => todo!(),
                    Interrupt::PMI => todo!(),
                    Interrupt::Timer => todo!(),
                    Interrupt::IPI => todo!(),
                },
                Trap::MachineError(machine_error) => panic!(
                    "Machine error: {machine_error:?}, badv: {badv:#x?}, badi: {badi:#x?}, era: {era:#x?}"
                ),
                Trap::Unknown => panic!(
                    "Unknown trap, badv: {badv:#x?}, badi: {badi:#x?}, era: {era:#x?}"
                ),
            }

            if has_kernel_event() {
                break ReturnReason::KernelEvent;
            }
        };

        crate::arch::irq::enable_local();
        ret
    }

    fn as_trap_frame(&self) -> TrapFrame {
        TrapFrame {
            general: self.user_context.general,
            prmd: self.user_context.prmd,
            era: self.user_context.era,
            euen: self.user_context.euen,
        }
    }
}

impl UserContextApi for UserContext {
    fn trap_number(&self) -> usize {
        todo!()
    }

    fn trap_error_code(&self) -> usize {
        todo!()
    }

    fn instruction_pointer(&self) -> usize {
        self.user_context.get_ip()
    }

    fn set_instruction_pointer(&mut self, ip: usize) {
        self.user_context.set_ip(ip);
    }

    fn stack_pointer(&self) -> usize {
        self.user_context.get_sp()
    }

    fn set_stack_pointer(&mut self, sp: usize) {
        self.user_context.set_sp(sp);
    }
}

macro_rules! cpu_context_impl_getter_setter {
    ( $( [ $field: ident, $setter_name: ident] ),*) => {
        impl UserContext {
            $(
                #[doc = concat!("Gets the value of ", stringify!($field))]
                #[inline(always)]
                pub fn $field(&self) -> usize {
                    self.user_context.general.$field
                }

                #[doc = concat!("Sets the value of ", stringify!($field))]
                #[inline(always)]
                pub fn $setter_name(&mut self, $field: usize) {
                    self.user_context.general.$field = $field;
                }
            )*
        }
    };
}

cpu_context_impl_getter_setter!(
    [ra, set_ra],
    [tp, set_tp],
    [sp, set_sp],
    [a0, set_a0],
    [a1, set_a1],
    [a2, set_a2],
    [a3, set_a3],
    [a4, set_a4],
    [a5, set_a5],
    [a6, set_a6],
    [a7, set_a7],
    [t0, set_t0],
    [t1, set_t1],
    [t2, set_t2],
    [t3, set_t3],
    [t4, set_t4],
    [t5, set_t5],
    [t6, set_t6],
    [t7, set_t7],
    [t8, set_t8],
    [r21, set_r21],
    [fp, set_fp],
    [s0, set_s0],
    [s1, set_s1],
    [s2, set_s2],
    [s3, set_s3],
    [s4, set_s4],
    [s5, set_s5],
    [s6, set_s6],
    [s7, set_s7],
    [s8, set_s8]
);

/// CPU exception.
pub type CpuException = Exception;

/// The FPU state of user task.
// FIXME: Implement FPU state on LoongArch64 platforms.
#[derive(Clone, Copy, Debug)]
pub struct FpuState;

impl FpuState {
    /// Saves CPU's current FPU state into this instance.
    pub fn save(&self) {
        // todo!()
    }

    /// Restores CPU's FPU state from this instance.
    pub fn restore(&self) {
        // todo!()
    }
}

impl Default for FpuState {
    fn default() -> Self {
        FpuState
    }
}
