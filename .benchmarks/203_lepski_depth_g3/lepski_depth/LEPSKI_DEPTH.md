# Lepski cascade adaptive depth — the i130 frozen read

| field | value |
|---|---|
| date | 2026-11-20T19:34:56Z |
| git | 26fcfba9 |
| datasets | /Users/kat/git/riir-reflex/.raw/datasets |
| eta | 0.05 |
| A | 1 |
| k_max cap | 512 |
| read rule | one invocation = one frozen read: cal premise gates (P-A headroom >= +1.0pp over the routed-whole reference; P-B cascade >= best-fixed -1.0pp and abstain <= 50%) decide whether the test slice is read at all; G3 = paired LB95 of (cascade - fixed-grid-max) per-case correctness > 0; forced-pick everywhere; case-correct = ALL its scorable questions correct |

## ag_news

- pool 256 docs · 4 domains (0 fallback) · k0 = 69 · grid [69, 138] · P1 inert in-domain grids: 4/4
- CAL premise: ref(routed-whole) 0.2500 · grid [0.2500, 0.2500] · best-fixed 0 0.2500 · cascade 0.2500 · abstain 0.895 · σ̂=0 rate 0.995
- gates: P-A(headroom ≥ +1pp) FAIL · P-B(selector ≥ best−1pp, abstain ≤ 0.5) FAIL → GATED OUT — test slice NOT read
- TEST: NOT READ (the premise gates decided it)
- latency (cal questions): adaptive pass p50 859µs / p99 1125µs (support 3) · routed-whole pass p50 151µs / p99 268µs · σ̂ p50 1.125µs / p99 1.458µs · cascade fold p50 0.042µs / p99 0.042µs

## emotion

- pool 384 docs · 6 domains (0 fallback) · k0 = 72 · grid [72, 144, 288] · P1 inert in-domain grids: 6/6
- CAL premise: ref(routed-whole) 0.1650 · grid [0.1650, 0.1650, 0.1650] · best-fixed 0 0.1650 · cascade 0.1650 · abstain 0.925 · σ̂=0 rate 1.000
- gates: P-A(headroom ≥ +1pp) FAIL · P-B(selector ≥ best−1pp, abstain ≤ 0.5) FAIL → GATED OUT — test slice NOT read
- TEST: NOT READ (the premise gates decided it)
- latency (cal questions): adaptive pass p50 1426µs / p99 2103µs (support 3) · routed-whole pass p50 110µs / p99 123µs · σ̂ p50 2.125µs / p99 2.834µs · cascade fold p50 0.042µs / p99 0.084µs

## sst5

- pool 320 docs · 5 domains (0 fallback) · k0 = 71 · grid [71, 142, 284] · P1 inert in-domain grids: 5/5
- CAL premise: ref(routed-whole) 0.2000 · grid [0.2000, 0.2000, 0.2000] · best-fixed 0 0.2000 · cascade 0.2000 · abstain 0.095 · σ̂=0 rate 0.100
- gates: P-A(headroom ≥ +1pp) FAIL · P-B(selector ≥ best−1pp, abstain ≤ 0.5) PASS → GATED OUT — test slice NOT read
- TEST: NOT READ (the premise gates decided it)
- latency (cal questions): adaptive pass p50 1108µs / p99 1503µs (support 3) · routed-whole pass p50 91µs / p99 108µs · σ̂ p50 2.042µs / p99 2.833µs · cascade fold p50 0.042µs / p99 0.125µs

## prompt_injections

- pool 128 docs · 2 domains (0 fallback) · k0 = 63 · grid [63, 126] · P1 inert in-domain grids: 2/2
- CAL premise: ref(routed-whole) 0.5100 · grid [0.5100, 0.5000] · best-fixed 0 0.5100 · cascade 0.5100 · abstain 0.940 · σ̂=0 rate 0.990
- gates: P-A(headroom ≥ +1pp) FAIL · P-B(selector ≥ best−1pp, abstain ≤ 0.5) FAIL → GATED OUT — test slice NOT read
- TEST: NOT READ (the premise gates decided it)
- latency (cal questions): adaptive pass p50 243µs / p99 515µs (support 2) · routed-whole pass p50 69µs / p99 87µs · σ̂ p50 1.000µs / p99 1.375µs · cascade fold p50 0.042µs / p99 0.084µs

