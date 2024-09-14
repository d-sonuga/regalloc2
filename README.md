## Info

This branch was used to measure the number of edits per instruction
to determine a reasonable guess to initialize the edits vector.

The measurements are in `edit-measure`. In `edit-measure`, each row represents
info recorded after a function was processed. `MaxOperandLen` is the number 
of operands in the instruction with the largest operand count. 
`NumInsts` is the number of instructions in the function.
`EditsLenGuess` is the number of edits that were guessed with the
initial `2 * max_operand_len * num_insts` guess.
`ActualEditsLen` is the number of edits that were actually added while processing
the function.

Some stats are computed in `edits_stats.rs`.

## regalloc2: another register allocator

This is a register allocator that started life as, and is about 50%
still, a port of IonMonkey's backtracking register allocator to
Rust. In many regards, it has been generalized, optimized, and
improved since the initial port.

In addition, it contains substantial amounts of testing infrastructure
(fuzzing harnesses and checkers) that does not exist in the original
IonMonkey allocator.

See the [design overview](doc/DESIGN.md) for (much!) more detail on
how the allocator works.

## License

This crate is licensed under the Apache 2.0 License with LLVM
Exception. This license text can be found in the file `LICENSE`.

Parts of the code are derived from regalloc.rs: in particular,
`src/checker.rs` and `src/domtree.rs`. This crate has the same license
as regalloc.rs, so the license on these files does not differ.
