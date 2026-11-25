# Contributing to Plumb 3D

Thank you for your interest in contributing to Plumb 3D. We are dedicated to building a high-performance, precision 3D DCC suite in 100% Safe, Pure Rust.

## Development Workflow

1. **Fork and Clone**: Ensure you have the latest Rust stable toolchain (`rustup update stable`).
2. **Deterministic Building**: Run `./scripts/build.sh` or `cargo build --workspace`.
3. **Running Tests**: Run `cargo test --workspace` (all tests must pass).
4. **Zero-Warning Gate**: Run `cargo clippy --workspace -- -D warnings`. Code with compiler warnings will not be merged.
5. **Code Formatting**: Ensure `cargo fmt --all -- --check` passes.

## Architectural Constitution

All contributions adhere to the **Zajfan Engineering Standard**:
- **Pure Safe Rust**: Zero raw pointers for ownership, zero `unsafe` blocks.
- **Topological Integrity**: Any geometric operator must preserve 2-manifold half-edge invariants.
- **Flat Flow**: Use early return guard clauses (maximum 2 levels of indentation).
- **Domain Modeling**: Illegal states must be unrepresentable via algebraic types (`enum`) rather than boolean flags.
