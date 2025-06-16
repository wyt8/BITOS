pub mod boot;
pub(crate) mod cpu;
pub mod device;
pub mod iommu;
pub(crate) mod irq;
pub(crate) mod mm;
pub(crate) mod pci;
pub mod qemu;
pub mod serial;
pub mod task;
pub mod timer;
pub mod trap;

/// Return the frequency of TSC. The unit is Hz.
pub fn tsc_freq() -> u64 {
    0
}

/// Reads the current value of the processor’s time-stamp counter (TSC).
pub fn read_tsc() -> u64 {
    0
}

pub(crate) unsafe fn late_init_on_bsp() {
    // // SAFETY: This function is called in the boot context of the BSP.
    // unsafe { trap::init() };
    // irq::init();

    // // SAFETY: We're on the BSP and we're ready to boot all APs.
    // unsafe { crate::boot::smp::boot_all_aps() };

    // timer::init();
    // let _ = pci::init();
}

pub(crate) fn interrupts_ack(irq_number: usize) {
    unimplemented!()
}

pub(crate) unsafe fn init_on_ap() {
    unimplemented!()
}

pub(crate) fn enable_cpu_features() {}

/// Reads a hardware generated 64-bit random value.
///
/// Returns None if no random value was generated.
pub fn read_random() -> Option<u64> {
    // FIXME: Implement a hardware random number generator on RISC-V platforms.
    None
}
