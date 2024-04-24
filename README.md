# This is title of my Transactify ;)

# Usage

## Requirements
rust compiler. See [rust-lang.org](https://www.rust-lang.org/tools/install) for installation instructions.
Latest stable toolchain should be good to go.

## Env
If explicitly declaring log level is not desirable - set env variable `RUST_LOG` to at least `info` level to see program's outputs into `stdout` and `stderr` if any.

## Running
```bash
RUST_LOG=info cargo run --release -- path/to/input.csv path/to/output.csv
```
or

```bash
cargo build --release && RUST_LOG=info ./targe/release/transactify path/to/input.csv path/to/output.csv
```

## Testing
```bash
cargo test
```