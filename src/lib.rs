#![cfg_attr(not(test), no_std)]

use core::arch::naked_asm;

use x86_64::registers::control::{Cr0Flags, Cr4Flags, EferFlags};

pub mod gdt;
pub mod paging;

#[cfg(not(target_arch = "x86_64"))]
compile_error!("This library requires a x86_64 target");

/// # Safety
#[unsafe(naked)]
#[unsafe(link_section = ".boot.text16")]
pub unsafe extern "C" fn start16() -> ! {
    naked_asm!(".code16",
        // Step 1: Disable interrupts
        "cli",

        // Step 2: Load the GDT
        // "mov ebx, offset {gdt_ptr}",
        "lgdtd cs:[{gdt_ptr}]",

        // Step 3: Set CRO.PE
        "mov eax, cr0",
        "or eax, {pe}",
        "mov cr0, eax", // must be followed by a branch

        // ljmpd {cs} : {setup32} (manually compiled for 32-bit mode)
        ".byte 0x66, 0xea",
        ".long {setup32}",
        ".short {cs}",
        ".code64",
        gdt_ptr = sym gdt::POINTER,
        pe = const Cr0Flags::PROTECTED_MODE_ENABLE.bits(),
        cs = const gdt::CS32.0,
        setup32 = sym setup32,
    )
}

#[unsafe(naked)]
#[unsafe(link_section = ".boot.text32")]
unsafe extern "C" fn setup32() -> ! {
    naked_asm!(".code32",
        "mov ax, {ds}",
        "mov ds, eax",
        "mov es, eax",
        "mov ss, eax",

        // Zero out ebx, as we don't have a PVH StartInfo struct.
        "xor ebx, ebx",
        "jmp short {start32}",
        ".code64",
        ds = const gdt::DS.0,
        start32 = sym start32
    )
}

/// # Safety
#[unsafe(naked)]
#[unsafe(link_section = ".boot.text32")]
pub unsafe extern "C" fn start32() {
    naked_asm!(".code32",
        // Point PML2s at the beginning of RAM
        "xor ecx, ecx",
        "mov eax, {l2_flags}",
        "2:",
        "mov [{pml2} + ecx * 8], eax",
        "add eax, (1 << 21)", // Each PML2 entry maps 2 MiB
        "inc ecx",
        "cmp ecx, {l2_entries}",
        "jb 2b",
        // Point PML3 entries at PML2s
        "xor ecx, ecx",
        "lea eax, [{pml2}]",
        "or eax, {flags}",
        "3:",
        "mov [{pml3} + ecx * 8], eax",
        "add eax, (1 << 12)", // Our PML2 pages are contiguous
        "inc ecx",
        "cmp ecx, {l3_entries}",
        "jb 3b",
        // Point PML4 entry at PML3
        "lea ecx, [{pml3}]",
        "or ecx, {flags}",
        "lea eax, [{pml4}]",
        "mov [eax], ecx",
        // Load our page tables
        "mov cr3, eax",
        // Setup our 64-bit GDT (not used until the long jump below)
        "lgdtd [{gdt_ptr}]",
        // Set CR4.PAE (Physical Address Extension)
        "mov eax, cr4",
        "or eax, {pae}",
        "mov cr4, eax",
        // Set EFER.LME (Long Mode Enable)
        "mov ecx, {efer}",
        "rdmsr",
        "or eax, {lme}",
        "wrmsr",
        // Set CRO.PG (Paging), must happen after the above 3 steps
        "mov eax, cr0",
        "or eax, {pg}",
        "mov cr0, eax", // must be followed by a branch
        // ljmpd {cs} : {start64} (manually compiled for 32-bit mode)
        ".byte 0xea",
        ".long {start64}",
        ".short {cs}",
        ".code64",
        l2_flags = const paging::L2_FLAGS.bits(),
        pml2 = sym paging::PML2,
        l2_entries = const paging::NUM_PML2_ENTRIES,
        flags = const paging::FLAGS.bits(),
        pml3 = sym paging::PML3,
        l3_entries = const paging::NUM_PML3_ENTRIES,
        pml4 = sym paging::PML4,
        gdt_ptr = sym gdt::POINTER,
        pae = const Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits(),
        efer = const 0xC000_0080u32,
        lme = const EferFlags::LONG_MODE_ENABLE.bits(),
        pg = const Cr0Flags::PAGING.bits(),
        cs = const gdt::CS64.0,
        start64 = sym start64,
    )
}

/// # Safety
#[unsafe(naked)]
#[unsafe(link_section = ".boot.text64")]
pub unsafe extern "C" fn start64() {
    naked_asm!(
        "lea     rax, [rip + 1f]",
        "1:",
        "movabs  r15, offset _GLOBAL_OFFSET_TABLE_",
        "add     r15, rax",
        "movabs  rsi, offset __init_data@GOTOFF",
        "add     rsi, r15",
        "movabs  rdi, offset __start_data@GOTOFF",
        "movabs  rcx, offset __stop_data@GOTOFF",
        "sub     rcx, rdi",
        "add     rdi, r15",
        "rep movsb [rdi], [rsi]",
        "movabs  rdi, offset __start_bss@GOTOFF",
        "movabs  rcx, offset __stop_bss@GOTOFF",
        "sub     rcx, rdi",
        "add     rdi, r15",
        "xor     eax, eax",
        "rep stosb [rdi], al",
        "movabs  rsp, offset __rust_stack_top@GOTOFF",
        "add     rsp, r15",
        "movabs  rax, offset __rust_start@GOTOFF",
        "add     rax, r15",
        "jmp     rax",
    )
}
