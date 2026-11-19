# Bench 200 — the 121 standing sweep: BM25 floor + nb/ridge selections, 6 suites, two independent runs

**Status:** RECORD — the 121 standing-sweep bench record + A3 adjudicated: **the floor lane ADOPTS (violations found + distinct signal — neither NEGATIVE-close leg fires)**. Two corpus-defect findings filed (issue 126: ag_news + semantic_defects engine<bm25).
**Lane:** `harness --suites ag_news,emotion,xnli_en,massive_intent_en,banking77,semantic_defects --bm25-floor --nb-select --ridge-select` (report-only; the flags landed `125a11c`+`041a8f4`+`777132e`, in-tree at `8c0439af`; binary rebuilt this session — the Nov-14 prebuild predated the flags).
**PROVENANCE:** box shikuwa (4090, Windows) · RAM-CONSTRAINED RUN DISCLOSED: FreePhysicalMemory 0.33 GB at build start (the record's numbers are deterministic accuracy/counts, not wall-clock; run2's load 7.20 footer marks latency NOT QUOTABLE — sibling load) · datasets `.raw/datasets` (local seat) · sha `8c0439af` + claim `54c3abae`.
**Run:** run1 2026-11-19T18:21Z · run2 2026-11-19T18:24Z · both EXIT 0, PASSED WITH ABSENCES (6 laya/english feature-off absences — loud, expected) · tables byte-identical across runs except timestamps/load footers.
**Exclusion (LOUD):** typed_decisions EXCLUDED — hard_metrics panic (`gold index 4 out of range for 4 probs`, metrics.rs:90): the corpus's 4 workflows carry heterogeneous gold schemas (issue **125**, filed before this record).

## The table (run1 == run2)

| suite | n | modelless | bm25 floor | nb-tables(sel) | nb-selected(sel) | ridge(sel) | engine>=bm25 |
|---|---|---|---|---|---|---|---|
| ag_news | 400 | 0.8825 | **0.8875** | 0.7450 | 0.7450 | 0.7450 | **NO (−0.5pp) → defect 126** |
| emotion | 400 | 0.8850 | 0.6000 | 0.5500 | 0.5500 | 0.7600 | yes |
| xnli_en | 300 | 0.5233 | 0.3867 | 0.5550 | 0.3400 | 0.5800 | yes |
| massive_intent_en | 300 | 0.7800 | 0.4667 | 0.5950 | 0.6050 | 0.6050 | yes |
| banking77 | 500 | 0.8420 | 0.8080 | 0.8000 | 0.8100 | 0.8080 | yes (thin +3.4pp) |
| semantic_defects | 102 | 0.4412 | **0.4706** | 0.4444 | 0.4444 | — | **NO (−2.9pp) → defect 126** |

## A-pre (the redundancy read)

NOT fully redundant: BM25 carries distinct signal — ag_news 0.8875 vs the nb/ridge comparators' flat 0.7450 (+14.25pp, the largest gap in the table); xnli_en it lands BELOW nb-tables; emotion below ridge. Mixed per-suite, never a flat domination → the NEGATIVE-close redundancy leg does not fire.

## A3 adjudication (the pre-registered bar)

Close NEGATIVE iff A-pre full redundancy AND no engine-vs-BM25 violation across two independent runs. **Both legs fail**: (a) A-pre not-redundant (above); (b) TWO violations, byte-identical across both runs (ag_news −0.5pp, semantic_defects −2.9pp). → **The lane's value is DELIVERED (a violation found OR a standing floor adopted — both happened).** The floor ADOPTS as the standing lexical floor (A2 row ships report-only, as landed); the two violating suites are corpus defects to fix at their sources (issue 126), re-measured on fix.
