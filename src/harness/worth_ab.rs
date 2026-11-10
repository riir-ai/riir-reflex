//! Issue 111 T3 — the matched-cost worthiness A/B (report-only, opt-in
//! `--worth-ab`, needs `--cascade --cascade-worthiness --collective-worth`).
//!
//! The GOAT question, pre-registered in `.issues/111` BEFORE any run: at
//! EQUAL test-side escalation spend, does cross-lane-informed arming (the
//! committee verdict) beat the shipped delta-only arming on serving
//! accuracy? Accuracy alone can never pass this gate — a committee runs
//! both lanes on every row while the delta gate spends only on escalated
//! rows — so the comparison is pinned to the cost axis and the postures
//! are traced to a matched spend before any accuracy word is said.
//!
//! Everything here reads per-suite records the cascade compose already
//! produced (one frozen test read; both postures are recomputed from the
//! same records — the arming rule is the only difference). No serving
//! behavior changes; the flags stay opt-in whatever the verdict.

use serde::Serialize;

/// Pre-registered relative cost-match tolerance (`.issues/111`, T3
/// pre-registration block): a `worth_only(t*)` posture matches
/// `delta_only` iff the relative gap is ≤ this.
pub const COST_MATCH_TOLERANCE: f64 = 0.05;

/// One-sided-95% z (the house paired-LB95 convention — matches
/// `probe_delta_lcb95`'s z in `cascade.rs`).
const PAIRED_LB95_Z: f64 = 1.644_934;

/// Issue 111 T3 — one suite's A/B inputs, derived from the cascade report
/// One suite's A/B inputs, derived from the cascade report + the
/// worthiness verdict's collective block.
///
/// `modelless_correct` is the all-modelless outcome over the suite (the
/// disarmed posture's outcome); the `hyp_*` counters are the WOULD-BE
/// armed posture over the covered abstains (arming-blind, so a disarmed
/// suite is priced at its rule's spend).
#[derive(Debug, Clone)]
pub struct SuiteAbInput {
    pub suite: String,
    /// The shipped delta rule's outcome for this suite (the worthiness
    /// verdict; a suite with no verdict reads armed — the gate's own
    /// named pre-lever behavior).
    pub delta_armed: bool,
    pub n_questions: usize,
    /// All-modelless correct over the suite.
    pub modelless_correct: usize,
    /// The would-be escalation spend when armed (covered abstains).
    pub hyp_escalated: usize,
    /// Escalator correct among the covered abstains.
    pub hyp_laya_correct: usize,
    /// Modelless correct among the covered abstains.
    pub hyp_ml_correct: usize,
    /// McNemar pair on the covered abstains:
    /// (escalator right ∧ modelless wrong, modelless right ∧ escalator wrong).
    pub hyp_mcnemar: (usize, usize),
    /// The committee verdict's worth-arm data: (improvement,
    /// wilson_supported). `None` = the suite has no collective block
    /// (no verdict / probe off) — the worth leg never arms it.
    pub worth_arm: Option<(f64, bool)>,
}

impl SuiteAbInput {
    /// The suite's outcome + spend under `armed`.
    fn posture(&self, armed: bool) -> (usize, usize) {
        if armed {
            (
                self.modelless_correct + self.hyp_laya_correct - self.hyp_ml_correct,
                self.hyp_escalated,
            )
        } else {
            (self.modelless_correct, 0)
        }
    }
}

/// One posture's accuracy/cost PAIR (the issue's own disclosure law: every
/// posture reports both, never accuracy alone).
#[derive(Debug, Clone, Serialize)]
pub struct WorthAbPosture {
    /// `delta_only` / `worth_only(t=…)` / `combined(t=…)`.
    pub rule: String,
    /// The threshold when the rule carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<f64>,
    pub armed_suites: Vec<String>,
    /// Total escalated questions on the frozen test read (the cost axis).
    pub cost: usize,
    pub correct: usize,
    pub n_questions: usize,
    pub accuracy: f64,
    /// `cost / n_questions` — the escalation rate the deployed cascade
    /// would pay.
    pub escalation_rate: f64,
}

