/* SPDX-License-Identifier: MPL-2.0 */

.section .text.entry
.globl _start
_start:
    # Set DMW0
    li.d    $t0, 0x8000000000000001 # UA, PLV0, 0x8000_xxxx_xxxx_xxxx
    csrwr   $t0, 0x180              # LOONGARCH_CSR_DMW0

    # Set DMW1
    li.d    $t0, 0x9000000000000011 # CA, PLV0, 0x9000_xxxx_xxxx_xxxx
    csrwr   $t0, 0x181              # LOONGARCH_CSR_DMW1

    # Set DMW2
    # li.d    $t0, 0x8000000000000001 # UA, PLV0, 0x8000_xxxx_xxxx_xxxx
    # csrwr   $t0, 0x182              # LOONGARCH_CSR_DMW2

    # Enable address translation and disable interrupts
    li.w    $t0, 0x000000b0		    # PLV=0, IE=0, PG=1
    csrwr   $t0, 0x0                # LOONGARCH_CSR_CRMD
    li.w    $t0, 0x00000000		    # PLV=0, PIE=0, PWE=0
    csrwr   $t0, 0x1                # LOONGARCH_CSR_PRMD

    # Disable floating point unit etc.
    li.w    $t0, 0x00000000		    # FPE=0, SXE=0, ASXE=0, BTE=0
    csrwr   $t0, 0x2                # LOONGARCH_CSR_EUEN

    # Set the first page table
    li.w    $t0, 0x0c               # 4KB
    csrwr   $t0, 0x1e               # LOONGARCH_CSR_STLBPS

    # PTbase=12, PTwidth=9, Dir1_base=12 + 9, Dir1_width=9, Dir2_base=12 + 9 + 9, Dir2_width=9, PTEWidth=0 (64 bits)
    li.w    $t0, 12 | 9 << 5 | 21 << 10 | 9 << 15 | 30 << 20 | 9 << 25
    csrwr   $t0, 0x1c               # LOONGARCH_CSR_PWCL
    # Dir3_base=12 + 9 + 9 + 9, Dir3_width=9
    li.w    $t0, 39 | 9 << 6
    csrwr   $t0, 0x1d               # LOONGARCH_CSR_PWCH

    la.global   $t0, boot_pagetable
    csrwr   $t0, 0x19               # LOONGARCH_CSR_PGDL
    la.global   $t0, boot_pagetable
    csrwr   $t0, 0x1a               # LOONGARCH_CSR_PGDH
    invtlb  0, $zero, $zero
    la.global   $t0, handle_tlb_refill
    csrwr   $t0, 0x88   # LOONGARCH_CSR_TLBRENTRY

    # Set the stack pointer (BSP only)
    la.global   $sp, boot_stack_top

    # Set gp (CPU-local address)
.extern __cpu_local_start
    la.global   $r21, __cpu_local_start

    # Jump to rust loongarch_boot (a0 = CoreId)
    csrrd   $a0, 0x20               # LOONGARCH_CSR_CPUID
.extern loongarch_boot
    la.global   $t0, loongarch_boot
    jr      $t0


.section .bss.stack

.globl boot_stack_bottom
boot_stack_bottom:
    .space 0x40000 # 64 KiB

.globl boot_stack_top
boot_stack_top:


.section .data

.align 12
boot_pagetable:
    .quad (0x00000 << 12) | 0x1d1 # VRWXAD
    .zero 8 * 255
    .quad (0x00000 << 12) | 0x1d1 # VRWXAD
    .zero 8 * 254
    .quad 0


// TLB Refill handler
.equ LA_CSR_PGDL,          0x19    /* Page table base address when VA[47] = 0 */
.equ LA_CSR_PGDH,          0x1a    /* Page table base address when VA[47] = 1 */
.equ LA_CSR_PGD,           0x1b    /* Page table base */
.equ LA_CSR_TLBRENTRY,     0x88    /* TLB refill exception entry */
.equ LA_CSR_TLBRBADV,      0x89    /* TLB refill badvaddr */
.equ LA_CSR_TLBRERA,       0x8a    /* TLB refill ERA */
.equ LA_CSR_TLBRSAVE,      0x8b    /* KScratch for TLB refill exception */
.equ LA_CSR_TLBRELO0,      0x8c    /* TLB refill entrylo0 */
.equ LA_CSR_TLBRELO1,      0x8d    /* TLB refill entrylo1 */
.equ LA_CSR_TLBREHI,       0x8e    /* TLB refill entryhi */

.section .text
.balign 4096
.global handle_tlb_refill
handle_tlb_refill:
    csrwr   $t0, LA_CSR_TLBRSAVE
    csrrd   $t0, LA_CSR_PGD
    lddir   $t0, $t0, 3
    lddir   $t0, $t0, 2
    lddir   $t0, $t0, 1
    ldpte   $t0, 0
    ldpte   $t0, 1
    tlbfill
    csrrd   $t0, LA_CSR_TLBRSAVE
    ertn