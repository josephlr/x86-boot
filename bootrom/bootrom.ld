ENTRY(reset)

MEMORY
{
  rom : ORIGIN = 0xFFFF0000 + SIZEOF_HEADERS, LENGTH = 64K
  ram : ORIGIN = 1M, LENGTH = 5M
}

PHDRS
{
  rodata PT_LOAD FLAGS(4); /* PF_R */
  data   PT_LOAD FLAGS(6); /* PF_R|PF_W */
  code   PT_LOAD FLAGS(5); /* PF_R|PF_X */
}

SECTIONS
{
  .rodata : {
    *(.rodata .rodata.*)
  } >rom :rodata
  .rodata : {
    *(.data.rel.ro .data.rel.ro.*)
  } >rom
  .got : {
    *(.got)
  } > rom
  .got.plt : {
    *(.got.plt)
  } > rom
  .gdt : {
    *(.boot.gdt)
  } > rom
  .pad : {
    . = ALIGN(4K);
  } > rom
  
  __init_data = .;
  data : {
    *(.data .data.*)
  } >ram AT>rom :data
  bss : {
    *(.bss .bss.*)
  } >ram :data
  
  .text : {
    *(.text .text.*)
    *(.boot.text64)
  } >rom :code

  .text32 : { *(.boot.text32)  } >rom
  .text16 : { *(.boot.text16)  } >rom

  .reset 0xFFFFFFF0 : {
    *(.boot.reset)
    . = 0x100000000;
  } >rom

  .stack 2M (NOLOAD) : ALIGN(4K) {
    KEEP(*(.stack));
    __rust_stack_top = .;
  } >ram :NONE
  
  .page_tables 4M (NOLOAD) : ALIGN(4K) {
    *(.boot.page_tables)
  } >ram :NONE
}
