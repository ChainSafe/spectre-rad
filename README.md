# Non-Halo2 implementation of Spectre for use with LLVM-compatible ZKVMs (Risc0, SP1)

Work in progress:

- Host function for sha2 in Risc0 / generic hashing
- AlignedLayer integration of the CommitteeUpdateCircuit for Risc0

## Naming convention

### Naming convention for crates

`*-iso`: Raw Rust implementation of a Spectre component in Isolation. Example: `committee-iso` for the `CommitteeUpdateCircuit` logic.

`circuit/rz-*`: Risc0 circuit implementation of an `iso` component.

`circuit/sp1-*`: SP1 circuit implementation of an `iso` component.


`prover`: A special crate that generates proofs using either of the `circuits`. This crate will be extended to support verification on `AlignedLayer`.

## Generate a proof for the Committee Circuit in Risc0

Prerequisites:

- Rust installation
- Risc0 toolchain
- Risc0 client `1.0.1`

```bash
cargo test test_risc0 -- --nocapture
```

Make sure to specify the path to `rotation_512.json` as an environment variable when running the circuit.

Example:

```bash
// in .bashrc
export COMMITTEE_UPDATE_TEST_PATH="/Users/USERNAME/Desktop/spectre-rad/data/rotation_512.json"
```


Example output:

```rust
   Compiling host v0.1.0 (/Users/chef/Desktop/spectre-rad/circuits/rz-committee/host)
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.95s
     Running `target/debug/host`
Verified Committee Root: [25, 122, 75, 125, 192, 12, 117, 238, 92, 109, 3, 192, 224, 63, 84, 28, 196, 131, 90, 32, 180, 39, 160, 7, 188, 177, 162, 100, 181, 205, 38, 142]
```

## Generate a Risc0 proof for the Committee circuit and verify it on AlignedLayer

Prerequisite:
- Holesky funded account & keystore (see [AlignedLayer docs](https://docs.alignedlayer.com/))
- Keystore should be placed in `./aligned/`

Generate and submit the proof:

```bash
cargo test test_committee_submit_aligned
```
-> Enter password for keystore


Example output:

```rust
successes:

---- test_risc0::test_committee_submit_aligned stdout ----
Proof submitted and verified successfully on batch ebe6ea81087c1f4063f0a1d3b632e64be6925d8903fd1acacfede0241427e459


successes:
    test_risc0::test_committee_submit_aligned

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 687.50s
```