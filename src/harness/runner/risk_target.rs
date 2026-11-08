//! Issue 112 T4 — the `--gate-risk-target <alpha>` harness lever
//! (REPORT-ONLY; the opt-in arming is the only emission path, so a lever-off
//! run carries no RiskTarget metadata — T5's additive-wire law).
//!
//! The fused gate's certified-abstention posture (`Posture::RiskTarget`,
//! `engine::threshold`) fits a cutoff whose accepted-subset error carries a
//! one-sided Clopper–Pearson guarantee at `(alpha, delta)` on the CAL slice.
//! This module is the harness's report half:
//!
//! - **Fit** — reuses the engine's `risk_target_fit` over the SAME
//!   `score_obs` the percentile fit saw (`fit_posture_inner`), so the
//!   certificate and the shipped gate share one observation surface.
//! - **Realization** — the certified cutoff is evaluated on the suite's
//!   TEST half with the SAME per-case convention the fit used (the last
//!   slot of each case; the noul wire flip) on the SAME confidence scale
//!   the fit read: the calibrated scale when `gate_fit_calibrated` is on
//!   (the default — `cal_eval_test`'s confidences), the raw scale when the
//!   fit-on-raw measurement posture is armed (`--no-gate-fit-calibrated`;
//!   `raw_eval`). The realized accepted-subset error carries a two-sided
//!   exact CP interval at per-tail `delta/2` (95% total).
//! - **Gates (pre-registered in the issue BEFORE any run; this code is
//!   their faithful reader, not their author)**:
//!   - **(a) CERTIFICATE** — per suite where coverage > 0: pass when the
//!     realized error's interval LOWER end ≤ alpha. `breach = lower >
//!     alpha` (a CONFIDENT violation — the whole interval above the
//!     target). The UPPER end is NOT the breach event (on small slices it
//!     clears alpha almost always even when the posture is correct — the
//!     paper's own Qwen3.8-27B row is per-suite noise of exactly that
//!     shape). Across suites, ≥ 2 LOWER-end breaches on ≤ 10 suites is
//!     the pre-registered "the posture is wrong" line (each certified
//!     suite breaches with probability ≤ delta, so δ·N ≤ 0.5 expected at
//!     N ≤ 10; 1 breach is within the guarantee's own probability mass —
//!     disclosed, never failed).
//!   - **(c1)/(c2)** — the shuffled-labels and planted-violation
//!     falsification arms live as unit tests on the engine
//!     (`threshold.rs`, landed with T1/T2); the harness run carries no
//!     second copy.
//!   - **(c3) DISTRIBUTION-SHIFT** — cal from one suite + test from the
//!     next suite in run order: the guarantee's exchangeability premise is
//!     broken there, so the row is RECORDED, never gated.
//! - **Verdict** — report-only; the exit code is untouched and the read
//!   feeds T6's expectations recording in the issue.
//!
//! `delta` is PINNED at 0.05 for v1 (disclosed in every row); alpha is the
//! lever's argument. Wire: the record rides `LaneResult.risk_target`
//! (skip-when-none) — additive-only; a lever-off run serializes nothing.

use serde::Serialize;

use crate::engine::{RecommendationStatus, RiskTargetFit};
use crate::harness::suites::{QKind, SuiteCase};

use super::SuiteResult;

/// The pinned v1 confidence level (Issue 112 T4, disclosed): the CP bound
/// holds with probability >= 1 − delta; the realized-error interval is
/// two-sided at delta/2 per tail.
pub const RISK_TARGET_DELTA: f64 = 0.05;

/// The realized accepted-subset error of the certified cutoff on the TEST
/// half, with its exact interval and the pre-registered breach event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCertificateRead {
    /// Test cases the read walked (the certificate's group unit: the case).
    pub groups: usize,
    /// Cases the certified cutoff ACCEPTS (`confidence >= cutoff`).
    pub accepted: usize,
    /// Wrong answers inside the accepted subset.
    pub errors: usize,
    /// `errors / accepted`; `None` when nothing is accepted (zero test
    /// coverage at the certified cutoff — an honest empty read, never a
    /// fabricated rate).
    pub err_rate: Option<f64>,
    /// Lower end of the two-sided exact CP interval (per-tail delta/2).
    /// `None` when nothing is accepted.
    pub ci_lower: Option<f64>,
    /// Upper end of the same interval.
    pub ci_upper: Option<f64>,
    /// The pre-registered breach event: LOWER end > alpha (a confident
    /// violation). Always `false` on an empty read.
    pub breach: bool,
}