/// The matched-cost comparison + its pre-registered verdict.
#[derive(Debug, Clone, Serialize)]
pub struct MatchedComparison {
    /// The threshold whose posture matched `delta_only`'s cost.
    pub worth_t: f64,
    pub delta_cost: usize,
    pub worth_cost: usize,
    /// `|worth_cost − delta_cost| / delta_cost` (disclosed against the
    /// 5% tolerance).
    pub cost_gap_rel: f64,
    pub delta_accuracy: f64,
    pub worth_accuracy: f64,
    /// McNemar counts composed over the flipped suites' covered abstains,
    /// both directions (worth-armed-only contributes (a, b); delta-armed-
    /// only contributes (b, a)).
    pub n_worth_right_delta_wrong: usize,
    pub n_worth_wrong_delta_right: usize,
    pub n_questions: usize,
    /// Mean per-question paired delta (worth − delta) over the WHOLE read.
    pub paired_mean: f64,
    /// One-sided-95% lower bound on `paired_mean` (normal approx, McNemar
    /// SE). The pre-registered promote bar reads THIS.
    pub paired_lb95: f64,
    /// `WORTH_BEATS_DELTA` / `NEGATIVE` / `UNDECIDABLE_COST_MATCH` /
    /// `UNDECIDABLE_ZERO_STRADDLE` (the pre-registered bars).
    pub verdict: String,
}

/// The run-level A/B block for one checkpoint (results.json, additive).
#[derive(Debug, Clone, Serialize)]
pub struct WorthAbBlock {
    /// Every evaluated posture's accuracy/cost pair.
    pub postures: Vec<WorthAbPosture>,
    /// The matched-cost comparison; None = no `worth_only(t)` posture
    /// matched within tolerance (the postures' pairs still report).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<MatchedComparison>,
    pub cost_match_tolerance: f64,
}

/// Sweep threshold grid: 0.00..=margin step 0.01 (the margin included —
/// there the worth leg is a subset of the delta rule, so the posture set
/// always contains the exact delta-only point).
fn threshold_grid(margin: f64) -> Vec<f64> {
    let mut grid: Vec<f64> = (0..=16)
        .map(|i| (i as f64) * 0.01)
        .filter(|t| *t <= margin + 1e-9)
        .collect();
    if let Some(&last) = grid.last()
        && (last - margin).abs() > 1e-9
    {
        grid.push(margin);
    }
    grid
}

/// Worth-arm leg at threshold t: the suite arms iff the committee verdict
/// reads worth at t — improvement ≥ t AND Wilson-supported.
fn worth_arms(worth_arm: &Option<(f64, bool)>, t: f64) -> bool {
    match worth_arm {
        Some((improvement, supported)) => *improvement >= t && *supported,
        None => false,
    }
}

