/* SPDX-License-Identifier: MPL-2.0 */

.equ LOONGARCH_CSR_CRMD,            0x0     /* Current mode */
.equ LOONGARCH_CSR_PRMD,            0x1     /* Previous mode */
.equ LOONGARCH_CSR_EUEN,            0x2     /* Extented unit enable */
.equ LOONGARCH_CSR_PWCL,            0x1c    /* Page table walk control low */
.equ LOONGARCH_CSR_PWCH,            0x1d    /* Page table walk control high */
.equ LOONGARCH_CSR_STLBPS,          0x1e    /* STLB page size */
.equ LOONGARCH_CSR_DMW0,            0x180   /* Direct mapping window 0 */
.equ LOONGARCH_CSR_DMW1,            0x181   /* Direct mapping window 1 */
.equ LOONGARCH_CSR_DMW2,            0x182   /* Direct mapping window 2 */
.equ LOONGARCH_CSR_DMW3,            0x183   /* Direct mapping window 3 */
.equ LOONGARCH_CSR_PGDL,            0x19    /* Page table base address when VA[47] = 0 */
.equ LOONGARCH_CSR_PGDH,            0x1a    /* Page table base address when VA[47] = 1 */
.equ LOONGARCH_CSR_PGD,             0x1b    /* Page table base */
.equ LOONGARCH_CSR_CPUID,           0x20    /* CPUID */
.equ LOONGARCH_CSR_TLBRENTRY,       0x88    /* TLB refill exception entry */
.equ LOONGARCH_CSR_TLBRBADV,        0x89    /* TLB refill badvaddr */
.equ LOONGARCH_CSR_TLBRERA,         0x8a    /* TLB refill ERA */
.equ LOONGARCH_CSR_TLBRSAVE,        0x8b    /* KScratch for TLB refill exception */
.equ LOONGARCH_CSR_TLBRELO0,        0x8c    /* TLB refill entrylo0 */
.equ LOONGARCH_CSR_TLBRELO1,        0x8d    /* TLB refill entrylo1 */
.equ LOONGARCH_CSR_TLBREHI,         0x8e    /* TLB refill entryhi */
.equ LOONGARCH_CSR_SAVE0,           0x30    /* Save 0 */
.equ LOONGARCH_CSR_SAVE1,           0x31    /* Save 1 */
.equ LOONGARCH_CSR_SAVE2,           0x32    /* Save 2 */
.equ LOONGARCH_CSR_SAVE3,           0x33    /* Save 3 */
.equ LOONGARCH_CSR_SAVE4,           0x34    /* Save 4 */
.equ LOONGARCH_CSR_SAVE5,           0x35    /* Save 5 */
.equ LOONGARCH_CSR_SAVE6,           0x36    /* Save 6 */
.equ LOONGARCH_CSR_SAVE7,           0x37    /* Save 7 */


.section .text.entry
.globl _start
_start:
    # Set DMW0 (kernel)
    li.d    $t0, 0x9000000000000011     # CA, PLV0, 0x9000_xxxx_xxxx_xxxx
    csrwr   $t0, LOONGARCH_CSR_DMW0

    # Set DMW2 (device)
    li.d    $t0, 0x8000000000000001     # UA, PLV0, 0x8000_xxxx_xxxx_xxxx
    csrwr   $t0, LOONGARCH_CSR_DMW2

    # Enable address translation and disable interrupts
    li.w    $t0, 0x00000010		        # PLV=0, IE=0, PG=1
    csrwr   $t0, LOONGARCH_CSR_CRMD
    li.w    $t0, 0x00000000		        # PPLV=0, PIE=0
    csrwr   $t0, LOONGARCH_CSR_PRMD

    # Disable floating point unit etc.
    li.w    $t0, 0x00000001		        # FPE=1, SXE=0, ASXE=0, BTE=0
    csrwr   $t0, LOONGARCH_CSR_EUEN

    # Set the first level (root level) page table size
    li.w    $t0, 0x0000000c             # 4KB
    csrwr   $t0, LOONGARCH_CSR_STLBPS
    li.w    $t0, 0x0000000c
    csrwr   $t0, LOONGARCH_CSR_TLBREHI

    # Set the structure of page table
    # PTbase=12, PTwidth=9, Dir1_base=12 + 9, Dir1_width=9, Dir2_base=12 + 9 + 9, Dir2_width=9, PTEWidth=0 (64 bits)
    li.w    $t0, 12 | 9 << 5 | 21 << 10 | 9 << 15 | 30 << 20 | 9 << 25
    csrwr   $t0, LOONGARCH_CSR_PWCL
    # Dir3_base=12 + 9 + 9 + 9, Dir3_width=9
    li.w    $t0, 39 | 9 << 6
    csrwr   $t0, LOONGARCH_CSR_PWCH

    # Set the entry of TLB refill exception
    la.global   $t0, _handle_tlb_refill
    # Convert the address to physical address