## xnli_en

- pool 192 docs · 3 domains (0 fallback) · k0 = 67 · grid [67, 134] · P1 inert in-domain grids: 3/3
- CAL premise: ref(routed-whole) 0.3350 · grid [0.3350, 0.3350] · best-fixed 0 0.3350 · cascade 0.3350 · abstain 0.725 · σ̂=0 rate 0.975
- gates: P-A(headroom ≥ +1pp) FAIL · P-B(selector ≥ best−1pp, abstain ≤ 0.5) FAIL → GATED OUT — test slice NOT read
- TEST: NOT READ (the premise gates decided it)
- latency (cal questions): adaptive pass p50 512µs / p99 850µs (support 3) · routed-whole pass p50 85µs / p99 106µs · σ̂ p50 1.042µs / p99 1.500µs · cascade fold p50 0.042µs / p99 0.042µs

## massive_intent_en

- pool 2703 docs · 59 domains (0 fallback) · k0 = 88 · grid [88, 176, 352] · P1 inert in-domain grids: 59/59
- CAL premise: ref(routed-whole) 0.0750 · grid [0.0650, 0.0650, 0.0650] · best-fixed 0 0.0650 · cascade 0.0650 · abstain 0.895 · σ̂=0 rate 0.990
- gates: P-A(headroom ≥ +1pp) FAIL · P-B(selector ≥ best−1pp, abstain ≤ 0.5) FAIL → GATED OUT — test slice NOT read
- TEST: NOT READ (the premise gates decided it)
- latency (cal questions): adaptive pass p50 4075µs / p99 4767µs (support 3) · routed-whole pass p50 89µs / p99 109µs · σ̂ p50 2.500µs / p99 3.125µs · cascade fold p50 0.042µs / p99 0.167µs

## banking77

- pool 3071 docs · 77 domains (0 fallback) · k0 = 89 · grid [89, 178, 356] · P1 inert in-domain grids: 77/77
- CAL premise: ref(routed-whole) 0.0550 · grid [0.0450, 0.0450, 0.0450] · best-fixed 0 0.0450 · cascade 0.0450 · abstain 0.965 · σ̂=0 rate 0.995
- gates: P-A(headroom ≥ +1pp) FAIL · P-B(selector ≥ best−1pp, abstain ≤ 0.5) FAIL → GATED OUT — test slice NOT read
- TEST: NOT READ (the premise gates decided it)
- latency (cal questions): adaptive pass p50 16895µs / p99 24811µs (support 3) · routed-whole pass p50 339µs / p99 643µs · σ̂ p50 2.542µs / p99 3.500µs · cascade fold p50 0.042µs / p99 0.208µs

## typed_decisions

- pool 192 docs · 4 domains (0 fallback) · k0 = 67 · grid [67, 134] · P1 inert in-domain grids: 4/4
- CAL premise: ref(routed-whole) 0.3100 · grid [0.2880, 0.2880] · best-fixed 0 0.2880 · cascade 0.2880 · abstain 0.680 · σ̂=0 rate 0.750
- gates: P-A(headroom ≥ +1pp) FAIL · P-B(selector ≥ best−1pp, abstain ≤ 0.5) FAIL → GATED OUT — test slice NOT read
- TEST: NOT READ (the premise gates decided it)
- latency (cal questions): adaptive pass p50 1514µs / p99 3307µs (support 6) · routed-whole pass p50 124µs / p99 478µs · σ̂ p50 1.083µs / p99 1.542µs · cascade fold p50 0.042µs / p99 0.166µs
