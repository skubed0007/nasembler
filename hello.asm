; Simple Hello World example

section .text
  global _start

_start:
  ; Exit with code 42
  mov rax, 60       ; syscall: exit
  mov rdi, 42       ; exit code 42
  syscall 