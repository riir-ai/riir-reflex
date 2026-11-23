//! riir-rethink Issue 065 — the frozen-transfer probe (REPORT-ONLY runner
//! half; the reflex family's "control/identity reds refuse" law applies).
//!
//! WHAT IT MEASURES (and what it does not). The transposition of the
//! Cross-Context Neural Transport instrument (Ndubuaku, NeurIPS 2026
//! NeuReps WS, PMLR; full protocol verified verbatim at the author's
//! companion blog, henryndubuaku.fyi/blog/tuning-fit-on-pictures-tested-
//! on-films.html, 2026-11-21) onto the decision harness's cal→test axis:
//! fit per-class heads on the cal front, FREEZE, score Pearson transfer
//! per class on the eval split, normalize by a within-target oracle into
//! the paper's transport ratio, and report both directions. ⚠ THE PAPER'S
//! "CONTEXT" WORD DOES NOT TRANSFER: cal front and eval split are the
//! train/eval halves of ONE distribution, so this measures cal→test
//! SAMPLE transfer (fit support + overfit decay), never a picture→film
//! context shift. The docs, the report and every number below say
//! "cal→test"; nothing here claims a context-shift read.
//!
//! The paper's protocol pieces carried verbatim:
//! - **Frozen scoring**: the frozen prediction uses NO target-fitted
//!   coefficients (the paper: "the first term alone with no gain
//!   fitted"); the score is Pearson correlation on held-out units (the
//!   paper: "Pearson correlation is the score" over ten held-out
//!   segments; ours: per class over the target rows).
//! - **Transport ratio**: frozen squared correlation ÷ an ORACLE fit on
//!   the target's squared correlation (the paper's definition); the
//!   paper's reported ratios are "cross-validated and regularised", so
//!   the ratio's denominator is the CV oracle — and the ratio is NOT
//!   bounded to [0, 1] out of sample (the paper says so; never clamped).
//! - **Calibrated/frozen marking**: every target-fitted quantity is
//!   labelled as such (`r_oracle_*`), never folded into the frozen arm
//!   (the paper marks gain-fitted analyses "calibrated").
//! - **Tentative directionality**: the paper flags its picture→film vs
//!   film→picture asymmetry tentative because the branches differ in
//!   feature-estimation quality, design width and response reliability.
//!   Ours differ in FIT SUPPORT (cal 200 vs eval 500 rows), so any
//!   directionality gap here is a SUPPORT ARTIFACT by default — a gap is
//!   a finding only if its sign survives the SUPPORT-MATCHED refit.
//!
//! Estimator delta (disclosed, deliberate): the paper fits per-vertex
//! ridge (closed-form normal equations) on encoding models; we fit the
//! stack's OWN per-class head family ([`crate::label_heads::LabelHeads`],
//! deterministic one-vs-all logistic over the hashed-bag features) — the
//! arming-relevant estimator, not a neuroscience replication. Point-
//! biserial correlation (score vs a rare-class gold indicator) yields
//! small r values at 1/77 prevalence; every ratio is printed beside its
//! r_frozen and r_oracle, never alone.
//!
//! Controls (exit 1 AFTER records are written, never a quiet pass):
//! - **NULL**: the gold of the TARGET side only is rotated by
//!   [`NULL_ROTATE`] row positions (cyclic, deterministic) and the frozen
//!   scores are re-scored against the rotated indicators. ⚠ TARGET-ONLY
//!   IS LOAD-BEARING: on the label-interleaved stratified front, rotating
//!   BOTH sides composes into a consistent relabel π (row i's gold moves
//!   to row i+1, and on a round-robin front that is label l → l+1), which
//!   the frozen arm learns, so the null transfers as well as the real run
//!   and falsely reds (the round-2 verdict's binding condition 1; the
//!   negative-arm unit test demonstrates exactly this hazard).
//! - **IDENTITY** (the paper's Lemma-2 shape): median in-sample-oracle r
//!   on the target ≥ median frozen r across classes. The in-sample
//!   oracle (fit on ALL target rows, scored on the same rows) is the
//!   Lemma-2 bound object; the CV oracle is the advisory denominator and
//!   is NEVER gated (a CV oracle on a small target can legitimately lose
//!   to a frozen fit from a larger source — the paper: "nothing forces it
//!   between 0 and 1").
//!
//! UNDECIDED (support) is a first-class per-direction verdict: at the
//! harness's thin cal slices (banking77: 200 rows / 77 classes ≈ 2.6
//! rows/class) most classes mask on support and a direction can carry no
//! ratio median at all. That outcome is recorded as such — it IS the
//! finding "the cal slice is too thin to measure cal→test transfer at
//! per-class granularity" — and never rescued by widening the split here.

use serde::Serialize;

use super::{
    Prepared, RunOptions, SUITES, git_sha, hostname_refusing_unknown, iso8601_utc, prepare,
};
use crate::embed::{EMBED_DIM, Embedder};
use crate::harness::suites::QKind;
use crate::label_heads::LabelHeads;

/// The probe's const-generic label ceiling — a suite whose option-key
/// universe exceeds it is SKIPPED with a named line (counted in the
/// report meta), never silently truncated. clinc150's 151 and banking77's
/// 77 fit; bfcl's per-row candidate union exceeds it.
pub const MAX_LABELS: usize = 256;

/// CV folds for the transport-ratio's oracle denominator (the paper's
/// "cross-validated and regularised" posture; contiguous folds over the
/// stratified front's interleaved row order).
pub const CV_FOLDS: usize = 5;

/// A class needs at least this many rows in BOTH contexts to enter the
/// ratio medians (the noise-ceiling-mask analogue; the paper restricts to
/// within-participant noise-ceiling vertices).
pub const MIN_SUPPORT_PER_CLASS: usize = 3;

