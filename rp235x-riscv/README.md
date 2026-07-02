# smoltcp-checksum-bench-rp2350

Run:
```bash
cargo run-riscv --release
```

Connect minicom:
```bash
minicom -D /dev/ttyACM0 -b 115200
```

typing should then cause the times to be printed.

# Attribution

The project layout was taken from [rp-hal](https://github.com/rp-rs/rp-hal).
