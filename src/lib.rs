#![no_std]
#![feature(asm, naked_functions)]
#![feature(const_in_array_repeat_expressions, const_mut_refs, const_panic)]

#[cfg(not(target_arch = "x86_64"))]
compile_error!("This library requires a x86_64 target");

#[cfg(feature = "start32")]
use x86_64::registers::control::{Cr0Flags, Cr4Flags, Efer, EferFlags};

#[cfg(feature = "start32")]
pub mod gdt;

#[cfg(feature = "build_page_tables")]
pub mod paging;

/// # Safety
#[naked]
#[cfg(feature = "start16")]
#[link_section = ".boot.text16"]
pub unsafe extern "C" fn start16() -> ! {
    asm!(".code16",
        // Step 1: Disable interrupts
        "cli",

        // Step 2: Load the GDT
        "movl ${gdt_ptr}, %ebx",
        "lgdtl %cs:(%bx)",

        // Step 3: Set CRO.PE
        "movl %cr0, %eax",
        "orb ${pe}, %al",
        "movl %eax, %cr0", // must be followed by a branch
        // Far jump to 32-bit code
        "ljmpl ${cs}, ${code32}",
        gdt_ptr = sym gdt::POINTER,
        pe = const Cr0Flags::PROTECTED_MODE_ENABLE.bits(),
        cs = const gdt::CS32.0,
        code32 = sym setup32,
        options(noreturn, att_syntax)
    )
}

#[naked]
#[cfg(feature = "start16")]
#[link_section = ".boot.text32"]
unsafe extern "C" fn setup32() -> ! {
    asm!(".code32",
        "mov ax, {ds}",
        "mov ds, eax",
        "mov es, eax",
        "mov ss, eax",

        // Zero out ebx, as we don't have a PVH StartInfo struct.
        "xor ebx, ebx",
        "jmp short {code32}",
        ds = const gdt::DS.0,
        code32 = sym start32,
        options(noreturn)
    )
}

/// # Safety
#[naked]
#[cfg(feature = "start32")]
#[link_section = ".boot.text32"]
pub unsafe extern "C" fn start32() {
    #[cfg(feature = "build_page_tables")]
    asm!(
        ".code32",
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
        l2_flags = const paging::L2_FLAGS.bits(),
        pml2 = sym paging::PML2,
        l2_entries = const paging::NUM_PML2_ENTRIES,
        flags = const paging::FLAGS.bits(),
        pml3 = sym paging::PML3,
        l3_entries = const paging::NUM_PML3_ENTRIES,
        pml4 = sym paging::PML4,
    );
    #[cfg(not(feature = "build_page_tables"))]
    asm!(
        ".code32",
        // Load our pre-built page tables
        "lea eax, [__rust_pml4]",
        "mov cr3, eax",
    );
    asm!(
        ".code32",
        // Setup our 64-bit GDT (not used until the long jump below)
        "lgdtl ({gdt_ptr})",
        // Set CR4.PAE (Physical Address Extension)
        "movl %cr4, %eax",
        "orb ${pae}, %al",
        "movl %eax, %cr4",
        // Set EFER.LME (Long Mode Enable)
        "movl ${efer}, %ecx",
        "rdmsr",
        "orl ${lme}, %eax",
        "wrmsr",
        // Set CRO.PG (Paging), must happen after the above 3 steps
        "movl %cr0, %eax",
        "orl ${pg}, %eax",
        "movl %eax, %cr0", // must be followed by a branch
        // Far jmp to 64-bit code
        "ljmpl ${cs}, ${code64}",
        gdt_ptr = sym gdt::POINTER,
        pae = const Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits(),
        efer = const Efer::MSR.0,
        lme = const EferFlags::LONG_MODE_ENABLE.bits(),
        pg = const Cr0Flags::PAGING.bits(),
        cs = const gdt::CS64.0,
        code64 = sym start64,
        options(noreturn, att_syntax)
    )
}

/// # Safety
#[naked]
#[link_section = ".boot.text64"]
pub unsafe extern "C" fn start64() {
    asm!(
        "lea     rax, [rip + 1f]",
        "1:",
        "movabs  r15, offset _GLOBAL_OFFSET_TABLE_",
        "add     r15, rax",
    );
    #[cfg(feature = "init_data")]
    asm!(
        "movabs  rsi, offset __init_data@GOTOFF",
        "add     rsi, r15",
        "movabs  rdi, offset __start_data@GOTOFF",
        "movabs  rcx, offset __stop_data@GOTOFF",
        "sub     rcx, rdi",
        "add     rdi, r15",
        "rep movsb [rdi], [rsi]",
    );
    #[cfg(feature = "zero_bss")]
    asm!(
        "movabs  rdi, offset __start_bss@GOTOFF",
        "movabs  rcx, offset __stop_bss@GOTOFF",
        "sub     rcx, rdi",
        "add     rdi, r15",
        "xor     eax, eax",
        "rep stosb [rdi], al",
    );
    asm!(
        "movabs  rsp, offset __rust_stack_top@GOTOFF",
        "add     rsp, r15",
        "movabs  rax, offset __rust_start@GOTOFF",
        "add     rax, r15",
        "jmp     rax",
        options(noreturn)
    )
}
