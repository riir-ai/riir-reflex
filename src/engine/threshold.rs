//! The threshold-recommendation surface (Issue 009 — the jimothy steal:
//! the per-task abstention cutoff as fit-time engine metadata).
//!
//! jimothy (`github.com/AndrewPrifer/jimothy` — the adjacent TRAINED
//! per-task classifier) ships the CONTRACT this module steals, not the
//! method:
//!
//! 1. **Recommend at fit time** — the cutoff is computed once from a
//!    labeled cal slice ([`threshold_recommendation`]), never hardcoded
//!    and never re-derived per call.
//! 2. **Return everything at run time** — the engine never silently drops
//!    to abstain; applying a cutoff stays the consumer's choice (the
//!    harness posture: fitted thresholds ride the result row, the engine
//!    keeps answering).
//! 3. **Null on thin support** — [`None`] (jimothy's `null`) when the cal
//!    slice is smaller than [`THIN_SUPPORT_FLOOR`], and every
//!    recommendation DISCLOSES its support ([`ThresholdSupport`]) — the
//!    percentile/tail-support law: never a confident number over 3 rows.
//! 4. **Validate in the deployment environment** — a threshold fitted under
//!    one corpus/runtime posture is a claim with a timestamp on it; the
//!    metadata carries the [`Posture`] so consumers cite it alongside the
//!    number.
//!
//! Modelless by construction: the fit is a deterministic scan over observed
//! sigmoid-gate scores (no RNG, no trained calibrator — jimothy's method is
//! the adjacent baseline named for comparison, never adopted).
//!
//! ## Percentile parity (the T2 migration bar)
//!
//! [`Posture::Percentile`] reproduces the harness runner's `quantile` law
//! VERBATIM — ascending `total_cmp` sort, `idx = floor(n·ρ)` clamped to
//! `n-1`, take `s[idx]` — so migrating the harness to this surface
//! (Issue 009 T2) is byte-identical on every current suite. The law's known
//! small-n coarseness (`idx` can land on `n-1`, the MAX, for high ρ at
//! small n) is not "fixed" here: fixing it would move published numbers,
//! which is a re-fit, not a migration. The [`ThresholdSupport`] disclosure
//! is the honest answer.
//!
//! NaN scores: `total_cmp` is a total order (NaN sorts above `+inf`), and a
//! NaN score never passes a threshold (`NaN >= t` is false) — NaN rows
//! deterministically abstain and are counted in
//! [`ThresholdSupport::n_abstain`].
//!
//! ## RiskTarget — certified abstention (Issue 112)
//!
//! [`Posture::RiskTarget`] is Learn-then-Test (arXiv:2110.01052; the
//! typed-decision application is ThinkJev §3.6, arXiv:2610.37732): the
//! loosest cutoff on a PRE-REGISTERED fixed grid
//! ([`RISK_TARGET_GRID_POINTS`]) whose accepted-subset error carries a
//! one-sided Clopper–Pearson upper bound `<= alpha` at level `delta`,
//! scanned strictest→loosest with the fixed-sequence stop. The group unit
//! is the cal case (one decision = one group — conservative only against
//! a duplicate-ful slice, which our cal slices do not carry today); a
//! candidate is testable only at or above the zero-error floor
//! `ceil(ln(delta)/ln(1−alpha))` (59 at α=5%, 299 at α=1%, δ=0.05). The
//! fixed-sequence stop has a shape worth naming: the strictest ADMISSIBLE
//! candidate sits at n ≈ floor — the loosest CP bound — so a slice dirty
//! there certifies NOTHING even if a looser large-n cutoff would pass
//! ("stop at the first failure", T1 — never a multiplicity correction).
//! The honest expectation: at our cal-slice sizes strict α certifies ~nothing
//! — the loud `NotCertified` IS the deliverable, never a failure. The
//! local exact-CP bound is TEMPORARY (superseded by the katgpt-core
//! export, Issue 112 T3a; the Bench-615 seed + the E=0 closed-form
//! canary); serving wiring is a separate owner-gated step.

use serde::{Deserialize, Serialize};

/// Minimum labeled observations before a recommendation exists (jimothy's
/// null rule). 16 is the floor the T1.5 harness measured in as its
/// `confs.len() < 16` fallback to the birth constants — frozen here so the
/// surface and the harness agree on what "thin" means.
pub const THIN_SUPPORT_FLOOR: usize = 16;

/// One labeled cal-slice observation: the gate score the cutoff applies to,
/// and whether the engine's answer on that case was correct. Percentile
/// postures select on `score` only; `correct` rides along as the accuracy
/// disclosure ([`ThresholdRecommendation::target_accuracy`]); the
/// target-accuracy posture selects on both.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GateObservation {
    /// The observed gate score (calibrated confidence for the score axis,
    /// corpus-distance confidence for the distance axis).
    pub score: f32,
    /// Whether the engine's top answer on the cal case was correct.
    pub correct: bool,
}

/// The fit posture: which deterministic selection the scan performs. Rides
/// the metadata (contract part 4 — cite the posture with the number).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "posture", rename_all = "camelCase")]
pub enum Posture {
    /// The arena posture (T1.6, ρ = 30%): threshold at the ρ-quantile of
    /// the observed scores — ρ of in-distribution questions abstain, the
    /// rest pass. Reproduces the harness `quantile` law exactly (the T2
    /// migration bar).
    Percentile {
        /// Target abstain rate, `0.0 <= rho <= 1.0`.
        rho: f64,
    },
    /// jimothy's shape: the LOWEST cutoff whose above-cutoff accuracy on
    /// the cal slice meets `target` (max answered volume at target
    /// quality). When no cutoff meets it, the best-achievable cutoff is
    /// returned with [`RecommendationStatus::TargetUnmet`] — disclosed,
    /// never silent.
    TargetAccuracy {
        /// Target accuracy, `0.0 < target <= 1.0`.
        target: f64,
    },
    /// Learn-then-Test certified abstention (Issue 112; arXiv:2110.01052,
    /// worked on typed decisions by ThinkJev §3.6): the LOOSEST cutoff on a
    /// PRE-REGISTERED fixed grid whose accepted-subset error carries a
    /// one-sided Clopper–Pearson upper bound `<= alpha` at level `delta`,
    /// scanned strictest→loosest with the fixed-sequence stop (first tested
    /// failure ends the scan — no multiplicity correction). When nothing
    /// certifies, the surface discloses
    /// [`RecommendationStatus::NotCertified`] and
    /// [`threshold_recommendation`] returns `None` — which geometry answers
    /// stays the caller's (the percentile fit's) business, never a silent
    /// point-estimate fallback. Opt-in: produced only when a lever arms it;
    /// no shipped default run ever emits this variant (the additive-only
    /// wire law — old readers fail on unknown enum variants).
    RiskTarget {
        /// Certified accepted-subset error target, `0.0 < alpha < 1.0`.
        alpha: f64,
        /// Confidence level: the bound holds with probability `>= 1 −
        /// delta`. The v1 harness lever pins `0.05`, disclosed.
        delta: f64,
    },
}

