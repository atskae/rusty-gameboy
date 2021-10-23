# Rusty-GameBoy
Another attempt at developing a GameBoy emulator.

## Usage

1. To build the emulator, run:
```
cargo build
```

2. To build and run the emulator, run:
```
cargo run
```

Once built, the executable can also be run directly:
```
./target/debug/rusty-gameboy
```

### Logs
To view logs, prepend `RUST_LOG=` to `cargo run` with the desired logging level:
```
RUST_LOG=debug cargo run
```
This will log all messages up to the `debug` level (available [logging levels](https://docs.rs/log/0.4.0/log/enum.Level.html)).

## Tests

To run the unit tests, go to the repository root, then run:
```
cargo test
```

To run the tests with loggging, prepend with `RUST_LOG=` and add the `--nocapture` flag:
```
RUST_LOG=debug cargo test -- --nocapture
```


## Pre-commit Hooks
This repository uses [pre-commit](https://pre-commit.com/) to apply code formatting and checking.

To install the hooks, install the `pre-commit` Python package, then install the custom hooks for this repository:
```
pip install pre-commit
pre-commit install
```

To run the hooks on modified files, run:
```
pre-commit run
```

To run the hooks on the entire repository, run:
```
pre-commit run --all-files
```
