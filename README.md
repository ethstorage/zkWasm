# Understanding zkWasm's Circuit layout
In zkWasm, the arrangement of each instruction's related states occupies **4 rows** within the circuit table, defined as `instruction_rows`. Constraints are established within these four rows and between successive instruction_rows. See the code [here](./crates/zkwasm/src/circuits/etable/mod.rs#L219).

| U32_Advice_Col | U32_Advice_Col | U64_Advice_Col | Bit_Advice_Col | …         | Bit_Advice_Col | U8_Advice_Col | … |
|----------------|----------------|----------------|----------------|-----------|----------------|---------------|---|
| curr_eid       | …              | …              |    ...         | GlobalGet | Select         |   ...         |   |
|   frame_id   | …              | …              | LocalGet       | GlobalSet | Return         |   ...         |   |
| …              | …              | …              | LocalSet       | Const     | Bin            |   ...         |   |
| …              | …              | …              | LocalTee       | Drop      | Unary          |   ...         |   |
| next_eid       | …              | …              |    ...         | ... | 
| next_frame_id  | …              | …              |    ...         | ... | 

While zkWasm employs Halo2's API, it defines a custom circuit layouter with three main differences from Halo2:

- Column Type Bounds: Each column is bound by a range constraint. This includes BitColumn, CommonRangeColumn (ranging from 0 to 1 << zkwasm_k() - 1), U16Column, U32Column, U64Column, JTableLookup, MTableLookup, etc. For ease of reference later, we collectively refer to all these columns as `TypeColumn`.

- Gate Constraint Building: Constructing gate constraints involves querying related cells and creating gates. To query a cell, an `alloc` function is utilized to obtain an unused cell of a TypeColumn within the current instruction_rows (akin to a conceptual region in Halo2). Moreover, after allocating one cell, the subsequent cell of the same type will be assigned to the next cell in that column. If allocation extends beyond the fourth row, allocation restarts from the first row of the subsequent column of the same type (See [code](./crates/zkwasm/src/circuits/etable/allocator.rs#L335)). The code utilizes a BTreeMap named `free_cells` to record the allocation of each type of cell up to which column's first row. When creating gates for an instruction, a `constraint builder` is encapsulated (e.g., integer addition [here](./crates/zkwasm/src/circuits/etable/op_configure/op_bin.rs#L168)) and then [`finalized`](./crates/zkwasm/src/circuits/etable/constraint_builder.rs#L61) to call Halo2's create gate API.

- Lookup: During a lookup, zkWasm encodes cells to be looked up into an auxiliary cell and constrains the encoding equation. Then, it looks up the auxiliary cell in another table's encoded cell. An example is the `op_br` instruction's `encode_memory_table_entry`.

For instance, let's consider `etable`. A `log_cell` macro has been integrated into the etable code to log the location of cells by their col and row.

After running the following simple test case, the cell's Column and Rotation will be printed.
```
cargo test test_uniform_verifier -- --show-output
```

# Instruction Circuits
## `i32.Add`` Instruction
`i32.Add` instruction is one typical [`binop`](https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-binop-mathit-binop) in wasm, it will execute the following:
1. pop value `lhs` from the stack 
2. pop value `rhs` from the stack 
3. compute `res = lhs + rhs`
4. push value `res` to the stack

Based on the op code definition, the constraints are defined as:
$$
lhs + rhs - res + isoverflow * 1<<32 = 0 \\
iaddr.curr + 1 - iaddr.next = 0 \\
sp.curr + 1 - sp.next = 0
Plookup(MemoryTable, (StackType, read, sp.curr + 1, rhs)) = 0
Plookup(MemoryTable, (StackType, read, sp.curr + 2, lhs)) = 0
Plookup(MemoryTable, (StackType, write, sp.curr + 1, res)) = 0
$$

### Wasm 