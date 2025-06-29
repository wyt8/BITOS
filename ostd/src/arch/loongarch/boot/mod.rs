// SPDX-License-Identifier: MPL-2.0

//! The LoongArch boot module defines the entrypoints of Asterinas.

pub mod smp;

use core::arch::global_asm;

use fdt::Fdt;

use crate::{
    arch::trap,
    boot::{
        memory_region::{MemoryRegion, MemoryRegionArray, MemoryRegionType},
        BootloaderAcpiArg, BootloaderFramebufferArg,
    },
    early_println,
    mm::paddr_to_vaddr,
};

global_asm!(include_str!("boot.S"));

fn parse_bootloader_name() -> &'static str {
    "Unknown"
}

fn parse_kernel_commandline() -> &'static str {
    "SHELL=/bin/sh LOGNAME=root HOME=/ USER=root PATH=/bin init=/bin/busybox ostd.log_level=trace -- ls -alh /"
}

fn parse_initramfs() -> Option<&'static [u8]> {
    let Some((start, end)) = parse_initramfs_range() else {
        return None;
    };

    let base_va = paddr_to_vaddr(start);
    let length = end - start;
    Some(unsafe { core::slice::from_raw_parts(base_va as *const u8, length) })
}

fn parse_acpi_arg() -> BootloaderAcpiArg {
    BootloaderAcpiArg::NotProvided
}

fn parse_framebuffer_info() -> Option<BootloaderFramebufferArg> {
    None
}

fn parse_memory_regions() -> MemoryRegionArray {
    let mut regions = MemoryRegionArray::new();

    // TODO: Parse memory regions other than specified value.
    // regions
    //     .push(MemoryRegion::new(
    //         0x0,
    //         0x1000_0000, // 256 MiB
    //         MemoryRegionType::Usable,
    //     ))
    //     .unwrap();

    regions
        .push(MemoryRegion::new(
            0x8000_0000,
            0x7000_0000, // 2 GiB - 256 MiB
            MemoryRegionType::Usable,
        ))
        .unwrap();

    // Add the kernel region.
    regions.push(MemoryRegion::kernel()).unwrap();

    // Add the initramfs region.
    if let Some((start, end)) = parse_initramfs_range() {
        regions
            .push(MemoryRegion::new(
                start,
                end - start,
                MemoryRegionType::Module,
            ))
            .unwrap();
    }

    regions.into_non_overlapping()
}

fn parse_initramfs_range() -> Option<(usize, usize)> {
    None
}

/// Print the CPU configuration using `cpucfg` instruction.
fn print_cpu_config() {
    let prid = loongArch64::cpu::get_prid();
    let palen = loongArch64::cpu::get_palen();
    assert!(palen == 48);
    let valen = loongArch64::cpu::get_valen();
    assert!(valen == 48);
    let mmu_support_page = loongArch64::cpu::get_mmu_support_page();
    let support_huge_page = loongArch64::cpu::get_support_huge_page();
    let save_num = loongArch64::register::prcfg1::read().save_num();

    early_println!("CPU Configuration:");
    early_println!("  PRID: 0x{:x}", prid);
    early_println!("  PA Width: {} bits", palen);
    early_println!("  VA Width: {} bits", valen);
    early_println!("  MMU Support Page: {}", mmu_support_page);
    early_println!("  Support Huge Page: {}", support_huge_page);
    early_println!("  CSR Save Num: {}", save_num);
}

fn parse_embeded_data() -> [Option<&'static [u8]>; 10] {
    let mut array = [None; 10];
    const BUSYBOX_BIN: &[u8] = include_bytes!("../../../../../initramfs/loongarch/busybox");
    array[0] = Some(BUSYBOX_BIN);
    const TEST_SCRIPT: &[u8] = include_bytes!("../../../../../initramfs/test.sh");
    array[1] = Some(TEST_SCRIPT);
    array
}

/// The entry point of the Rust code portion of Asterinas.
#[no_mangle]
pub extern "C" fn loongarch_boot(_core_id: usize) -> ! {
    print_cpu_config();

    trap::trap::init();

    use crate::boot::{call_ostd_main, EarlyBootInfo, EARLY_INFO};

    EARLY_INFO.call_once(|| EarlyBootInfo {
        bootloader_name: parse_bootloader_name(),
        kernel_cmdline: parse_kernel_commandline(),
        initramfs: parse_initramfs(),
        acpi_arg: parse_acpi_arg(),
        framebuffer_arg: parse_framebuffer_info(),
        memory_regions: parse_memory_regions(),
        embeded_data: parse_embeded_data(),
    });

    call_ostd_main();
}
