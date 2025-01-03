# Useful Commands

## Run

```bash
cargo run
```

## Generate Documentation

```bash
cargo doc --no-deps --open
```

## Emulation

<http://localhost:1234/>

```cmd
cargo build
cd tests
renode --console run_clock.resc
s
```
