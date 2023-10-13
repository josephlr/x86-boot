use x86_64::structures::gdt::{
    Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector,
};
use x86_64::PrivilegeLevel::Ring0;

pub const CS32: SegmentSelector = SegmentSelector::new(1, Ring0);
pub const CS64: SegmentSelector = SegmentSelector::new(2, Ring0);
pub const DS: SegmentSelector = SegmentSelector::new(3, Ring0);

pub(crate) struct DescriptorTablePointer {
    _limit: u16,
    _base: &'static GlobalDescriptorTable,
}

#[link_section = ".boot.gdt"]
pub(crate) static POINTER: DescriptorTablePointer = DescriptorTablePointer {
    _limit: (4 * core::mem::size_of::<u64>() - 1) as u16,
    _base: &TABLE,
};
#[link_section = ".boot.gdt"]
pub static TABLE: GlobalDescriptorTable = make_gdt();

const fn make_gdt() -> GlobalDescriptorTable {
    let kernel_code32 = Descriptor::UserSegment(DescriptorFlags::KERNEL_CODE32.bits());

    let mut gdt = GlobalDescriptorTable::new();
    assert!(CS32.0 == gdt.add_entry(kernel_code32).0);
    assert!(CS64.0 == gdt.add_entry(Descriptor::kernel_code_segment()).0);
    assert!(DS.0 == gdt.add_entry(Descriptor::kernel_data_segment()).0);
    gdt
}