/// A class whose |CV-oracle r| falls under this is masked from the ratio
/// median (denominator near zero); its r values stay in the table with
/// `ratio: null`.
pub const MIN_ORACLE_R: f64 = 0.05;

/// Below this many classes surviving the mask, a direction's median is
/// NOT reported — the direction reads `UNDECIDED (support)` (never a
/// median of a handful).
pub const MIN_RATIO_CLASSES: usize = 8;

/// The support-matched arm caps each class's source rows at this (the
/// deterministic first-k in row order; both directions then fit at the
/// same per-class support).
pub const MATCH_CAP: usize = 8;

/// The null rotation's row offset (target-only; see the module docs for
/// why both-side rotation is the hazard).
pub const NULL_ROTATE: usize = 1;

/// The identity control's SGD-convergence noise allowance (the control
/// catches BACKWARDS instruments, which miss by large margins; near the
/// separability ceiling the frozen-vs-in-sample median gap is 4th-decimal
/// fitting noise, and pinning zero there would red on noise).
pub const IDENTITY_EPS: f64 = 0.01;

/// The probe's default suite set: the two ESC-armed suites whose Bench 089
/// caveat this instrument informs. `--suites` overrides.
pub const TRANSFER_PROBE_SUITES: &[&str] = &["massive_intent_en", "banking77"];

/// The report's honesty block (rides the JSON meta and the MD header).
pub const INSTRUMENT_NOTE: &str = "cal->test SAMPLE transfer: one distribution, two halves \
 (stratified train front vs eval split) — NOT a context shift. Frozen per-class heads \
 (the shipped LabelHeads one-vs-all family over hashed-bag features), Pearson score on \
 the target rows, transport ratio = frozen r^2 / CV-oracle r^2 (unbounded, per the \
 paper), both directions, support-masked medians disclosed beside unmasked. \
 Directionality is a support artifact by default: a gap is a finding only if its sign \
 survives the support-matched refit. The paper flags its own directionality tentative; \
 that flag is carried.";

/// Ratios whose |CV-oracle r| sits under this are not "computable" for
/// the unmasked median (division by a noise floor is not a number).
const MIN_COMPUTABLE_DENOM: f64 = 1e-6;

// ───────────────────────────────────────────── report shapes ──