/// One suite's risk-target record: the cal-side fit disclosure plus the
/// test-half realization. Absent entirely (`LaneResult.risk_target == None`)
/// unless the lever armed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskTargetSuiteRecord {
    pub suite: String,
    pub alpha: f64,
    pub delta: f64,
    /// The engine fit over the cal `score_obs`. `None` = below
    /// `THIN_SUPPORT_FLOOR` — no fit at all (the engine's null rule).
    pub fit: Option<RiskTargetFit>,
    /// The test-half realization; `None` when the fit is absent or nothing
    /// certified (no cutoff to realize).
    pub test: Option<TestCertificateRead>,
    /// The test-half (confidence, correct) rows the certificate read —
    /// report-only state the c3 shift row re-reads. Never serialized.
    #[serde(skip)]
    pub test_pairs: Vec<(f64, bool)>,
}

/// The c3 DISTRIBUTION-SHIFT report row (cal from one suite + test from the
/// next suite in run order — exchangeability broken by construction;
/// recorded, never gated).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionShiftRow {
    pub cal_suite: String,
    pub cutoff: f32,
    pub test_suite: String,
    pub accepted: usize,
    pub errors: usize,
    pub err_rate: Option<f64>,
    pub ci_lower: Option<f64>,
    pub ci_upper: Option<f64>,
}

/// The aggregated lever report (the `risk_target.json` document).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskTargetReport {
    pub alpha: f64,
    pub delta: f64,
    pub rows: Vec<RiskTargetSuiteRecord>,
    /// Suites with a certified cutoff (coverage > 0).
    pub certified: usize,
    /// Suites whose realized LOWER end breached alpha.
    pub breaches: usize,
    /// The pre-registered breach-budget verdict (T4(a)) — the exact read
    /// the T6 recording quotes.
    pub verdict: String,
    pub shift_row: Option<DistributionShiftRow>,
}

/// Per-case (confidence, correct) rows over the TEST cases with the SAME
/// convention the cal-side `score_obs` were built with: the LAST slot of
/// each case, the noul wire flip on the pick (`eval_engine`'s convention —
/// engine internal [yes, no] vs gold [no, yes]), correctness vs the case's
/// last gold index. Index-aligned with `cases`; cases without an eval row
/// are skipped (never fabricated).
pub fn last_slot_pairs(
    picks: &[Vec<usize>],
    confs: &[Vec<f64>],
    cases: &[SuiteCase],
) -> Vec<(f64, bool)> {
    let mut rows = Vec::with_capacity(cases.len());
    for (ci, case) in cases.iter().enumerate() {
        let (Some(picks_ci), Some(confs_ci)) = (picks.get(ci), confs.get(ci)) else {
            continue;
        };
        let (Some(pick_raw), Some(conf)) = (picks_ci.last().copied(), confs_ci.last().copied())
        else {
            continue;
        };
        let pick = match case.questions.last().map(|q| q.kind) {
            Some(QKind::Noul) => 1 - pick_raw,
            _ => pick_raw,
        };
        let correct = case.gold.last().is_some_and(|g| g.idx == pick);
        rows.push((conf, correct));
    }
    rows.shrink_to_fit();
    rows
}

/// The test-half realization of a certified cutoff: accepted = the score
/// axis alone (`confidence >= cutoff`, the fit's own tie-inclusive
/// semantics), errors inside it, exact interval, breach event.
pub fn evaluate_test(pairs: &[(f64, bool)], cutoff: f32, alpha: f64) -> TestCertificateRead {
    let mut accepted = 0usize;
    let mut errors = 0usize;
    for (conf, correct) in pairs {
        if *conf >= f64::from(cutoff) {
            accepted += 1;
            errors += (!*correct) as usize;
        }
    }
    if accepted == 0 {
        return TestCertificateRead {
            groups: pairs.len(),
            accepted: 0,
            errors: 0,
            err_rate: None,
            ci_lower: None,
            ci_upper: None,
            breach: false,
        };
    }
    let (lo, hi) =
        crate::engine::threshold::cp_two_sided_ends(errors, accepted, RISK_TARGET_DELTA / 2.0);
    TestCertificateRead {
        groups: pairs.len(),
        accepted,
        errors,
        err_rate: Some(errors as f64 / accepted as f64),
        ci_lower: Some(lo),
        ci_upper: Some(hi),
        breach: lo > alpha,
    }
}

