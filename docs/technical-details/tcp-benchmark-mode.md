# TCP benchmark mode

## parallel streams model

with **n streams**:
- client opens **n TCP sockets**
- server runs **1 listener** + **n accepted connections**
- each stream independent

## test boundaries

client sends for exactly **t seconds** (e.g., 10s).

## throughput calculation

```
throughput = bytes × 8 / duration
```

includes TCP retransmissions (real behavior).

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

## `--verify` option

performs data integrity validation for received TCP stream:
- uses the deterministic `stream_seed` to verify data
- verification buffer size is configurable (default: `1 GiB`, max: available RAM)
- available only with a single stream (`--n-streams 1`)
