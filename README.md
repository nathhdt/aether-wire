# aether-wire

native cross-platform E2E network performance and benchmarking tool

## overview

**aether-wire** is a lightweight native cross-platform tool written in Rust for end-to-end (E2E) network performance measurement.

it provides:
- **realistic TCP benchmarking** using the native OS TCP stack
- **deterministic UDP benchmarking** with precise packet pacing and embedded telemetry
- an upcoming **link qualification pipeline** for automated network profiling (throughput, MTU, jitter, packet loss, bufferbloat, stability)

this project is under development.

## quick start

### run a server

```bash
aw server -p 9000
```

### run a TCP benchmark

```bash
aw client benchmark tcp -s 192.168.1.11 -p 9000 -t 10s -n 4
```

## platform support

**tested and working:**
- macOS 12.0+
- Windows 10+

## documentation

- [CLI reference](./docs/cli-reference.md)
- [roadmap](./docs/roadmap.md)
- [technical details](./docs/technical-details.md)
