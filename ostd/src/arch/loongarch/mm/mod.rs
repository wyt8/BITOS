// SPDX-License-Identifier: MPL-2.0

use alloc::fmt;
use core::ops::Range;

use crate::{
    mm::{
        page_prop::{CachePolicy, PageFlags, PageProperty, PrivilegedPageFlags as PrivFlags},
        page_table::PageTableEntryTrait,
        Paddr, PagingConstsTrait, PagingLevel, PodOnce, Vaddr, PAGE_SIZE,
    },
    util::marker::SameSizeAs,
    Pod,
};

#[derive(Clone, Debug, Default)]
pub struct PagingConsts {}

impl PagingConstsTrait for PagingConsts {
    const BASE_PAGE_SIZE: usize = 4096;
    const NR_LEVELS: PagingLevel = 4;
    const ADDRESS_WIDTH: usize = 48;
    const HIGHEST_TRANSLATION_LEVEL: PagingLevel = 4;
    const PTE_SIZE: usize = core::mem::size_of::<PageTableEntry>();
    const VA_SIGN_EXT: bool = true;
}

bitflags::bitflags! {
    #[derive(Pod)]
    #[repr(C)]
    /// Possible flags for a page table entry.
    pub struct PageTableFlags: usize {
        /// Specifies whether the mapped frame or page table is valid.
        const VALID =           1 << 0;
        /// Whether the memory area represented by this entry is modified.
        const DIRTY =           1 << 1;
        /// Privilege level corresponding to the page table entry.
        /// When `RPLV` = 0, the page table entry can be accessed by any program
        /// with a privilege level not lower than `PLV`;
        /// When `RPLV` = 1, this page table entry can only be accessed by programs
        /// with privilege level equal to `PLV`.
        const PLVL =            1 << 2;
        const PLVH =            1 << 3;
        /// Control the memory access type of the memory access operation
        /// falling on the address space of the table page entry.
        const MATL =            1 << 4;
        const MATH =            1 << 5;
        /// Whether the memory area represented by this entry is accessed.
        const GLOBAL_HUGE =          1 << 6;
        /// Specifies whether the mapped frame or page table is loaded in memory.
        const PRESENT =         1 << 7;
        /// Controls whether writes to the mapped frames are allowed.
        const WRITABLE =        1 << 8;
        /// ...
        const GLOBAL =          1 << 12;
        /// Controls whether reads to the mapped frames are not allowed.
        const NOT_READABLE =    1 << 61;
        /// Controls whether execution code in the mapped frames are not allowed.
        const NOT_EXECUTABLE =  1 << 62;
        /// Whether the PageTableEntry can only be accessed by the privileged level `PLV` field inferred 
        const RPLV =            1 << 63;
    }
}

/// Flush any TLB entry that contains the map of the given virtual address.
///
/// This flush performs regardless of the global-page bit. So it can flush both global
/// and non-global entries.
pub(crate) fn tlb_flush_addr(vaddr: Vaddr) {
    // unsafe {
    //     riscv::asm::sfence_vma(0, vaddr);
    // }
}

/// Flush any TLB entry that intersects with the given address range.
pub(crate) fn tlb_flush_addr_range(range: &Range<Vaddr>) {
    for vaddr in range.clone().step_by(PAGE_SIZE) {
        tlb_flush_addr(vaddr);
    }
}

/// Flush all TLB entries except for the global-page entries.
pub(crate) fn tlb_flush_all_excluding_global() {
    // // TODO: excluding global?
    // riscv::asm::sfence_vma_all()
}

/// Flush all TLB entries, including global-page entries.
pub(crate) fn tlb_flush_all_including_global() {
    // riscv::asm::sfence_vma_all()
}


#[derive(Clone, Copy, Pod, Default)]
#[repr(C)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    const PHYS_ADDR_MASK: usize = 0x0000_FFFF_FFFF_F000;

    fn new_paddr(paddr: Paddr) -> Self {
        let ppn = paddr >> 12;
        Self(ppn << 10)
    }
}

/// Activate the given level 4 page table.
///
/// "satp" register doesn't have a field that encodes the cache policy,
/// so `_root_pt_cache` is ignored.
///
/// # Safety
///
/// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
/// changing the page mapping.
pub unsafe fn activate_page_table(root_paddr: Paddr, _root_pt_cache: CachePolicy) {
    assert!(root_paddr % PagingConsts::BASE_PAGE_SIZE == 0);
    // let ppn = root_paddr >> 12;
    // riscv::register::satp::set(riscv::register::satp::Mode::Sv48, 0, ppn);
}

pub fn current_page_table_paddr() -> Paddr {
    let pgdl = loongArch64::register::pgdl::read().raw();
    let pgdh = loongArch64::register::pgdh::read().raw();
    let pgd = loongArch64::register::pgd::read().raw();
    assert_eq!(pgdl, pgdh, "Only support to share the same page table for both user and kernel space");
    pgdl & 0x0000_FFFF_FFFF_F000
}