/// Build one suite's record from the engine fit and the test-half eval.
/// `fit` comes from `FittedPosture` (the lever's arming decided upstream);
/// the picks/confs follow the fit's scale (calibrated by default — see the
/// module doc).
pub fn suite_record(
    suite: &str,
    alpha: f64,
    fit: Option<RiskTargetFit>,
    picks: &[Vec<usize>],
    confs: &[Vec<f64>],
    cases: &[SuiteCase],
) -> RiskTargetSuiteRecord {
    let test_pairs = last_slot_pairs(picks, confs, cases);
    let test = fit
        .as_ref()
        .and_then(|f| f.recommendation.as_ref())
        .map(|rec| evaluate_test(&test_pairs, rec.threshold, alpha));
    RiskTargetSuiteRecord {
        suite: suite.to_string(),
        alpha,
        delta: RISK_TARGET_DELTA,
        fit,
        test,
        test_pairs,
    }
}

/// The pre-registered breach-budget verdict (T4(a)/T6): the posture is
/// wrong iff >= 2 LOWER-end breaches land on <= 10 certified suites; 1
/// breach is the guarantee's own probability mass (disclosed, not failed).
/// Certified suites beyond the pre-registered N <= 10 window carry the
/// delta-scaled expectation instead — the >=2 line was registered for the
/// board-size runs, never extrapolated.
pub fn verdict_for(breaches: usize, certified: usize) -> String {
    let scale = if certified > 10 {
        format!(
            " ({certified} certified suites exceed the pre-registered N <= 10 window — read against the delta-scaled expectation {:.1})",
            0.05 * certified as f64
        )
    } else {
        String::new()
    };
    match breaches {
        0 => format!(
            "breach budget clean — 0 breaches on {certified} certified suite(s) (pre-registered line: >=2 LOWER-end breaches on <=10 suites = posture wrong){scale}"
        ),
        1 => format!(
            "1 breach on {certified} certified suite(s) — within the guarantee's own probability mass (each certified suite breaches with probability <= delta); disclosed, never failed{scale}"
        ),
        n => format!(
            "BREACH BUDGET FIRES — {n} LOWER-end breaches on {certified} certified suite(s); per the pre-registered line (>=2 on <=10 suites) the posture is wrong and Issue 112 closes NEGATIVE{scale}"
        ),
    }
}

/// Pull the lever's records out of a run's suites (the modelless lane only
/// — the certificate is a modelless-gate surface), preserving run order.
pub fn extract_rows(suites: &[SuiteResult]) -> Vec<RiskTargetSuiteRecord> {
    suites
        .iter()
        .filter_map(|s| s.modelless.as_ref().and_then(|m| m.risk_target.clone()))
        .collect()
}

/// Aggregate the records into the report: counts, the breach verdict, and
/// the c3 distribution-shift row (the first certified suite's cutoff on the
/// NEXT record's test rows, run order — skipped loud when no such pair).
pub fn aggregate(rows: &[RiskTargetSuiteRecord], alpha: f64) -> RiskTargetReport {
    let certified = rows
        .iter()
        .filter(|r| r.test.as_ref().is_some_and(|t| t.accepted > 0))
        .count();
    let breaches = rows
        .iter()
        .filter(|r| r.test.as_ref().is_some_and(|t| t.breach))
        .count();
    let shift_row = rows.windows(2).find_map(|w| {
        let (a, b) = (&w[0], &w[1]);
        let rec = a.fit.as_ref().and_then(|f| f.recommendation.as_ref())?;
        if b.test_pairs.is_empty() {
            return None;
        }
        let read = evaluate_test(&b.test_pairs, rec.threshold, alpha);
        Some(DistributionShiftRow {
            cal_suite: a.suite.clone(),
            cutoff: rec.threshold,
            test_suite: b.suite.clone(),
            accepted: read.accepted,
            errors: read.errors,
            err_rate: read.err_rate,
            ci_lower: read.ci_lower,
            ci_upper: read.ci_upper,
        })
    });
    RiskTargetReport {
        alpha,
        delta: RISK_TARGET_DELTA,
        certified,
        breaches,
        verdict: verdict_for(breaches, certified),
        rows: rows.to_vec(),
        shift_row,
    }
}

