# aether-wire

native cross-platform E2E network performance and benchmarking tool

## overview

**aether-wire** is a lightweight, native cross-platform tool built in Rust for measuring end-to-end (E2E) network performance. it provides two modes: a **raw TCP/UDP benchmark** for quick throughput measurement, and a **full link qualification** (not yet implemented) pipeline that automatically profiles a network path (throughput, MTU, jitter, bufferbloat, packet loss).

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
