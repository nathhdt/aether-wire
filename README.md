# aether-wire

native Linux end-to-end network performance and benchmarking tool

> [!NOTE]
> under active development; interfaces may change before the first stable release.

documentation:
- [CLI reference](docs/cli-reference.md)

## runtime requirements

* Linux kernel **5.15+** required
* Linux kernel **6.2+** recommended (XDP metadata support)

you can verify system compatibility with:

```bash
aw check
```

## build requirements

optimized for maximum throughput and minimum latency.

build configuration:
- fat LTO
- codegen-units = 1
- clang + lld
- x86-64-v3 (AVX2, FMA3, BMI2)

> [!IMPORTANT]
> CPU restriction: running on architectures < x86-64-v3 will trigger `SIGILL`.

### prerequisites

requires a C toolchain with `clang` and the `lld` linker:

* **Ubuntu/Debian:** `sudo apt install clang lld build-essential`
* **Fedora/RHEL:** `sudo dnf install clang lld gcc g++`
* **Arch Linux:** `sudo pacman -S clang lld base-devel`

### build

```bash
cargo build --release
```
