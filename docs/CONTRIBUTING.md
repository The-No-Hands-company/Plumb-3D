# Contributing to DCC Suite

Thank you for your interest in contributing to the pure-Rust DCC Suite! This guide outlines our architectural standards, code style, testing expectations, and contribution process.

---

## 1. Code Standards & Architectural Principles

### 100% Safe Pure Rust (Zero Foreign C/C++ FFI)
All code submitted to this repository must be pure safe Rust. We strictly disallow foreign C/C++ FFI dependencies, wrapper bindings around native C libraries, or unsafe system shims. Everything must build cleanly via Cargo across Linux, macOS, and Windows.

### The ~600-700 Line Limit Rule
To maintain clean Separation of Concerns (SoC) and prevent the re-emergence of monolithic "God files":
- **No operational source file may exceed ~650-700 lines**.
- If a module approaches 600 lines, decompose it into focused domain submodules organized within a folder module (e.g. `foo/mod.rs`, `foo/bar.rs`).
- High-level coordinator functions must remain thin dispatchers.

### Zero Compiler Warnings Policy
All code must compile with **zero warnings**:
```bash
cargo check --workspace --all-targets
```
Code that produces unused variable, dead code, or deprecated API warnings will not be merged.

### Deterministic Topological Invariants
Any function that mutates mesh topology (`geo-kernel`) must preserve 2-manifold half-edge integrity:
- Every half-edge must have a valid twin pointer.
- Vertex outgoing half-edge pointers must remain valid.
- Face boundary loops must form closed, counter-clockwise cycles.
- Always include automated unit tests verifying topology and Euler characteristics.

---

## 2. Development Setup

1. **Install Rust**:
   Ensure you have the latest stable Rust toolchain:
   ```bash
   rustup update stable
   ```

2. **Clone the Repository**:
   ```bash
   git clone https://github.com/The-No-hands-Company/3dmodeling.git
   cd 3dmodeling
   ```

3. **Run the Test Suite**:
   ```bash
   cargo test --workspace
   ```

4. **Run the Application**:
   ```bash
   cargo run -p app --release
   ```

---

## 3. Pull Request Checklist

Before submitting a pull request, ensure:
- [ ] `cargo check --workspace --all-targets` reports **0 errors** and **0 warnings**.
- [ ] `cargo test --workspace` passes all tests.
- [ ] Any new feature, modifier, or operator includes corresponding unit tests in its crate.
- [ ] No file exceeds ~650-700 lines.
- [ ] Any new hotkey or user-facing feature is documented in `docs/KEYBOARD_SHORTCUTS.md` or `docs/USER_GUIDE.md` and added to `crates/app/src/ui/help_dialog.rs`.
- [ ] In-code doc comments (`///`) are provided for all newly added public items.
