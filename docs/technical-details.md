# technical details

## design philosophy

aether-wire measures network performance under realistic conditions.

principles:
- **no optimization**: emulate standard client/server behavior
- **TCP as-is**: let TCP handle flow control, congestion, retransmissions
- **telemetry embedded**: timestamps for precise measurements
- **benchmark-quality first**: all heavy calculations done after benchmark


## TCP benchmark

### parallel streams model

with **N streams**:
- client opens **N TCP sockets**
- server runs **1 listener** + **N accepted connections**
- each stream independent

### test boundaries

client sends for exactly **T seconds** (e.g., 10s).

server measures using **client timestamps** (exact timing).

no reliance on clock synchronization.

## packet format

### initialization (once per connection)

```
┌────────────────┐
│ stream_id: u16 │  2 bytes
└────────────────┘
```

sent at connection start. identifies stream for the entire session.

### data packets (repeated)

```
┌──────────────────────────┐
│ seq_num: u64             │  8 bytes
├──────────────────────────┤
│ sender_timestamp_ns: u64 │  8 bytes
├──────────────────────────┤
│ payload: ChaCha8         │  N bytes (typically 1472)
└──────────────────────────┘

total: 16 + N bytes
overhead: ~1%
```

**fields**:
- `seq_num`: packet sequence number (0, 1, 2, ...)
- `sender_timestamp_ns`: monotonic timestamp from sender
- `payload`: pseudo-random data (ChaCha8)

## timestamps and timing

### reference establishment

**first packet** establishes time reference:
```
ref_sender_time = first_packet.sender_timestamp_ns
ref_receiver_instant = Instant::now()
```

both sides now share a common t=0.

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

ROWD = 0 → latency stable  
ROWD > 0 → latency increased (congestion)  
ROWD < 0 → latency decreased (route change)

## metrics

### throughput

```
throughput = bytes × 8 / sender_duration
```

calculated using sender timestamps (exact 10s window). includes TCP retransmissions (real behavior).

### jitter

```
jitter = stddev(ROWD values)
```

measures latency stability:
- < 10ms: stable line
- 10-50ms: moderate instability
- \> 50ms: severe instability (e.g., bufferbloat)

### packet loss (indirect with TCP)

```
loss = (bytes_sent - bytes_received) / bytes_sent
```

TCP retransmits automatically. loss detected if retransmissions don't complete within test window.

## directional modes

### unidirectional (default)

```
client ─── [n streams] ──> server
```

measures: upload capacity.

### reverse

```
client <── [n streams] ─── server
```

measures: download capacity.

### both (sequential)

```
phase 1: client ─── [n streams] ──> server (10s)
phase 2: client <── [n streams] ─── server (10s)

total: 2n connections, 20s
```

detects asymmetry without interference. separate measurements per direction.

### bidirectional (simultaneous)

```
client ──── [n streams] (upload) ───> server
client <── [n streams] (download) ─── server

total: 2n connections, 10s
```

reveals:
- bidirectional saturation
- mutual congestion impact
- realistic interactive workload

## benchmark quality optimizations

### during benchmark (critical path)

**sender**:
- construct packet on stack (no heap allocation)
- single syscall per packet (`write_all`)
- minimal arithmetic

**receiver**:
- single syscall per read (`read`)
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

## use cases by mode

### unknown line (first diagnostic)

```bash
aw client ipv4 tcp -s server -p 9000 --bidirectional
```

quick bidirectional test (10s).

### suspected asymmetry

```bash
aw client ipv4 tcp -s server -p 9000 --both
```

precise per-direction measurements (20s).

### bulk transfer (backup, file transfer)

```bash
aw client ipv4 tcp -s server -p 9000 -t 60s
```

unidirectional, long duration.

### interactive applications (SSH, gaming)

```bash
aw client ipv4 tcp -s server -p 9000 -n 4 --bidirectional
```

multiple streams, bidirectional load.

## UDP benchmark

TO-DO