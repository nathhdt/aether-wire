# roadmap

## CLI

- [x] server (TCP)
- [x] server (UDP)
- [x] client benchmark (TCP)
- [x] client benchmark (UDP)
- [ ] client qualify (full pipeline)

## TUI

- [ ] server
- [ ] client benchmark (TCP)
- [ ] client benchmark (UDP)
- [ ] client qualify (full pipeline)

## protocol support

### layer 3

- [x] IPv4
- [ ] IPv6

### layer 4

- [x] TCP
- [x] UDP

## TCP benchmark mode

- [x] TCP base benchmark
- [x] multi-stream sessions
- [x] TCP payload integrity verification option (`--verify`)
- [ ] TCP reverse-directional mode
- [ ] TCP both-directional mode
- [ ] TCP bidirectional mode
- [ ] TCP benchmark quality optimizations
- [ ] JSON export (`--json`)

## UDP benchmark mode

- [x] multi-stream sessions
- [x] UDP base benchmark
- [ ] UDP benchmark quality optimizations
- [ ] packet length option
- [ ] out-of-order packet count statistics
- [ ] duplicate packet count statistics
- [ ] configurable UDP sending buffer size
- [ ] JSON export (`--json`)

## server mode

- [x] configurable UDP receiving buffer size

## qualify mode (not implemented yet)

### step 1 - TCP probe

- [x] single stream throughput test
- [x] multi stream throughput test
- [x] $T_{ref}$ (reference throughput) calculation

### step 2 - MTU sweep

- [ ] UDP path MTU discovery
- [ ] encapsulation fingerprinting
- [ ] known MTU signature table

### step 3 - health check (UDP CBR)

- [ ] constant bitrate sender (80% $T_{ref}$)
- [ ] jitter measurement
- [ ] stability measurement
- [ ] packet loss measurement

### step 4 - stress test

- [ ] ramp from 80% to 110% $T_{ref}$
- [ ] ROWD (Relative One-Way Delay) calculation
- [ ] bufferbloat detection
- [ ] loss threshold detection

### step 5 - report

- [ ] performance matrix
- [ ] physical link profile
- [ ] reliability matrix

### step 6 - diagnostic

- [ ] automated analysis and recommendations
- [ ] JSON export (`--json`)
