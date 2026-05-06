# CLI reference

## command pattern

```bash
aw <mode> <ip-version> <protocol> [options]
```

# modes

## server

starts a benchmark server.

### example

```bash
aw server ipv4 tcp -p 9000
```

### server options

| flag | description |
|---|---|
| `-b, --bind` | bind address |
| `-p, --port` | listening port |
| `--once` | exit after serving one benchmark session |

### `--once`

terminates the server after the first completed benchmark.

useful for: scripting, CI pipelines, automated benchmark orchestration.

## client

starts a benchmark client.

### example

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -t 10s -n 4
```

### client options

| flag | description |
|---|---|
| `-s, --server` | target server address |
| `-p, --port` | target server port |
| `-t, --time` | benchmark duration (e.g., `10s`, `1m`) |
| `-n, --n-streams` | number of parallel streams (1-128) |
| `--verify` | enable payload integrity verification |

# directional modes

## unidirectional (default)

tests **client -> server** throughput.

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -t 10s
```

default mode. measures upload capacity.

## reverse

tests **server -> client** throughput.

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -t 10s --reverse
```

measures download capacity.

## both (sequential)

tests both directions **sequentially**.

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -t 10s --both
```

behavior:
1. client -> server (10s)
2. server -> client (10s)
3. total duration: 20s

detects line asymmetry without interference.

## bidirectional (simultaneous)

tests both directions **simultaneously**.

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -t 10s --bidirectional
```

behavior:
- client -> server (n streams)
- server -> client (n streams)
- runs for 10s
- total: 2n active connections

reveals:
- bidirectional saturation
- mutual congestion impact
- realistic behavior for interactive applications

# payload verification

## `--verify`

enables payload integrity verification.

payloads generated with ChaCha8 are verified on receiver side.

detects:
- corrupted transfers
- payload mutations
- network stack anomalies

### example

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 --verify
```

### performance impact

increases CPU usage on both sides. **may reduce measured throughput** at high speeds.

# combining options

## multiple streams + verification

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -n 8 --verify
```

## bidirectional with multiple streams

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -n 4 --bidirectional
```

creates 8 total connections (4 upload + 4 download).

## long test with reverse

```bash
aw client ipv4 tcp -s 192.168.1.11 -p 9000 -t 15m --reverse
```