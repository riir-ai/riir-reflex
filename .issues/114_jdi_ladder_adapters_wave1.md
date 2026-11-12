# Issue 114 — JDI ladder adapters, wave 1: the decision-shaped tools three (BFCL / When2Call / API-Bank) + CLINC150 (retrieval)

**Status:** OPEN — plan-half of rethink Issue 060's R1a/R2a; adapters unbulleted, one suite at a time. **Wave state after the 2026-11-10 third session (zed-64abf0dd@m3):** T1 probes DONE (all licences verified) · **T2 clinc150 LANDED** ([Bench 100](../.benchmarks/100_clinc150_suite_birth.md)) · **T3 when2call LANDED** ([Bench 101](../.benchmarks/101_when2call_suite_birth.md): the CORPUS-OPTIONAL posture chose re-open path (b) — the only self-serve path; the suite + the runner change landed, laya-only full-population read recorded, honestly below the majority floor) · **T4 bfcl LANDED** + **T5 api_bank LANDED** ([Bench 180](../.benchmarks/180_bfcl_api_bank_suite_birth.md) — the direct-resolve fetch lane + the two builders, corpus-optional; bfcl reads 0.5612 / skill +0.31, api_bank 0.6081 / skill ≈ 0 — the honest floor posture) · T6 clinc150+when2call+bfcl+api_bank reflex halves DONE; the rethink fold = the rethink-side booking (owner-gated amo hours, unchanged). The wave's remaining DECISIONS are unchanged: the modelless corpus for T3/T4/T5 (paths (a)/(d) = owner/lane-gated spend or rethink-side synthesis) and the owner-gated amo booking. Wave-1 reflex-side work COMPLETE — only the corpus re-opens + the rethink fold remain.
**Lease closed:** zed-64abf0dd@m3 (riir-refine idle loop continuation, m3), 1794350458, expires 2026-11-12T22:40:58+00:00 — discharged by the T4+T5 landing (Bench 180 + the fetch lane + the builders, this session); the wave's residue is the rethink-side booking, not a reflex claim.
**Claimed:** zed-3aa7e916@amo (amo), 1794451200, expires 2026-11-14T02:40:00+00:00
  **Claim scope:** the R3a TRIAGE half of rethink Issue 060 (the seven language board rows — ContractNLI / ACOS / FinEntity / iSarcasmEval / VAST / NLI4CT / RAGTruth — wire-scoreable vs scoreable-partial vs refused verdicts, recorded as a wave-3 triage section here under the R2b law; board facts quoted with the snapshot's panel_id + generated_utc; licence reads at the shape level only — the T1-pattern build-time probes stay with the adapters). NO adapter construction, NO fetch-stanza work, NO corpus decisions (owner-gated) in this claim.

Rethink Issue 060 (the beat-the-JDI ladder) orders the areas most-use × quick-win first:
**tools (18.3% fold weight, ZERO coverage today)**, then retrieval (20.0%, one served
suite). This issue carries the reflex-side half: real harness adapters for the
decision-shaped suites, so the teacher capture + ESC fit + arming wave (rethink Issue
059's machinery) can ride them exactly like banking77 rides today. **Boundary law
rethink 060 R1a: adapters are FILED here and built here — rethink never edits reflex.**

## The board rows we are adapting (snapshot 66d25721, edition decision-index-0.3)

| board row | chance | gold weight | what it asks | wire verdict (R2b law, triaged here) |
|---|---|---|---|---|
| BFCL | 0.2592 | ×1.2 | given a query + tool schemas, pick the function to call (or irrelevance-detect) | **scoreable-partial**: the pick-a-tool tier is choice-shaped; the arg-filling tiers are NOT choice-shaped — out of scope for wave 1, never forced |
| When2Call | 0.25 | ×1.0 | given a query + tool set: call a tool (which one?) or answer directly | **scoreable**: this IS the abstention-first shape — the wire's own contract (choice with an abstain option) |
| API-Bank | 0.0189 | ×1.2 | API-call selection in a dialogue | **scoreable-partial**: the which-API tier is choice-shaped; multi-turn state carry is out of scope wave 1 |
| CLINC150 | 0.006 | ×1.2 | 150-intent (+ OOS) classification | **scoreable**: banking77's nearest neighbour — same intent lane, one more suite on existing machinery |

Home appliances (chance 0) and ToolRet (0.1341) stay TRIAGE-LATER per rethink 060 R1
(the shapes are weakest fits; the decision-shaped three carry the rung's points).
**TRIAGE LANDED 2026-11-11 (amo-idle, the R1a tail + rethink 060 R2b) — see "The wave-2
triage" below.**

## The wave-2 triage (the ranking + case-exact rows; R1a tail + 060 R2b, 2026-11-11)

Grounded in the board's own release record
(`riir-rethink/.research/data/jdi_index_release-v4_2026-10-22_da1586d5.json`, rows 2/9/36)
and the harness's question grammar (`src/harness/suites.rs`: Choice = object-as-ordered-labels,
Score = array-as-levels, Noul; the engine scores `k = options.len()` per question, option-name
routing needs `options.len() <= N` = 8):

| board row | chance | metric + shape (the board's own explainer) | wire verdict |
|---|---|---|---|
| BRIGHT (id 36) | 0.116 | **nDCG@10 over 32 candidate documents per query**, answerable queries only (≥1 of the 32 relevant), 220 sampled cases (arXiv:2407.12883, xlangai/BRIGHT) | **scoreable-partial — wave 2.** The wire speaks it as per-(query, doc) `Score` questions over a pinned relevance ladder, ranked by level with the identity order as the deterministic tie-break; the harness needs ONE new scorer (nDCG@10 — every existing scorer is accuracy/F1-class) + the fetch stanza + the answerable filter. NOT wave 1: the machinery is real, and the honest expectation is weak — BRIGHT is reasoning-intensive retrieval by construction (the gold docs share little vocabulary with the query), so the modelless half lands near the lexical floor; the laya encoder half is the real read. Corpus decision (paths (a)/(d)) rides the same owner-gated rethink booking as T3/T4/T5 |
| ToolRet (id 2) | 0.1341 | **nDCG@10 over 32 candidate tools per request**, answerable queries only, 685 cases (arXiv:2503.01763) | **scoreable-partial — wave 2, the SAME shape and the SAME new nDCG scorer as BRIGHT** (rank the 32 tools per request). One scorer buys both rows; the suites differ only in the fetch stanza + the candidate text renderer |
| Home appliances (id 9) | 0 | **case exact accuracy**: given a simulated home + a command, answer ~18 handling questions, ALL must be right; duplicate + dev rows removed | **scoreable-partial — wave 2.** The wire speaks it: the multi-question envelope carries the ~18 sub-questions (each Choice/Noul over the sim state as the case's `state` JSON); the new machinery is an all-sub-answers-exact case scorer (the current scorers grade per question). The kept-rows rule (duplicate + dev removed) rides the fetch stanza's dedupe | 

Why wave 2 and never forced: every one of the three needs new harness machinery (an nDCG
scorer for the two ranking rows; an all-exact case scorer for the case-exact row) — wave 1's
law was "no new machinery invented", and the decision-shaped three + CLINC150 already carry
the tools rung's points. The triage is a shape verdict ONLY: the licence + contamination
probes remain build-time gates (the T1 pattern), and the corpus decision stays the rethink-
side owner-gated booking. Never force the wire.

## The adapter recipe (exactly what banking77 got — no new machinery invented)

1. **`scripts/fetch_datasets.sh` stanza** — the `want_suite`/`fetch_suite`/
   `probe_license`/`dedupe_train` pattern per suite; parquet-backed mirrors preferred
   (the PolyAI/banking77 script-dataset 404 lesson — probe /rows BEFORE committing the
   source; a script-based dataset needs its mteb-class mirror named in the stanza
   comment). Candidate sources to probe: BFCL `gorilla/cs-bench-...` — no: the BFCL
   test set lives in the gorilla repo's `berkeley-function-call-leaderboard` data dir
   (JSON, NOT a parquet HF dataset — the fetcher may need a direct-JSON lane or an
   HF mirror; probe first, record the verdict, never invent a transform silently).
   When2Call: `nvidia/when2call`-family (probe /rows). API-Bank: ` Backbone-Models/...`
   mirrors exist; probe. CLINC150: `clinc_oos` is script-based on the Hub (the
   banking77 404 shape) — the parquet-backed mirror is `FaizaanAliza/clinc_oos`-class;
   probe /splits + /rows first, record the chosen mirror + licence in the manifest.
2. **`.docs/02_protocols/dataset_manifest.md`** — every fetched file's blake3 digest +
   a VERIFIED licence row; `license.json` probes ride the fetch (the drift tripwire).
   Licence verdicts recorded BEFORE any number is published; fail-closed.
3. **`src/harness/suites.rs` builders** — one `build_<suite>` fn per suite, keyed on
   the mirror's real columns (banking77's `label_text` precedent); `Suite::name` is
   the OUR_SUITES spelling (`bfcl_jdi`? — NO: name them what the board names them
   minus fold-mapping noise: `bfcl`, `when2call`, `api_bank`, `clinc150`. The
   `_jdi` suffix exists because the four rethink panel suites are re-captures of
   board rows under fresh digests; the ladder suites are FIRST-CLASS suites — plain
   names, the fold maps them).
4. **`src/harness/runner.rs` SuiteSpec rows** — test/cal/corpus caps per suite shape:
   BFCL pick-tier test rows cluster by API domain — apply the sampling law
   (hash-ordered sampling if rows cluster, the 060 gate); When2Call test_cap ~its
   real split; CLINC150 test_cap 500 first-row convention (banking77 parity),
   cal_cap 200.
5. **Option-key universes**: tools suites carry LARGE option sets (BFCL 100+ APIs,
   CLINC150 150+1 intents) — reuse banking77's derived-universe rule (labels = sorted
   unique label keys, gold = key position); the seat's corpus_cap_per_label bounds
   the train side.
6. **The served-cells path (already live)** — rethink's `build_jdi_data.py`
   `served_composed` tier + rethink `scripts/jdi_aggregate.py` `OUR_SUITES` extend by
   ONE row per new suite (suite → gid, metric, label); the ESC fit instrument
   (`esc_mammoth_items_fit`) extends per the 087 pattern. That half is RETHINK-side
   (060 R1d) — reflex only ships the harness suite + the feed cells.

## Gates (bind every suite, in order)

- **Licence first, fail-closed** — a suite whose source licence does not read clean
  is refused, noted in this issue, never "annotated in" later.
- **grammar-not-answers** — no test rows in any training pool; a future distill corpus
  for the tools rung builds from TRAIN splits only, echo-gated + leak-scanned before
  any number.
- **Sampling law** — clustered test rows → hash-ordered sampling, disclosed in the
  record.
- **B5 law** — the JDI public read and the our-suite paired read ship together.
- **No silent transforms** — a source needing a normalization (BFCL's JSON lane)
  records the transform in the stanza comment + the manifest row; the digest pins
  the OUTPUT.

## Disposition

- [x] T1 — fetch probes for all four sources: /splits + /rows + licence per dataset;
      verdicts recorded here (mirror + config + split per suite), sources that 404
      named honestly. **DONE 2026-11-09 (m3, datasets-server + HF API + gorilla HF mirror):**
  - **CLINC150** — `clinc/clinc_oos` (the CANONICAL name now — the old bare `clinc_oos`
    404s with "renamed", and the issue's `FaizaanAliza/clinc_oos`-class mirror guess
    does NOT exist; no mirror needed, the canonical is parquet-backed on
    datasets-server). Splits: configs `imbalanced` + `plus`, each train/validation/test.
    Licence **CC-BY-3.0**, not gated. Test rows = the 4500-row test split (150 intents
    + oos); config `plus` (the balanced 100/intent version) is the candidate — builder
    decides at T2, the probe pins both.
  - **When2Call** — `nvidia/When2Call` (the capital-W spelling is live; the lower-case
    probe 404s "renamed"). Splits: config `test` with splits `llm_judge` + `mcq`, plus
    `train_pref` + `train_sft`. The pick-shaped suite = **config test / split mcq**.
    Licence **CC-BY-4.0**, not gated. /rows probe hit transient 502/503s
    (datasets-server flaked mid-probe); the /splits + /tree + licence probes all
    resolved — columns read at T3 from the parquet directly.
  - **BFCL** — the gorilla GitHub data dir 404s (data moved INTO the `bfcl-eval` PyPI
    package — the fetcher's direct-JSON lane would need pip, refused); the OFFICIAL
    HF mirror is `gorilla-llm/Berkeley-Function-Calling-Leaderboard` (the gorilla
    org itself, 85k downloads, licence **Apache-2.0**, not gated). NOT
    datasets-server-backed ("No supported data files found" — it is raw JSON-LINES,
    one .json per category), so the stanza needs the direct-resolve lane
    (`/resolve/main/<file>.json`) — a new fetcher pattern, the banking77 parquet
    stanza does NOT cover it; recorded as the transform-to-name. Pick-tier files
    probed with row counts: live_simple 258, live_multiple 1053, multiple 200,
    simple 400, irrelevance 240, live_irrelevance 882. The pick tier = simple +
    multiple (+ their live twins); the irrelevance files are the abstain tier —
    the When2Call-shaped abstention signal rides THERE (the R2b triage's
    "irrelevance-detect" is scoreable after all, as a second option-key universe).
  - **API-Bank** — `Backbone-Models/...` mirrors: NOT probed this session (the T5
    suite is last in the wave; the probe rides T5's start, one less stale verdict
    to re-check at build time). Named honestly as UNPROBED, not a 404.
  - **Fetch infra note (T2 prerequisite):** datasets-server flaked 502/503
    mid-probe (recovered for /splits, not for /rows) — the fetch script's retry
    + probe-then-fetch order (the banking77 lesson) is load-bearing; the stanza
    probes /splits BEFORE /rows, and a red /rows is a retry, never a verdict.
- [x] T2 — CLINC150 first (the 060 verdict: it reuses banking77's machinery as it
      stands): stanza + builder + SuiteSpec + manifest row + a smoke `harness`
      run; the teacher capture for it rides the NEXT amo trip (rethink 059 lineage,
      owner-gated amo hours — book it with the R1b tools capture, one booking).
      **DONE 2026-11-09 (m3, this session):** stanza (probe splits/size + licence +
      test/train WHOLE — the plus split is intent-ordered, so the train pull is
      uncapped or the corpus starves); `CLINC150_KEYS` (151, mechanical from the
      FETCHED features — the card's YAML differs in TWO spellings: `no'`/`yes'`
      card-typo vs the served `no`/`yes`; the verify-at-port caught it live);
      `build_clinc150` + the `train_row_label`/`train_docs` arms + the 151 dispatch
      arm + the expected-key pin + the 151-universe guard. **Two envelope root-fixes
      rode the lane** (both additive): `wrap_rows` threads the source features
      through split envelopes, and `load_rows` carries the first page's features
      block (verify-at-port builders need them on BOTH envelopes — the PolyAI
      banking77 shape has wanted this since its panic message was written).
      Smoke: 500 cases / 151 domains, slice-integrity OK (test `fnv1a64-317d15fe`
      · cal `fnv1a64-7dc66be9` · pool 15,050 `fnv1a64-705b33da`), modelless acc
      **0.3520** at p50 0.567 ms (chance 0.0066 — 53× above random on the 151-way
      task, zero-shot modelless). 560 lib tests + clippy `-D warnings` + rustfmt
      clean; fetch self-test PASS. Manifest §18 + the licence row landed
      (CC BY 3.0, card-stub caveat). Teacher capture: still rides the owner-gated
      amo booking (unchanged).
- [x] T3 — When2Call second (the abstention-first shape is the wire's own; highest
      product-value of the three tools suites). **SHAPE + CORPUS DISPOSITION 2026-11-09
      (this session, one probe pass):** the wire is CONFIRMED scoreable — the repo
      README + the Artificial-Analysis read agree: MCQ = ONE choice among FOUR fixed
      categories (`direct` / `tool_call` / `request_for_info` / `cannot_answer`; the
      row's `answers` object carries all four responses; `correct_answer` names the
      gold). test/mcq = 3,652 rows (the board's population; eval cap = all — the
      faithful paired-read posture). **BUT the suite is BLOCKED on a lawful corpus
      source** — three refusals, each measured:
  1. `test/llm_judge` (300 labeled rows) is **documented in the repo README as "a
     subset of the MCQ evaluation data"** (and uuid-overlap MEASURED 11/100 in the
     first pages) — the slice-integrity law refuses it as a train pool, full stop.
  2. `train_pref` (9,000) / `train_sft` (15,000) carry **no class label** — the
     README documents a tool-call MARKER (` ... `) for one class only; deriving
     direct/request_for_info/cannot_answer from response shapes would be an
     INVENTED transform (refused per the no-silent-transforms gate; a guessed
     label poisons every corpus it feeds).
  3. The mcq rows' own `answers` objects are the only per-class labeled text —
     and they are TEST rows (grammar-not-answers: never a training pool).
  **RESOLUTION 2026-11-09 (second session): re-open path (b) CHOSEN — the runner
  no-corpus (corpus-optional) posture — because it is the only self-serve path:**
  (a) is teacher spend (owner-gated), (d) is rethink-side synthesis (the ESC lane's
  machinery, owner-gated amo hours), (c) demotes the wave's highest-value row to
  nothing. The posture landed: `modelless_lane: false` on a dataset suite now
  means corpus-optional (prepare skips train/cal/pool entirely; the modelless lane
  skips loud; only the LLM lanes answer; the no-LLM-lane build refuses loud —
  verified). **The suite LANDED**: fetch stanza (mcq WHOLE, 3,652 rows, stored as
  `test`), the task-faithful builder (the official MCQ form: the row's own four
  response candidates as the choices, gold = correct_answer's position in the
  uniform answers-key order, pinned per row; state = query + tools AS STORED,
  258 empty-tools rows kept verbatim), WHEN2CALL_KEYS + the expected-key pin, the
  population/gold guards, manifest §19 + the CC-BY-4.0 licence row (commercial-use
  declared). First read ([Bench 101](../.benchmarks/101_when2call_suite_birth.md)):
  laya[english] 0.2757 on ALL 3,652 rows — BELOW the majority floor 0.3546, an
  honest negative for the encoder on the response-pick rendering (the diagnosis
  arms: state truncation over multi-KB tool schemas, response-length bias, or the
  category-classification comparison rendering — the ESC lane's question).
  **The modelless corpus REMAINS OPEN** — the recorded owner/lane decisions
  (a)/(d) are unchanged; when a corpus lands, the corpus-optional flag flips
  back and the train_row_label/train_docs arms + the modelless dispatch join.
- [x] T4 — BFCL pick-tier third (the arg-filling tiers REFUSED wave 1 — recorded as
      the R2b-class wire verdict, never forced). **CORPUS DISPOSITION 2026-11-09
      (same probe pass as T3):** the distribution is SINGLE-FILE-PER-CATEGORY
      (id/question/function — verified live on `BFCL_v3_live_simple.jsonl`); NO
      train split exists anywhere in it, so the corpus wall bites the same way:
      the candidate functions' names/descriptions ARE the test rows' content
      (grammar-not-answers). Structural note for the re-open: BFCL's option
      universe is PER-ROW (each row carries its own candidate list — the
      mmlupro/s1mb variable-breadth shape, `coverage_audit: false` posture), and
      the options carry their own descriptions in the criteria, so the corpus-less
      scoring question is thinner than When2Call's. **UPDATE 2026-11-09 (second
      session): the T3 corpus-optional posture UNBLOCKS this build** — the same
      `modelless_lane: false` posture applies (no train split exists → none is
      lawful to invent). The remaining build work is the FETCH lane, not the
      posture: the gorilla-llm mirror is raw JSON-LINES, not datasets-server-backed
      — the stanza needs the direct-resolve pattern (`/resolve/main/<file>.json`),
      a new fetcher shape (the banking77 parquet stanza does not cover it), plus
      the per-row candidate-list builder (the mmlupro breadth machinery). Ride a
      follow-up lane. **LANDED 2026-11-10 (zed-64abf0dd@m3, the follow-up lane —
      T4+T5 as the ONE shared work item):** the direct-resolve fetch lane
      (`fetch_raw_json` + `fetch_bfcl_join` + `_raw_convert`, self-test scenario 4)
      + the six wave-1 files fetched + the gold join (all-or-nothing by exact id,
      ONE pinned upstream-typo normalization `live_multiple_1052-79-0` →
      `live_multiple_1052-279-0` — the gold id-chain …277/…278/…279 proves the
      eval file dropped a digit; any other mismatch still fails the fetch loud) +
      `build_bfcl` (per-row candidate-name options; the irrelevance tier appends
      the fixed `no_appropriate_function` option LAST and golds it; the pick tier
      never carries it; gold = full-name then UNIQUE-leaf match — the official ast
      evals' semantics; 3,033 fetched → 3,029 built, 4 empty-candidate
      live_irrelevance rows skipped) + the prepare pins (3,029 / 1,118 abstain
      golds). First read [Bench 180](../.benchmarks/180_bfcl_api_bank_suite_birth.md):
      laya[english] **0.5612** on ALL 3,029 (jdi_chance 0.3665, **jdi_skill
      +0.3075** — a POSITIVE first read for the pick shape). The corpus re-open
      stays the rethink-side synthesis route (unchanged).
- [x] T5 — API-Bank which-API tier (multi-turn refused wave 1). **CORPUS
      DISPOSITION 2026-11-09:** not probed further this session (T5 is last in
      the wave); the T3/T4 corpus wall is structural to the tools rung — expect
      the same disposition; probe its splits + corpus posture when its build
      lane opens. **PROBE + DISPOSITION 2026-11-09 (second session — the build
      lane opened via T3's corpus-optional posture):** the canonical mirror is
      `liminghao1630/API-Bank` (4,720 downloads, **MIT**, the original authors'
      org; configs default/{train,test}). ⛔ NOT datasets-server-servable: the
      raw JSON files have inconsistent columns across files — the /rows call
      dies with `DatasetGenerationCastError` (some files carry
      `id/file/expected_output`, others `output`), so the banking77 parquet
      stanza cannot fetch it. **T4+T5 are now ONE shared work item:** the
      direct-JSON fetch lane (the `/resolve/main/<file>.json` pattern BFCL
      also needs) + per-suite builders. The corpus posture rides the same
      T3 decision tree — the which-API tier's candidate functions are the
      test rows' content (grammar-not-answers), so a corpus for the modelless
      lane needs synthesis (the rethink-side routes) or the suite ships
      corpus-optional like when2call. Wave-1 reflex-side work ends here;
      the remaining builds are the follow-up lane's (the direct-JSON fetcher
      + the BFCL/API-Bank builders), filed as this issue's T4/T5 residuals.
      **LANDED 2026-11-10 (zed-64abf0dd@m3, the same lane as T4):**
      level-1-api (399) + level-2-api (135) fetched whole via the
      direct-resolve lane (JSON-array shape). ROW-LEVEL SCOREABLE-PARTIAL
      (the R2b law at row level, census 2026-11-10): every level-1 row's
      gold IS among its own candidates; the level-2 rows are
      ToolSearcher-gated and present exactly ONE candidate — the 45 rows
      whose expected_output names that candidate are choice-shaped (KEPT),
      the 90 whose expected_output names the DOWNSTREAM API drop (the
      official eval is generation-based there — never forced into the pick
      wire). `build_api_bank`: gold parsed per row from the row's own
      `expected_output` (the recorded transform), candidates parsed from
      the instruction's `API descriptions:` block, state = the instruction
      VERBATIM; **444 cases** built (399 + 45; the prepare pin). First read
      [Bench 180](../.benchmarks/180_bfcl_api_bank_suite_birth.md):
      laya[english] **0.6081** on ALL 444 — ≈ the majority class (jdi_chance
      0.6059, **jdi_skill +0.0057**), an honest near-zero-skill floor read
      at the corpus-less posture (the corpus re-open stays the rethink-side
      route, unchanged).
  **Wave-level corpus note (path (d), NEW this session):** the When2Call train
  data derives from **Salesforce/xlam-function-calling-60k (Apache-2.0)** — a
  DIFFERENT dataset from the test surface (the README's synthetic-data-gen
  section documents the lineage). A corpus for the four BEHAVIOR domains could
  be constructed from xlam rows WITHOUT touching When2Call's test rows — but
  that is corpus-synthesis work (the rethink ESC lane's machinery, the
  grammar-not-answers echo-gate on every pass), never a fetch stanza. Routed to
  the rethink-side follow-up; reflex only ships the harness suite once a
  corpus path exists.
- [x] T6 — per-suite paired read (modelless + laya on the new suites, the standard
      harness posture) + the results.json cells land in the feed; rethink's fold
      extends (060 R1d/R2a) — separate booking, gate, and record on the rethink side.
      **clinc150's reflex-side half DONE 2026-11-09** ([Bench 100](../.benchmarks/100_clinc150_suite_birth.md):
      modelless 0.3520 n=500 / laya 0.6500 n=100-capped, chance 0.0066, slice
      digests pinned; latency NOT QUOTABLE — the box-state disclosure + preflight
      refusal quoted in the record; re-run at the quiet posture before quoting
      any p50). The feed cells + the rethink fold (060 R1d/R2a) stay the
      rethink-side booking, gated on the T3/T4/T5 corpus decisions + owner-gated
      amo hours.

### Landscape note (2026-11-12, amo idle lane zed-c21902cf — wave-2/3 candidate, recorded not restated)

**PolyRoute (arXiv:2611.13843, 2026-11-10)** — a 12-language + code-mixed benchmark for TYPED-DECISION ROUTING with baselines and a leaderboard. Adjacent to the JDI family's typed_decisions lane (multilingual + code-mixed axes the current board rows don't carry). Not a wave-1/wave-2 adapter candidate as recorded (no fetch stanza scored, no licence read yet); the decision-index-beat lane owns any future board/lane read. Recorded here because this file is the adapters' triage home.
