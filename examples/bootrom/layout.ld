ENTRY(reset)

SECTIONS
{
  . = 1M;
  .stack (NOLOAD) : ALIGN(4K) { . += 1M; } :NONE
  rust_stack_top = .;

  /* Map everything into ROM */
  . = 0xFFFF0000;
  . += SIZEOF_HEADERS;
  .rodata      : {
    *(.rodata .rodata.*)
    *(.boot.l2) *(.boot.l3) *(.boot.l4)
  } :rodata
  .got         : { *(.got)              }
  .got.plt     : { *(.got.plt)          }
  .data.rel.ro : { *(.data.rel.ro .data.rel.ro.*) }
  
  .text   : { *(.text .text.*)     } :code
  .plt     : { *(.plt)             }
  .plt.got : { *(.plt.got  )       }
  
  rom_data_start = ALIGN(4K);
  data_start = rust_stack_top + 1M;
  . = data_start;
  .data : AT(rom_data_start) {
    *(.data .data.*)
    *(.bss .bss.*)
  } :data
  data_size = . - data_start;
  . = rom_data_start + data_size;

  .text64 : { *(.boot.text64)      } :early
  .text32 : { *(.boot.text32)      }
  .text16 : { *(.boot.text16)      }
  . = 0xFFFFFFF0;
  .reset  : {
    *(.boot.reset)
    . = 0x100000000;
  }

  /DISCARD/ : {
    /* Sometimes emitted event for panic = "abort" */
    *(.eh_frame)
    *(.eh_frame_hdr)
    /* Inserted of lld, unnecessary */
    *(.comment)
  }
}

PHDRS
{
    rodata PT_LOAD FLAGS(4); /* PF_R */
    code   PT_LOAD FLAGS(5); /* PF_R|PF_X */
    data   PT_LOAD FLAGS(6); /* PF_R|PF_W */
    pt     PT_LOAD;
    early  PT_LOAD FLAGS(5); /* PF_R|PF_X */
}