/// Parse a bit-flag bits `val` in the representation of `from` to `to` in bits.
macro_rules! parse_flags {
    ($val:expr, $from:expr, $to:expr) => {
        ($val as usize & $from.bits() as usize) >> $from.bits().ilog2() << $to.bits().ilog2()
    };
}

// SAFETY: `PageTableEntry` has the same size as `usize`
unsafe impl SameSizeAs<usize> for PageTableEntry {}

impl PodOnce for PageTableEntry {}

impl PageTableEntryTrait for PageTableEntry {
    fn is_present(&self) -> bool {
        self.0 & PageTableFlags::VALID.bits() != 0
    }

    fn new_page(paddr: Paddr, _level: PagingLevel, prop: PageProperty) -> Self {
        let mut pte = Self(paddr & Self::PHYS_ADDR_MASK);
        pte.set_prop(prop);
        pte
    }

    fn new_pt(paddr: Paddr) -> Self {
        // In LoongArch, non-leaf PTE only contains the physical address of the next page table.
        Self(paddr & Self::PHYS_ADDR_MASK)
    }

    fn paddr(&self) -> Paddr {
        let ppn = (self.0 & Self::PHYS_ADDR_MASK) >> 10;
        ppn << 12
    }

    fn prop(&self) -> PageProperty {
    let flags = (parse_flags!(!(self.0), PageTableFlags::NOT_READABLE, PageFlags::R))
            | (parse_flags!(self.0, PageTableFlags::WRITABLE, PageFlags::W))
            | (parse_flags!(!(self.0), PageTableFlags::NOT_EXECUTABLE, PageFlags::X))
            | (parse_flags!(self.0, PageTableFlags::PRESENT, PageFlags::ACCESSED))
            | (parse_flags!(self.0, PageTableFlags::DIRTY, PageFlags::DIRTY));
            // | (parse_flags!(self.0, PageTableFlags::, PageFlags::AVAIL1))
            // | (parse_flags!(self.0, PageTableFlags::RSV2, PageFlags::AVAIL2));
        let priv_flags = (parse_flags!(self.0, PageTableFlags::PLVL, PrivFlags::USER))
            | (parse_flags!(self.0, PageTableFlags::GLOBAL, PrivFlags::GLOBAL));

        let cache = if self.0 & PageTableFlags::MATL.bits() != 0 {
            CachePolicy::Uncacheable
        } else {
            CachePolicy::Writeback
        };

        PageProperty {
            flags: PageFlags::from_bits(flags as u8).unwrap(),
            cache,
            priv_flags: PrivFlags::from_bits(priv_flags as u8).unwrap(),
        }
    }

    fn set_prop(&mut self, prop: PageProperty) {
        let mut flags = PageTableFlags::VALID.bits()
            | parse_flags!(prop.flags.bits(), PageFlags::R, PageTableFlags::NOT_READABLE)
            | parse_flags!(prop.flags.bits(), PageFlags::W, PageTableFlags::WRITABLE)
            | parse_flags!(prop.flags.bits(), PageFlags::X, PageTableFlags::NOT_EXECUTABLE)
            | parse_flags!(
                prop.priv_flags.bits(),
                PrivFlags::USER,
                PageTableFlags::PLVL
            )
            | parse_flags!(
                prop.priv_flags.bits(),
                PrivFlags::GLOBAL,
                PageTableFlags::GLOBAL
            )
            | parse_flags!(prop.flags.bits(), PageFlags::AVAIL1, PageTableFlags::MATL)
            | parse_flags!(prop.flags.bits(), PageFlags::AVAIL2, PageTableFlags::MATH);

        match prop.cache {
            CachePolicy::Writeback => (),
            CachePolicy::Uncacheable => {
                // // Currently, Asterinas uses `Uncacheable` for I/O memory.
                // flags |= PageTableFlags::PBMT_IO.bits()
            }
            _ => panic!("unsupported cache policy"),
        }

        self.0 = (self.0 & Self::PHYS_ADDR_MASK) | flags;
    }

    fn is_last(&self, level: PagingLevel) -> bool {
        let rwx = PageTableFlags::NOT_READABLE | PageTableFlags::WRITABLE | PageTableFlags::NOT_EXECUTABLE;
        level == 1 || (self.0 & rwx.bits()) != 0
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("raw", &format_args!("{:#x}", self.0))
            .field("paddr", &format_args!("{:#x}", self.paddr()))
            .field("present", &self.is_present())
            .field(
                "flags",
                &PageTableFlags::from_bits_truncate(self.0 & !Self::PHYS_ADDR_MASK),
            )
            .field("prop", &self.prop())
            .finish()
    }
}

pub(crate) fn __memcpy_fallible(dst: *mut u8, src: *const u8, size: usize) -> usize {
    // TODO: implement fallible
    unsafe { core::ptr::copy(src, dst, size) };
    0
}

pub(crate) fn __memset_fallible(dst: *mut u8, value: u8, size: usize) -> usize {
    // TODO: implement fallible
    unsafe { core::ptr::write_bytes(dst, value, size) };
    0
}