/// Whether the posture's target was reachable (percentile postures have no
/// accuracy target and are always [`RecommendationStatus::Fitted`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecommendationStatus {
    /// The posture's selection held (percentile fit, or a target that was
    /// met).
    Fitted,
    /// No cutoff achieved the target — the recommendation is the
    /// best-achievable cutoff and `target_accuracy` reports what it
    /// achieves.
    TargetUnmet,
    /// The RiskTarget posture certified NOTHING (Issue 112): no candidate
    /// on the fixed grid carries a Clopper–Pearson upper bound `<= alpha`
    /// at level `delta` (or none had the accepted-group count the
    /// zero-error floor demands). A met target without a guarantee and a
    /// failed certification are different disclosures — never reused for
    /// each other. Lives on [`RiskTargetFit`]; the `Option`-shaped
    /// recommendation surface answers with absence instead, so the
    /// caller's declared geometry stays in charge.
    NotCertified,
}

/// Support disclosure (the tail-support law): what the fit actually saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdSupport {
    /// Observations the fit saw (`n >= THIN_SUPPORT_FLOOR` or the fit is
    /// `None`).
    pub n: usize,
    /// Observations at or above the recommended threshold (would pass).
    pub n_pass: usize,
    /// Observations below it (would abstain).
    pub n_abstain: usize,
}

/// A fit-time cutoff recommendation (jimothy's
/// `metadata.thresholdRecommendation` shape; [`None`] is his `null`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdRecommendation {
    /// The recommended cutoff: `score >= threshold` passes, below abstains.
    pub threshold: f32,
    /// [`RecommendationStatus::Fitted`] /
    /// [`RecommendationStatus::TargetUnmet`].
    pub status: RecommendationStatus,
    /// The realized accuracy at-or-above the cutoff on the cal slice — for
    /// a met target posture this is `>= target`; percentile postures
    /// report the accuracy their cutoff happens to keep (a disclosure,
    /// never a selection criterion).
    pub target_accuracy: f32,
    /// What the fit saw.
    pub support: ThresholdSupport,
}

/// The fused-gate pair (both axes) under one posture — the shape the
/// harness fits (Issue 009 T2's entry point) and the metadata surface
/// cites: `thresholdRecommendation: { posture, score, distance }`, `null`
/// per axis on thin support.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FusedGateRecommendation {
    /// The posture that produced both numbers (cite it with them — contract
    /// part 4).
    pub posture: Posture,
    /// Score-axis recommendation (calibrated-confidence cutoff).
    pub score: Option<ThresholdRecommendation>,
    /// Distance-axis recommendation (corpus-distance-confidence cutoff).
    pub distance: Option<ThresholdRecommendation>,
}

/// Recommend a cutoff for one gate axis from a labeled cal slice.
///
/// Deterministic: no RNG, no time, no ambient state — the same observations
/// and posture always produce a bit-identical recommendation (pinned by
/// gate). Fit-time only; never a per-call cost.
pub fn threshold_recommendation(
    observations: &[GateObservation],
    posture: Posture,
) -> Option<ThresholdRecommendation> {
    // jimothy's null rule — thin support is disclosed by ABSENCE, never a
    // confident number over a handful of rows.
    if observations.len() < THIN_SUPPORT_FLOOR {
        return None;
    }
    match posture {
        Posture::Percentile { rho } => {
            assert!(
                (0.0..=1.0).contains(&rho),
                "percentile posture requires rho in [0, 1]"
            );
            let threshold = percentile_law(observations, rho);
            Some(finish(
                observations,
                threshold,
                RecommendationStatus::Fitted,
            ))
        }
        Posture::TargetAccuracy { target } => {
            assert!(
                target > 0.0 && target <= 1.0,
                "target-accuracy posture requires target in (0, 1]"
            );
            let (threshold, status) = target_scan(observations, target);
            Some(finish(observations, threshold, status))
        }
        Posture::RiskTarget { alpha, delta } => {
            assert!(
                alpha > 0.0 && alpha < 1.0,
                "risk-target posture requires alpha in (0, 1)"
            );
            assert!(
                delta > 0.0 && delta < 1.0,
                "risk-target posture requires delta in (0, 1)"
            );
            // NotCertified answers with ABSENCE — which geometry answers
            // stays the caller's (the percentile fit's) business, never a
            // silent point-estimate fallback. The full disclosure state
            // (status, floor, the certified count) lives on
            // [`risk_target_fit`], which the harness lever reports.
            risk_target_fit(observations, alpha, delta).and_then(|f| f.recommendation)
        }
    }
}

/// Fit both fused-gate axes under one posture (the harness T2 entry point).
///
/// The axes are fitted independently (the T1.5 law: score threshold from
/// calibrated confidences, distance threshold from corpus-distance
/// confidences, both at the same ρ); each axis carries its own thin-support
/// null.
pub fn recommend_fused_gate(
    score_obs: &[GateObservation],
    distance_obs: &[GateObservation],
    posture: Posture,
) -> FusedGateRecommendation {
    FusedGateRecommendation {
        posture,
        score: threshold_recommendation(score_obs, posture),
        distance: threshold_recommendation(distance_obs, posture),
    }
}

