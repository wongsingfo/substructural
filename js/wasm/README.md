# Flexible Linear Type System

![Build](https://github.com/wongsingfo/substructural/workflows/release/badge.svg)

## Build Commands

Build:

```
cargo build
```

Build WASM (the output files are under `/pkg`):

```
cargo install wasm-pack
wasm-pack build --target web
```

Test all:

```
cargo test
```

Test a specific testcase and checkout the standard output, say  `test_conditional`:

```
cargo test test_conditional -- --nocapture
```



