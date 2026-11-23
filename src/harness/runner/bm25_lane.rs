//! Issue 121 A-pre/A2 — the harness half of the BM25 lexical floor lane
//! (REPORT-ONLY): the floor row beside the modelless lane, and the
//! redundancy cell against the standalone hashed-lexicon scorers.
//!
//! - **A2 floor row** — the index builds over the FULL registry pool
//!   (`inp.train`, the same docs the engine's corpora bind), answers the
//!   suite's TEST cases once, and lands `floor: bm25 <acc>%` beside the
//!   modelless row. A suite where the engine < bm25 is a CORPUS-DEFECT
//!   finding to file (issue 121 A2), never an auto-fix; the wire is
//!   additive-only and the exit code untouched.
//! - **A2 verdict** — every real floor row carries the paired read and its
//!   verdict label (PASS / FLOOR-PARITY@n / CORPUS-DEFECT) from
//!   [`crate::bm25_floor::floor_parity`]: the small-n parity clause is
//!   COMPUTED here, never typed into a record by hand.
//! - **A-pre redundancy** — the floor vs the STANDALONE hashed-lexicon
//!   scorers. ⚠ The modelless-engine-vs-NB/ridge margins CANNOT establish
//!   redundancy — the engine CONTAINS those scorers, so that comparison is
//!   engine-vs-its-own-parts. The honest comparator here is the
//!   `--nb-select` ladder's high-scale candidates (scale 64 ≈ the count
//!   tables decide alone — the ladder's own doc) measured on the SAME
//!   selection slice, plus (when `--ridge-select` ran) the ridge lane's
//!   selection rows. The cell reports per-suite paired deltas +
//!   cross-suite rank correlation (aggregated in the report renderer over
//!   all suites' rows) so A3 can adjudicate WITHOUT a second sweep. Runs
//!   on whatever levers the caller armed; a floor-only run carries
//!   `redundancy: None` (not a pass — an UNMEASURED, disclosed as such).
//!
//! Question-shape semantics (query text, option order) live at the ONE
//! home ([`crate::bm25_floor`]) — this module consumes them, never
//! re-derives them. `k1`/`b`/`top_k` are pinned there too (disclosed in
//! every row). Zero new dependencies.

use serde::Serialize;

use super::ModellessInput;
#[cfg(feature = "nb_ridge")]
use super::ridge_lane::RidgeSelection;
use crate::bm25_floor::{
    BM25_B, BM25_K1, BM25_TOP_K, Bm25Index, FloorParity, floor_contract_violation, floor_parity,
    floor_rows, question_options, question_query,
};

/// One suite's floor row + (when the selection levers ran) the A-pre
/// redundancy read. Rides `LaneResult.bm25_floor` (skip-when-None — the
/// additive-wire law).
#[derive(Debug, Clone, Serialize)]
pub struct Bm25FloorReport {
    /// Forced accuracy of the floor over the suite's test questions.
    pub accuracy: f64,
    /// Questions scored (the hard-metrics row count).
    pub n_questions: usize,
    /// Questions whose top-k vote was empty (they picked 0 — disclosed,
    /// never hidden; a high share means the corpus and the states share no
    /// vocabulary, which is itself a corpus finding).
    pub empty_votes: usize,
    /// Pinned constants (disclosure in-band — the row is self-describing).
    pub k1: f32,
    pub b: f32,
    pub top_k: usize,
    /// Docs indexed (the registry pool size the floor saw).
    pub n_docs: usize,
    /// Typed non-applicability (issue 125): `Some(reason)` = the suite is
    /// outside the floor's label-vote contract (per-question option spaces
    /// that are not the doc-label universe — the typed_decisions shape),
    /// so NO rows were computed and `accuracy` is a meaningless 0.0
    /// placeholder. Renderers print n/a; the engine-vs-floor corpus-defect
    /// comparison (issue 121 A2) must skip this row. Absent = a real
    /// floor row. Additive wire (skip-when-None).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_applicable: Option<String>,
    /// The A2 verdict (issue 121 + issue 126's parity clause): the paired
    /// engine-vs-floor read — `b`, `c`, the agreement cells, the exact
    /// McNemar p, the point delta + Wald 95% interval and the verdict
    /// label. Present on every real floor row; absent on a typed n/a row.
    /// Additive wire (skip-when-None).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parity: Option<FloorParity>,
    /// The A-pre redundancy read (None = the selection levers didn't run —
    /// UNMEASURED, never folded into a pass).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redundancy: Option<RedundancyRead>,
}

/// The A-pre redundancy cell for one suite: the floor vs the standalone
/// hashed-lexicon comparators measured on the SAME selection slice.
/// Cross-suite rank correlation aggregates in the renderer.
#[derive(Debug, Clone, Serialize)]
pub struct RedundancyRead {
    /// The nb ladder's highest-scale candidate accuracy (scale 64 = the
    /// count tables decide alone). None when `--nb-select` didn't run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nb_tables_alone_sel_acc: Option<f64>,
    /// The nb ladder's selected posture's sel-slice accuracy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nb_selected_sel_acc: Option<f64>,
    /// The ridge lane's best sel-slice accuracy (None when
    /// `--ridge-select` didn't run or the lane skipped).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ridge_sel_acc: Option<f64>,
}

