#![no_std]
#![no_main]
#![feature(asm, naked_functions)]
#![feature(const_panic, const_ptr_offset, const_mut_refs)]

use core::panic::PanicInfo;
use core::{fmt, mem::align_of};
use uart_16550::SerialPort;
use x86_64::{
    structures::paging::{PageSize, PageTableFlags as Flags, Size2MiB},
    PhysAddr,
};

#[naked]
#[no_mangle]
#[link_section = ".boot.reset"]
unsafe extern "C" fn reset() {
    asm!(
        ".code16",
        ".align 16",
        "mov eax, offset {code16}",
        "jmp ax",
        code16 = sym x86_boot::start16,
        options(noreturn),
    )
}

#[no_mangle]
extern "C" fn rust_start() {
    use core::{fmt::Write, ptr::{read_volatile, write_volatile}};
    let mut serial = unsafe { SerialPort::new(0x3f8) };
    serial.init();

    let p = ((1u64 << 21) + 0x1000) as *mut u64;
    unsafe { write_volatile(p, 0xff) };
    let b = unsafe { read_volatile(p) };
    writeln!(serial, "{:p} - 0x{:x}", p, b).unwrap();
    let p = ((1u64 << 39) + 0x1000) as *mut u64;
    let b = unsafe { read_volatile(p) };
    writeln!(serial, "{:p} - 0x{:x}", p, b).unwrap();

    writeln!(serial, "Old L2 {:?}", L2_FLAGS).unwrap();
    writeln!(serial, "Old L3 {:?}", FLAGS).unwrap();
    writeln!(serial, "Old L4 {:?}", FLAGS).unwrap();

    let l2 = unsafe { read_volatile(&PML2[3].0[511]) };
    let l3 = unsafe { read_volatile(&PML3[0].0[3])};
    let l4 = unsafe { read_volatile(&PML4[0].0[0])};
    writeln!(serial, "New L2 {:?}", l2.flags()).unwrap();
    writeln!(serial, "New L3 {:?}", l3.flags()).unwrap();
    writeln!(serial, "New L4 {:?}", l4.flags()).unwrap();

    let l2 = unsafe { read_volatile(&PML2[0].0[0]) };
    let l3 = unsafe { read_volatile(&PML3[0].0[0])};
    let l4 = unsafe { read_volatile(&PML4[0].0[0])};
    writeln!(serial, "New L2 {:?}", l2.flags()).unwrap();
    writeln!(serial, "New L3 {:?}", l3.flags()).unwrap();
    writeln!(serial, "New L4 {:?}", l4.flags()).unwrap();

    for b in b"Hello, World!\nMy name is Joe!\n" {
        serial.send(*b);
    }
    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

/// Alternative to x86_64::PageTableEntry that can be used with static pointers.
struct Entry(*const u8);
unsafe impl Send for Entry {}
unsafe impl Sync for Entry {}

impl Entry {
    const fn new() -> Self {
        Self(core::ptr::null())
    }
    const fn leaf_entry(addr: PhysAddr, flags: Flags) -> Self {
        assert!((addr.as_u64() as usize) % align_of::<Table>() == 0);
        Self((addr.as_u64() + flags.bits()) as *const _)
    }
    const fn ptr_entry(table: &Table, flags: Flags) -> Self {
        let ptr: *const u8 = table as *const Table as *const u8;
        Self(ptr.wrapping_add(flags.bits() as usize))
    }
    fn addr(&self) -> PhysAddr {
        PhysAddr::new((self.0 as u64) - self.flags().bits())
    }
    fn flags(&self) -> Flags {
        let bits = (self.0 as usize) % align_of::<Table>();
        Flags::from_bits_truncate(bits as u64)
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Entry");
        f.field("addr", &self.addr());
        f.field("flags", &self.flags());
        f.finish()
    }
}

const ENTRY_COUNT: usize = 512;

/// Alternative to x86_64::PageTable that uses alternative Entry
#[repr(C, align(4096))]
pub struct Table([Entry; ENTRY_COUNT]);

impl Table {
    const fn new() -> Self {
        const ENTRY: Entry = Entry::new();
        Self([ENTRY; ENTRY_COUNT])
    }
}

// Number of GiB to identity map (max 512 GiB)
const ADDRESS_SPACE_GIB: usize = 4;

// Common flags to all page table entries.
const FLAGS: Flags = Flags::from_bits_truncate(Flags::PRESENT.bits() | Flags::WRITABLE.bits() | Flags::ACCESSED.bits());
// Set HUGE_PAGE in our L2 so we don't need to have a PML1
const L2_FLAGS: Flags = Flags::from_bits_truncate(FLAGS.bits() | Flags::DIRTY.bits() | Flags::HUGE_PAGE.bits());

// Make a PML2 that maps virtual addresses [0, ADDRESS_SPACE_GIB) to a
// contiguous range of physical addresses starting at `start`.
const fn make_pml2s(start: u64) -> [Table; ADDRESS_SPACE_GIB] {
    const TABLE: Table = Table::new();
    let mut pml2s = [TABLE; ADDRESS_SPACE_GIB];
    let mut addr = start;

    let mut i = 0;
    while i < ADDRESS_SPACE_GIB {
        let pml2 = &mut pml2s[i];

        let mut j = 0;
        while j < pml2.0.len() {
            pml2.0[j] = Entry::leaf_entry(PhysAddr::new_truncate(addr), L2_FLAGS);
            addr += Size2MiB::SIZE;
            j += 1;
        }
        i += 1;
    }
    pml2s
}

// Make a page table pointing to the provided array of tables.
const fn make_ptr_table(tables: &[Table]) -> Table {
    let mut table = Table::new();
    let mut j = 0;
    while j < tables.len() {
        table.0[j] = Entry::ptr_entry(&tables[j], FLAGS);
        j += 1;
    }
    table
}

#[link_section = ".boot.l2"]
static PML2: [Table; ADDRESS_SPACE_GIB] = make_pml2s(0);
#[link_section = ".boot.l2"]
static PML2A: [Table; ADDRESS_SPACE_GIB] = make_pml2s(2 * (1 << 20));
#[link_section = ".boot.l3"]
static PML3: [Table; 2] = [make_ptr_table(&PML2), make_ptr_table(&PML2A)];

#[export_name = "rust_page_tables"]
#[link_section = ".boot.l4"]
pub static PML4: [Table; 1] = [make_ptr_table(&PML3)];
