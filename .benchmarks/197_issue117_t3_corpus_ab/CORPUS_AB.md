# corpus A/B (Plan 426 T5 — the V5 gate)

- cap 48/label · synth extra-cap 128 · datasets /Users/katopz/git/riir-reflex/.raw/datasets_t20k · read @ 1796ad6
- one frozen test read; two engine builds differing ONLY in corpus content; forced-pick hard accuracy; paired LB95 (synth − gold) > 0 is the V5 gate; both arms reported (C3)

## massive_intent_en — gold 0.7800 → synth 0.8133 · paired LB95 +0.0110

- arm A reproduces the published 0.78 exactly — the instrument is alive (the seat posture, selected knobs + fitted gate)
- V5 PASS — synthesis lifts the modelless row (paired LB95 > 0)
- corpus: gold 11314 doc(s) → synth 13362 doc(s) (2048 in scope of 2048 artifact rows)
- latency: gold p50 82 µs / p99 103 µs · synth p50 81 µs / p99 147 µs (V6: the lane stays sub-ms)
- flips: 11 synth-only wins, 1 gold-only wins · determinism true

## Issue-064 gates — abstention PASS · OOD transfer-ok

- abstention-entropy: KL(gold‖synth) 0.0000 · reverse 0.0000 · abstain 0.5133 → 0.4067
- OOD word-dropout ladder:

| p | acc gold | acc synth | Δ | paired LB95 |
|---|---|---|---|---|
| 0.10 | 0.7567 | 0.7933 | +0.0367 | +0.0134 |
| 0.20 (gate) | 0.7567 | 0.7900 | +0.0333 | +0.0130 |
| 0.30 | 0.6767 | 0.6933 | +0.0167 | -0.0005 |

- retention @ gate 1 · rule: deterministic seeded word-dropout ladder p∈{0.10,0.20,0.30}; gate rung p=0.20; a clean V5 PASS whose gate-rung paired LB95 < 0 reads ECHO (recorded NEGATIVE, the lane dies); a non-negative gate-rung LB95 reads transfer-ok; retention = corrupted Δ / clean Δ
- KL(gold‖synth) ≤ 0.05 nats over the 10-bin normalized answer-entropy histogram (Laplace-smoothed) — the issue's own wording gates the KL alone; the abstain-rate drift is a DISCLOSURE column, not a gate leg (first real reading, 2026-10-04: corpus growth raises answer confidence BY DESIGN — a 6pp rate drop at a 0.93 baseline with KL 0.0000 is the system working, not a collapse; an absolute-rate leg would mis-fire at high-abstain baselines and under-bind at low ones)
- no echo verdict fired

| label | gained | lost | net |
|---|---|---|---|
| audio_volume_other | 3 | 0 | +3 |
| audio_volume_mute | 2 | 0 | +2 |
| audio_volume_down | 1 | 0 | +1 |
| general_greet | 1 | 0 | +1 |
| iot_hue_lighton | 1 | 0 | +1 |
| lists_createoradd | 1 | 0 | +1 |
| music_likeness | 1 | 0 | +1 |
| qa_maths | 1 | 0 | +1 |
| news_query | 0 | 1 | -1 |

synth-only wins: massive_intent_en:7, massive_intent_en:12, massive_intent_en:85, massive_intent_en:120, massive_intent_en:143, massive_intent_en:176, massive_intent_en:223, massive_intent_en:233, massive_intent_en:256, massive_intent_en:257, massive_intent_en:283

gold-only wins: massive_intent_en:136