.extern KERNEL_VMA_OFFSET
    la.global   $t1, KERNEL_VMA_OFFSET
    sub.d   $t0, $t0, $t1
    csrwr   $t0, LOONGARCH_CSR_TLBRENTRY

    # Set the boot page table
    la.global   $t0, boot_pagetable
    sub.d   $t0, $t0, $t1
    csrwr   $t0, LOONGARCH_CSR_PGDL
    la.global   $t0, boot_pagetable
    sub.d   $t0, $t0, $t1
    csrwr   $t0, LOONGARCH_CSR_PGDH

    # Initialize TLB
    invtlb  0, $zero, $zero

    # Set the stack pointer (BSP only)
    la.global   $sp, boot_stack_top

    # Set gp (CPU-local address)
.extern __cpu_local_start
    la.global   $r21, __cpu_local_start

    # Jump to rust loongarch_boot (a0 = CoreId)
    csrrd   $a0, LOONGARCH_CSR_CPUID
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
    .zero 8 * 512


.section .text
.align 12
.global _handle_tlb_refill
_handle_tlb_refill:
    # Save $t0, $t1, $t2
    csrwr   $t0, LOONGARCH_CSR_TLBRSAVE
    // csrwr   $t1, LOONGARCH_CSR_SAVE0
    // csrwr   $t2, LOONGARCH_CSR_SAVE1

    csrrd $t0, LOONGARCH_CSR_TLBRBADV  # Read badvaddr
    csrrd $t0, LOONGARCH_CSR_TLBRERA  # Read ERA

    # Read PGD
    csrrd   $t0, LOONGARCH_CSR_PGD

    # Walk page table
    // lddir   $t0, $t0, 4
    lddir   $t0, $t0, 3
    lddir   $t0, $t0, 2
    lddir   $t0, $t0, 1

    # Load PTEs
    ldpte   $t0, 0
    ldpte   $t0, 1

//     # Set page size
//     # Check if t0[6] == 1 -> huge page
//     andi    $t1, $t0, 0x40
//     beqz    $t1, _not_hugepage

//     # Extract bits t0[14:13] -> t1
//     srli.d  $t1, $t0, 12
//     andi    $t1, $t1, 0x3

//     # Check if t1 == 0 -> t1 = 4
//     beqz    $t1, _t1_4
//     b       _cal_ps

// _t1_4:
//     li.d    $t1, 4
//     b       _cal_ps

// _not_hugepage:
//     li.d    $t1, 0
//     b       _cal_ps

// _cal_ps:
//     # Compute PS = 12 + 9 * n
//     li.d    $t0, 9
//     mul.w   $t1, $t1, $t0       # t1 = 9 * n
//     addi.d  $t1, $t1, 12        # t1 = PS

//     # Read TLBREHI
//     csrrd   $t0, LOONGARCH_CSR_TLBREHI

//     # Clear old PS [5:0]
//     li.d    $t2, ~0x3F
//     and     $t0, $t0, $t2

//     # Insert new PS
//     or      $t0, $t0, $t1

//     # Write back TLBREHI
//     csrwr   $t0, LOONGARCH_CSR_TLBREHI

    # Fill TLB
    tlbfill

    # Restore $t0, $t1, $t2
    csrrd   $t0, LOONGARCH_CSR_TLBRSAVE
    // csrwr   $t1, LOONGARCH_CSR_SAVE0
    // csrwr   $t2, LOONGARCH_CSR_SAVE1

    ertn
