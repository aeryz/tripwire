
# TripWire

`TripWire` is a GDB-like debugging tool targeting JIT-compiled WASM.

It's in the PoC phase right now where I experiment with discovering the memory addresses in JIT-compiled WASM, injecting traps and tracing with ptrace.

Right now, I only target JIT-compiled WASM with System V calling convention but working on a design to make `TripWire` generic over the targets and the calling conventions.
