.equ CURR_SLOT_HEIGHT, 0x0060
.equ MAX_SLOT_HEIGHT, 0x2898
.globl entrypoint
entrypoint:
  ldxdw r2, [r1 + CURR_SLOT_HEIGHT]
  ldxdw r1, [r1 + MAX_SLOT_HEIGHT]
  jle r2, r1, end
  mov64 r0, 1
  end:
    exit
