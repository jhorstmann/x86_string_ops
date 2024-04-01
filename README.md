# X86/x86_64 String operations

This crate exposes the `REP MOVS/STOS/CMPS/SCAS` strings functions supported by x86 and x86_64
and implements safe wrappers for them.

These implementations are mostly useful for experimenting and benchmarking. In most circumstances,
the implementations in the standard library are likely faster, or are using the same instructions.
The main benefit then is that these implementations can be inlined, while llvm would generate a
call to `memcpy` or `memset` for example.

**NOTE:** The codegen option `-Cllvm-args=-x86-use-fsrm-for-memcpy` is also worth investigating,
but will make llvm replace all memory copies with `REP MOVS` instructions.