/// The `RISK_TARGET.md` rendering (also printed to stdout).
pub fn render_markdown(report: &RiskTargetReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "## Issue 112 T4 — risk-target certificate report (report-only)\n\n\
         alpha {} · delta {} (pinned v1) · realized-error interval two-sided exact CP at delta/2 per tail · \
         breach = LOWER end > alpha · score-axis certificate (the fused gate's served set also folds the distance axis) · \
         group unit = the cal case (template families would count once — none in these slices today)\n\n",
        report.alpha, report.delta
    ));
    out.push_str("| suite | fit | cutoff | cal n/accepted/errors | floor | test accepted/errors | err rate | 95% CP CI | breach |\n");
    out.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for r in &report.rows {
        let (
            fit_cell,
            cutoff_cell,
            cal_cell,
            floor_cell,
            test_cell,
            rate_cell,
            ci_cell,
            breach_cell,
        ) = match (&r.fit, &r.test) {
            (Some(fit), Some(test)) => {
                let status = match fit.status {
                    RecommendationStatus::Fitted => "certified",
                    _ => "not certified",
                };
                let rec = fit.recommendation.as_ref();
                (
                    status.to_string(),
                    rec.map_or("—".to_string(), |rec| format!("{:.3}", rec.threshold)),
                    format!("{}/{}/{}", fit.n_total, fit.n_groups, fit.n_errors),
                    fit.floor_groups.to_string(),
                    format!("{}/{}", test.accepted, test.errors),
                    test.err_rate
                        .map_or("—".to_string(), |v| format!("{:.4}", v)),
                    match (test.ci_lower, test.ci_upper) {
                        (Some(lo), Some(hi)) => format!("[{:.4}, {:.4}]", lo, hi),
                        _ => "—".to_string(),
                    },
                    if test.breach { "YES" } else { "no" }.to_string(),
                )
            }
            (Some(fit), None) => (
                "not certified".to_string(),
                "—".to_string(),
                format!("{}/—/—", fit.n_total),
                fit.floor_groups.to_string(),
                "—".to_string(),
                "—".to_string(),
                "—".to_string(),
                "—".to_string(),
            ),
            (None, _) => (
                "no fit (thin support)".to_string(),
                "—".to_string(),
                "—".to_string(),
                "—".to_string(),
                "—".to_string(),
                "—".to_string(),
                "—".to_string(),
                "—".to_string(),
            ),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            r.suite,
            fit_cell,
            cutoff_cell,
            cal_cell,
            floor_cell,
            test_cell,
            rate_cell,
            ci_cell,
            breach_cell
        ));
    }
    out.push_str(&format!("\n**Breach budget (T4a):** {}\n", report.verdict));
    match &report.shift_row {
        Some(s) => out.push_str(&format!(
            "**c3 distribution-shift row (report-only):** certified cutoff {:.3} ({} cal) applied to {} test — \
             accepted {} (errors {}), rate {}, CI {} — the guarantee's exchangeability premise is broken here \
             by construction; recorded, never gated.\n",
            s.cutoff,
            s.cal_suite,
            s.test_suite,
            s.accepted,
            s.errors,
            s.err_rate.map_or("—".to_string(), |v| format!("{:.4}", v)),
            match (s.ci_lower, s.ci_upper) {
                (Some(lo), Some(hi)) => format!("[{:.4}, {:.4}]", lo, hi),
                _ => "—".to_string(),
            }
        )),
        None => out.push_str(
            "**c3 distribution-shift row:** skipped — no certified suite with a following suite's test rows.\n",
        ),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::threshold::{
        GateObservation, ThresholdRecommendation, ThresholdSupport, risk_target_fit,
    };
    use crate::harness::suites::{GoldAnswer, SuiteQuestion};
    use serde_json::Value;

    fn case(questions: &[(QKind, usize)]) -> SuiteCase {
        SuiteCase {
            id: "c".to_string(),
            state: Value::Null,
            questions: questions
                .iter()
                .map(|(kind, _)| SuiteQuestion {
                    qid: "q".to_string(),
                    kind: *kind,
                    instructions: String::new(),
                    criteria: Value::Null,
                })
                .collect(),
            gold: questions
                .iter()
                .map(|(_, idx)| GoldAnswer {
                    idx: *idx,
                    soft: Vec::new(),
                    gold_score: None,
                })
                .collect(),
        }
    }

    #[test]
    fn last_slot_pairs_read_the_last_slot_with_the_noul_flip() {
        // Case A: [Choice, Noul] — the eval rows are per question; the pair
        // reads the LAST slot and flips the noul pick (internal [yes, no]
        // vs gold [no, yes]).
        let cases = vec![
            case(&[(QKind::Choice, 0), (QKind::Noul, 1)]),
            case(&[(QKind::Choice, 2)]),
        ];
        let picks = vec![vec![0, 0], vec![2]];
        // A: last slot noul pick 0 (internal yes) → wire pick 1 == gold 1 →
        // correct. B: pick 2 == gold 2 → correct.
        let confs = vec![vec![0.9, 0.85], vec![0.5]];
        let pairs = last_slot_pairs(&picks, &confs, &cases);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (0.85, true));
        assert_eq!(pairs[1], (0.5, true));
        // A wrong arm: internal pick 1 (no) → wire pick 0 != gold 1.
        let picks = vec![vec![0, 1], vec![2]];
        let pairs = last_slot_pairs(&picks, &confs, &cases);
        assert_eq!(pairs[0], (0.85, false));
        // Mismatched eval rows (a skipped case) never fabricate a pair.
        let pairs = last_slot_pairs(&picks[..1], &confs[..1], &cases);
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn evaluate_test_reads_the_accepted_subset_and_breaches() {
        let pairs = vec![(0.9, true), (0.8, true), (0.7, false), (0.6, false)];
        // cutoff 0.75 accepts the top two, both correct — zero-error read,
        // breach impossible (the lower end is exactly 0).
        let read = evaluate_test(&pairs, 0.75, 0.10);
        assert_eq!(read.groups, 4);
        assert_eq!(read.accepted, 2);
        assert_eq!(read.errors, 0);
        assert_eq!(read.err_rate, Some(0.0));
        assert_eq!(read.ci_lower, Some(0.0));
        assert!(!read.breach);
        // The tie is INCLUSIVE (the fit's own `score >= t` semantics).
        let read = evaluate_test(&pairs, 0.7, 0.10);
        assert_eq!(read.accepted, 3);
        // Planted violation: the accepted subset is mostly wrong — the
        // whole interval sits above alpha, so the LOWER end breaches.
        let planted = vec![(0.9, false), (0.85, false), (0.8, false), (0.7, true)];
        let read = evaluate_test(&planted, 0.7, 0.10);
        assert_eq!(read.accepted, 4);
        assert_eq!(read.errors, 3);
        let lo = read.ci_lower.expect("interval");
        assert!(lo > 0.10, "lower {lo} must clear alpha on a 75% error read");
        assert!(read.breach);
        // Zero acceptance is an honest empty read, never a fake rate.
        let empty = evaluate_test(&pairs, 0.95, 0.10);
        assert_eq!(empty.accepted, 0);
        assert_eq!(empty.err_rate, None);
        assert!(!empty.breach);
    }

    #[test]
    fn verdict_lines_match_the_pre_registered_budget() {
        assert!(verdict_for(0, 3).starts_with("breach budget clean"));
        assert!(verdict_for(1, 3).starts_with("1 breach"));
        assert!(verdict_for(1, 3).contains("disclosed, never failed"));
        assert!(verdict_for(2, 3).starts_with("BREACH BUDGET FIRES"));
        assert!(verdict_for(2, 3).contains("closes NEGATIVE"));
        // Beyond the pre-registered N <= 10 window the line is scaled, not
        // extrapolated.
        assert!(verdict_for(0, 12).contains("exceed the pre-registered"));
        assert!(verdict_for(0, 10).starts_with("breach budget clean"));
    }

    fn fitted_record(suite: &str, threshold: f32) -> RiskTargetSuiteRecord {
        RiskTargetSuiteRecord {
            suite: suite.to_string(),
            alpha: 0.10,
            delta: 0.05,
            fit: Some(RiskTargetFit {
                alpha: 0.10,
                delta: 0.05,
                status: RecommendationStatus::Fitted,
                recommendation: Some(ThresholdRecommendation {
                    threshold,
                    status: RecommendationStatus::Fitted,
                    target_accuracy: 1.0,
                    support: ThresholdSupport {
                        n: 40,
                        n_pass: 30,
                        n_abstain: 10,
                    },
                }),
                n_total: 40,
                n_groups: 30,
                n_errors: 0,
                floor_groups: 29,
            }),
            test: Some(TestCertificateRead {
                groups: 30,
                accepted: 25,
                errors: 1,
                err_rate: Some(0.04),
                ci_lower: Some(0.001),
                ci_upper: Some(0.2),
                breach: false,
            }),
            test_pairs: vec![(0.9, true), (0.4, false)],
        }
    }

    #[test]
    fn aggregate_counts_and_builds_the_shift_row() {
        let certified = fitted_record("a", 0.5);
        let thin = RiskTargetSuiteRecord {
            suite: "b".to_string(),
            alpha: 0.10,
            delta: 0.05,
            fit: None,
            test: None,
            test_pairs: vec![(0.9, false), (0.8, false), (0.2, true)],
        };
        let report = aggregate(&[certified.clone(), thin.clone()], 0.10);
        assert_eq!(report.certified, 1);
        assert_eq!(report.breaches, 0);
        let shift = report.shift_row.expect("a certifies and b follows");
        assert_eq!(shift.cal_suite, "a");
        assert_eq!(shift.test_suite, "b");
        assert_eq!(shift.cutoff, 0.5);
        // b's pairs at cutoff 0.5: two accepted, both wrong — the shift row
        // reads them without gating.
        assert_eq!(shift.accepted, 2);
        assert_eq!(shift.errors, 2);
        // A certified LAST suite has no following record — c3 skipped.
        let report = aggregate(std::slice::from_ref(&certified), 0.10);
        assert!(report.shift_row.is_none());
    }

    #[test]
    fn suite_record_applies_the_certified_cutoff_to_the_test_half() {
        // The engine fit over a clean cal slice; the record must evaluate
        // the SAME cutoff on the test pairs and keep the fit disclosure.
        let obs: Vec<GateObservation> = (0..400)
            .map(|i| GateObservation {
                score: 0.4 + (i as f32 / 400.0) * 0.6,
                correct: i >= 40,
            })
            .collect();
        let fit = risk_target_fit(&obs, 0.10, RISK_TARGET_DELTA).expect("support");
        assert_eq!(fit.status, RecommendationStatus::Fitted);
        let cutoff = fit.recommendation.as_ref().expect("certified").threshold;
        let cases: Vec<SuiteCase> = (0..50)
            .map(|i| {
                if i < 45 {
                    case(&[(QKind::Choice, 0)])
                } else {
                    case(&[(QKind::Choice, 1)])
                }
            })
            .collect();
        // Pick 0 everywhere: the first 45 cases (gold 0) are correct at
        // conf 0.9, the last 5 (gold 1) are wrong at conf 0.3.
        let picks: Vec<Vec<usize>> = (0..50).map(|_| vec![0]).collect();
        let confs: Vec<Vec<f64>> = (0..50)
            .map(|i| if i < 45 { vec![0.9] } else { vec![0.3] })
            .collect();
        let rec = suite_record("s", 0.10, Some(fit), &picks, &confs, &cases);
        let test = rec.test.expect("certified fit realizes a test read");
        // 45 accepted at the certified cutoff (all correct), 5 below it.
        assert_eq!(test.accepted, 45);
        assert_eq!(test.errors, 0);
        assert!(!test.breach);
        assert_eq!(rec.test_pairs.len(), 50);
        assert!(cutoff > 0.3 && cutoff <= 0.9);
    }

    #[test]
    fn lever_off_emits_no_rows_and_the_wire_stays_clean() {
        // T5's no-default-emission law: the lever's arming is the only
        // emission path — a lanes_off posture carries no alpha, an
        // empty-suite extraction carries no rows, and the aggregator
        // answers the clean verdict, never a fabricated row.
        let opts = crate::harness::runner::RunOptions::lanes_off(
            std::path::PathBuf::from(".raw/datasets"),
            Vec::new(),
        );
        assert!(opts.gate_risk_target.is_none());
        let rows = extract_rows(&[]);
        assert!(rows.is_empty());
        let report = aggregate(&rows, 0.10);
        assert_eq!(report.rows.len(), 0);
        assert_eq!(report.certified, 0);
        assert_eq!(report.breaches, 0);
        assert!(report.verdict.contains("clean"));
        assert!(report.shift_row.is_none());
        // The record type itself must serialize without the pairs (the
        // report-only state never rides the wire) — the additive-wire law.
        let json = serde_json::to_string(&fitted_record("a", 0.5)).expect("serialize");
        assert!(!json.contains("test_pairs"), "{json}");
    }
}
