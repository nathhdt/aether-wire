# technical details

## design philosophy

aether-wire measures network performance under realistic conditions.

principles:
- **no optimization**: emulate standard client/server behavior
- **TCP as-is**: let TCP handle flow control, congestion, retransmissions
- **telemetry embedded**: timestamps for precise measurements
- **benchmark-quality first**: all heavy calculations done after benchmark

## modes overview

aether-wire offers two modes:

**benchmark**: raw TCP throughput measurement.

**qualify**: automated multi-step link qualification pipeline. profiles a network path end-to-end: throughput, MTU, jitter, stability, bufferbloat, packet loss, with automated diagnostics.

## benchmark mode (TCP)

### parallel streams model

with **n streams**:
- client opens **n TCP sockets**
- server runs **1 listener** + **n accepted connections**
- each stream independent

### test boundaries

client sends for exactly **T seconds** (e.g., 10s).

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

### directional modes

#### unidirectional (default)

```
client ─── [n streams] ──> server
```

measures: upload capacity.

#### reverse

```
client <── [n streams] ─── server
```

measures: download capacity.

#### both (sequential)

```
phase 1: client ─── [n streams] ──> server (10s)
phase 2: client <── [n streams] ─── server (10s)

total: 2n connections, 20s
```

detects asymmetry without interference. separate measurements per direction.

#### bidirectional (simultaneous)

```
client ──── [n streams] (upload) ───> server
client <── [n streams] (download) ─── server

total: 2n connections, 10s
```

reveals:
- bidirectional saturation
- mutual congestion impact
- realistic interactive workload


## qualify mode (pipeline)

the qualification pipeline runs automatically. the client orchestrates all steps sequentially, using the same server endpoint.

```
┌──────────────────────────────────────────────────────────┐
│                    aw client qualify                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  step 1: TCP probe ─────────────────────────────→ Vref   │
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

### step 1 — TCP probe

establishes a reference throughput ($V_{ref}$) for the link.

**procedure**:
- single stream TCP test (15s) → throughput_single
- multi stream TCP test, 4 streams (15s) → throughput_multi
- $V_{ref}$ = max(throughput_single, throughput_multi)

$V_{ref}$ is used by all subsequent steps to calibrate their sending rates.

**reuses**: existing benchmark mode TCP implementation.

### step 2 — MTU sweep

discovers the path maximum transmission unit.

**procedure**:
- send UDP packets with DF (Don't Fragment) bit set
- binary search from 1500 down to find largest packet that passes
- compare discovered MTU against known encapsulation signatures

**known MTU signatures**:

| MTU | encapsulation |
|---|---|
| 1500 | standard ethernet |
| 1460 | TCP over standard ethernet (MSS) |
| 1450 | VXLAN |
| 1418 | GRE |
| 1400 | IPsec (ESP + tunnel mode, typical) |
| 1380 | IPsec + NAT-T |
| 1370 | GRE + IPsec |

**output**: path MTU + detected encapsulation (if any).

### step 3 — health check (UDP CBR)

measures link quality under moderate, sustained load.

**procedure**:
- send UDP at constant bitrate = 80% of $V_{ref}$
- duration: 15s
- packet size: discovered MTU (step 2)

**metrics collected**:
- **jitter**: standard deviation of inter-packet delay variation
- **stability**: consistency of throughput over time windows
- **packet loss**: (packets_sent - packets_received) / packets_sent

**UDP packet format**:

```
┌──────────────────────────┐
│ seq_num: u64             │  8 bytes
├──────────────────────────┤
│ sender_timestamp_ns: u64 │  8 bytes
├──────────────────────────┤
│ payload: ChaCha8         │  n bytes
└──────────────────────────┘
```

timestamps embedded per-packet for precise jitter/ROWD measurement.

### step 4 — stress test

finds degradation thresholds by progressively increasing load.

**procedure**:
- start at 80% of $V_{ref}$
- increase by 5% per step
- each step: 10s constant bitrate
- continue up to 110% of $V_{ref}$
- record metrics at each step

**metrics per step**:
- **ROWD (Relative One-Way Delay)**: latency variation relative to first packet
- **jitter**: standard deviation of ROWD values
- **packet loss**: at this specific bitrate
- **throughput**: effective received throughput vs sent

**detection**:
- **bufferbloat**: ROWD increasing steadily across steps -> queuing in network devices
- **loss threshold**: bitrate at which packet loss exceeds acceptable levels
- **physical capacity**: throughput at which performance degrades

### step 5 — report

displays structured results on stdout.

**performance matrix**: throughput per step, single vs multi stream comparison.

**physical link profile**: discovered MTU, detected encapsulation, estimated physical capacity (throughput before degradation).

**reliability matrix**: jitter, loss, and ROWD at each tested bitrate level.

### step 6 — diagnostic

automated analysis of collected data. produces human-readable recommendations.

**example output**:
```
physical capacity: 15.2 Mbps before degradation
TCP behavior: single stream limited, applications should use multi stream
stability: excellent jitter (< 2ms)
encapsulation: IPsec detected (MTU 1400)
recommendation: set MTU to 1360 to avoid fragmentation
```

**JSON export** (`--json`): all raw metrics from every step, machine-parseable.


## timestamps and timing

### reference establishment (qualify mode, steps 3-4)

**first packet** establishes time reference:
```
ref_sender_time = first_packet.sender_timestamp_ns
ref_receiver_instant = Instant::now()
```

both sides now share a common $t=0$.

### relative one-way delay (ROWD)

for each subsequent packet:
```
sender_elapsed = packet.sender_timestamp_ns - ref_sender_time
receiver_elapsed = receiver_instant - ref_receiver_instant

ROWD = receiver_elapsed - sender_elapsed
```

**ROWD measures**:
- variation of latency relative to first packet
- NOT absolute latency (impossible without client/server clock sync)

ROWD = 0 -> latency stable

ROWD > 0 -> latency increased (congestion)

ROWD < 0 -> latency decreased (route change)

### jitter

```
jitter = stddev(ROWD values)
```

measures latency stability:
- < 10ms: stable line
- 10-50ms: moderate instability
- \> 50ms: severe instability (e.g., bufferbloat)


## benchmark quality optimizations

### during benchmark (critical path)

**sender**:
- construct packet on stack (no heap allocation)
- single syscall per packet (`write_all` / `send_to`)
- minimal arithmetic

**receiver**:
- single syscall per read (`read` / `recv_from`)
- minimal parsing (2× `u64::from_be_bytes`)
- store raw data in pre-allocated `Vec`

**avoided during benchmark**:
- statistics calculations
- jitter/variance computation
- logging/printing
- complex operations

### after benchmark

all metrics calculated post-reception:
- ROWD for all packets
- mean, variance, jitter
- sorting (if needed)
- integrity verification (if `--verify`)


## limitations

### no absolute latency

impossible without clock synchronization. we measure **variation** (jitter), not absolute values. sufficient for line qualification.

### no RTT

measures one-way delay only. for round-trip time: use separate ping tool.

### clock drift

assumes negligible drift on short benchmarks (<60s). modern clocks: **~1µs/s drift**, 10µs over 10s (negligible).