/// Run the floor over the suite: index the pool, answer the test cases,
/// and (when the selection levers ran) collect the A-pre comparators.
/// `engine_rows` = the modelless lane's FORCED rows over the same test
/// cases in the same question order — the A2 verdict joins them with the
/// floor's rows. The ridge comparator rides the `nb_ridge` feature (its
/// lane's home); off-feature every caller passes `None` and the arm
/// compiles out.
#[cfg(feature = "nb_ridge")]
pub(crate) fn bm25_floor_pass(
    inp: &ModellessInput<'_>,
    engine_rows: &[(usize, Vec<f64>)],
    nb_selection: Option<&super::NbSelection>,
    ridge_selection: Option<&RidgeSelection>,
) -> Result<Bm25FloorReport, String> {
    bm25_floor_pass_inner(inp, engine_rows, nb_selection, ridge_selection)
}

#[cfg(not(feature = "nb_ridge"))]
pub(crate) fn bm25_floor_pass(
    inp: &ModellessInput<'_>,
    engine_rows: &[(usize, Vec<f64>)],
    nb_selection: Option<&super::NbSelection>,
    _ridge_none: (),
) -> Result<Bm25FloorReport, String> {
    bm25_floor_pass_inner(inp, engine_rows, nb_selection, None)
}

fn bm25_floor_pass_inner(
    inp: &ModellessInput<'_>,
    engine_rows: &[(usize, Vec<f64>)],
    nb_selection: Option<&super::NbSelection>,
    ridge_selection: Option<&RidgeSelection>,
) -> Result<Bm25FloorReport, String> {
    // Issue 125 — the floor-contract gate BEFORE any row is computed: a
    // suite whose questions carry per-question option spaces outside the
    // doc-label universe gets a typed n/a row, never the pick-0
    // degenerate number and never the gold-vs-arity panic. Measured on the
    // standing sweep (Bench 205 addendum): five suites carry a real floor
    // row (ag_news, emotion, xnli_en, banking77, semantic_defects — their
    // choice keys ARE the label universe); typed_decisions and
    // massive_intent_en (20-option subsets of a 59-label universe) are
    // typed n/a — a disclosed gap in the floor's coverage, never a pass.
    if let Some(reason) = floor_contract_violation(&inp.suite.cases, inp.labels) {
        eprintln!("    bm25-floor: n/a — {reason}");
        return Ok(Bm25FloorReport {
            accuracy: 0.0,
            n_questions: 0,
            empty_votes: 0,
            k1: BM25_K1,
            b: BM25_B,
            top_k: BM25_TOP_K,
            n_docs: 0,
            redundancy: redundancy_read(nb_selection, ridge_selection),
            not_applicable: Some(reason),
            parity: None,
        });
    }
    let index = Bm25Index::build(inp.train, inp.labels);
    let rows = floor_rows(&index, &inp.suite.cases, inp.labels);
    let hard = crate::harness::metrics::hard_metrics(&rows);
    let mut empty_votes = 0usize;
    for case in &inp.suite.cases {
        for q in &case.questions {
            let Some(options) = question_options(q, inp.labels) else {
                continue;
            };
            if options.is_empty() {
                continue;
            }
            if index.vote_is_empty(&question_query(case, q), BM25_TOP_K) {
                empty_votes += 1;
            }
        }
    }
    let redundancy = redundancy_read(nb_selection, ridge_selection);
    // The A2 verdict: join the floor's rows with the engine's forced rows.
    // A mis-joined read refuses the pass loud (the whole floor row fails)
    // rather than print a verdict over misaligned questions.
    let parity = floor_parity(engine_rows, &rows)?;
    Ok(Bm25FloorReport {
        accuracy: hard.accuracy,
        n_questions: rows.len(),
        empty_votes,
        k1: BM25_K1,
        b: BM25_B,
        top_k: BM25_TOP_K,
        n_docs: index.n_docs(),
        redundancy,
        not_applicable: None,
        parity: Some(parity),
    })
}

fn redundancy_read(
    nb_selection: Option<&super::NbSelection>,
    ridge_selection: Option<&super::ridge_lane::RidgeSelection>,
) -> Option<RedundancyRead> {
    if nb_selection.is_none() && ridge_selection.is_none() {
        return None;
    }
    Some(RedundancyRead {
        nb_tables_alone_sel_acc: nb_selection.and_then(|sel| {
            sel.candidates
                .iter()
                .filter(|c| c.scale >= 64.0)
                .map(|c| c.cal_acc)
                .fold(None, |best: Option<f64>, a| {
                    Some(best.map_or(a, |b: f64| b.max(a)))
                })
        }),
        nb_selected_sel_acc: nb_selection
            .and_then(|sel| {
                sel.candidates
                    .iter()
                    .find(|c| c.scale == sel.selected_scale)
            })
            .map(|c| c.cal_acc),
        ridge_sel_acc: ridge_selection.and_then(|r| {
            r.candidates
                .iter()
                .filter(|c| c.scale == r.selected_scale)
                .map(|c| c.cal_acc)
                .fold(None, |best: Option<f64>, a| {
                    Some(best.map_or(a, |b: f64| b.max(a)))
                })
        }),
    })
}
