#![no_std]
#![no_main]

use core::{arch::naked_asm, fmt::Write, panic::PanicInfo};
use uart_16550::SerialPort;
use x86_64::structures::paging::page_table::PageTableEntry;

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot.reset")]
unsafe extern "C" fn reset() {
    naked_asm!(
        ".code16",
        ".align 16",
        "jmp {code16}",
        ".code64",
        code16 = sym x86_boot::start16,
    )
}

const STACK_SIZE: usize = 512 * 1024; // 512 KiB
#[used]
#[unsafe(link_section = ".stack")]
static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

#[unsafe(export_name = "__rust_start")]
extern "C" fn main() {
    let mut serial = unsafe { SerialPort::new(0x3f8) };
    serial.init();
    for b in b"Hello, World!\nMy name is Joe!\n" {
        serial.send(*b);
    }

    static mut DATA: usize = 3;
    static mut BSS: usize = 0;
    unsafe {
        DATA += 1;
        BSS += 1;
        writeln!(serial, "{} {}", *&raw const DATA, *&raw const BSS).unwrap();
    }

    for i in 0..=4 {
        let entry = get_pte(i);
        writeln!(serial, "{:6x} - {:?}", entry.addr(), entry.flags()).unwrap();
    }

    loop {}
}

#[inline(never)]
fn get_pte(i: usize) -> &'static PageTableEntry {
    unsafe { &x86_boot::paging::PML2[0][i] }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
