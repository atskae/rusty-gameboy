# Rusty-GameBoy
Another attempt at developing a GameBoy emulator.

## Usage

1. To build the emulator, run:
```
cargo build
```

2. To run the emulator, run:
```
./target/debug/rusty-gameboy
```

Alternatively, to build *and* run the emulator, run:
```
cargo build
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
