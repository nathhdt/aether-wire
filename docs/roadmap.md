# roadmap

## CLI

- [x] server (TCP)
- [ ] server (unified TCP + UDP)
- [x] client benchmark (TCP)
- [ ] client qualify (full pipeline)

## protocol support

### layer 3

- [x] IPv4
- [ ] IPv6

## benchmark mode

- [x] multi-stream sessions
- [x] payload integrity verification option (`--verify`)
- [ ] reverse-directional mode
- [ ] both-directional mode
- [ ] bidirectional mode
- [ ] benchmark quality optimizations

## qualify mode

### step 1 - TCP probe

- [ ] single stream throughput test
- [ ] multi stream throughput test
- [ ] $V_{ref}$ (reference throughput) calculation

### step 2 - MTU sweep

- [ ] UDP path MTU discovery
- [ ] encapsulation fingerprinting
- [ ] known MTU signature table

### step 3 - health check (UDP CBR)

- [ ] constant bitrate sender (80% $V_{ref}$)
- [ ] jitter measurement
- [ ] stability measurement
- [ ] packet loss measurement

### step 4 - stress test

- [ ] ramp from 80% to 110% Vref ($V_{ref}$)
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
