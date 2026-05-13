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

### TCP benchmark

```bash
aw client benchmark tcp -s 192.168.1.11 -p 9000
```

#### TCP benchmark options

| flag | description | default |
|---|---|---|
| `-s, --server` | target server address | required |
| `-p, --port` | target server port | required |
| `-t, --time` | benchmark duration (e.g., `10s`, `1m`) | `10s` |
| `-n, --n-streams` | number of parallel streams (1-128) | `1` |
| `--verify` | enable payload integrity verification | off |

### UDP benchmark

```bash
aw client benchmark udp -s 192.168.1.11 -p 9000 -b 50M
```

#### UDP benchmark options

| flag | description | default |
|---|---|---|
| `-s, --server` | target server address | required |
| `-p, --port` | target server port | required |
| `-t, --time` | benchmark duration (e.g., `10s`, `1m`) | `10s` |
| `-n, --n-streams` | number of parallel streams (1-128) | `1` |
| `-b, --bandwidth` | target bandwidth (e.g., `1K`, `10M`, `1G`) | required |
| `-l, --length` | UDP payload size in bytes | `1400` |

### qualify (not yet implemented)

automated link qualification. runs a multi-step pipeline to fully profile a network path.

```bash
aw client qualify -s 192.168.1.11 -p 9000
```
