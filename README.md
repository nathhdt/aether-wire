# aether-wire

native cross-platform E2E network performance and benchmarking tool

## overview

**aether-wire** is a lightweight, native cross-platform tool built in Rust for measuring end-to-end (E2E) network performance. it provides two modes: a **raw TCP benchmark** for quick throughput measurement, and a **full link qualification** pipeline that automatically profiles a network path (throughput, MTU, jitter, bufferbloat, packet loss).

this project is under development.

## quick start

### run a server

```bash
aw server -p 9000
```

### run a TCP benchmark

```bash
aw client benchmark -s 192.168.1.11 -p 9000 -t 10s -n 4
```

### run a full link qualification

```bash
aw client qualify -s 192.168.1.11 -p 9000
```

## modes

### benchmark

raw TCP throughput measurement. can open parallel streams (`-n 4`), sends data for a fixed duration (`-t 10s`), reports throughput. no optimization, measures the wire as-is.

### qualify

automated multi-step link qualification pipeline:

- **baseline performance**: establishment of reference throughput via single and multi-stream TCP probes.
- **physical footprint**: path MTU discovery and encapsulation signatures (e.g., IPsec, GRE, ...)
- **stability analysis**: jitter and ROWD measurement
- **saturation limits**: ramp-up stress testing to locate packet loss thresholds and bufferbloat
- **automated diagnostics**: human-readable performance profiling and JSON exports

## documentation

- [CLI reference](./docs/cli-reference.md)
- [roadmap](./docs/roadmap.md)
- [technical details](./docs/technical-details.md)
