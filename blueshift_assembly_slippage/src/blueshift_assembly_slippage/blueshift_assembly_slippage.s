.equ TOKEN_ACCOUNT_BALANCE, 0x00A0
.equ MINIMUM_BALANACE, 0x2918
.globl entrypoint
entrypoint:
  ldxdw r3, [r1 + MINIMUM_BALANACE]
  ldxdw r4, [r1 + TOKEN_ACCOUNT_BALANCE]
  jge r4, r3, end
  lddw r1, e 
  lddw r2, 17
  call sol_log_
  mov64 r0, 1
  end:
    exit
.rodata
  e: .ascii "Slippage exceeded"
