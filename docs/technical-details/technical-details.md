# technical details

## design philosophy

aether-wire measures network performance under realistic conditions.

## modes overview

aether-wire offers two modes:

- **benchmark mode**: direct TCP/UDP performance measurement
- **qualify mode**: automated end-to-end network qualification

## benchmark mode principles

the project distinguishes between:
- **TCP realism**: preserve native TCP stack behavior
- **UDP determinism**: minimize scheduler and pacing artifacts to obtain stable latency and jitter measurements

### TCP benchmark mode principles

- native OS TCP stack
- no artificial pacing
- no CPU affinity by default
- pseudo-random payloads to prevent middlebox optimization

see [documentation](./docs/technical-details/tcp-benchmark-mode.md).

### UDP benchmark mode principles

- deterministic packet pacing
- dedicated per-stream worker threads
- platform-specific scheduling optimizations
- embedded packet telemetry
- pseudo-random payloads to prevent middlebox optimization

see [documentation](./docs/technical-details/udp-benchmark-mode.md).
