# aether-wire

native cross-platform E2E network performance and benchmarking tool

## overview

**aether-wire** is a lightweight, native cross-platform tool built in Rust for measuring end-to-end (E2E) network performance. it provides **low-level benchmarking capabilities** for TCP (currently IPv4), targeting developers and network engineers who need precise control over network behavior.

this project is under development.

## example usage - TCP test

### start a TCP server

```bash
aw server ipv4 tcp -p 9000
```

### run a simple TCP client benchmark

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -t 10s -n 4
```

the client will open multiple (`n = 4`) concurrent TCP connections and send data for a fixed duration (`t = 10s`) to measure throughput under load.

## documentation

- [CLI reference](./docs/cli-reference.md)
- [roadmap](./docs/roadmap.md)
- [technical details](./docs/technical-details.md)