/// Compose the posture table + the matched-cost comparison for one
/// checkpoint's suite set. Pure — a function of the per-suite records
/// only.
pub fn evaluate(suites: &[SuiteAbInput], margin: f64) -> Result<WorthAbBlock, String> {
    if suites.is_empty() {
        return Err("worth-ab: no suites with cascade records".to_string());
    }
    let n_questions: usize = suites.iter().map(|s| s.n_questions).sum();
    if n_questions == 0 {
        return Err("worth-ab: empty question set".to_string());
    }

    let posture_of = |armed: &[bool]| -> WorthAbPosture {
        let mut cost = 0usize;
        let mut correct = 0usize;
        let mut armed_suites = Vec::new();
        for (s, &a) in suites.iter().zip(armed) {
            let (c, spend) = s.posture(a);
            correct += c;
            cost += spend;
            if a {
                armed_suites.push(s.suite.clone());
            }
        }
        WorthAbPosture {
            rule: String::new(),
            t: None,
            armed_suites,
            cost,
            correct,
            n_questions,
            accuracy: correct as f64 / n_questions as f64,
            escalation_rate: cost as f64 / n_questions as f64,
        }
    };

    // delta_only (the shipped gate).
    let delta_armed: Vec<bool> = suites.iter().map(|s| s.delta_armed).collect();
    let mut delta_posture = posture_of(&delta_armed);
    delta_posture.rule = "delta_only".to_string();

    // worth_only(t) sweep + combined(t) diagnostics.
    let mut postures = vec![delta_posture.clone()];
    let mut worth_postures: Vec<(f64, WorthAbPosture, Vec<bool>)> = Vec::new();
    for t in threshold_grid(margin) {
        let worth_armed: Vec<bool> = suites.iter().map(|s| worth_arms(&s.worth_arm, t)).collect();
        let mut p = posture_of(&worth_armed);
        p.rule = format!("worth_only(t={t:.2})");
        p.t = Some(t);
        postures.push(p.clone());
        worth_postures.push((t, p, worth_armed.clone()));

        let combined: Vec<bool> = delta_armed
            .iter()
            .zip(worth_armed.iter())
            .map(|(d, w)| *d || *w)
            .collect();
        let mut cp = posture_of(&combined);
        cp.rule = format!("combined(t={t:.2})");
        cp.t = Some(t);
        postures.push(cp);
    }

    // Matched-cost pick: the worth_only(t*) nearest delta_only's cost,
    // valid iff the relative gap ≤ tolerance. The t = margin posture is
    // the exact delta-only set by construction, so the grid always
    // contains a zero-gap point — the sweep exists to price everything
    // BETWEEN and disclose the curve, not to hunt for a reachable match.
    let delta_cost = delta_posture.cost;
    let mut matched = None;
    if delta_cost > 0 {
        let best = worth_postures
            .iter()
            .min_by_key(|(_, p, _)| {
                let gap = (p.cost as f64 - delta_cost as f64).abs() / delta_cost as f64;
                // Stable ordering: gap, then prefer the HIGHER t (closer
                // to the delta-only point — fewer extra suites).
                (
                    (gap * 1e9).round() as i64,
                    (-p.t.unwrap_or(0.0) * 1e9).round() as i64,
                )
            })
            .expect("non-empty grid");
        let (t, p, worth_armed) = best;
        let cost_gap_rel = (p.cost as f64 - delta_cost as f64).abs() / delta_cost as f64;
        let mut m = MatchedComparison {
            worth_t: *t,
            delta_cost,
            worth_cost: p.cost,
            cost_gap_rel,
            delta_accuracy: delta_posture.accuracy,
            worth_accuracy: p.accuracy,
            n_worth_right_delta_wrong: 0,
            n_worth_wrong_delta_right: 0,
            n_questions,
            paired_mean: 0.0,
            paired_lb95: f64::NEG_INFINITY,
            verdict: String::new(),
        };
        // Per-question paired counts composed from the flipped suites'
        // covered-abstain McNemar pairs, both directions.
        for (s, (w, d)) in suites.iter().zip(worth_armed.iter().zip(&delta_armed)) {
            if w == d {
                continue;
            }
            let (a, b) = s.hyp_mcnemar;
            if *w {
                // worth arms, delta does not: worth takes the escalator,
                // delta keeps the modelless pick.
                m.n_worth_right_delta_wrong += a;
                m.n_worth_wrong_delta_right += b;
            } else {
                m.n_worth_right_delta_wrong += b;
                m.n_worth_wrong_delta_right += a;
            }
        }
        let (a, b) = (m.n_worth_right_delta_wrong, m.n_worth_wrong_delta_right);
        let discordant = (a + b) as f64;
        m.paired_mean = (a as f64 - b as f64) / n_questions as f64;
        m.paired_lb95 = m.paired_mean - PAIRED_LB95_Z * discordant.sqrt() / n_questions as f64;
        m.verdict = if cost_gap_rel > COST_MATCH_TOLERANCE {
            "UNDECIDABLE_COST_MATCH".to_string()
        } else if a + b == 0 {
            "UNDECIDABLE_ZERO_STRADDLE".to_string()
        } else if m.paired_lb95 > 0.0 && m.worth_accuracy >= m.delta_accuracy {
            "WORTH_BEATS_DELTA".to_string()
        } else {
            "NEGATIVE".to_string()
        };
        matched = Some(m);
    }

    Ok(WorthAbBlock {
        postures,
        matched,
        cost_match_tolerance: COST_MATCH_TOLERANCE,
    })
}

