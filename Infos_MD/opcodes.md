# 🔍 x86-64 Assembly Opcodes Reference Guide 🔍

<div align="center">
  
  *A comprehensive reference for x86-64 assembly language machine code*
  
  ![Assembly Language](https://img.shields.io/badge/Assembly-x86__64-blue)
  ![Usage](https://img.shields.io/badge/Usage-Educational-green)
  
</div>

---

## 🔄 Data Movement Instructions

<details open>
<summary><b>MOV - Move Data</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
mov          │  48 B8         │  Move data between registers or memory
rax          │  48 B8         │  Move to RAX register
rbx          │  48 89 C3      │  Move to RBX register
rcx          │  48 B9         │  Move to RCX register
rdx          │  48 89 D3      │  Move to RDX register
```

</div>
</details>

<details open>
<summary><b>LEA - Load Effective Address</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
lea          │  48 8D         │  Calculate address and store in register
rax          │  05            │  Store address in RAX
rbx          │  1C 03         │  Store address in RBX
rcx          │  1C 12         │  Store address in RCX
```

</div>
</details>

<details open>
<summary><b>PUSH - Push to Stack</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
push         │  50            │  Push value onto the stack
rax          │  50            │  Push RAX register value
rbx          │  53            │  Push RBX register value
rcx          │  51            │  Push RCX register value
rdx          │  52            │  Push RDX register value
```

</div>
</details>

<details open>
<summary><b>POP - Pop from Stack</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
pop          │  58            │  Pop value from the stack
rax          │  58            │  Pop to RAX register
rbx          │  5B            │  Pop to RBX register
rcx          │  59            │  Pop to RCX register
rdx          │  5A            │  Pop to RDX register
```

</div>
</details>

---

## 🧮 Arithmetic Instructions

<details open>
<summary><b>ADD - Addition</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
add          │  48 83 C0      │  Add source to destination
rax          │  48 83 C0      │  Add to RAX register
rbx          │  48 01 C3      │  Add to RBX register
rcx          │  48 83 C1      │  Add to RCX register
```

</div>
</details>

<details open>
<summary><b>SUB - Subtraction</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
sub          │  48 83 E8      │  Subtract source from destination
rax          │  48 83 E8      │  Subtract from RAX register
rbx          │  48 29 C3      │  Subtract from RBX register
rcx          │  48 83 E9      │  Subtract from RCX register
```

</div>
</details>

<details open>
<summary><b>MUL - Multiplication</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
mul          │  48 F7 E0      │  Multiply by accumulator (RAX)
rax          │  48 F7 E0      │  Multiply RAX by operand
rbx          │  48 F7 E3      │  Multiply RBX by operand
rcx          │  48 F7 E1      │  Multiply RCX by operand
```

</div>
</details>

<details open>
<summary><b>DIV - Division</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
div          │  48 F7 F0      │  Divide accumulator by source
rax          │  48 F7 F0      │  Divide using RAX
rbx          │  48 F7 F3      │  Divide using RBX
rcx          │  48 F7 F1      │  Divide using RCX
```

</div>
</details>

---

## ⤴️ Control Flow Instructions

<details open>
<summary><b>JMP - Unconditional Jump</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
jmp          │  E9            │  Jump to specified location
rax          │  FF E0         │  Jump to address in RAX
rbx          │  FF E3         │  Jump to address in RBX
```

</div>
</details>

<details open>
<summary><b>JE - Jump if Equal</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
je           │  74            │  Jump if ZF=1 (equal)
rax          │  FF E0         │  Jump to address in RAX if equal
rbx          │  FF E3         │  Jump to address in RBX if equal
```

</div>
</details>

<details open>
<summary><b>JNE - Jump if Not Equal</b></summary>
<div>

```asm
Instruction  │  Machine Code  │  Description
─────────────┼────────────────┼────────────────────
jne          │  75            │  Jump if ZF=0 (not equal)
rax          │  FF E0         │  Jump to address in RAX if not equal
rbx          │  FF E3         │  Jump to address in RBX if not equal
```

</div>
</details>

---

<div align="center">

## 📝 Notes

- This reference is intended for educational purposes in understanding x86-64 assembly language
- Machine codes are shown in hexadecimal format
- Most operations work on 64-bit registers in this reference
- Specific addressing modes and operands may require additional machine code bytes

</div>

---

<div align="center">
  <p>
    <i>Created for the NASimulator assembler project</i>
  </p>
</div> 