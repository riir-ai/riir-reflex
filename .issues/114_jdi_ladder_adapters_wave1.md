# Issue 114 — JDI ladder adapters, wave 1: the decision-shaped tools three (BFCL / When2Call / API-Bank) + CLINC150 (retrieval)

**Status:** OPEN — plan-half of rethink Issue 060's R1a/R2a; adapters unbulleted, one suite at a time. **Wave state after the 2026-11-09 second session (lease zed-464454b5):** T1 probes DONE (all licences verified) · **T2 clinc150 LANDED** ([Bench 100](../.benchmarks/100_clinc150_suite_birth.md)) · **T3 when2call LANDED** ([Bench 101](../.benchmarks/101_when2call_suite_birth.md): the CORPUS-OPTIONAL posture chose re-open path (b) — the only self-serve path; the suite + the runner change landed, laya-only full-population read recorded, honestly below the majority floor) · T4/T5 UNBLOCKED by the same posture (adapter builds ride a follow-up lane; BFCL needs the direct-JSON fetch lane) · T6 clinc150+when2call reflex halves DONE; the rethink fold = the rethink-side booking. The wave's remaining DECISIONS are unchanged: the modelless corpus for T3 (paths (a)/(d) = owner/lane-gated spend or rethink-side synthesis) and the owner-gated amo booking.
**Claimed:** zed-464454b5 (JDI ESC serve capture monitoring), 1794138600, expires 2026-11-10T00:00:00Z

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
- [ ] T4 — BFCL pick-tier third (the arg-filling tiers REFUSED wave 1 — recorded as
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
      follow-up lane.
- [ ] T5 — API-Bank which-API tier (multi-turn refused wave 1). **CORPUS
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
