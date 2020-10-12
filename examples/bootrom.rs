#![no_std]
#![no_main]
#![feature(asm, naked_functions)]
#![feature(const_panic, const_ptr_offset, const_mut_refs)]

use core::{fmt::Write, panic::PanicInfo};
use uart_16550::SerialPort;

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

const STACK_SIZE: usize = 512 * 1024; // 512 KiB
#[used]
#[link_section = ".stack"]
static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

#[export_name = "__rust_start"]
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
        writeln!(serial, "{} {}", DATA, BSS).unwrap();
    }

    for i in 0..=4 {
        let entry = unsafe { &x86_boot::paging::PML2[0][i] };
        writeln!(serial, "{:6x} - {:?}", entry.addr(), entry.flags()).unwrap();
    }

    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
