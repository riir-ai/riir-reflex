# Bench 177 — Issue 110 T1: the CONFLICT-band falsification cell

Issue 110 T1 (the falsification arm, run before any wiring): measure the
share of ANSWERED questions whose top-k nearest corpus references split
across labels while the deployed DistanceGate passed — with the witness
picked per the issue's own discipline: two INDEPENDENT readers, the
served corpus-centroid pick vs the reference-label majority (the
issue's witness (b); witness (a) needs a heads-armed posture, (b) rides
the default one). **Verdict: the naive band is huge (69% of answered),
and the gold split shows WHY — the corpus at the served caps does not
ground an independent second reading. The witness (b) reader is
degenerate at this corpus geometry: the band measures the hashed-bag
embedding's granularity, not two credible readings. The CONFLICT status
premise FAILS at this seat — closed NEGATIVE** (the issue's own
negligible-residual clause, inverted: the residual is huge but
illegitimate).

## Run posture

- Items run: `target/release/harness --skip-laya --dump-items` over the
  7 dataset suites, default posture (calibrated gates, registry caps),
  canonical `.raw/datasets` pool — `items_run/` beside this record
  (`results.json` carries each suite's `corpus_cap.effective`; the
  per-question `pick`/`gold`/`ok`/`abstain_cal` are the SERVED verdicts).
- Probe: `examples/conflict_band_probe.rs` (`3254602`-lineage, landed
  with this record) — joins the items dump against `prepare_seat`'s
  byte-identical eval slice, mirrors the served corpora (per-label
  `take(cap)` over the cal-excluded pool), embeds the modelless context
  (`state + "\n" + prompt`, the engine's own composition), kNN top-5
  over all capped refs (cosine on the shared 256-bucket hashed bag),
  first-seen tie rules pinned.
- Box state: m3, AC power, load ≈ 5 (preflight posture; the accuracy /
  membership counts are contention-immune — no latency claim rides this
  record).
- `prompt_injections` and `sst5` are 100%-abstained at the calibrated
  gates — the witness pool is EMPTY there (the gates already refuse
  everything; disclosed absence, not a zero-answer).
- `typed_decisions` excluded (5q/case workflow shape — the option-key/
  label join is a different witness; disclosed).

## The numbers

| suite | answered | unanim | split==pick | split≠pick (band) | band% of answered | cap |
|---|---|---|---|---|---|---|
| ag_news | 243 | 4 | 79 | 160 | 65.8% | 64 |
| banking77 | 237 | 32 | 106 | 99 | 41.8% | 40 |
| emotion | 268 | 0 | 43 | 225 | 84.0% | 64 |
| massive_intent_en | 196 | 12 | 4 | 180 | 91.8% | 48 |
| prompt_injections | 0 | — | — | — | (100% abstained) | 64 |
| sst5 | 0 | — | — | — | (100% abstained) | 64 |
| xnli_en | 133 | 1 | 50 | 82 | 61.7% | 64 |
| TOTAL | 1077 | 49 | 282 | 746 | **69.3%** | |

Band gold split (who is right where the two readers disagree):

| suite | pick right | majority right | both right | 
|---|---|---|---|
| ag_news | 57/160 (35.6%) | 40/160 (25.0%) | **0** |
| banking77 | 17/99 (17.2%) | 51/99 (51.5%) | **0** |
| emotion | 62/225 (27.6%) | 36/225 (16.0%) | **0** |
| massive_intent_en | 106/180 (58.9%) | 0/180 (0.0%) | **0** |
| xnli_en | 26/82 (31.7%) | 28/82 (34.1%) | **0** |

`both right = 0` EVERYWHERE is the structural tell: by construction a
split-majority question cannot have both readers right (the readers
disagree with each other, so at most one matches gold) — the cells to
read are pick-right vs majority-right, and NEITHER side reaches
credible-reader levels (ag_news 35.6% vs 25.0% where the answered-pool
accuracy is 39.5%; massive_intent_en 58.9% vs 0.0% — the majority
reader is COMPLETELY wrong there, the served pick dominates).

## Why the band is illegitimate (the falsification finding)

1. **The kNN-majority reader is degenerate at the served caps.** The
   corpora are tiny (40–64 docs/label) and the 256-bucket hashed-bag
   space is coarse: the independent-python re-derivation of the same
   reader (ag_news, mirrored corpora, k=5) lands the kNN-majority at
   ~25% gold agreement in-band vs the pick's ~36% — both far below the
   answered pool's own accuracy. A "second reading" that is right
   16–51% of the time (and 0% on massive) is not a credible reading;
   its disagreements measure reader weakness, not corpus conflict.
2. **The served pick distribution shows the same degeneracy from the
   other side.** emotion's answered pool: the pick chose "joy" on 227
   of 268 answered questions (the readout collapsed toward one option);
   ag_news answered picks: 153/243 are label 0. The answered pool
   itself is not a set of confident grounded answers — it is the
   subset the gates let through, with a collapsed argmax. A CONFLICT
   status on top of this would rename gate-leakage as disagreement.
3. **The gates already own the honest-refusal surface.** The
   100%-abstained suites (sst5, prompt_injections) show the calibrated
   gates refuse wholesale where grounding fails; the residual band the
   probe found is not "the gates missed conflict" — it is "the gates
   passed questions whose corpus grounding is too weak for ANY second
   reading to be credible." XERJ's CONFLICT needs two records that each
   LOOK grounded; this corpus never gets there at served scale.

## What would re-open it

The premise could become measurable only with a corpus that actually
grounds two readers: larger per-label corpora (the caps are the
binding constraint — 40–64 docs/label), or a reader pair where BOTH
sides individually clear the answered-pool accuracy (the
pre-registered gate any re-run inherits: a band is real only if the
second reader's in-band gold rate is within noise of the first's).
Neither is a wire change — the corpus posture would have to move
first. Until then: no wire shape decision (T2) is owed, T3 evidence
pointers stay unwired, T5's second consumer never opens.

## Record

- Probe: `examples/conflict_band_probe.rs` (report-only example; no
  gate, no harness wiring — a one-shot falsification cell).
- Items + results: `items_run/` beside this record.
- The issue closes NEGATIVE with these numbers (T1's own clause).
