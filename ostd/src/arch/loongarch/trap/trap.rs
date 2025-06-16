use crate::Pod;

/// General registers
#[derive(Debug, Default, Clone, Copy, Pod)]
#[repr(C)]
#[expect(missing_docs)]
pub struct GeneralRegs {
    pub zero: usize,
    pub ra: usize,
    pub tp: usize,
    pub sp: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub t7: usize,
    pub t8: usize,
    pub u0: usize,
    pub fp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
}

/// Trap frame of kernel interrupt
///
/// # Trap handler
///
/// You need to define a handler function like this:
///
/// ```no_run
/// #[no_mangle]
/// pub extern "C" fn trap_handler(tf: &mut TrapFrame) {
///     println!("TRAP! tf: {:#x?}", tf);
/// }
/// ```
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    /// General registers
    pub general: GeneralRegs,
    /// Supervisor Status
    pub sstatus: usize,
    /// Supervisor Exception Program Counter
    pub sepc: usize,
}

/// Saved registers on a trap.
#[derive(Debug, Default, Clone, Copy, Pod)]
#[repr(C)]
pub struct UserContext {
    /// General registers
    pub general: GeneralRegs,
    /// Supervisor Status
    pub sstatus: usize,
    /// Supervisor Exception Program Counter
    pub sepc: usize,
}


impl UserContext {
    /// Get number of syscall.
    pub fn get_syscall_num(&self) -> usize {
        self.general.a7
    }

    /// Get return value of syscall.
    pub fn get_syscall_ret(&self) -> usize {
        self.general.a0
    }

    /// Set return value of syscall.
    pub fn set_syscall_ret(&mut self, ret: usize) {
        self.general.a0 = ret;
    }

    /// Get syscall args.
    pub fn get_syscall_args(&self) -> [usize; 6] {
        [
            self.general.a0,
            self.general.a1,
            self.general.a2,
            self.general.a3,
            self.general.a4,
            self.general.a5,
        ]
    }

    /// Set instruction pointer.
    pub fn set_ip(&mut self, ip: usize) {
        // self.general.rip = ip;
    }

    /// Set stack pointer.
    pub fn set_sp(&mut self, sp: usize) {
        // self.general.rsp = sp;
    }

    /// Get stack pointer.
    pub fn get_sp(&self) -> usize {
        self.general.sp
    }

    /// Set thread-local storage pointer.
    pub fn set_tls(&mut self, tls: usize) {
        // self.general.fsbase = tls;
    }
}