/// The harness runner's `quantile`, frozen verbatim (the T2 migration bar —
/// see the module doc). Callers guarantee `n >= 1` and `0 <= rho <= 1`.
fn percentile_law(observations: &[GateObservation], rho: f64) -> f32 {
    let mut s: Vec<f32> = observations.iter().map(|o| o.score).collect();
    s.sort_by(|a, b| a.total_cmp(b));
    let idx = ((s.len() as f64) * rho) as usize;
    s[idx.min(s.len() - 1)]
}

/// Deterministic target-accuracy scan: sweep DISTINCT score boundaries in
/// descending order; the pass set (`score >= t`) grows as the threshold
/// drops. Returns the lowest threshold whose above-cutoff accuracy still
/// meets `target`; when none does, the best-achievable threshold (accuracy
/// argmax, ties broken toward the larger pass set). O(B·n) over B distinct
/// boundaries — fit-time only, cal slices are tens of rows.
fn target_scan(observations: &[GateObservation], target: f64) -> (f32, RecommendationStatus) {
    let mut boundaries: Vec<f32> = observations.iter().map(|o| o.score).collect();
    boundaries.sort_by(|a, b| b.total_cmp(a));
    boundaries.dedup();
    let mut chosen: Option<f32> = None;
    let mut best: Option<(f32, f32)> = None; // (threshold, accuracy)
    for &t in &boundaries {
        let (n_pass, n_correct) = observations.iter().fold((0usize, 0usize), |(p, c), o| {
            if o.score >= t {
                (p + 1, c + o.correct as usize)
            } else {
                (p, c)
            }
        });
        // n_pass >= 1 at every boundary: the boundary value itself passes.
        let acc = n_correct as f32 / n_pass as f32;
        if acc >= target as f32 {
            chosen = Some(t); // keep overwriting → the LOWEST meeting threshold
        }
        match best {
            Some((_, best_acc)) if acc < best_acc => {}
            // equal accuracy → prefer the LOWER threshold (larger pass set)
            _ => best = Some((t, acc)),
        }
    }
    chosen
        .map(|t| (t, RecommendationStatus::Fitted))
        .unwrap_or_else(|| {
            let (t, _) = best.expect("at least one distinct score exists");
            (t, RecommendationStatus::TargetUnmet)
        })
}

/// The v1 candidate grid (Issue 112 T1, pre-registered): 1000 evenly
/// spaced points in `[0, 1]` over the calibrated score. The candidate set
/// MUST be fixed before the data is inspected — a cal-derived grid would
/// test hypotheses chosen by the data, and the fixed-sequence guarantee's
/// premise would break.
pub const RISK_TARGET_GRID_POINTS: usize = 1000;

/// The certified-abstention fit state (Issue 112). [`Option`]-shaped
/// consumers see only `recommendation` (absence when nothing certified);
/// the full state is the disclosure surface the harness lever reports.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskTargetFit {
    /// Certified accepted-subset error target (echoed for the disclosure).
    pub alpha: f64,
    /// Confidence level: the bound holds with probability `>= 1 − delta`.
    pub delta: f64,
    /// [`RecommendationStatus::Fitted`] when a candidate certified,
    /// [`RecommendationStatus::NotCertified`] otherwise.
    pub status: RecommendationStatus,
    /// `Some` iff `status == Fitted` — the loosest certified cutoff with
    /// its support/accuracy disclosures (`finish`-computed, tie-inclusive
    /// `score >= threshold`, same semantics as the other postures).
    pub recommendation: Option<ThresholdRecommendation>,
    /// Accepted groups at the certified cutoff (0 when nothing certified).
    pub n_groups: usize,
    /// The zero-error pass floor: the smallest group count whose ALL-CORRECT
    /// accepted subset still certifies, `ceil(ln(delta) / ln(1 − alpha))` —
    /// below it no candidate is even testable, and the disclosure names the
    /// floor rather than guessing.
    pub floor_groups: usize,
}

/// Fit the RiskTarget posture (Issue 112): the certified-abstention scan
/// over the pre-registered fixed grid.
///
/// Deterministic: fixed grid, fixed scan order (strictest → loosest),
/// fixed-sequence stop, bounded deterministic bisection — the same
/// observations always produce a bit-identical fit. `None` below
/// [`THIN_SUPPORT_FLOOR`] (jimothy's null rule, same as the other
/// postures).
pub fn risk_target_fit(
    observations: &[GateObservation],
    alpha: f64,
    delta: f64,
) -> Option<RiskTargetFit> {
    assert!(
        alpha > 0.0 && alpha < 1.0,
        "risk-target requires alpha in (0, 1)"
    );
    assert!(
        delta > 0.0 && delta < 1.0,
        "risk-target requires delta in (0, 1)"
    );
    if observations.len() < THIN_SUPPORT_FLOOR {
        return None;
    }
    let floor_groups = zero_error_floor(alpha, delta);
    // Strictest → loosest over the fixed grid. Candidates below the
    // zero-error floor are INADMISSIBLE (they can never pass — skip, they
    // are not test failures and do not stop the fixed sequence); the first
    // TESTED failure stops the scan (fixed-sequence testing — no
    // multiplicity correction needed).
    let mut certified: Option<(f32, usize)> = None;
    for i in (0..RISK_TARGET_GRID_POINTS).rev() {
        let t = i as f32 / (RISK_TARGET_GRID_POINTS - 1) as f32;
        let (n_groups, n_errors) = observations.iter().fold((0usize, 0usize), |(n, k), o| {
            if o.score >= t {
                (n + 1, k + (!o.correct) as usize)
            } else {
                (n, k)
            }
        });
        if n_groups < floor_groups {
            continue;
        }
        if cp_upper_bound(n_errors, n_groups, delta) <= alpha {
            certified = Some((t, n_groups));
        } else {
            break;
        }
    }
    let fit = match certified {
        Some((t, n_groups)) => RiskTargetFit {
            alpha,
            delta,
            status: RecommendationStatus::Fitted,
            recommendation: Some(finish(observations, t, RecommendationStatus::Fitted)),
            n_groups,
            floor_groups,
        },
        None => RiskTargetFit {
            alpha,
            delta,
            status: RecommendationStatus::NotCertified,
            recommendation: None,
            n_groups: 0,
            floor_groups,
        },
    };
    Some(fit)
}

