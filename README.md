# HEX FILE FLASHER

## Build

### Prerequisites

1. Install the [rustup toolchain](https://rustup.rs/)
2. Compile the code with `cargo build --release`

## Test

### Master

```bash
./target/debug/hex-file-flasher-rs /dev/ttyUSB0 green master --factory-number 142427-0307
```

### Relay

```bash
./target/build/hex-file-flasher-rs /dev/ttyUSB0 green rel-mk1.5 -s /dev/ttyUSB1 --factory-number 142427-0307 -f 50
```

## Example use cases
