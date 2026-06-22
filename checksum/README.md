# checksum

Variety of implementations of internet checksum

## Criterion benchmarks

```bash
RUSTFLAGS="-C target-cpu=native" cargo bench
```

### cross-compiling

I had luck using [cross](https://github.com/cross-rs/cross), e.g. for my raspberry pi zero:

```bash
# commit when I installed: 29d00c7803f221f1b3f35e561b03792368fb8339
cargo install cross --git https://github.com/cross-rs/cross
RUSTFLAGS="-C target-cpu=arm1176jzf-s" cross bench --no-run --target arm-unknown-linux-musleabihf
```

I can then `scp` the built benchmark binary to the pi and run there.

I need to check if I should pass an `mcpu` LLVM arg.