/// The zero-error pass floor (Issue 112 T2): the smallest accepted-group
/// count whose ALL-CORRECT subset still certifies at `(alpha, delta)` —
/// `ceil(ln(delta) / ln(1 − alpha))`, computed by the closed form then
/// verified by exact integer powering (59 at α=5%, 299 at α=1%, both at
/// δ=0.05 — pinned by test). CEIL, never an approximation.
fn zero_error_floor(alpha: f64, delta: f64) -> usize {
    let mut m = (delta.ln() / (1.0 - alpha).ln()).ceil() as usize;
    // f64 correction loop: shift until the boundary is unambiguous in the
    // exact powering direction (at most one step either way in practice).
    while m > 1 && (1.0 - alpha).powi(m as i32 - 1) <= delta {
        m -= 1;
    }
    while (1.0 - alpha).powi(m as i32) > delta {
        m += 1;
    }
    m
}

// TEMPORARY: superseded by the katgpt-core export (Issue 112 T3a — the
// filed katgpt-rs issue; retire this block when the export lands and
// consume `katgpt_core::…cp_upper` instead). The seed is Bench 615's
// private `cp_upper` bisection + E=0 closed form (katgpt-rs
// `crates/katgpt-core/benches/bench_615_two_fidelity_bai_goat.rs`, the
// one-sided Clopper–Pearson PAC precedent; `certified_frontier` exports
// only the closed-form `beta_union_bound`, whose own doc names the exact
// CP quantile as the upgrade).
/// One-sided Clopper–Pearson UPPER bound on the error rate: the smallest
/// `p` with `P(X <= errors | p) = delta`, i.e. the `(1 − delta)`-quantile
/// of `Beta(errors + 1, n − errors)`. `errors >= n` answers 1.0 (the data
/// cannot rule out `p = 1`); `errors == 0` takes the E=0 closed form
/// `1 − delta^(1/n)` (pinned by test); otherwise a bounded deterministic
/// bisection on the regularized incomplete beta.
fn cp_upper_bound(errors: usize, n: usize, delta: f64) -> f64 {
    assert!(n >= 1, "cp_upper_bound needs n >= 1");
    assert!(delta > 0.0 && delta < 1.0, "delta in (0, 1)");
    if errors >= n {
        return 1.0;
    }
    if errors == 0 {
        return 1.0 - delta.powf(1.0 / n as f64);
    }
    let a = (errors + 1) as f64;
    let b = (n - errors) as f64;
    // f(p) = I_{1−p}(b, a) is the binomial CDF at `errors` under `p`,
    // strictly decreasing on (0, 1) from 1 to 0; bisect f(p) = delta.
    let mut lo = 0.0f64;
    let mut hi = 1.0f64;
    while hi - lo > 1e-15 {
        let mid = 0.5 * (lo + hi);
        if reg_inc_beta(b, a, 1.0 - mid) > delta {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Regularized incomplete beta `I_x(a, b)` — continued fraction with the
/// symmetry split (the Numerical Recipes shape), f64, deterministic.
fn reg_inc_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let ln_front = ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (-x).ln_1p();
    let front = ln_front.exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        front * betacf(a, b, x) / a
    } else {
        1.0 - front * betacf(b, a, 1.0 - x) / b
    }
}

/// The continued fraction for the incomplete beta (Lentz's method, the
/// Numerical Recipes `betacf` shape), f64, bounded-iteration deterministic.
fn betacf(a: f64, b: f64, x: f64) -> f64 {
    const MAX_IT: usize = 300;
    const EPS: f64 = 3.0e-12;
    const FPMIN: f64 = 1.0e-300;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0f64;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < FPMIN {
        d = FPMIN;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=MAX_IT {
        let m = m as f64;
        let m2 = 2.0 * m;
        let mut aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        h *= d * c;
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < EPS {
            break;
        }
    }
    h
}

/// Lanczos ln-gamma (g=7, n=9 coefficients — the Numerical Recipes
/// shape), f64, deterministic. Valid for x > 0.
fn ln_gamma(x: f64) -> f64 {
    const COF: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_5,
        12.507_343_278_686_9,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_6e-6,
        1.505_632_735_149_3e-7,
    ];
    if x < 0.5 {
        // Reflection for the small-x branch (unused by the CP bisection,
        // whose arguments are integers >= 1, but kept complete).
        (std::f64::consts::PI / x.sin()).ln() - ln_gamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut a = COF[0];
        let t = x + 7.5;
        for (i, &c) in COF.iter().enumerate().skip(1) {
            a += c / (x + i as f64);
        }
        0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
    }
}

/// Recompute support and realized accuracy AT the threshold (tie-inclusive:
/// every observation with `score >= threshold` passes, including duplicates
/// of the boundary value) — the disclosed numbers always match the actual
/// threshold semantics, never the scan's prefix arithmetic.
fn finish(
    observations: &[GateObservation],
    threshold: f32,
    status: RecommendationStatus,
) -> ThresholdRecommendation {
    let mut n_pass = 0usize;
    let mut n_correct = 0usize;
    for o in observations {
        if o.score >= threshold {
            n_pass += 1;
            n_correct += o.correct as usize;
        }
    }
    let target_accuracy = if n_pass > 0 {
        n_correct as f32 / n_pass as f32
    } else {
        0.0
    };
    ThresholdRecommendation {
        threshold,
        status,
        target_accuracy,
        support: ThresholdSupport {
            n: observations.len(),
            n_pass,
            n_abstain: observations.len() - n_pass,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic labeled observations (no RNG dep, no seed drift).
    fn obs_from_scores(scores: &[f32], correct_above: f32) -> Vec<GateObservation> {
        scores
            .iter()
            .enumerate()
            .map(|(i, &s)| GateObservation {
                score: s,
                correct: (s / 100.0) >= correct_above || (i % 7 == 0),
            })
            .collect()
    }

    /// Distinct descending scores 100, 99, ... (n rows).
    fn distinct_scores(n: usize) -> Vec<f32> {
        (0..n).map(|i| 100.0 - i as f32).collect()
    }

    #[test]
    fn thin_support_is_null_below_floor() {
        let floor_pin = 16; // the T1.5 harness fallback (`confs.len() < 16`)
        assert_eq!(THIN_SUPPORT_FLOOR, floor_pin);
        let thin = obs_from_scores(&distinct_scores(15), 0.5);
        let enough = obs_from_scores(&distinct_scores(16), 0.5);
        assert!(threshold_recommendation(&thin, Posture::Percentile { rho: 0.30 }).is_none());
        assert!(threshold_recommendation(&thin, Posture::TargetAccuracy { target: 0.9 }).is_none());
        assert!(threshold_recommendation(&enough, Posture::Percentile { rho: 0.30 }).is_some());
        assert!(
            threshold_recommendation(&enough, Posture::TargetAccuracy { target: 0.1 }).is_some()
        );
    }

    #[test]
    fn determinism_bit_identical() {
        let obs = obs_from_scores(&distinct_scores(50), 0.6);
        for posture in [
            Posture::Percentile { rho: 0.30 },
            Posture::TargetAccuracy { target: 0.7 },
        ] {
            let a = threshold_recommendation(&obs, posture).unwrap();
            let b = threshold_recommendation(&obs, posture).unwrap();
            assert_eq!(a, b);
            assert_eq!(a.threshold.to_bits(), b.threshold.to_bits());
            assert_eq!(a.target_accuracy.to_bits(), b.target_accuracy.to_bits());
        }
    }

    #[test]
    fn percentile_parity_with_the_harness_quantile_law() {
        // The harness runner's `quantile`, replicated verbatim — the T2
        // migration bar pinned from the T1 side. If either side drifts,
        // this reds.
        let harness_quantile = |sample: &[f32], q: f64| -> f32 {
            let mut s = sample.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            let idx = ((s.len() as f64) * q) as usize;
            s[idx.min(s.len() - 1)]
        };
        // Deterministic mixed populations (duplicates + spread), not just
        // ladders — ties and clamping are where quantile laws diverge.
        let cases: Vec<Vec<f32>> = vec![
            distinct_scores(16),
            distinct_scores(31),
            distinct_scores(100),
            (0..64)
                .map(|i| ((i * 37) % 13) as f32 + if i % 3 == 0 { 0.5 } else { 0.0 })
                .collect(),
        ];
        for scores in cases {
            let obs = obs_from_scores(&scores, 0.5);
            for rho in [0.30, 0.5, 0.99] {
                let got = threshold_recommendation(&obs, Posture::Percentile { rho })
                    .unwrap()
                    .threshold;
                let want = harness_quantile(&scores, rho);
                assert_eq!(got.to_bits(), want.to_bits(), "n={}", scores.len());
            }
        }
    }

    #[test]
    fn percentile_support_discloses_the_posture() {
        // Distinct scores → n_abstain is exactly the quantile index
        // floor(n·ρ) — the posture's realized abstain rate, disclosed.
        for n in [16usize, 31, 50] {
            let obs = obs_from_scores(&distinct_scores(n), 0.5);
            let r = threshold_recommendation(&obs, Posture::Percentile { rho: 0.30 }).unwrap();
            assert_eq!(r.support.n, n);
            assert_eq!(r.support.n_pass + r.support.n_abstain, n);
            assert_eq!(r.support.n_abstain, (n as f64 * 0.30) as usize);
            assert!(r.support.n_pass >= 1);
            assert_eq!(r.status, RecommendationStatus::Fitted);
        }
    }

    #[test]
    fn target_met_picks_lowest_meeting_cutoff() {
        // 10 correct (high scores) → 10 wrong → 10 correct → 6 wrong tail.
        // At target 0.8 the meeting boundaries are k=10 (acc 1.0, t=81),
        // k=11 (0.909, t=70), k=12 (0.833, t=69); k=13 (0.769) and
        // everything deeper fall short — the recommendation is the LOWEST
        // meeting cutoff (k=12, threshold 69.0, the 12th distinct score
        // descending: the score series jumps 81 → 70 between blocks).
        let mut scores: Vec<f32> = Vec::new();
        for i in 0..10 {
            scores.push(90.0 - i as f32); // correct block
        }
        for i in 0..10 {
            scores.push(70.0 - i as f32); // wrong block
        }
        for i in 0..10 {
            scores.push(50.0 - i as f32); // correct block
        }
        for i in 0..6 {
            scores.push(30.0 - i as f32); // wrong tail (n = 36 ≥ floor)
        }
        let obs: Vec<GateObservation> = scores
            .iter()
            .map(|&s| GateObservation {
                score: s,
                correct: s > 80.0 || (40.0..60.0).contains(&s),
            })
            .collect();
        let r = threshold_recommendation(&obs, Posture::TargetAccuracy { target: 0.8 }).unwrap();
        assert_eq!(r.status, RecommendationStatus::Fitted);
        assert_eq!(r.support.n_pass, 12);
        assert!((r.target_accuracy - 10.0 / 12.0).abs() < 1e-6);
        assert_eq!(r.threshold, 69.0);
    }

    #[test]
    fn target_unmet_returns_best_achievable() {
        // All wrong: no cutoff reaches 0.5 — the best-achievable accuracy
        // is 0.0 at every boundary, tie broken toward the largest pass set
        // (the lowest threshold).
        let scores = distinct_scores(20);
        let obs: Vec<GateObservation> = scores
            .iter()
            .map(|&s| GateObservation {
                score: s,
                correct: false,
            })
            .collect();
        let r = threshold_recommendation(&obs, Posture::TargetAccuracy { target: 0.5 }).unwrap();
        assert_eq!(r.status, RecommendationStatus::TargetUnmet);
        assert_eq!(r.target_accuracy, 0.0);
        assert_eq!(r.support.n_pass, obs.len());
        assert_eq!(r.support.n_abstain, 0);
    }

    #[test]
    fn ties_at_threshold_pass_together() {
        // All-equal scores: ρ=0.30 lands on the shared value, everything
        // passes (>= threshold), nothing abstains, accuracy is the honest
        // population rate.
        let scores = [0.6f32; 24];
        let obs: Vec<GateObservation> = scores
            .iter()
            .enumerate()
            .map(|(i, &s)| GateObservation {
                score: s,
                correct: i % 2 == 0,
            })
            .collect();
        let r = threshold_recommendation(&obs, Posture::Percentile { rho: 0.30 }).unwrap();
        assert_eq!(r.threshold, 0.6);
        assert_eq!(r.support.n_pass, 24);
        assert_eq!(r.support.n_abstain, 0);
        assert_eq!(r.target_accuracy, 0.5);
    }

    #[test]
    fn nan_rows_abstain_deterministically() {
        let mut scores = distinct_scores(18);
        scores[3] = f32::NAN;
        scores[17] = f32::NAN;
        let obs = obs_from_scores(&scores, 0.5);
        let r = threshold_recommendation(&obs, Posture::Percentile { rho: 0.30 }).unwrap();
        assert_eq!(r.support.n_pass + r.support.n_abstain, r.support.n);
        // NaN sorts above +inf under total_cmp but never passes a cutoff —
        // both NaN rows sit in the abstain side deterministically.
        assert_eq!(r.support.n_abstain, 2 + (18.0f64 * 0.30) as usize);
    }

    #[test]
    fn serde_metadata_shape_matches_jimothy() {
        // Numbers only, camelCase — the `metadata.thresholdRecommendation`
        // shape downstream tables + the arena site cite verbatim.
        let obs = obs_from_scores(&distinct_scores(20), 0.5);
        let fused = recommend_fused_gate(&obs, &obs, Posture::Percentile { rho: 0.30 });
        let json = serde_json::to_string(&fused).unwrap();
        assert!(json.contains("\"posture\":\"percentile\""));
        assert!(json.contains("\"rho\":0.3"));
        assert!(json.contains("\"targetAccuracy\""));
        assert!(json.contains("\"nPass\""));
        assert!(json.contains("\"nAbstain\""));
        // The null rule renders as real JSON null per axis.
        let thin = obs_from_scores(&distinct_scores(3), 0.5);
        let fused_thin = recommend_fused_gate(&thin, &thin, Posture::Percentile { rho: 0.30 });
        let json_thin = serde_json::to_string(&fused_thin).unwrap();
        assert!(json_thin.contains("\"score\":null"));
        assert!(json_thin.contains("\"distance\":null"));
    }

    // ── Issue 112 — the RiskTarget posture ─────────────────────────────────

    /// The zero-error floor at (alpha, delta): smallest m with
    /// (1 − alpha)^m <= delta — the independent integer-powering oracle
    /// the closed form is verified against.
    fn floor_by_powering(alpha: f64, delta: f64) -> usize {
        let mut m = 1usize;
        let keep = 1.0 - alpha;
        let mut p = keep;
        while p > delta {
            p *= keep;
            m += 1;
        }
        m
    }

    /// Independent one-sided binomial upper bound: direct search over p via
    /// the EXACT pmf sum P(X <= k | p) — no incomplete beta anywhere. The
    /// bisection's answer is validated against elementary probability.
    fn binom_cdf(k: usize, n: usize, p: f64) -> f64 {
        // pmf(0) = (1-p)^n; pmf(i+1) = pmf(i) * (n-i)/(i+1) * p/(1-p)
        let q = 1.0 - p;
        let mut pmf = q.powi(n as i32);
        let mut cdf = pmf;
        for i in 0..k {
            pmf *= (n - i) as f64 / (i + 1) as f64 * p / q;
            cdf += pmf;
        }
        cdf.min(1.0)
    }

    #[test]
    fn zero_error_floor_pins_both_values() {
        // CEIL, never approx — the ThinkJev §3.6 values, both pinned.
        assert_eq!(floor_by_powering(0.05, 0.05), 59);
        assert_eq!(floor_by_powering(0.01, 0.05), 299);
        // The posture's closed form + correction loop agree with the
        // powering oracle at the pinned points and across a small sweep.
        let floor = |a: f64, d: f64| super::zero_error_floor(a, d);
        assert_eq!(floor(0.05, 0.05), 59);
        assert_eq!(floor(0.01, 0.05), 299);
        for &(a, d) in &[
            (0.02, 0.05),
            (0.05, 0.10),
            (0.10, 0.05),
            (0.10, 0.10),
            (0.20, 0.05),
            (0.01, 0.01),
        ] {
            assert_eq!(floor(a, d), floor_by_powering(a, d), "alpha={a} delta={d}");
        }
    }

    #[test]
    fn cp_upper_e0_closed_form_and_monotone_bracketing() {
        // E=0 closed form pinned verbatim.
        for &(n, d) in &[(59usize, 0.05f64), (299, 0.05), (100, 0.10), (16, 0.05)] {
            let want = 1.0 - d.powf(1.0 / n as f64);
            let got = super::cp_upper_bound(0, n, d);
            assert!((got - want).abs() < 1e-15, "n={n} d={d}");
        }
        // All-wrong answers 1.0; k=n convention.
        assert_eq!(super::cp_upper_bound(10, 10, 0.05), 1.0);
        // Monotone in errors, never undercuts the observed rate, below 1
        // when k < n.
        for n in [16usize, 59, 120] {
            let mut prev = 0.0f64;
            for k in 0..=n {
                let u = super::cp_upper_bound(k, n, 0.05);
                assert!(u >= prev - 1e-12, "monotone k={k}");
                assert!(u >= k as f64 / n as f64 - 1e-12, ">= rate k={k}");
                if k < n {
                    assert!(u < 1.0);
                }
                prev = u;
            }
        }
    }

    #[test]
    fn cp_upper_matches_the_independent_binomial_cdf_root() {
        // For several (k, n, delta): P(X <= k | cp_upper) must EQUAL delta —
        // the bound is the root of the exact binomial CDF (validated by the
        // elementary pmf sum, an independent method from the incomplete
        // beta bisection).
        for &(k, n, d) in &[
            (1usize, 100, 0.05),
            (3, 59, 0.05),
            (7, 299, 0.05),
            (2, 30, 0.10),
            (5, 200, 0.01),
            (11, 500, 0.05),
        ] {
            let u = super::cp_upper_bound(k, n, d);
            let cdf = binom_cdf(k, n, u);
            assert!((cdf - d).abs() < 1e-9, "k={k} n={n} d={d}: cdf(p_u)={cdf}");
        }
    }

    /// A clean synthetic slice: high scores almost all correct, low scores
    /// wrong (n = 400; the boundary is sharp so strict cutoffs certify).
    fn clean_slice() -> Vec<GateObservation> {
        (0..400)
            .map(|i| GateObservation {
                score: 0.4 + (i as f32 / 400.0) * 0.6,
                correct: i >= 40, // bottom decile wrong, everything above right
            })
            .collect()
    }

    #[test]
    fn risk_target_certifies_a_clean_slice() {
        let obs = clean_slice();
        let fit = risk_target_fit(&obs, 0.10, 0.05).unwrap();
        assert_eq!(fit.status, RecommendationStatus::Fitted);
        let rec = fit.recommendation.expect("certified");
        // The certified cutoff must sit inside the clean geometry: accepted
        // error exactly 0 below the wrong block, well under alpha.
        assert!(rec.threshold > 0.4);
        assert!(rec.target_accuracy >= 0.9);
        assert_eq!(fit.alpha, 0.10);
        assert_eq!(fit.delta, 0.05);
        assert_eq!(fit.floor_groups, 29); // ceil(ln .05 / ln .9)
        assert!(fit.n_groups >= fit.floor_groups);
        assert_eq!(fit.n_groups, rec.support.n_pass);
        // The Option-shaped surface carries the same cutoff, status Fitted.
        let shaped = threshold_recommendation(
            &obs,
            Posture::RiskTarget {
                alpha: 0.10,
                delta: 0.05,
            },
        )
        .expect("certified");
        assert_eq!(shaped.threshold.to_bits(), rec.threshold.to_bits());
        assert_eq!(shaped.status, RecommendationStatus::Fitted);
    }

    #[test]
    fn risk_target_certified_cutoff_is_the_loosest_passing() {
        // Reproduce the fixed-sequence law by hand over the same grid: the
        // certified threshold must equal the LAST passing candidate before
        // the first tested failure.
        let obs = clean_slice();
        let alpha = 0.10f64;
        let delta = 0.05f64;
        let floor = super::zero_error_floor(alpha, delta);
        let mut want: Option<f32> = None;
        for i in (0..RISK_TARGET_GRID_POINTS).rev() {
            let t = i as f32 / (RISK_TARGET_GRID_POINTS - 1) as f32;
            let (n, k) = obs.iter().fold((0usize, 0usize), |(n, k), o| {
                if o.score >= t {
                    (n + 1, k + (!o.correct) as usize)
                } else {
                    (n, k)
                }
            });
            if n < floor {
                continue;
            }
            if super::cp_upper_bound(k, n, delta) <= alpha {
                want = Some(t);
            } else {
                break;
            }
        }
        let fit = risk_target_fit(&obs, alpha, delta).unwrap();
        let rec = fit.recommendation.expect("certified");
        assert_eq!(rec.threshold.to_bits(), want.unwrap().to_bits());
    }

    #[test]
    fn risk_target_not_certified_is_absence_with_disclosure() {
        // Every cutoff keeps ~20% error: nothing certifies at alpha = 0.1,
        // and the Option-shaped surface answers None (the caller's declared
        // geometry stays in charge) while the fit discloses NotCertified +
        // the floor.
        let obs: Vec<GateObservation> = (0..400)
            .map(|i| GateObservation {
                score: 0.1 + (i as f32 / 400.0) * 0.8,
                correct: i % 5 != 0, // 20% wrong, uniformly over the score axis
            })
            .collect();
        let fit = risk_target_fit(&obs, 0.10, 0.05).unwrap();
        assert_eq!(fit.status, RecommendationStatus::NotCertified);
        assert!(fit.recommendation.is_none());
        assert_eq!(fit.n_groups, 0);
        assert_eq!(fit.floor_groups, 29);
        assert!(
            threshold_recommendation(
                &obs,
                Posture::RiskTarget {
                    alpha: 0.10,
                    delta: 0.05
                }
            )
            .is_none(),
            "not certified -> absence, never a point-estimate fallback"
        );
    }

    #[test]
    fn below_floor_group_counts_refuse_loud() {
        // n between THIN_SUPPORT_FLOOR and the 59-floor at alpha=5%: the
        // fit runs, every candidate is inadmissible, the disclosure names
        // the floor.
        let scores = distinct_scores(40);
        let obs: Vec<GateObservation> = scores
            .iter()
            .map(|&s| GateObservation {
                score: s / 100.0,
                correct: true,
            })
            .collect();
        let fit = risk_target_fit(&obs, 0.05, 0.05).unwrap();
        assert_eq!(fit.status, RecommendationStatus::NotCertified);
        assert_eq!(fit.floor_groups, 59);
        assert!(fit.n_groups < fit.floor_groups);
        // Even an ALL-CORRECT slice cannot certify below the floor — the
        // zero-error pass open is a COUNT law, not an accuracy one.
        assert!(
            threshold_recommendation(
                &obs,
                Posture::RiskTarget {
                    alpha: 0.05,
                    delta: 0.05
                }
            )
            .is_none()
        );
        // At 300 all-correct groups the floor opens and the loosest
        // candidate certifies.
        let scores = distinct_scores(300);
        let obs: Vec<GateObservation> = scores
            .iter()
            .map(|&s| GateObservation {
                score: s / 100.0,
                correct: true,
            })
            .collect();
        let fit = risk_target_fit(&obs, 0.05, 0.05).unwrap();
        assert_eq!(fit.status, RecommendationStatus::Fitted);
        assert_eq!(fit.recommendation.unwrap().target_accuracy, 1.0);
    }

    #[test]
    fn shuffled_labels_certify_nothing_below_the_base_rate() {
        // Falsification arm c1: permute the correctness labels against the
        // score order (deterministic stride) so accepted-subset error ≈ the
        // 25% base rate; the posture must return NotCertified for every
        // alpha below that rate.
        let scores = distinct_scores(400);
        let obs: Vec<GateObservation> = scores
            .iter()
            .enumerate()
            .map(|(i, &s)| GateObservation {
                score: s / 100.0,
                correct: (i * 13 + 7) % 4 != 0, // 25% wrong, score-independent
            })
            .collect();
        for alpha in [0.05, 0.10, 0.20] {
            let fit = risk_target_fit(&obs, alpha, 0.05).unwrap();
            assert_eq!(
                fit.status,
                RecommendationStatus::NotCertified,
                "alpha={alpha} below the 25% base rate"
            );
        }
    }

    #[test]
    fn planted_violation_does_not_certify() {
        // Falsification arm c2: the accepted subset carries error ABOVE
        // alpha at every testable cutoff — certification must refuse. A
        // uniform 20%-wrong slice refuses at every alpha below the base
        // rate + the strict end's CP margin — INCLUDING alphas a looser
        // large-n cutoff could pass, because the fixed-sequence stop (T1:
        // "stop at the first failure") fires at the strictest admissible
        // candidate (n ≈ floor — the loosest bound) and never reaches them.
        // That wholesale refusal is the method working as specified; the
        // positive contrast (a clean strict end certifies) lives in
        // `risk_target_certifies_a_clean_slice` + the all-correct arm of
        // `below_floor_group_counts_refuse_loud`.
        let scores = distinct_scores(400);
        let obs: Vec<GateObservation> = scores
            .iter()
            .enumerate()
            .map(|(i, &s)| GateObservation {
                score: s / 100.0,
                correct: i % 5 != 0, // 20% wrong, score-independent
            })
            .collect();
        for alpha in [0.10, 0.15, 0.30] {
            let fit = risk_target_fit(&obs, alpha, 0.05).unwrap();
            assert_eq!(
                fit.status,
                RecommendationStatus::NotCertified,
                "alpha={alpha}: the 20% planted violation must refuse"
            );
        }
    }

    #[test]
    fn honesty_risktarget_coverage_is_a_subset_of_target_accuracy() {
        // T4(b), by construction: a CP-passing cutoff's observed error is
        // <= alpha (the upper bound never undercuts the observed rate), so
        // it also passes TargetAccuracy's point screen at target =
        // 1 − alpha — the certified cutoff's coverage is at most the
        // accuracy posture's (its loosest meeting cutoff).
        let slices: Vec<Vec<GateObservation>> = vec![
            clean_slice(),
            (0..300)
                .map(|i| GateObservation {
                    score: ((i * 37) % 100) as f32 / 100.0,
                    correct: i % 3 != 0,
                })
                .collect(),
            (0..200)
                .map(|i| GateObservation {
                    score: 0.05 + (i as f32 / 200.0) * 0.9,
                    correct: i >= 30 && i % 17 != 0,
                })
                .collect(),
        ];
        for obs in &slices {
            for alpha in [0.05, 0.10] {
                let rt = risk_target_fit(obs, alpha, 0.05).unwrap();
                let ta = threshold_recommendation(
                    obs,
                    Posture::TargetAccuracy {
                        target: 1.0 - alpha,
                    },
                )
                .unwrap();
                let rt_cov = match rt.recommendation {
                    Some(r) => r.support.n_pass as f64 / obs.len() as f64,
                    None => 0.0,
                };
                let ta_cov = ta.support.n_pass as f64 / obs.len() as f64;
                assert!(
                    rt_cov <= ta_cov + 1e-12,
                    "alpha={alpha}: rt={rt_cov} ta={ta_cov}"
                );
            }
        }
    }

    #[test]
    fn determinism_bit_identical_includes_risk_target() {
        let obs = clean_slice();
        let posture = Posture::RiskTarget {
            alpha: 0.10,
            delta: 0.05,
        };
        let a = risk_target_fit(&obs, 0.10, 0.05).unwrap();
        let b = risk_target_fit(&obs, 0.10, 0.05).unwrap();
        assert_eq!(a, b);
        let shaped_a = threshold_recommendation(&obs, posture).unwrap();
        let shaped_b = threshold_recommendation(&obs, posture).unwrap();
        assert_eq!(shaped_a.threshold.to_bits(), shaped_b.threshold.to_bits());
        assert_eq!(
            shaped_a.target_accuracy.to_bits(),
            shaped_b.target_accuracy.to_bits()
        );
        // The grid is the pre-registered 1000-point v1.
        assert_eq!(RISK_TARGET_GRID_POINTS, 1000);
    }

    #[test]
    fn serde_wire_carries_risk_target_and_not_certified() {
        // The additive variant round-trips (camelCase, tagged).
        let posture = Posture::RiskTarget {
            alpha: 0.1,
            delta: 0.05,
        };
        let json = serde_json::to_string(&posture).unwrap();
        assert!(json.contains("\"posture\":\"riskTarget\""), "{json}");
        assert!(json.contains("\"alpha\":0.1"), "{json}");
        assert!(json.contains("\"delta\":0.05"), "{json}");
        let back: Posture = serde_json::from_str(&json).unwrap();
        assert_eq!(back, posture);
        // The NotCertified disclosure state round-trips with the floor.
        let fit = risk_target_fit(&clean_slice(), 0.005, 0.05).unwrap(); // floor >> n
        assert_eq!(fit.status, RecommendationStatus::NotCertified);
        let fit_json = serde_json::to_string(&fit).unwrap();
        assert!(
            fit_json.contains("\"status\":\"notCertified\""),
            "{fit_json}"
        );
        assert!(fit_json.contains("\"floorGroups\""), "{fit_json}");
        let fit_back: RiskTargetFit = serde_json::from_str(&fit_json).unwrap();
        assert_eq!(fit_back, fit);
    }

    #[test]
    fn risk_target_thin_support_is_null_like_the_other_postures() {
        let thin = obs_from_scores(&distinct_scores(15), 0.5);
        assert!(
            threshold_recommendation(
                &thin,
                Posture::RiskTarget {
                    alpha: 0.10,
                    delta: 0.05
                }
            )
            .is_none()
        );
        assert!(risk_target_fit(&thin, 0.10, 0.05).is_none());
    }

    #[test]
    fn junk_risk_target_parameters_refuse_loud() {
        let obs = clean_slice();
        assert!(
            std::panic::catch_unwind(|| {
                let _ = threshold_recommendation(
                    &obs,
                    Posture::RiskTarget {
                        alpha: 0.0,
                        delta: 0.05,
                    },
                );
            })
            .is_err()
        );
        assert!(
            std::panic::catch_unwind(|| {
                let _ = risk_target_fit(&obs, 0.10, 1.0);
            })
            .is_err()
        );
    }
}
