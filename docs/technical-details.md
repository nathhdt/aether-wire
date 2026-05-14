# technical details

## design philosophy

aether-wire measures network performance under realistic conditions.

### modes overview

aether-wire offers two modes:

- **benchmark mode**: direct TCP/UDP performance measurement.
- **qualify mode**: automated end-to-end network qualification.

### benchmark mode principles

the project distinguishes between:
- **TCP realism**: preserve native TCP stack behavior
- **UDP determinism**: minimize scheduler and pacing artifacts to obtain stable latency and jitter measurements

#### TCP benchmark mode principles

- native OS TCP stack
- no artificial pacing
- no CPU affinity by default
- pseudo-random payloads to prevent middlebox optimization

#### UDP benchmark mode principles

- deterministic packet pacing
- dedicated per-stream worker threads
- platform-specific scheduling optimizations
- embedded packet telemetry
- pseudo-random payloads to prevent middlebox optimization

## TCP benchmark mode

### parallel streams model

with **n streams**:
- client opens **n TCP sockets**
- server runs **1 listener** + **n accepted connections**
- each stream independent

### test boundaries

client sends for exactly **t seconds** (e.g., 10s).

### throughput calculation

```
throughput = bytes × 8 / duration
```

includes TCP retransmissions (real behavior).

### payload generation

payloads generated using [ChaCha8](https://docs.rs/rand_chacha).

pseudo-random data prevents:
- network device caching
- compression by middleboxes
- unrealistic optimization

seed derivation:
```
stream_seed = session_seed ⊕ (stream_id × golden_ratio_constant)
```

ensures each stream has unique, reproducible payload.

## UDP benchmark mode

### parallel streams model

- client: **n UDP sockets** (ephemeral ports)
- server: **1 UDP socket** (listening port)
- streams identified by **stream_id** in header

### packet format (18-byte header)

```
[u16 stream_id][u64 seq_num][u64 timestamp_ns][payload]
```

### bandwidth control

```
bits_per_packet = (18 + payload_size) × 8
interval_ns = 1_000_000_000 / (bandwidth / bits_per_packet)
```

packets are paced using a spin-loop timer for sub-microsecond precision.

#### CPU affinity model

each UDP stream runs on its own dedicated thread, hardware permitting.

stream count is capped at:
- available logical CPUs
- hard limit: 32 streams

platform behavior:

| platform | stream limit | scheduling |
|---|---|---|
| Linux | logical CPUs | hard CPU affinity |
| Windows | logical CPUs | hard CPU affinity |
| macOS Intel | logical CPUs | QoS |
| macOS Apple Silicon | mostly performance cores | QoS |

on Apple Silicon Macs, threads use `QOS_CLASS_USER_INTERACTIVE` and the stream count is limited to the number of performance cores, encouraging the scheduler to run benchmark workloads primarily on P-cores for more deterministic measurements. See [optimize for Apple Silicon with performance and efficiency cores](https://developer.apple.com/news/?id=vk3m204o).

on Intel Macs, the same QoS flag is used, but the maximum stream count extends to all logical cores.

### jitter

interarrival jitter calculation (based on [RFC 3550 §6.4.1](https://www.rfc-editor.org/rfc/rfc3550.html#section-6.4.1)) using an exponentially weighted moving average:
```
D = |(recv_delta - send_delta)|
J = J + (D - J) / 16
```

no clock synchronization required (relative deltas).

### statistics

- **packet loss**: `(max_seq - min_seq + 1) - packets_received` (not implemented yet)
- **out-of-order**: arrival order ≠ sequential order (not implemented yet)
- **duplicates**: (not implemented yet)

### payload generation

same ChaCha8 approach as TCP:

```
stream_seed = session_seed ⊕ (stream_id × golden_ratio)
```

### reception timeout

4-second timeout after last packet to detect transmission end.

### UDP stream limits

to preserve deterministic packet pacing and avoid scheduler oversubscription, the maximum number of UDP streams (max. 32) depends on platform capabilities.

| platform | maximum UDP streams |
|---|---|
| Linux | logical CPUs |
| Windows | logical CPUs |
| macOS Intel | logical CPUs |
| macOS Apple Silicon | performance cores only |

## qualify mode (not implemented yet)

the qualification pipeline runs automatically. the client orchestrates all steps sequentially, using the same server endpoint.

```
┌──────────────────────────────────────────────────────────┐
│                    aw client qualify                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  step 1: TCP probe ─────────────────────────────→ Bref   │
│                                                          │
│  step 2: MTU sweep ─────────────────────────────→ MTU    │
│                                                          │
│  step 3: health check (UDP CBR) ────────────────→ stats  │
│                                                          │
│  step 4: stress test (UDP ramp) ────────────────→ stats  │
│                                                          │
│  step 5: report ────────────────────────────────→ stdout │
│                                                          │
│  step 6: diagnostic ────────────────────────────→ JSON   │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### step 1: TCP probe

establishes a reference throughput ($B_{ref}$) for the link.

**procedure**:
- single stream TCP test (15s) → throughput_single
- multi stream TCP test, 4 streams (15s) → throughput_multi
- $B_{ref}$ = max(throughput_single, throughput_multi)

$B_{ref}$ is used by all subsequent steps to calibrate their sending rates.
