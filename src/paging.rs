use x86_64::{
    align_up,
    structures::paging::{PageTable, PageTableFlags as Flags},
};

// 2^20 identity mapped 4K pages is 4 GiB.
const MAPPED_PAGES: u64 = 1 << 20;
pub(crate) const NUM_PML2_ENTRIES: u64 = align_up(MAPPED_PAGES, 512) / 512;
pub(crate) const NUM_PML3_ENTRIES: u64 = align_up(NUM_PML2_ENTRIES, 512) / 512;
const NUM_PML4_ENTRIES: u64 = align_up(NUM_PML3_ENTRIES, 512) / 512;

// Common flags to all page table entries.
pub const FLAGS: Flags = Flags::from_bits_truncate(
    Flags::PRESENT.bits() | Flags::WRITABLE.bits() | Flags::ACCESSED.bits(),
);
// Set HUGE_PAGE in our L2 so we don't need to have a PML1
pub const L2_FLAGS: Flags =
    Flags::from_bits_truncate(FLAGS.bits() | Flags::DIRTY.bits() | Flags::HUGE_PAGE.bits());

#[link_section = ".boot.page_tables"]
pub static mut PML4: PageTable = PageTable::new();
#[link_section = ".boot.page_tables"]
pub static mut PML3: [PageTable; NUM_PML4_ENTRIES as _] = [PageTable::new()];
#[link_section = ".boot.page_tables"]
pub static mut PML2: [PageTable; NUM_PML3_ENTRIES as _] =
    [const { PageTable::new() }; NUM_PML3_ENTRIES as _];
