# Bench 186 — the wave-2 nDCG machinery + the BRIGHT/ToolRet lane verdicts: the scorer lands, the suite birth REFUSED on the 512-token wire (issue 114 T12)

**Date:** 2026-11-12 · **Repo:** riir-reflex (the scorer lands in the same commit as this record) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` T12 (the wave-2 nDCG lane — BRIGHT board row 36 + ToolRet board row 2; rethink 060 R2b)
**Posture:** probe/verdict record + pure-function machinery. **NO suite run exists — there are no latency or accuracy numbers to quote, and none are fabricated here.** The claim's suite-birth half resolved REFUSED-AS-SHAPED (the lane-feasibility verdict below); the claim's machinery half LANDED with known-answer tests only.

## What landed

1. **The nDCG@10 scorer** (`ndcg_at_10_metrics` + `NdcgCase` +
   `NdcgMetrics` in `src/harness/metrics.rs`, 6 known-answer tests) —
   the port of the board's retrieval scorer (`scoring/index.py`
   `score_query` + `random_ndcg_baseline`), the wave-2 triage's "one
   scorer buys both rows" item:
   - Documents rank by (score desc, doc_id asc) — the board's (-p, id)
     key, the identity order as the deterministic tie-break (pinned by
     the `score_ties_break_by_ascending_doc_id_not_input_order` test:
     input order must not matter, only the identity order does).
   - DCG@10 folds the graded gain 2^g−1 over 1/log2(rank+2) for the
     top 10 (binary gains are the g ∈ {0,1} case — the BRIGHT ladder
     stays representable); IDCG is the same fold over gains sorted
     descending, also truncated at 10.
   - **Answerable filter at the scorer:** a case whose every gain is 0
     has `idcg == 0` and is SKIPPED from the means, counted in
     `skipped_zero_positive` (the board's answerable-queries filter,
     ≥1 relevant of the candidates) — never folded as a fabricated 0.0;
     zero answerable cases → `None` (nothing measured, never a rate).
   - `random_baseline` is the board's CLOSED FORM for a uniform random
     permutation: E[gain at any rank] = mean gain over all n docs, so
     E[DCG@10] = mean_gain · Σ_{i<min(10,n)} 1/log2(i+2), divided by
     the case's IDCG (the §B3 re-derivation law — the board's pinned
     chance values 0.116 / 0.1341 stay reference columns).
   - **NO lane seam** (deliberate): `case_exact_metrics` got a
     `case_exact_of` seam because home_appliance CONSUMES it; nothing
     consumes the nDCG scorer (the verdict below), and a
     `CASE_EXACT_SUITES`-style gate with an empty suite set would be a
     green-zero — the seam lands WITH the first servable consumer.

2. **BRIGHT licence probe: PASS.** `xlangai/BRIGHT` (the board's
   arXiv:2407.12883 source) carries **cc-by-4.0** with the hub tag and
   the card data agreeing (one authoritative surface family); the
   documents tree is ≈470 MB — feasible to fetch. The
   `retrieval-subsets.json` pin names BRIGHT 550 kept group_ids of
   1,297 — the same release-v2.1 subset file that cryptographically
   pinned home_appliance's kept set (T11).

3. **THE LANE-FEASIBILITY VERDICT: the suite birth is REFUSED-AS-SHAPED
   — no suite exists.** The board's BRIGHT wire is per-(query, doc)
   `Score` questions: each question's sequence carries the query text,
   ONE full-text candidate document, and the score criteria. The laya
   lane's sequence budget truncates:
   - `riir-infer/crates/riir-infer-laya/src/laya/config.rs` L20-21:
     the `english` checkpoint is ModernBERT-large with **`max_len`
     512** (L59-60: "total sequence budget (512 EN / 1024 ML+TD)") —
     the serving posture the reflex laya lane boots.
   - `.../laya/tokenize.rs` L245-254 (`build_sequence`): the serialized
     state is truncated to the remaining room, then the whole id
     sequence to `max_len` — the overflow is dropped SILENTLY.
   - BRIGHT is reasoning-intensive retrieval BY CONSTRUCTION (the
     triage's own note: gold docs share little vocabulary with the
     query); its documents (arXiv abstracts, stackexchange answers,
     math excerpts) routinely exceed a few hundred tokens ALONE. With
     query + doc + state riding one 512-token sequence, the truncated
     tail drops exactly the evidence the score must weigh — **the lane
     would measure truncation, not retrieval capability.** The
     modelless half has no lawful signal either: corpus-optional
     ranking over full-text relevance is the grammar-not-answers
     class (no corpus is the model for a doc the lane never saw).
     The 1024 checkpoints (multilingual / typed) do not rescue it —
     long BRIGHT documents exceed 1024 too, and neither is this lane's
     serving posture.
   - **The named re-opens (never silent drops):** (a) a long-context
     encoder posture on the laya lane (or a dedicated dense-embedder
     lane — the riir-rethink Research 014 rerank-tier falsifier is the
     natural vehicle); (b) an OWNER call re-scoping the board wire to
     excerpted candidates — a transform the board does not sanction,
     never filed unilaterally. Until one lands, BRIGHT stays
     SCOREABLE-PER-MACHINERY (the scorer above) and UNSERVABLE on the
     current wire; the board's row stays unperturbed.

4. **ToolRet (board row 2): UNDECIDED.** The candidate pool is an
   AGGREGATE of upstream tool-documentation sources — the licence is
   not one grant but many upstream terms, and the board's own suite.md
   row punts the same way. Aggregate-of-upstream-terms needs a
   per-source licence census before any fetch could lawfully start;
   that census is its own probe lane (the same shape the ACOS/VAST
   refusals went through, one source at a time). The nDCG scorer buys
   ToolRet's row shape when (if) the licence census clears AND a
   servable lane exists — the same 512-token verdict applies to its
   full-text tool descriptions.

## The machinery's coverage after this landing

- **BRIGHT / ToolRet (nDCG@10)**: scorer LANDED; both suite births
  verdict-refused as shaped (BRIGHT on the token-budget trace, ToolRet
  licence-undecided first). The rows leave wave-2's residue and join
  the RE-OPEN ledger.
- **iSarcasm track B (the multilabel envelope)**: still open — the
  multilabel option scorer, next claimable.
- **The Arabic tracks**: the posture decision (the triage's
  English-first posture stands).
- **ACOS / VAST**: REFUSED (no licence); **NLI4CT**: UNDECIDED
  (licence-pending) — owner-call re-opens, unchanged.

## Gates

`cargo test -p riir-reflex --lib metrics` — 23 passed (the 6 new nDCG
known-answer tests green); `cargo clippy --all-targets -- -D warnings`
clean; rustfmt per-file (never `cargo fmt --all`). No box-state record
is required — no run, no numbers.
