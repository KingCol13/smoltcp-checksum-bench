# smoltcp-checksum-bench-rp2350

Run RISC-V:
```bash
cargo run-riscv --release
```

Connect minicom:
```bash
minicom -D /dev/ttyACM0 -b 115200
```

Typing "r" will reboot the pi, allowing it to be re-flashed.

Typing any other character causes the times to be printed.

Run ARM:
```bash
cargo run-arm --release
```

# Attribution

The project layout was taken from [rp-hal](https://github.com/rp-rs/rp-hal).
