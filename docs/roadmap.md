# roadmap

## CLI

- [x] server (TCP)
- [x] server (UDP)
- [x] client benchmark (TCP)
- [x] client benchmark (UDP)
- [ ] client qualify (full pipeline)

## protocol support

### layer 3

- [x] IPv4
- [ ] IPv6

### layer 4

- [x] TCP
- [x] UDP

## benchmark mode

- [x] multi-stream sessions
- [x] UDP base benchmark
- [x] TCP base benchmark
- [x] TCP payload integrity verification option (`--verify`)
- [ ] TCP reverse-directional mode
- [ ] TCP both-directional mode
- [ ] TCP bidirectional mode
- [ ] TCP benchmark quality optimizations
- [ ] UDP benchmark quality optimizations

## qualify mode (not implemented yet)

### step 1 - TCP probe

- [x] single stream throughput test
- [x] multi stream throughput test
- [x] $B_{ref}$ (reference throughput) calculation

### step 2 - MTU sweep

- [ ] UDP path MTU discovery
- [ ] encapsulation fingerprinting
- [ ] known MTU signature table

### step 3 - health check (UDP CBR)

- [ ] constant bitrate sender (80% $B_{ref}$)
- [ ] jitter measurement
- [ ] stability measurement
- [ ] packet loss measurement

### step 4 - stress test

- [ ] ramp from 80% to 110% $B_{ref}$
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
