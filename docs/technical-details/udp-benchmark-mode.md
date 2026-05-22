# UDP benchmark mode

## parallel streams model

- client: **n UDP sockets** (ephemeral ports)
- server: **1 UDP socket** (listening port)
- streams identified by **stream_id** in header

## UDP stream limits

to preserve deterministic packet pacing and avoid scheduler oversubscription, the maximum number of UDP streams (max. 32) depends on platform capabilities.

| platform | maximum UDP streams |
|---|---|
| Linux | logical CPUs |
| Windows | logical CPUs |
| macOS Intel | logical CPUs |
| macOS Apple Silicon | performance cores |

## packet format (18-byte header)

```
[u16 stream_id][u64 seq_num][u64 timestamp_ns][payload]
```

## bandwidth control

```
bits_per_packet = (18 + payload_size) × 8
interval_ns = 1_000_000_000 / (bandwidth / bits_per_packet)
```

packets are paced using a spin-loop timer for sub-microsecond precision.

if the sender falls behind (OS jitter), it resets to `now` instead of bursting. this prevents spikes and caps IAT deviation.

### CPU affinity model

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

## payload generation

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

## reception timeout

4-second timeout after last packet to detect transmission end.

## statistics

- **packet loss**: `(max_seq - min_seq + 1) - packets_received` (not implemented yet)
- **out-of-order**: arrival order ≠ sequential order (not implemented yet)
- **duplicates**: (not implemented yet)

## jitter

interarrival jitter calculation (based on [RFC 3550](https://www.rfc-editor.org/rfc/rfc3550.html#section-6.4.1)) using an exponentially weighted moving average:
```
D = |(recv_delta - send_delta)|
J = J + (D - J) / 16
```

no clock synchronization required (relative deltas).

## kernel-level receive timestamping

to reduce user-space scheduling jitter, packet arrival timestamps are captured directly from the kernel network stack when supported.

| platform | implementation | precision | source |
|---|---|---|---|
| Linux | `SO_TIMESTAMPNS` + `recvmsg` | nanoseconds | kernel |
| macOS | `SO_TIMESTAMP` + `recvmsg` | microseconds | kernel |
| Windows | `recv_from` fallback | scheduler-dependent | user-space |

unsupported platforms fall back to user-space receive timing.
