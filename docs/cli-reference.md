# CLI reference

## command pattern

```bash
aw <command> [subcommand] [options]
```

# commands

## server

starts an aether-wire server. listens for both TCP and UDP sessions on a single port.

```bash
aw server -p 9000
```

### server options

| flag | description | default |
|---|---|---|
| `-b, --bind` | bind address | `0.0.0.0` |
| `-p, --port` | listening port | required |
| `--once` | exit after serving one session | off |

### `--once`

terminates the server after the first completed session.

useful for: scripting, CI pipelines, automated benchmark orchestration.

## client

### benchmark

raw TCP throughput measurement.

```bash
aw client benchmark -s 192.168.1.11 -p 9000
```

#### benchmark options

| flag | description | default |
|---|---|---|
| `-s, --server` | target server address | required |
| `-p, --port` | target server port | required |
| `-t, --time` | benchmark duration (e.g., `10s`, `1m`) | `10s` |
| `-n, --n-streams` | number of parallel streams (1-128) | `1` |
| `--verify` | enable payload integrity verification | off |

### qualify

automated link qualification. runs a multi-step pipeline to fully profile a network path.

```bash
aw client qualify -s 192.168.1.11 -p 9000
```

#### qualify options

| flag | description | default |
|---|---|---|
| `-s, --server` | target server address | required |
| `-p, --port` | target server port | required |
| `--json` | export full metrics to JSON file | off |

#### qualification pipeline

1. **TCP probe**: establishes reference throughput ($V_{ref}$) via single and multi stream tests (~30s)
2. **MTU sweep**: discovers path MTU using UDP with DF bit, identifies encapsulation signatures (~10s)
3. **health check (UDP CBR)**: constant bitrate at 80% $V_{ref}$, measures jitter and stability (~15s)
4. **stress test**: ramp from 80% to 110% $V_{ref}$ by 5% steps, measures ROWD, detects bufferbloat and loss thresholds (~3m)
5. **report**: displays performance matrix, physical link profile, reliability matrix
6. **diagnostic**: automated analysis and recommendations

# directional modes (benchmark command)

## unidirectional (default)

tests **client -> server** throughput.

```bash
aw client benchmark -s 192.168.1.11 -p 9000 -t 10s
```

default mode. measures upload capacity.

## reverse

tests **client <- server** throughput.

```bash
aw client benchmark -s 192.168.1.11 -p 9000 -t 10s --reverse
```

measures download capacity.

## both (sequential)

tests both directions **sequentially**.

```bash
aw client benchmark -s 192.168.1.11 -p 9000 -t 10s --both
```

behavior:
- client -> server (10s)
- client <- server (10s)

total duration: 20s

detects line asymmetry without interference.

## bidirectional (simultaneous)

tests both directions **simultaneously**.

```bash
aw client benchmark -s 192.168.1.11 -p 9000 -t 10s --bidirectional
```

behavior:
- client -> server
- client <- server

total duration: 10s

reveals:
- bidirectional saturation
- mutual congestion impact
- realistic behavior for interactive applications

# payload verification (benchmark)

## `--verify`

enables payload integrity verification. payloads generated with ChaCha8 are verified on receiver side.

detects:
- corrupted transfers
- payload mutations
- network stack anomalies

### example

```bash
aw client benchmark -s 192.168.1.11 -p 9000 --verify
```

### performance impact

increases CPU usage on both sides. **may reduce measured throughput** at high speeds.
