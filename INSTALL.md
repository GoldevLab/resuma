# Installation

Resuma needs the Rust toolchain. Node only matters if you want to rebuild
the JavaScript runtime (a fallback bundle ships in the repo).

## Windows (PowerShell)

```powershell
# Install rustup (the official Rust installer)
winget install --id Rustlang.Rustup -e
# Or via rustup directly:
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe -y
```

After installing, restart your terminal and verify:

```powershell
rustc --version   # should print 1.74+
cargo --version
```

## macOS / Linux

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build the workspace

```sh
cd Resuma
cargo build
```

Cargo will pull every transitive dependency (`axum`, `serde`, `tokio`, `syn`, …) and compile every crate in the workspace.

## Run an example

```sh
cargo run -p example-counter   # http://127.0.0.1:3000
cargo run -p example-todo
```

## Optional: rebuild the JS runtime

The repo includes a hand-written fallback at `crates/resuma-server/assets/runtime.js`. To rebuild the optimized bundle from the TypeScript source:

```sh
cd runtime
npm install
npm run build
# Bundle ends up at runtime/dist/runtime.js — copy it back over the fallback
cp dist/runtime.js ../crates/resuma-server/assets/runtime.js
```

## Install the CLI

```sh
cargo install --path crates/resuma-cli
resuma --help
```

## Troubleshooting

* **`error: linker not found`** on Windows — install MSVC build tools via `Visual Studio Installer` (Workloads → "Desktop development with C++").
* **`cargo` not on PATH** after install — close & reopen your terminal, or `. "$HOME/.cargo/env"`.
* **`syn` version mismatch** — Resuma requires `syn 2.x`. If a downstream crate forces 1.x, `cargo` will resolve them in parallel; no action needed.
