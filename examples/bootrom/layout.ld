ENTRY(reset)

SECTIONS
{
  . = 1M;
  rust_stack_start = .;
  .stack (NOLOAD) : ALIGN(4K) { . += 1M; } :NONE
  rust_stack_top = .;

  /* Map everything into ROM */
  . = 0xFFFF0000;
  . += SIZEOF_HEADERS;
  .rodata      : { *(.rodata .rodata.*) } :rodata
  .got         : { *(.got)              }
  .got.plt     : { *(.got.plt)          }
  .data.rel.ro : { *(.data.rel.ro .data.rel.ro.*) }
  
  .text   : { *(.text .text.*)     } :code
  .plt     : { *(.plt)             }
  .plt.got : { *(.plt.got  )       }
  
  .data   : { *(.data .data.*)     } :data
  .bss    : { *(.bss .bss.*)       }

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
    early  PT_LOAD FLAGS(5); /* PF_R|PF_X */
}
