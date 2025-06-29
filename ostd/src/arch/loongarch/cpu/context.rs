use core::{arch::asm, fmt::Debug};

use loongArch64::register::estat::{self, Exception, Interrupt, Trap};

pub use crate::arch::trap::GeneralRegs as RawGeneralRegs;
use crate::{
    arch::{mm::tlb_flush_addr, trap::{TrapFrame, UserContext as RawUserContext}},
    user::{ReturnReason, UserContextApi, UserContextApiInternal},
};

/// CPU exception information.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CpuExceptionInfo {
    /// The type of the exception.
    pub code: Exception,
    /// The error code associated with the exception.
    pub page_fault_addr: usize,
    pub error_code: usize, // TODO
    pub instruction: usize,
}

impl Default for CpuExceptionInfo {
    fn default() -> Self {
        CpuExceptionInfo {
            code: Exception::Breakpoint,
            page_fault_addr: 0,
            error_code: 0,
            instruction: 0,
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
    trap: usize,
    fpu_state: FpuState,
    cpu_exception_info: CpuExceptionInfo,
}

impl Default for UserContext {
    fn default() -> Self {
        UserContext {
            user_context: RawUserContext::default(),
            trap: 0,
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

    pub fn init_fpu_state(&self) {
        // unsafe {
        // // 获取原始指针
        // let sstatus_ptr = core::ptr::addr_of!(self.user_context.sstatus) as *mut usize;

        // // 读取当前值
        // let current = sstatus_ptr.read();

        // // 设置 FS 字段为 Dirty
        // let modified = (current & !(0b11 << 13)) | (0b11 << 13);

        // // 写回
        // sstatus_ptr.write(modified);
        // }
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
            let pgd = loongArch64::register::pgd::read().raw();
            let tlbehi = loongArch64::register::tlbehi::read().raw();
            let tlbelo0 = loongArch64::register::tlbelo0::read().raw();
            let tlbelo1 = loongArch64::register::tlbelo1::read().raw();

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
                        tlb_flush_addr(badv);
                        // Handle page fault
                        self.cpu_exception_info = CpuExceptionInfo {
                            code: exception,
                            page_fault_addr: badv,
                            error_code: 0, // TODO: Set error code if needed
                            instruction: badi,
                        };
                        break ReturnReason::UserException;
                    }
                    Exception::FetchInstructionAddressError
                    | Exception::MemoryAccessAddressError
                    | Exception::AddressNotAligned
                    | Exception::BoundsCheckFault
                    | Exception::Breakpoint
                    | Exception::InstructionNotExist
                    | Exception::InstructionPrivilegeIllegal
                    | Exception::FloatingPointUnavailable => {
                        // Handle other exceptions
                        self.cpu_exception_info = CpuExceptionInfo {
                            code: exception,
                            page_fault_addr: 0,
                            error_code: 0, // TODO: Set error code if needed
                            instruction: badi,
                        };
                        log::warn!(
                            "Exception {exception:?} occurred, badv: {badv:#x?}, badi: {badi:#x?}, era: {era:#x?}"
                        );
                        break ReturnReason::UserException;
                    }
                    Exception::TLBRFill => panic!("Shuld not happen"),
                },
                Trap::Interrupt(interrupt) => match interrupt {
                    Interrupt::SWI0 => todo!(),
                    Interrupt::SWI1 => todo!(),
                    Interrupt::HWI0 => {
                        log::info!("Handling HWI0 interrupt");
                    },
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
        unimplemented!()
        // TrapFrame {
        //     general: self.user_context.general,
        //     // sstatus: self.user_context.sstatus,
        //     // sepc: self.user_context.sepc,
        //     prmd: 0,
        //     era: 0,
        //     badv: 0,
        //     crmd: 0,
        //     ktp: 0,
        //     kr21: 0,
        //     fs: [0, 0],
        // }
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
///
/// This could be used for saving both legacy and modern state format.
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
