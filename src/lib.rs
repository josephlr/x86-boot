#![no_std]
#![feature(asm, naked_functions)]
#![feature(const_mut_refs, const_panic)]

use x86_64::registers::control::{Cr0Flags, Cr4Flags, EferFlags};
use x86_64::structures::{
    gdt::{
        Descriptor, DescriptorFlags as Flags, GlobalDescriptorTable as Table,
        SegmentSelector as Selector,
    },
    DescriptorTablePointer,
};
use x86_64::PrivilegeLevel::Ring0;

#[naked]
#[link_section = ".boot.text16"]
pub unsafe extern "C" fn start16() -> ! {
    asm!(".code16",
        // Step 1: Disable interrupts
        "cli",

        // Step 2: Load the GDT
        "lea bx, [{gdt_ptr}]",
        "lgdtd cs:[bx]",

        // Step 3: Set CRO.PE
        "mov eax, cr0",
        "or al, {pe}",
        "mov cr0, eax", // must be followed by a branch
        // Far jump to 32-bit code
        "jmpl {cs}, OFFSET {code32}",
        gdt_ptr = sym GDT_PTR,
        pe = const Cr0Flags::PROTECTED_MODE_ENABLE.bits(),
        cs = const CS32.0,
        code32 = sym setup_pvh,
        options(noreturn)
    )
}

#[naked]
#[link_section = ".boot.text32"]
unsafe extern "C" fn setup_pvh() -> ! {
    asm!(".code32",
        "mov ax, {ds}",
        "mov ds, eax",
        "mov es, eax",
        "mov ss, eax",

        // Zero out ebx, as we don't have a PVH StartInfo struct.
        "xor ebx, ebx",
        "jmp {pvh_start}",
        ds = const DS.0,
        pvh_start = sym start32,
        options(noreturn)
    )
}

#[naked]
#[link_section = ".boot.text32"]
pub unsafe extern "C" fn start32() {
    asm!(".code32",
        // Setup our 64-bit GDT (not used until the long jump below)
        "lgdt [{gdt_ptr}]",

        // This is equivalent to: memcpy(pt_virt_start, pt_phys_start, pt_size)
        "lea edi, [data_start]",
        "lea esi, [rom_data_start]",
        "mov ecx, offset data_size",
        "rep movsb [edi], [esi]",

        // Load our static page tables
        "lea eax, [rust_page_tables]",
        "mov cr3, eax",
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
        // Far jmp to 64-bit code
        "jmp {cs}, OFFSET {code64}",
        gdt_ptr = sym GDT_PTR,
        efer = const 0xC0000080u32,
        pae = const Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits(),
        lme = const EferFlags::LONG_MODE_ENABLE.bits(),
        pg = const Cr0Flags::PAGING.bits(),
        cs = const CS64.0,
        code64 = sym start64,
        options(noreturn)
    )
}

#[naked]
#[link_section = ".boot.text64"]
pub unsafe extern "C" fn start64() {
    asm!(
        "lea     rax, [rip + 1f]",
        "1:",
        "movabs  rcx, offset _GLOBAL_OFFSET_TABLE_",
        "add     rcx, rax",
        "movabs  rsp, offset rust_stack_top@GOTOFF",
        "add     rsp, rcx",
        "movabs  rax, offset rust_start@GOTOFF",
        "add     rax, rcx",
        "jmp     rax",
        options(noreturn)
    )
    // asm!(
    //     "lea     rcx, [rip + 1f]",
    //     "1:",
    //     "movabs  rax, offset _GLOBAL_OFFSET_TABLE_",
    //     "add     rax, rcx",
    //     "mov     rsp, [rax + offset rust_stack_top@GOT]",
    //     "jmp     [rax + offset rust_start@GOT]",
    //     options(noreturn)
    // )
    // asm!(
    //     "movabs  rsp, offset rust_stack_top",
    //     "movabs  rax, offset rust_start",
    //     "jmp     rax",
    //     options(noreturn)
    // )
}

const CS32: Selector = Selector::new(1, Ring0);
const CS64: Selector = Selector::new(2, Ring0);
const DS: Selector = Selector::new(3, Ring0);

// struct Pointer 
#[repr(C, packed)]
pub struct Pointer {
    pub limit: u16,
    pub base: *const Flags,
}
unsafe impl Send for Pointer {}
unsafe impl Sync for Pointer {}

// #[link_section = ".boot.gdt_ptr"]
static GDT_PTR: Pointer = Pointer{limit: 31, base: GDT.as_ptr()};
// #[link_section = ".boot.gdt"]
static GDT: [Flags; 4] = [Flags::empty(), Flags::KERNEL_CODE32, Flags::KERNEL_CODE64, Flags::KERNEL_DATA];

// const fn make_gdt() -> Table {
//     let kernel_code32 = Descriptor::UserSegment(Flags::KERNEL_CODE32.bits());

//     let mut gdt = Table::new();
//     assert!(CS32.0 == gdt.add_entry(kernel_code32).0);
//     assert!(CS64.0 == gdt.add_entry(Descriptor::kernel_code_segment()).0);
//     assert!(DS.0 == gdt.add_entry(Descriptor::kernel_data_segment()).0);
//     gdt
// }
