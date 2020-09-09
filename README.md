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

## Generate report

### Prerequisites

1. Install [Python 3.8](https://www.python.org/) on your system
2. Install [Pipenv](https://pipenv.pypa.io/en/latest/) on your system

### Use case

```bash
pipenv install
pipenv shell
python3 report.py --start-date 2020-07-08_12:00 --end-date 2020-09-09_12:00
```