/// The per-suite A/B inputs derived from one cascade report (the
/// runner's join — kept beside the report's fields so a field rename
/// breaks here first, not silently downstream).
pub fn suite_input(
    suite: &str,
    rep: &crate::harness::cascade::CascadeReport,
) -> Result<SuiteAbInput, String> {
    let Some(ab) = rep.ab.as_ref() else {
        return Err(format!(
            "suite {suite}: cascade report carries no ab counters — run with --worth-ab"
        ));
    };
    // The all-modelless outcome: the report's composition swaps the
    // escalator's picks in on the escalated set — swap them back out.
    let modelless_correct =
        rep.n_correct - rep.escalated_laya_correct + rep.escalated_modelless_correct;
    let worth_arm = rep.worthiness.as_ref().and_then(|w| {
        w.collective.as_ref().map(|c| {
            (
                c.committee.improvement,
                c.committee.committee_wilson_lcb > c.committee.floor_acc,
            )
        })
    });
    Ok(SuiteAbInput {
        suite: suite.to_string(),
        delta_armed: rep.worthiness.as_ref().is_none_or(|w| !w.disarmed),
        n_questions: rep.n_questions,
        modelless_correct,
        hyp_escalated: ab.covered_abstains,
        hyp_laya_correct: ab.laya_correct_covered,
        hyp_ml_correct: ab.ml_correct_covered,
        hyp_mcnemar: ab.mcnemar,
        worth_arm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suite(
        name: &str,
        n_questions: usize,
        delta_armed: bool,
        modelless_correct: usize,
        hyp: (usize, usize, usize, (usize, usize)),
        worth_arm: Option<(f64, bool)>,
    ) -> SuiteAbInput {
        SuiteAbInput {
            suite: name.to_string(),
            delta_armed,
            n_questions,
            modelless_correct,
            hyp_escalated: hyp.0,
            hyp_laya_correct: hyp.1,
            hyp_ml_correct: hyp.2,
            hyp_mcnemar: hyp.3,
            worth_arm,
        }
    }

    #[test]
    fn delta_only_matches_the_run_posture() {
        // Two suites; the A/B's delta_only row must reproduce the
        // armed/disarmed outcomes the reports carried.
        let suites = vec![
            suite("a", 100, true, 60, (30, 24, 12, (18, 6)), None),
            suite("b", 100, false, 55, (20, 6, 8, (2, 4)), None),
        ];
        let out = evaluate(&suites, 0.16).expect("evaluate");
        let delta = out.postures.first().expect("delta row");
        assert_eq!(delta.rule, "delta_only");
        assert_eq!(delta.armed_suites, vec!["a".to_string()]);
        // armed a: 60 + 24 - 12 = 72; disarmed b: 55 → 127.
        assert_eq!(delta.correct, 127);
        assert_eq!(delta.cost, 30);
        assert!((delta.accuracy - 0.635).abs() < 1e-9);
    }

    #[test]
    fn worth_arm_leg_respects_threshold_and_support() {
        // Suite worth_arm (improvement 0.105, supported): arms at t ≤ 0.10,
        // not at 0.11; unsupported never arms.
        let suites = vec![
            suite(
                "a",
                100,
                false,
                60,
                (30, 24, 12, (18, 6)),
                Some((0.105, true)),
            ),
            suite(
                "b",
                100,
                false,
                55,
                (20, 10, 10, (4, 4)),
                Some((0.20, false)),
            ),
        ];
        let out = evaluate(&suites, 0.16).expect("evaluate");
        let at_010 = out
            .postures
            .iter()
            .find(|p| p.rule == "worth_only(t=0.10)")
            .expect("t=0.10 row");
        let at_011 = out
            .postures
            .iter()
            .find(|p| p.rule == "worth_only(t=0.11)")
            .expect("t=0.11 row");
        // a arms at 0.10 (60 + 24 - 12 = 72; b stays disarmed at 55 → the
        // posture total is 127, identical to delta_only here — the arm set
        // is the distinguishing fact), b never (unsupported).
        assert_eq!(at_010.armed_suites, vec!["a".to_string()]);
        assert_eq!(at_010.correct, 127);
        assert_eq!(at_010.cost, 30);
        assert!(at_011.armed_suites.is_empty());
        assert_eq!(at_011.correct, 115);
    }

    #[test]
    fn matched_counts_compose_both_flip_directions() {
        // a: worth-armed only at t≤0.10 (cost 4); b: delta-armed (cost 100)
        // with no worth leg. Nearest worth posture to delta's 100: t=0.10
        // ({a} cost 4, gap 96%) beats t=0.11+ ({} cost 0, gap 100%) →
        // UNDECIDABLE_COST_MATCH, and the paired counts compose over BOTH
        // flipped suites: a (worth−delta += (24,6)), b reversed += (2,12).
        let suites = vec![
            suite("a", 100, false, 60, (4, 3, 0, (3, 0)), Some((0.10, true))),
            suite("b", 100, true, 55, (100, 80, 60, (24, 6)), None),
        ];
        let out = evaluate(&suites, 0.16).expect("evaluate");
        let m = out.matched.expect("matched (delta cost > 0)");
        assert_eq!(m.verdict, "UNDECIDABLE_COST_MATCH");
        assert_eq!(m.worth_t, 0.10);
        assert_eq!(m.n_worth_right_delta_wrong, 3 + 6);
        assert_eq!(m.n_worth_wrong_delta_right, 24);
        // Paired mean over 200 questions: (9 − 24) / 200.
        assert!((m.paired_mean - (-15.0) / 200.0).abs() < 1e-9);
    }

    #[test]
    fn zero_straddle_reads_undecidable() {
        // The matched posture IS the delta set (worth arms exactly the
        // delta-armed suite at every t) → no flipped suite → a+b = 0 →
        // the zero-straddle undecidable, never folded into either verdict.
        let suites = vec![suite(
            "a",
            100,
            true,
            60,
            (30, 24, 12, (18, 6)),
            Some((0.30, true)),
        )];
        let out = evaluate(&suites, 0.16).expect("evaluate");
        let m = out.matched.expect("matched at gap 0");
        assert_eq!(m.n_worth_right_delta_wrong + m.n_worth_wrong_delta_right, 0);
        assert_eq!(m.verdict, "UNDECIDABLE_ZERO_STRADDLE");
    }

    #[test]
    fn worth_beats_delta_fires_on_a_clear_lb95() {
        // d: delta-armed AND worth-armed at t≤0.15 (cost 100); t: worth
        // arms at t≤0.10 only (cost 4, escalator dominates: a=4 b=0).
        // Matched at t=0.15: {d, t} cost 104, gap 4% ≤ 5% → the flipped
        // t suite contributes (4, 0) → LB95 > 0 and worth ≥ delta →
        // WORTH_BEATS_DELTA.
        let suites = vec![
            suite(
                "d",
                100,
                true,
                50,
                (100, 60, 40, (30, 10)),
                Some((0.15, true)),
            ),
            suite("t", 100, false, 50, (4, 4, 0, (4, 0)), Some((0.15, true))),
        ];
        let out = evaluate(&suites, 0.16).expect("evaluate");
        let m = out.matched.expect("matched at t=0.15");
        assert_eq!(m.worth_t, 0.15);
        assert!(m.cost_gap_rel <= 0.05);
        assert_eq!(m.n_worth_right_delta_wrong, 4);
        assert_eq!(m.n_worth_wrong_delta_right, 0);
        assert_eq!(m.verdict, "WORTH_BEATS_DELTA");
        assert!(m.paired_lb95 > 0.0);
        assert!(m.worth_accuracy >= m.delta_accuracy);
    }

    #[test]
    fn negative_fires_when_the_escalator_loses() {
        // Same shape as the worth-beats cell but the flipped suite's
        // escalator loses (a=0 b=4) → LB95 < 0 → NEGATIVE.
        let suites = vec![
            suite(
                "d",
                100,
                true,
                50,
                (100, 60, 40, (30, 10)),
                Some((0.15, true)),
            ),
            suite("t", 100, false, 50, (4, 0, 4, (0, 4)), Some((0.15, true))),
        ];
        let out = evaluate(&suites, 0.16).expect("evaluate");
        let m = out.matched.expect("matched at t=0.15");
        assert!(m.cost_gap_rel <= 0.05);
        assert_eq!(m.verdict, "NEGATIVE");
        assert!(m.paired_lb95 <= 0.0);
    }

    #[test]
    fn empty_inputs_refuse_loud() {
        assert!(evaluate(&[], 0.16).is_err());
    }
}
