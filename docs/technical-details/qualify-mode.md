# qualify mode (not implemented yet)

the qualification pipeline runs automatically. the client orchestrates all steps sequentially, using the same server endpoint.

```
┌──────────────────────────────────────────────────────────┐
│                    aw client qualify                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  step 1: TCP probe ─────────────────────────────→ Tref   │
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

## step 1: TCP probe

establishes a reference throughput ($T_{ref}$) for the link.

**procedure**:
- single stream TCP test (15s) → throughput_single
- multi stream TCP test, 4 streams (15s) → throughput_multi
- $T_{ref}$ = max(throughput_single, throughput_multi)

$T_{ref}$ is used by all subsequent steps to calibrate their sending rates.
