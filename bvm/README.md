# bvm
the **brandon virtual machine** is a register based virtual machine inspired by LC-3/Lua
- there exists 2<sup>24</sup> memory locations, each address storing upto 32 bits
- there exists 29 general purpose registers
- a link register
- a remainder register
- a program counter register
- each instruction is 29 bits
    - opcode occupies 5 bits, or `ceil(log_2(29))`

hello world assembly
```
#LFH 0x2929 ; load file at address 0x2929
MRX R00     ; move data to R00
ARG STR     ; provide memory address of data to move
PNT         ; alias for CAL 0x9A - print R00 to terminal
HLT         ; alias for CAL 0x9D - halt program
STR: #STR "hello world\n" ; define hello world string
#END        ; end of file
```
```
00000000 00 29 29 02 00 00 00 0B 00 29 2D 12 00 00 9A 12
00000010 00 00 9D 00 68 00 65 00 6C 00 6C 00 6F 00 20 00
00000020 77 00 6F 00 72 00 6C 00 64 00 0A
```

a more complicated hello
```
#LFH 0x002929   ; load file at 0x002929
MRX R29         ; load into R28
ARG STR         ; the address of STR
MOV R00 R29     ; move data of R28 to R00
MEX             ; move data in memory
ARG 0x002938    ; to 0x002938
ARG 0x002939    ; from 0x002939
MMX R01         ; move data from R01
ARG 0x002939    ; to memory at 0x002939
PNT             ; print starting at stored address in R00
HLT             ; halt program
STR: #STR "hello world\n"
#END            ; end of file
```
```
00000000 00 29 29 02 00 00 1C 0B 00 29 33 00 80 00 1C 01
00000010 00 00 00 0B 00 29 38 0B 00 29 39 03 00 00 01 0B
00000020 00 29 39 12 00 00 9A 12 00 00 9D 00 68 00 65 00
00000030 6C 00 6C 00 6F 00 20 00 77 00 6F 00 72 00 6C 29
00000040 29 29 29 00 64 00 0A
```

## bvm instruction set
| opcode | hex | name | args | action |
| - | - | :--  | - | :-- |
| 00000 | 0x00 | MOV | DST, SRC | DST := SRC |
| 00001 | 0x01 | MEX | - | MEM[VAR1] := MEM[VAR2] |
| 00010 | 0x02 | MRX | DST | DST := VAR1 |
| 00011 | 0x03 | MMX | SRC | MEM[VAR1] := SRC
| 00100 | 0x04 | NIL | DST | DST := NIL |
| 00101 | 0x05 | LFX | DST | DST := MEM[VAR1] |
| 00110 | 0x06 | SWX | - | MEM[VAR1] := MEM[VAR2]<br> MEM[VAR2] := SWP |
| 00111 | 0x07 | JMP | DST, DAT | PC := DAT |
| 01000 | 0x08 | JSR | imm24 | LNK := PC<br>PC := imm24 |
| 00111 | 0x07 | RET | - | PC := LNK |
| 01001 | 0x09 | CEQ | CMP1, CMP2 | IF (CMP1 == CMP2), PC := PC + 1 |
| 01001 | 0x09 | CEL | CMP1, CMP2 | IF (CMP1 <= CMP2), PC := PC + 1 |
| 01001 | 0x09 | CEG | CMP1, CMP2 | IF (CMP1 >= CMP2), PC := PC + 1 |
| 01001 | 0x09 | CLT | CMP1, CMP2 | IF (CMP1 < CMP2), PC := PC + 1 |
| 01001 | 0x09 | CGT | CMP1, CMP2 | IF (CMP1 > CMP2), PC := PC + 1 |
| 01010 | 0x0A | CEZ | CMP | IF (CMP == 0), PC := PC + 1 |
| 01010 | 0x0A | CNZ | CMP | IF (CMP <= 0), PC := PC + 1 |
| 01010 | 0x0A | CPZ | CMP | IF (CMP >= 0), PC := PC + 1 |
| 01010 | 0x0A | CLZ | CMP | IF (CMP < 0), PC := PC + 1 |
| 01010 | 0x0A | CGZ | CMP | IF (CMP > 0), PC := PC + 1 |
| 01011 | 0x0B | ARG | imm24 | - |
| 01100 | 0x0C | ADD | DST, A, B | DST := A + B |
| 01101 | 0x0D | SUB | DST, A, B | DST := A - B |
| 01110 | 0x0E | MUL | DST, A, B | DST := A * B |
| 01111 | 0x0F | DIV | DST, A, B | DST := A // B |
| 10000 | 0x10 | AND | DST, A, B | DST := A & B |
| 10001 | 0x11 | NOT | DST, A | DST := NOT(A)
| 10010 | 0x12 | CAL | VEC | LNK := PC<br>PC=VEC |
| 10011 | 0x13 | JPA | imm24 | PC := imm24 |
| 10100 | 0x14 | FLX | - | MEM[VAR2] <- read(MEM[VAR1])
| 10101 | 0x15 | ILX | - | f := read(MEM[VAR1]) <br> MEM[f[0:3]] <- f[3:]