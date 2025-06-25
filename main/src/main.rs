#![no_std]
#![no_main]

#![feature(linkage)]

extern crate aster_nix;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe extern "Rust" {
        pub fn __ostd_panic_handler(info: &core::panic::PanicInfo) -> !;
    }
    unsafe { __ostd_panic_handler(info); }
}

mod default_frame_allocator {
    use ostd::mm::frame::GlobalFrameAllocator;

    use osdk_frame_allocator::FrameAllocator;
    static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator;

    #[unsafe(no_mangle)]
    #[linkage = "weak"]
    static __GLOBAL_FRAME_ALLOCATOR_REF: &'static dyn GlobalFrameAllocator = &FRAME_ALLOCATOR;
}

mod default_heap_allocator {
    use ostd::mm::heap::GlobalHeapAllocator;

    use osdk_heap_allocator::{HeapAllocator, type_from_layout};
    static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator;

    #[unsafe(no_mangle)]
    #[linkage = "weak"]
    static __GLOBAL_HEAP_ALLOCATOR_REF: &'static dyn GlobalHeapAllocator = &HEAP_ALLOCATOR;

    #[unsafe(no_mangle)]
    #[linkage = "weak"]
    #[expect(non_snake_case)]
    fn __GLOBAL_HEAP_SLOT_INFO_FROM_LAYOUT(layout: core::alloc::Layout) -> Option<ostd::mm::heap::SlotInfo> {
        type_from_layout(layout)
    }
}