/// One class's row in a direction table. The r values are the load-bearing
/// columns; `ratio` is a derived view and is `None` whenever masked.
#[derive(Debug, Clone, Serialize)]
pub struct ClassRow {
    pub label: String,
    pub n_src: usize,
    pub n_tgt: usize,
    pub r_frozen: Option<f64>,
    pub r_oracle_insample: Option<f64>,
    pub r_oracle_cv: Option<f64>,
    /// `r_frozen² / r_oracle_cv²` — `None` when masked (support, oracle
    /// floor) or incomputable (zero-variance side).
    pub ratio: Option<f64>,
    /// "support" | "oracle" — why the ratio is masked. `None` = carried.
    pub masked_reason: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectionReport {
    /// "cal->test" | "test->cal".
    pub direction: String,
    /// "MEASURED" | "UNDECIDED (support)".
    pub verdict: String,
    pub n_classes: usize,
    pub n_masked_support: usize,
    pub n_masked_oracle: usize,
    pub n_ratio_classes: usize,
    /// Median transport ratio over the mask-carrying classes.
    pub median_ratio_masked: Option<f64>,
    /// Median over EVERY computable ratio (denominator nonzero), floors
    /// aside — disclosed so a floor choice cannot hide the mass.
    pub median_ratio_unmasked: Option<f64>,
    /// Median frozen r over support-carrying classes.
    pub median_r_frozen: Option<f64>,
    pub per_class: Vec<ClassRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NullControl {
    /// Median |r_frozen| over support-carrying classes, real gold.
    pub real_median_abs_r: Option<f64>,
    /// Same classes, gold rotated by [`NULL_ROTATE`] in the TARGET side
    /// only (the source fit stays on real labels).
    pub null_median_abs_r: Option<f64>,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityControl {
    /// Median in-sample-oracle r on the target (the Lemma-2 bound object).
    pub median_insample_oracle: Option<f64>,
    /// Median frozen r over the SAME class set.
    pub median_r_frozen: Option<f64>,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedReport {
    /// Classes entering the matched medians (m(d) ≥
    /// [`MIN_SUPPORT_PER_CLASS`], capped at [`MATCH_CAP`], and surviving
    /// both directions' ratio masks).
    pub n_classes: usize,
    pub median_ratio_cal_to_test: Option<f64>,
    pub median_ratio_test_to_cal: Option<f64>,
    /// The raw-median gap over the same intersection (disclosure).
    pub gap_raw: Option<f64>,
    /// The matched-median gap (support-matched refit both directions).
    pub gap_matched: Option<f64>,
    /// `Some(true)` only when BOTH gaps are finite and share a sign —
    /// the only posture under which a directionality gap is reported as a
    /// finding; `None` when either side is UNDECIDED or the intersection
    /// is under [`MIN_RATIO_CLASSES`].
    pub gap_survives: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuiteReport {
    pub name: String,
    pub n_cal_rows: usize,
    pub n_test_rows: usize,
    pub n_classes: usize,
    /// Rows whose gold resolved outside the option-key universe (skipped,
    /// counted — the coverage audit's sibling disclosure).
    pub n_unknown_gold: usize,
    pub directions: [DirectionReport; 2],
    pub null: NullControl,
    pub identity: IdentityControl,
    pub matched: MatchedReport,
    pub controls_pass: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkippedSuite {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferProbeMeta {
    pub date_utc: String,
    /// `None` when the build did not carry git metadata (the house
    /// `git_sha()` shape — disclosure, never a fabricated sha).
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub suites: Vec<String>,
    pub skipped: Vec<SkippedSuite>,
    pub max_labels: usize,
    pub cv_folds: usize,
    pub min_support_per_class: usize,
    pub min_oracle_r: f64,
    pub min_ratio_classes: usize,
    pub match_cap: usize,
    pub null_rotate: usize,
    pub instrument_note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferProbeOutput {
    pub meta: TransferProbeMeta,
    pub suites: Vec<SuiteReport>,
    /// Every suite's controls passed — the probe family's exit-code
    /// coupling (exit 1 on red; the records are already on disk).
    pub controls_ok: bool,
}

// ───────────────────────────────────────────── rows ──

/// One scored question row: the case's hashed-bag feature + gold class.
struct ProbeRow {
    feature: [f32; EMBED_DIM],
    class: usize,
}

/// The two contexts + label universe the probe scores.
struct SuiteRows {
    labels: Vec<String>,
    cal: Vec<ProbeRow>,
    test: Vec<ProbeRow>,
}

/// One context's derived scoring inputs.
struct Side {
    /// Per-class feature lists (label-sized; padded to [`MAX_LABELS`] at
    /// fit time).
    per_class: Vec<Vec<[f32; EMBED_DIM]>>,
    /// All features in row order (the scoring population).
    feats: Vec<[f32; EMBED_DIM]>,
    /// Row order → gold class (the CV fold assembly keys on this).
    classes: Vec<usize>,
    /// Per-class gold indicators aligned with `feats`.
    inds: Vec<Vec<f64>>,
    /// Per-class support (`per_class[d].len()`).
    n_per: Vec<usize>,
}

fn build_side(rows: &[ProbeRow], n_labels: usize) -> Side {
    let mut per_class: Vec<Vec<[f32; EMBED_DIM]>> = vec![Vec::new(); n_labels];
    let mut feats: Vec<[f32; EMBED_DIM]> = Vec::with_capacity(rows.len());
    let mut classes: Vec<usize> = Vec::with_capacity(rows.len());
    for row in rows {
        per_class[row.class].push(row.feature);
        feats.push(row.feature);
        classes.push(row.class);
    }
    let mut inds: Vec<Vec<f64>> = vec![vec![0.0; feats.len()]; n_labels];
    for (i, row) in rows.iter().enumerate() {
        inds[row.class][i] = 1.0;
    }
    let n_per: Vec<usize> = per_class.iter().map(Vec::len).collect();
    Side {
        per_class,
        feats,
        classes,
        inds,
        n_per,
    }
}

/// The label-count guard (standalone so the ceiling is unit-testable
/// without a `Prepared`).
fn check_label_count(labels: &[String]) -> Result<(), String> {
    if labels.len() > MAX_LABELS {
        return Err(format!(
            "label universe {} > MAX_LABELS {MAX_LABELS}",
            labels.len()
        ));
    }
    Ok(())
}

/// Extract the probe rows from a prepared suite: one row per CHOICE
/// question with resolvable gold; the feature is the case's serialized
/// state embedding (multi-question cases share the case feature —
/// disclosed; the ESC-armed suites are one question per case). Gold
/// resolves through the case's OWN criteria-key order (robust to per-row
/// universes); a key outside the label universe is counted, not fatal.
fn rows_from_prepared(p: &Prepared) -> Result<(SuiteRows, usize), String> {
    check_label_count(&p.labels)?;
    let label_idx: std::collections::HashMap<&str, usize> = p
        .labels
        .iter()
        .enumerate()
        .map(|(i, l)| (l.as_str(), i))
        .collect();
    let embedder = Embedder;
    let mut unknown_gold = 0usize;
    let mut rows_of = |cases: &[crate::harness::suites::SuiteCase],
                       strs: &[String]|
     -> Result<Vec<ProbeRow>, String> {
        let mut out = Vec::with_capacity(cases.len());
        for (ci, case) in cases.iter().enumerate() {
            let Some(text) = strs.get(ci) else {
                return Err(format!(
                    "case {}: no state string at position {ci} (misaligned envelope)",
                    case.id
                ));
            };
            let mut feature = [0.0f32; EMBED_DIM];
            embedder.embed_into(text.as_bytes(), &mut feature);
            for (qi, q) in case.questions.iter().enumerate() {
                if !matches!(q.kind, QKind::Choice) {
                    continue;
                }
                let Some(obj) = q.criteria.as_object() else {
                    continue;
                };
                let Some(g) = case.gold.get(qi) else {
                    continue;
                };
                let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
                let Some(&key) = keys.get(g.idx) else {
                    continue;
                };
                let Some(&class) = label_idx.get(key) else {
                    unknown_gold += 1;
                    continue;
                };
                out.push(ProbeRow { feature, class });
            }
        }
        Ok(out)
    };
    let cal = rows_of(&p.cal_cases, &p.cal_state_strs)?;
    let test = rows_of(&p.suite.cases, &p.state_strs)?;
    Ok((
        SuiteRows {
            labels: p.labels.clone(),
            cal,
            test,
        },
        unknown_gold,
    ))
}

// ───────────────────────────────────────────── stats ──

/// Pearson correlation, f64 sequential folds over the given row order
/// (the determinism law: plain `+=`, fixed order, no SIMD lane). `None`
/// when either side is constant or the lengths disagree.
fn pearson(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.len() < 2 {
        return None;
    }
    let n = x.len();
    let mut sx = 0.0f64;
    let mut sy = 0.0f64;
    for i in 0..n {
        sx += x[i];
        sy += y[i];
    }
    let mx = sx / n as f64;
    let my = sy / n as f64;
    let mut sxy = 0.0f64;
    let mut sxx = 0.0f64;
    let mut syy = 0.0f64;
    for i in 0..n {
        let dx = x[i] - mx;
        let dy = y[i] - my;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if !matches!((sxx, syy), (a, b) if a > 0.0 && b > 0.0) {
        return None;
    }
    Some(sxy / (sxx.sqrt() * syy.sqrt()))
}

/// Median by ascending `total_cmp` (odd → middle; even → mean of the two
/// middings). Empty → `None`.
fn median(mut vals: Vec<f64>) -> Option<f64> {
    if vals.is_empty() {
        return None;
    }
    vals.sort_by(f64::total_cmp);
    let n = vals.len();
    if n % 2 == 1 {
        Some(vals[n / 2])
    } else {
        Some((vals[n / 2 - 1] + vals[n / 2]) / 2.0)
    }
}

// ───────────────────────────────────────────── fits ──

fn fit_padded(per_class: &[Vec<[f32; EMBED_DIM]>]) -> LabelHeads<MAX_LABELS, EMBED_DIM> {
    debug_assert!(per_class.len() <= MAX_LABELS);
    let mut rows: Vec<Vec<[f32; EMBED_DIM]>> = Vec::with_capacity(MAX_LABELS);
    for d in 0..MAX_LABELS {
        rows.push(per_class.get(d).cloned().unwrap_or_default());
    }
    LabelHeads::fit(&rows)
}

/// Frozen scores for every class over one context's rows, in row order.
fn score_all(
    heads: &LabelHeads<MAX_LABELS, EMBED_DIM>,
    feats: &[[f32; EMBED_DIM]],
    n_labels: usize,
) -> Vec<Vec<f64>> {
    (0..n_labels)
        .map(|d| {
            feats
                .iter()
                .map(|q| heads.raw_logit(d, q) as f64)
                .collect::<Vec<f64>>()
        })
        .collect()
}

/// Contiguous CV fold of row `i` over `n` rows (deterministic; the small
/// tail folds into the last fold).
fn fold_of(i: usize, n: usize) -> usize {
    i * CV_FOLDS / n.max(1)
}

// ───────────────────────────────────────────── direction cell ──

#[allow(clippy::too_many_lines)]
fn direction_cell(
    direction: &'static str,
    src: &Side,
    tgt: &Side,
    labels: &[String],
    frozen_scores: &[Vec<f64>],
) -> DirectionReport {
    let n_labels = labels.len();
    let n_tgt_rows = tgt.feats.len();

    // In-sample oracle on the target (the Lemma-2 control object).
    let heads_in = fit_padded(&tgt.per_class);
    let scores_in = score_all(&heads_in, &tgt.feats, n_labels);

    // CV oracle on the target (the ratio's denominator): contiguous folds
    // over the target's row order; each fold scores through a fit that
    // never saw it. A fold absent when n < CV_FOLDS simply scores nothing.
    let mut scores_cv: Vec<Vec<f64>> = vec![vec![0.0; n_tgt_rows]; n_labels];
    for k in 0..CV_FOLDS {
        let mut train: Vec<Vec<[f32; EMBED_DIM]>> = vec![Vec::new(); n_labels];
        let mut fold_rows: Vec<usize> = Vec::new();
        for (i, feat) in tgt.feats.iter().enumerate() {
            if fold_of(i, n_tgt_rows) == k {
                fold_rows.push(i);
            } else {
                train[tgt.classes[i]].push(*feat);
            }
        }
        if fold_rows.is_empty() {
            continue;
        }
        let heads_k = fit_padded(&train);
        for &i in &fold_rows {
            let feat = &tgt.feats[i];
            for (d, slot) in scores_cv.iter_mut().enumerate().take(n_labels) {
                slot[i] = heads_k.raw_logit(d, feat) as f64;
            }
        }
    }

    let mut per_class: Vec<ClassRow> = Vec::with_capacity(n_labels);
    let mut n_masked_support = 0usize;
    let mut n_masked_oracle = 0usize;
    let mut carried_ratios: Vec<f64> = Vec::new();
    let mut all_ratios: Vec<f64> = Vec::new();
    let mut support_frozen: Vec<f64> = Vec::new();

    for d in 0..n_labels {
        let n_src = src.n_per[d];
        let n_tgt = tgt.n_per[d];
        let r_frozen = pearson(&frozen_scores[d], &tgt.inds[d]);
        let r_insample = pearson(&scores_in[d], &tgt.inds[d]);
        let r_cv = pearson(&scores_cv[d], &tgt.inds[d]);

        let mut masked_reason: Option<&'static str> = None;
        if n_src < MIN_SUPPORT_PER_CLASS || n_tgt < MIN_SUPPORT_PER_CLASS {
            masked_reason = Some("support");
            n_masked_support += 1;
        } else {
            let under_floor = match r_cv {
                Some(r) => r.abs() < MIN_ORACLE_R,
                None => true,
            };
            if under_floor {
                masked_reason = Some("oracle");
                n_masked_oracle += 1;
            }
        }

        // The computable set (denominator off the noise floor) feeds the
        // unmasked median; the carried set (mask None) feeds the masked
        // median and the table's ratio column.
        if let (Some(f), Some(c)) = (r_frozen, r_cv)
            && c.abs() >= MIN_COMPUTABLE_DENOM
        {
            let r = (f * f) / (c * c);
            all_ratios.push(r);
            if masked_reason.is_none() {
                carried_ratios.push(r);
            }
        }
        if n_tgt >= MIN_SUPPORT_PER_CLASS
            && let Some(f) = r_frozen
        {
            support_frozen.push(f);
        }

        // The ratio column: carried classes only (masked rows keep their
        // r values with `ratio: null`).
        let ratio = if masked_reason.is_none() {
            match (r_frozen, r_cv) {
                (Some(f), Some(c)) if c.abs() >= MIN_COMPUTABLE_DENOM => Some((f * f) / (c * c)),
                _ => None,
            }
        } else {
            None
        };
        per_class.push(ClassRow {
            label: labels[d].clone(),
            n_src,
            n_tgt,
            r_frozen,
            r_oracle_insample: r_insample,
            r_oracle_cv: r_cv,
            ratio,
            masked_reason,
        });
    }

    let n_ratio_classes = carried_ratios.len();
    let verdict = if n_ratio_classes >= MIN_RATIO_CLASSES {
        "MEASURED"
    } else {
        "UNDECIDED (support)"
    };
    DirectionReport {
        direction: direction.to_string(),
        verdict: verdict.to_string(),
        n_classes: n_labels,
        n_masked_support,
        n_masked_oracle,
        n_ratio_classes,
        median_ratio_masked: median(carried_ratios),
        median_ratio_unmasked: median(all_ratios),
        median_r_frozen: median(support_frozen),
        per_class,
    }
}

// ───────────────────────────────────────────── suite cell ──

#[allow(clippy::too_many_lines)]
fn probe_suite(name: &str, rows: &SuiteRows, unknown_gold: usize) -> SuiteReport {
    let n_labels = rows.labels.len();
    let cal_side = build_side(&rows.cal, n_labels);
    let test_side = build_side(&rows.test, n_labels);

    // Frozen fits: one per direction, computed once (the cal->test scores
    // are reused by the null control — the source fit stays on REAL
    // labels; only the target indicators rotate).
    let heads_c2t = fit_padded(&cal_side.per_class);
    let scores_c2t = score_all(&heads_c2t, &test_side.feats, n_labels);
    let heads_t2c = fit_padded(&test_side.per_class);
    let scores_t2c = score_all(&heads_t2c, &cal_side.feats, n_labels);

    let c2t = direction_cell(
        "cal->test",
        &cal_side,
        &test_side,
        &rows.labels,
        &scores_c2t,
    );
    let t2c = direction_cell(
        "test->cal",
        &test_side,
        &cal_side,
        &rows.labels,
        &scores_t2c,
    );

    // NULL control (target-only rotation — binding condition 1).
    let null_ctrl = {
        let n_test = test_side.feats.len();
        let mut real: Vec<f64> = Vec::new();
        let mut null: Vec<f64> = Vec::new();
        for (d, scores_d) in scores_c2t.iter().enumerate().take(n_labels) {
            if test_side.n_per[d] < MIN_SUPPORT_PER_CLASS {
                continue;
            }
            if let Some(f) = c2t.per_class[d].r_frozen {
                real.push(f.abs());
            }
            let rot: Vec<f64> = (0..n_test)
                .map(|i| test_side.inds[d][(i + NULL_ROTATE) % n_test])
                .collect();
            if let Some(r) = pearson(scores_d, &rot) {
                null.push(r.abs());
            }
        }
        let real_median = median(real);
        let null_median = median(null);
        let pass = matches!((real_median, null_median), (Some(a), Some(b)) if a > b);
        NullControl {
            real_median_abs_r: real_median,
            null_median_abs_r: null_median,
            pass,
        }
    };

    // IDENTITY control (in-sample oracle vs frozen, Lemma-2 shape).
    let identity_ctrl = {
        let mut ins: Vec<f64> = Vec::new();
        let mut fro: Vec<f64> = Vec::new();
        for d in 0..n_labels {
            if test_side.n_per[d] < MIN_SUPPORT_PER_CLASS {
                continue;
            }
            if let Some(v) = c2t.per_class[d].r_oracle_insample {
                ins.push(v);
            }
            if let Some(v) = c2t.per_class[d].r_frozen {
                fro.push(v);
            }
        }
        let m_in = median(ins);
        let m_fro = median(fro);
        let pass = matches!((m_in, m_fro), (Some(a), Some(b)) if a >= b - IDENTITY_EPS);
        IdentityControl {
            median_insample_oracle: m_in,
            median_r_frozen: m_fro,
            pass,
        }
    };

    // SUPPORT-MATCHED arm (binding round-1 change 3): both directions fit
    // at the same per-class support; a directionality gap is a finding
    // only if its sign survives the match.
    let matched = {
        let mut trunc_cal: Vec<Vec<[f32; EMBED_DIM]>> = vec![Vec::new(); n_labels];
        let mut trunc_test: Vec<Vec<[f32; EMBED_DIM]>> = vec![Vec::new(); n_labels];
        for d in 0..n_labels {
            let m = cal_side.n_per[d].min(test_side.n_per[d]);
            if m < MIN_SUPPORT_PER_CLASS {
                continue;
            }
            let m = m.min(MATCH_CAP);
            trunc_cal[d] = cal_side.per_class[d][..m].to_vec();
            trunc_test[d] = test_side.per_class[d][..m].to_vec();
        }
        let heads_mc = fit_padded(&trunc_cal);
        let scores_mc = score_all(&heads_mc, &test_side.feats, n_labels);
        let heads_mt = fit_padded(&trunc_test);
        let scores_mt = score_all(&heads_mt, &cal_side.feats, n_labels);

        let mut m_c2t: Vec<Option<f64>> = vec![None; n_labels];
        let mut m_t2c: Vec<Option<f64>> = vec![None; n_labels];
        for d in 0..n_labels {
            let r_mc = pearson(&scores_mc[d], &test_side.inds[d]);
            if c2t.per_class[d].masked_reason.is_none()
                && let (Some(f), Some(c)) = (r_mc, c2t.per_class[d].r_oracle_cv)
                && c.abs() >= MIN_COMPUTABLE_DENOM
            {
                m_c2t[d] = Some((f * f) / (c * c));
            }
            let r_mt = pearson(&scores_mt[d], &cal_side.inds[d]);
            if t2c.per_class[d].masked_reason.is_none()
                && let (Some(f), Some(c)) = (r_mt, t2c.per_class[d].r_oracle_cv)
                && c.abs() >= MIN_COMPUTABLE_DENOM
            {
                m_t2c[d] = Some((f * f) / (c * c));
            }
        }

        // The comparable intersection: both directions' masks carried AND
        // both matched ratios computed.
        let inter: Vec<usize> = (0..n_labels)
            .filter(|&d| {
                c2t.per_class[d].masked_reason.is_none()
                    && t2c.per_class[d].masked_reason.is_none()
                    && m_c2t[d].is_some()
                    && m_t2c[d].is_some()
            })
            .collect();
        let n_inter = inter.len();
        let enough = n_inter >= MIN_RATIO_CLASSES;
        let med_c2t = if enough {
            median(inter.iter().filter_map(|&d| m_c2t[d]).collect())
        } else {
            None
        };
        let med_t2c = if enough {
            median(inter.iter().filter_map(|&d| m_t2c[d]).collect())
        } else {
            None
        };
        let raw_c2t = median(
            inter
                .iter()
                .filter_map(|&d| c2t.per_class[d].ratio)
                .collect(),
        );
        let raw_t2c = median(
            inter
                .iter()
                .filter_map(|&d| t2c.per_class[d].ratio)
                .collect(),
        );
        let gap_raw = match (raw_c2t, raw_t2c) {
            (Some(a), Some(b)) => Some(a - b),
            _ => None,
        };
        let gap_matched = match (med_c2t, med_t2c) {
            (Some(a), Some(b)) => Some(a - b),
            _ => None,
        };
        let gap_survives = match (gap_raw, gap_matched) {
            (Some(a), Some(b)) => {
                Some(a.signum() == b.signum() && a.abs() > 1e-12 && b.abs() > 1e-12)
            }
            _ => None,
        };
        MatchedReport {
            n_classes: n_inter,
            median_ratio_cal_to_test: med_c2t,
            median_ratio_test_to_cal: med_t2c,
            gap_raw,
            gap_matched,
            gap_survives,
        }
    };

    let controls_pass = null_ctrl.pass && identity_ctrl.pass;
    SuiteReport {
        name: name.to_string(),
        n_cal_rows: rows.cal.len(),
        n_test_rows: rows.test.len(),
        n_classes: n_labels,
        n_unknown_gold: unknown_gold,
        directions: [c2t, t2c],
        null: null_ctrl,
        identity: identity_ctrl,
        matched,
        controls_pass,
    }
}

// ───────────────────────────────────────────── runner half ──

/// Run the probe over the named suites (default: the two ESC-armed
/// suites). Suites skip LOUD with a named reason (synthetic, label
/// ceiling, no cal slice, no scoreable rows) — counted in the meta,
/// never a silent zero.
pub fn run_transfer_probe(opts: &RunOptions) -> Result<TransferProbeOutput, String> {
    let names: Vec<String> = if opts.suites.is_empty() {
        TRANSFER_PROBE_SUITES
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    } else {
        opts.suites.clone()
    };
    let mut suites: Vec<SuiteReport> = Vec::new();
    let mut skipped: Vec<SkippedSuite> = Vec::new();
    for name in &names {
        let Some(spec) = SUITES.iter().find(|s| s.name == name.as_str()) else {
            return Err(format!(
                "transfer-probe: suite {name} is not in the registry"
            ));
        };
        if spec.synthetic.is_some() {
            skipped.push(SkippedSuite {
                name: name.clone(),
                reason: "synthetic suite (no dataset rows)".into(),
            });
            continue;
        }
        let prepared = prepare(spec, &opts.datasets_dir)?;
        if prepared.labels.len() > MAX_LABELS {
            skipped.push(SkippedSuite {
                name: name.clone(),
                reason: format!(
                    "label universe {} > MAX_LABELS {MAX_LABELS}",
                    prepared.labels.len()
                ),
            });
            continue;
        }
        if prepared.cal_cases.is_empty() {
            skipped.push(SkippedSuite {
                name: name.clone(),
                reason: "no cal slice (cal_cap 0 — corpus-optional suite)".into(),
            });
            continue;
        }
        let (rows, unknown) = rows_from_prepared(&prepared)?;
        if rows.cal.is_empty() || rows.test.is_empty() {
            skipped.push(SkippedSuite {
                name: name.clone(),
                reason: "no scoreable choice-question rows on a side (noul-only?)".into(),
            });
            continue;
        }
        let report = probe_suite(name, &rows, unknown);
        let [c2t, t2c] = &report.directions;
        eprintln!(
            "  [transfer-probe] {}: cal->test {} ({} ratio classes) · test->cal {} ({}) · \
             null {} · identity {}",
            name,
            c2t.verdict,
            c2t.n_ratio_classes,
            t2c.verdict,
            t2c.n_ratio_classes,
            if report.null.pass { "PASS" } else { "RED" },
            if report.identity.pass { "PASS" } else { "RED" },
        );
        suites.push(report);
    }
    let controls_ok = suites.iter().all(|s| s.controls_pass);
    Ok(TransferProbeOutput {
        meta: TransferProbeMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            suites: names,
            skipped,
            max_labels: MAX_LABELS,
            cv_folds: CV_FOLDS,
            min_support_per_class: MIN_SUPPORT_PER_CLASS,
            min_oracle_r: MIN_ORACLE_R,
            min_ratio_classes: MIN_RATIO_CLASSES,
            match_cap: MATCH_CAP,
            null_rotate: NULL_ROTATE,
            instrument_note: INSTRUMENT_NOTE,
        },
        suites,
        controls_ok,
    })
}

// ───────────────────────────────────────────── render ──

fn opt(v: Option<f64>) -> String {
    v.map_or_else(|| "n/a".to_string(), |x| format!("{x:.4}"))
}

fn pass_str(p: bool) -> &'static str {
    if p { "PASS" } else { "RED" }
}

/// The markdown record (the JSON carries the same rows verbatim).
#[must_use]
pub fn render_transfer_probe_markdown(out: &TransferProbeOutput) -> String {
    use std::fmt::Write as _;
    let mut md = String::new();
    let _ = writeln!(
        md,
        "# transfer-probe (riir-rethink Issue 065) — cal-to-test frozen-transfer readout\n"
    );
    let _ = writeln!(md, "> {}\n", INSTRUMENT_NOTE);
    let m = &out.meta;
    let _ = writeln!(
        md,
        "- consts: max_labels {} · cv_folds {} · min_support/class {} · min_oracle_r {} · \
         min_ratio_classes {} · match_cap {} · null_rotate {} (TARGET-only)",
        m.max_labels,
        m.cv_folds,
        m.min_support_per_class,
        m.min_oracle_r,
        m.min_ratio_classes,
        m.match_cap,
        m.null_rotate
    );
    let _ = writeln!(
        md,
        "- datasets: {} · sha {} · host {} · {} UTC",
        m.datasets_dir,
        m.git_sha.as_deref().unwrap_or("unknown"),
        m.host,
        m.date_utc
    );
    for s in &m.skipped {
        let _ = writeln!(md, "- SKIP {}: {}", s.name, s.reason);
    }
    let _ = writeln!(
        md,
        "- controls: {}",
        if out.controls_ok {
            "OK".to_string()
        } else {
            "RED (exit 1 — the records above are the finding)".to_string()
        }
    );
    for s in &out.suites {
        let _ = writeln!(
            md,
            "\n## {} — cal {} rows · test {} rows · {} classes · unknown-gold {}\n",
            s.name, s.n_cal_rows, s.n_test_rows, s.n_classes, s.n_unknown_gold
        );
        let _ = writeln!(
            md,
            "- null control: real median |r| {} vs target-rotated {} — {}",
            opt(s.null.real_median_abs_r),
            opt(s.null.null_median_abs_r),
            pass_str(s.null.pass)
        );
        let _ = writeln!(
            md,
            "- identity control: in-sample oracle median {} vs frozen {} — {}",
            opt(s.identity.median_insample_oracle),
            opt(s.identity.median_r_frozen),
            pass_str(s.identity.pass)
        );
        for dir in &s.directions {
            let _ = writeln!(md, "\n### {} — {}\n", dir.direction, dir.verdict);
            let _ = writeln!(
                md,
                "- ratio classes {} (support-masked {}, oracle-masked {}) · median ratio \
                 (masked) {} · (unmasked) {} · median r_frozen {}",
                dir.n_ratio_classes,
                dir.n_masked_support,
                dir.n_masked_oracle,
                opt(dir.median_ratio_masked),
                opt(dir.median_ratio_unmasked),
                opt(dir.median_r_frozen)
            );
            let carried: Vec<&ClassRow> = dir
                .per_class
                .iter()
                .filter(|r| r.masked_reason.is_none())
                .collect();
            if !carried.is_empty() {
                let _ = writeln!(
                    md,
                    "\n| label | n_src | n_tgt | r_frozen | r_insample | r_cv | ratio |"
                );
                let _ = writeln!(md, "|---|---|---|---|---|---|---|");
                for r in carried {
                    let _ = writeln!(
                        md,
                        "| {} | {} | {} | {} | {} | {} | {} |",
                        r.label,
                        r.n_src,
                        r.n_tgt,
                        opt(r.r_frozen),
                        opt(r.r_oracle_insample),
                        opt(r.r_oracle_cv),
                        opt(r.ratio)
                    );
                }
            }
        }
        let mt = &s.matched;
        let _ = writeln!(
            md,
            "\n- matched arm: {} classes · c2t median {} · t2c median {} · gap raw {} · \
             gap matched {} · survives {}",
            mt.n_classes,
            opt(mt.median_ratio_cal_to_test),
            opt(mt.median_ratio_test_to_cal),
            opt(mt.gap_raw),
            opt(mt.gap_matched),
            match mt.gap_survives {
                Some(true) => "YES",
                Some(false) => "no",
                None => "—",
            }
        );
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::{GoldAnswer, Suite, SuiteCase, SuiteQuestion};
    use crate::pyjson::serialize_state;
    use serde_json::json;

    /// A separable fixture: `n_labels` classes, each with its own word
    /// set plus a row-specific token (within-class variance, so the
    /// in-sample oracle has real room over the frozen fit), interleaved
    /// round-robin (the stratified front's order shape — the row rotation
    /// IS a consistent relabel on this order, which the hazard test
    /// exercises).
    fn fixture_rows(per_class_cal: usize, per_class_test: usize) -> SuiteRows {
        const N_CLASSES: usize = 10;
        let embedder = Embedder;
        let row = |class: usize, k: usize| {
            let text =
                format!("alpha{class} beta{class} gamma{class} delta{class} token{class}x{k}");
            let mut feature = [0.0f32; EMBED_DIM];
            embedder.embed_into(text.as_bytes(), &mut feature);
            ProbeRow { feature, class }
        };
        let mut cal = Vec::with_capacity(per_class_cal);
        let mut test = Vec::with_capacity(per_class_test);
        for k in 0..per_class_cal {
            for c in 0..N_CLASSES {
                cal.push(row(c, k));
            }
        }
        for k in 0..per_class_test {
            for c in 0..N_CLASSES {
                test.push(row(c, k));
            }
        }
        SuiteRows {
            labels: (0..N_CLASSES).map(|i| format!("c{i}")).collect(),
            cal,
            test,
        }
    }

    /// The BROKEN null the module docs warn about: rotate the gold of BOTH
    /// sides by one row position, fit on the rotated source, score against
    /// the rotated target indicators. On an interleaved front this is a
    /// consistent relabel π, which the frozen arm learns — the null then
    /// transfers as well as the real run.
    fn broken_null_median_abs_r(rows: &SuiteRows) -> Option<f64> {
        let rotate = |side: &[ProbeRow]| -> Vec<ProbeRow> {
            let n = side.len();
            (0..n)
                .map(|i| ProbeRow {
                    feature: side[i].feature,
                    class: side[(i + NULL_ROTATE) % n].class,
                })
                .collect()
        };
        let cal_r = rotate(&rows.cal);
        let test_r = rotate(&rows.test);
        let cal_side = build_side(&cal_r, rows.labels.len());
        let test_side = build_side(&test_r, rows.labels.len());
        let heads = fit_padded(&cal_side.per_class);
        let scores = score_all(&heads, &test_side.feats, rows.labels.len());
        let mut abs_r: Vec<f64> = Vec::new();
        for (d, scores_d) in scores.iter().enumerate() {
            if test_side.n_per[d] < MIN_SUPPORT_PER_CLASS {
                continue;
            }
            if let Some(r) = pearson(scores_d, &test_side.inds[d]) {
                abs_r.push(r.abs());
            }
        }
        median(abs_r)
    }

    #[test]
    fn frozen_transfer_measures_and_controls_pass() {
        let rows = fixture_rows(3, 4);
        let rep = probe_suite("fixture", &rows, 0);
        assert!(rep.null.pass, "real gold must beat the target-rotated null");
        assert!(
            rep.identity.pass,
            "the in-sample oracle must not lose to the frozen fit (Lemma 2)"
        );
        let c2t = &rep.directions[0];
        assert_eq!(c2t.direction, "cal->test");
        assert_eq!(c2t.verdict, "MEASURED", "10 classes ≥ MIN_RATIO_CLASSES");
        assert_eq!(c2t.n_ratio_classes, rep.n_classes);
        let mrf = c2t
            .median_r_frozen
            .expect("separable fixture carries frozen r");
        assert!(mrf > 0.2, "separable fixture must transfer, got {mrf}");
        assert!(rep.controls_pass);
    }

    #[test]
    fn null_rotation_hazard_negative_arm() {
        let rows = fixture_rows(3, 4);
        let rep = probe_suite("fixture", &rows, 0);
        let real = rep.null.real_median_abs_r.expect("real median");
        let shipped_null = rep.null.null_median_abs_r.expect("null median");
        // The SHIPPED null (target-only rotation) collapses far below real.
        assert!(
            shipped_null < 0.5 * real,
            "target-only rotation must collapse (real {real}, null {shipped_null})"
        );
        // The HAZARD: a consistent relabel (both-side rotation) transfers
        // as well as the real run — the broken null does NOT separate.
        let broken = broken_null_median_abs_r(&rows).expect("broken-null median");
        assert!(
            broken > 0.8 * real,
            "a consistent relabel must transfer like the real run \
             (real {real}, broken null {broken}) — this is why the null is target-only"
        );
    }

    #[test]
    fn thin_support_is_undecided_not_a_control_failure() {
        // 2 cal rows/class < MIN_SUPPORT_PER_CLASS: every class masks on
        // support, the direction reads UNDECIDED — while the controls
        // still pass (an undecided direction is a finding, not a red).
        let rows = fixture_rows(2, 4);
        let rep = probe_suite("fixture", &rows, 0);
        let c2t = &rep.directions[0];
        assert_eq!(c2t.verdict, "UNDECIDED (support)");
        assert_eq!(c2t.n_ratio_classes, 0);
        assert_eq!(c2t.median_ratio_masked, None);
        assert!(rep.controls_pass);
    }

    #[test]
    fn label_ceiling_refuses() {
        let labels: Vec<String> = (0..=MAX_LABELS).map(|i| format!("l{i}")).collect();
        assert!(labels.len() > MAX_LABELS);
        assert!(check_label_count(&labels).is_err());
        let ok: Vec<String> = (0..MAX_LABELS).map(|i| format!("l{i}")).collect();
        assert!(check_label_count(&ok).is_ok());
    }

    fn fixture_case(id: &str, gold_key: &str, criteria: serde_json::Value) -> SuiteCase {
        let keys: Vec<String> = criteria
            .as_object()
            .expect("choice criteria object")
            .keys()
            .cloned()
            .collect();
        let idx = keys
            .iter()
            .position(|k| k == gold_key)
            .expect("gold key present in criteria");
        SuiteCase {
            id: id.to_string(),
            state: json!({ "text": format!("fixture utterance for {id}") }),
            questions: vec![SuiteQuestion {
                qid: "q".to_string(),
                kind: QKind::Choice,
                instructions: String::new(),
                criteria,
            }],
            gold: vec![GoldAnswer {
                idx,
                soft: Vec::new(),
                gold_score: None,
            }],
        }
    }

    #[test]
    fn rows_from_prepared_extracts_counts_unknown_gold() {
        let labels = vec!["a".to_string(), "b".to_string()];
        // "zz" resolves to a criteria key outside the label universe —
        // counted, never fatal.
        let test_cases = vec![
            fixture_case("t0", "a", json!({ "a": 1, "b": 2 })),
            fixture_case("t1", "zz", json!({ "zz": 1, "b": 2 })),
        ];
        let cal_cases = vec![fixture_case("c0", "b", json!({ "a": 1, "b": 2 }))];
        let suite = Suite {
            name: "fixture",
            cases: test_cases,
            option_counts_note: "",
        };
        let state_strs: Vec<String> = suite
            .cases
            .iter()
            .map(|c| serialize_state(&c.state))
            .collect();
        let cal_state_strs: Vec<String> = cal_cases
            .iter()
            .map(|c| serialize_state(&c.state))
            .collect();
        let prepared = Prepared {
            suite,
            train: Vec::new(),
            state_strs,
            cal_cases,
            cal_state_strs,
            labels: labels.clone(),
            pool_rows: serde_json::Value::Null,
            slices: None,
        };
        let (rows, unknown) = rows_from_prepared(&prepared).expect("fixture rows");
        assert_eq!(unknown, 1, "the out-of-universe gold is counted");
        assert_eq!(rows.cal.len(), 1);
        assert_eq!(rows.test.len(), 1);
        assert_eq!(rows.test[0].class, 0, "gold key 'a' resolves to class 0");
        assert_eq!(rows.cal[0].class, 1, "gold key 'b' resolves to class 1");
    }
}
