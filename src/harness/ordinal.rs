//! Ordinal arena metrics (issue 132 Item A — SSR distill, arXiv:2510.08338).
//!
//! The comparison arena's distributional-realism + ranking-signal axis for
//! the ordinal suites (sst5 / sst5_syn — the 5 fixed score levels whose
//! index IS the level; `build_sst5_as`). Two lanes that tie on accuracy can
//! differ sharply on realism: a "regress to suite prior" lane matches the
//! gold label distribution (high KS_sim) while carrying no per-case ranking
//! signal (Spearman ≈ 0) — the paper's zero-demography persona. Both axes
//! are report-only; no serving-path change, no exit-code change.
//!
//! Suite taxonomy (pinned at the issue's filing, AGREE round 1): the
//! cumulative statistic below is meaningful ONLY on ordered labels, so the
//! arm list here is the ONE source of ordinal membership — a suite joins by
//! appending its registry name in the same commit its spec lands (the
//! CASE_EXACT_SUITES law). Unordered suites never enter this arm list; the
//! order-free total-variation axis (`tv`) is still reported beside `ks_sim`
//! for them by the caller (the report carries both, separately labeled —
//! never one pooled "KS" column across ordered and unordered label sets).

/// The ordinal arm list: suites whose labels are ordered levels (index =
/// level). sst5_syn shares `build_sst5_as`'s 5-level ordinal wire, so it is
/// ordinal by construction, not by name-matching alone.
pub const ORDINAL_SUITES: &[&str] = &["sst5", "sst5_syn"];

/// True when `suite`'s labels are ordered levels (the cumulative KS axis is
/// meaningful).
#[must_use]
pub fn is_ordinal(suite: &str) -> bool {
    ORDINAL_SUITES.contains(&suite)
}

/// The report for one lane's eval on one ordinal suite. `spearman` is the
/// per-case ranking axis (expected label `E[r] = Σ r·p(r)` vs gold);
/// `ks_sim` the aggregate distributional-similarity axis (picked labels vs
/// gold histogram). `tv` is the order-free twin of `ks_sim` — reported for
/// EVERY suite so an unordered suite still carries a distributional row
/// (never a cumulative one; see the module doc).
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrdinalReport {
    /// `1 − sup_r |F_gold(r) − F_pred(r)|` over the ordered levels — 1.0
    /// when the picked-label distribution matches the gold distribution
    /// exactly. None when the suite carried no cases (an honest absence).
    pub ks_sim: Option<f64>,
    /// Order-free total variation `½ Σ|p_gold − p_pred|` (0.0 = identical
    /// distributions) — the axis an UNORDERED suite reports in place of the
    /// cumulative KS. None when the suite carried no cases.
    pub tv: Option<f64>,
    /// Spearman correlation between the lane's expected label and gold,
    /// over cases. None on constant inputs (no ranking signal is
    /// measurable — a lane that answers one label everywhere carries none,
    /// and a constant gold column has none to find).
    pub spearman: Option<f64>,
    /// Seeded bootstrap 95% CI over cases (percentile, 2000 resamples,
    /// SplitMix64 — deterministic per report). None exactly when
    /// `spearman` is None.
    pub spearman_ci95: Option<(f64, f64)>,
    /// Cases (score questions) the report read.
    pub n_cases: usize,
}

/// The one-home ordinal scorer (the `case_exact_of` shape): arm-gated on
/// [`ORDINAL_SUITES`], fed the forced rows `(gold_idx, probs)` — the same
/// rows `hard_metrics` reads — and shared by BOTH lane assemblies (the
/// modelless walk and `assemble_laya_lane_result`), so the two lanes'
/// numbers cannot diverge. sst5's grid is one score question per case, so
/// the forced rows align 1:1 with cases; a non-uniform probability width
/// refuses loud (a ragged grid would silently misalign the level walk).
#[must_use]
pub fn ordinal_of(suite_name: &str, forced: &[(usize, Vec<f64>)]) -> Option<OrdinalReport> {
    if !is_ordinal(suite_name) {
        return None;
    }
    let n = forced.len();
    if n == 0 {
        return None;
    }
    let levels = forced[0].1.len();
    assert!(
        levels > 1 && forced.iter().all(|(_, p)| p.len() == levels),
        "ordinal_of: probability width is ragged or trivial ({levels}) — the \
         level walk would silently misalign"
    );

    let picks: Vec<usize> = forced
        .iter()
        .map(|(_, p)| {
            p.iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map_or(0, |(i, _)| i)
        })
        .collect();
    let golds: Vec<usize> = forced.iter().map(|(g, _)| *g).collect();

    let exp_labels: Vec<f64> = forced
        .iter()
        .map(|(_, p)| {
            p.iter()
                .enumerate()
                .map(|(r, &x)| r as f64 * x)
                .sum::<f64>()
        })
        .collect();

    let (spearman, ci) = if exp_labels.len() >= 2 {
        match spearman_of(
            &exp_labels,
            &golds.iter().map(|&g| g as f64).collect::<Vec<f64>>(),
        ) {
            Some(rho) => {
                let ci = bootstrap_spearman_ci(
                    &exp_labels,
                    &golds.iter().map(|&g| g as f64).collect::<Vec<f64>>(),
                    2000,
                    0x1322_5D35,
                );
                (Some(rho), Some(ci))
            }
            None => (None, None),
        }
    } else {
        (None, None)
    };

    Some(OrdinalReport {
        ks_sim: ks_sim_of(&golds, &picks, levels),
        tv: tv_of(&golds, &picks, levels),
        spearman,
        spearman_ci95: ci,
        n_cases: n,
    })
}

/// The order-free twin reported for EVERY suite (see the module doc):
/// `½ Σ|p_gold − p_pred|` over the label histograms of gold vs picked
/// labels. None on an empty walk.
#[must_use]
pub fn tv_of(golds: &[usize], preds: &[usize], levels: usize) -> Option<f64> {
    if golds.is_empty() || golds.len() != preds.len() {
        return None;
    }
    let (mut g, mut p) = (vec![0.0f64; levels], vec![0.0f64; levels]);
    for (&gi, &pi) in golds.iter().zip(preds.iter()) {
        assert!(gi < levels && pi < levels, "tv_of: label out of range");
        g[gi] += 1.0;
        p[pi] += 1.0;
    }
    let n = golds.len() as f64;
    let tv: f64 = g
        .iter()
        .zip(p.iter())
        .map(|(&a, &b)| (a / n - b / n).abs())
        .sum::<f64>()
        * 0.5;
    Some(tv)
}

/// `KS_sim = 1 − sup_r |F_gold(r) − F_pred(r)|` — the cumulative
/// distribution functions of gold vs picked labels walked over the ordered
/// levels. Meaningful ONLY on ordered labels (`is_ordinal`); on unordered
/// label sets it changes under any reordering and must never be pooled
/// with an ordered suite's row.
#[must_use]
pub fn ks_sim_of(golds: &[usize], preds: &[usize], levels: usize) -> Option<f64> {
    if golds.is_empty() || golds.len() != preds.len() {
        return None;
    }
    let (mut fg, mut fp) = (vec![0.0f64; levels], vec![0.0f64; levels]);
    for (&gi, &pi) in golds.iter().zip(preds.iter()) {
        assert!(gi < levels && pi < levels, "ks_sim_of: label out of range");
        fg[gi] += 1.0;
        fp[pi] += 1.0;
    }
    let n = golds.len() as f64;
    let mut cg = 0.0f64;
    let mut cp = 0.0f64;
    let mut sup = 0.0f64;
    for r in 0..levels {
        cg += fg[r] / n;
        cp += fp[r] / n;
        sup = sup.max((cg - cp).abs());
    }
    Some(1.0 - sup)
}

/// Spearman's ρ between two paired samples — average-rank tie handling,
/// Pearson over the ranks. Sequential deterministic folds throughout
/// (the house fold-order hygiene). None when either input is constant
/// (zero variance — there is no ranking signal to correlate).
#[must_use]
pub fn spearman_of(xs: &[f64], ys: &[f64]) -> Option<f64> {
    if xs.len() != ys.len() || xs.is_empty() {
        return None;
    }
    let rx = average_ranks(xs)?;
    let ry = average_ranks(ys)?;
    let n = xs.len() as f64;
    let (mut mx, mut my) = (0.0f64, 0.0f64);
    for i in 0..xs.len() {
        mx += rx[i];
        my += ry[i];
    }
    mx /= n;
    my /= n;
    let (mut cov, mut vx, mut vy) = (0.0f64, 0.0f64, 0.0f64);
    for i in 0..xs.len() {
        let dx = rx[i] - mx;
        let dy = ry[i] - my;
        cov += dx * dy;
        vx += dx * dx;
        vy += dy * dy;
    }
    if vx <= 0.0 || vy <= 0.0 {
        return None;
    }
    Some(cov / (vx.sqrt() * vy.sqrt()))
}

/// Average ranks (1-based, ties share the mean of their positions) or None
/// on an empty slice.
fn average_ranks(vs: &[f64]) -> Option<Vec<f64>> {
    if vs.is_empty() {
        return None;
    }
    let mut idx: Vec<usize> = (0..vs.len()).collect();
    idx.sort_by(|&a, &b| vs[a].total_cmp(&vs[b]));
    let mut ranks = vec![0.0f64; vs.len()];
    let mut i = 0usize;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && vs[idx[j + 1]] == vs[idx[i]] {
            j += 1;
        }
        // positions i..=j (0-based) share the average of ranks i+1..=j+1.
        let mean = (i + j) as f64 / 2.0 + 1.0;
        for &k in &idx[i..=j] {
            ranks[k] = mean;
        }
        i = j + 1;
    }
    Some(ranks)
}

/// SplitMix64 — the house deterministic PRNG (std-only, no rand dep).
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform `[0, n)` — the rejection-free modulo is fine for a report
    /// axis (no adversarial input; the bias is far below the CI's
    /// resolution at n ≤ a few thousand).
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Seeded case-resample bootstrap percentile CI for the Spearman estimate
/// (2000 resamples, 2.5/97.5 percentiles). Deterministic per (input, seed).
#[must_use]
pub fn bootstrap_spearman_ci(xs: &[f64], ys: &[f64], iters: usize, seed: u64) -> (f64, f64) {
    let n = xs.len();
    assert!(n > 1 && ys.len() == n, "bootstrap needs ≥2 paired samples");
    let mut rng = SplitMix64(seed);
    let mut rhos: Vec<f64> = Vec::with_capacity(iters);
    let mut bx = vec![0.0f64; n];
    let mut by = vec![0.0f64; n];
    for _ in 0..iters {
        for cell in bx.iter_mut().zip(by.iter_mut()) {
            let k = rng.below(n);
            *cell.0 = xs[k];
            *cell.1 = ys[k];
        }
        if let Some(r) = spearman_of(&bx, &by) {
            rhos.push(r);
        }
    }
    if rhos.is_empty() {
        return (0.0, 0.0);
    }
    rhos.sort_by(|a, b| a.total_cmp(b));
    let lo = rhos[(iters as f64 * 0.025) as usize % rhos.len()];
    let hi = rhos[((iters as f64 * 0.975) as usize).min(rhos.len() - 1)];
    // Tail support: with iters=2000 the 2.5% index is 50 — the percentile
    // reads interpolation-free at this width and the support is well past
    // the n ≤ 1/(1−p) degeneracy the p99 audit warns about.
    (lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn ks_sim_known_answers() {
        // Identical histograms → KS_sim 1.0.
        let golds = vec![0, 1, 2, 3, 4, 0, 1, 2, 3, 4];
        let picks = golds.clone();
        assert!(approx(ks_sim_of(&golds, &picks, 5).unwrap(), 1.0, 1e-12));

        // Gold [0,0,1,1], pred all 0: F_gold(0)=0.5, F_pred(0)=1.0 → sup=0.5
        // → KS_sim 0.5; F_gold(1)=1.0, F_pred(1)=1.0.
        let g = vec![0, 0, 1, 1];
        let p = vec![0, 0, 0, 0];
        assert!(approx(ks_sim_of(&g, &p, 2).unwrap(), 0.5, 1e-12));

        // Pred shifted one level DOWN against a uniform gold: gold uniform
        // over 0..5 (2 each), pred uniform over 1..5 shifted — worst single
        // sup at r=0: F_gold=0.2, F_pred=0.0 → KS_sim 0.8.
        let g2: Vec<usize> = (0..5).flat_map(|l| [l, l]).collect();
        let p2: Vec<usize> = (1..5).flat_map(|l| [l, l]).chain([4, 4]).collect();
        assert!(approx(ks_sim_of(&g2, &p2, 5).unwrap(), 0.8, 1e-12));
    }

    #[test]
    fn tv_known_answers() {
        let g = vec![0, 0, 1, 1];
        let p = vec![0, 0, 0, 0];
        // p_gold = [0.5, 0.5], p_pred = [1.0, 0.0] → ½(|.5|+|.5|) = 0.5.
        assert!(approx(tv_of(&g, &p, 2).unwrap(), 0.5, 1e-12));
        assert!(approx(tv_of(&g, &g.clone(), 2).unwrap(), 0.0, 1e-12));
    }

    #[test]
    fn spearman_known_answers() {
        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(approx(spearman_of(&xs, &ys).unwrap(), 1.0, 1e-12));

        let ys_inv = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        assert!(approx(spearman_of(&xs, &ys_inv).unwrap(), -1.0, 1e-12));

        // Ties: x = [1,1,2], y = [1,2,3] — average ranks [1.5,1.5,3] vs
        // [1,2,3]: ρ = cov/(σxσy). mx=2, my=2; cov = (−.5)(−1)+0+1·1 = 1.5;
        // vx = .25+0+.25... wait: dx = [−0.5, −0.5, 1.0] → vx = 0.25+0.25+1.0
        // = 1.5; dy = [−1,0,1] → vy = 2.0 → ρ = 1.5/sqrt(3.0) ≈ 0.8660.
        let xt = vec![1.0, 1.0, 2.0];
        let yt = vec![1.0, 2.0, 3.0];
        assert!(approx(
            spearman_of(&xt, &yt).unwrap(),
            1.5 / (1.5f64 * 2.0).sqrt(),
            1e-12
        ));

        // Constant input → None (no ranking signal).
        assert!(spearman_of(&vec![1.0, 1.0, 1.0], &ys).is_none());
    }

    #[test]
    fn bootstrap_ci_covers_point_estimate() {
        let xs: Vec<f64> = (0..60).map(f64::from).collect();
        let ys: Vec<f64> = (0..60)
            .map(|i| i as f64 * 0.5 + ((i * 7) % 5) as f64)
            .collect();
        let rho = spearman_of(&xs, &ys).unwrap();
        let (lo, hi) = bootstrap_spearman_ci(&xs, &ys, 500, 42);
        assert!(
            lo <= rho + 1e-9 && hi >= rho - 1e-9,
            "CI ({lo},{hi}) misses ρ {rho}"
        );
    }

    /// Issue 132 A4 — the falsification arm: a synthetic lane sampling
    /// labels from the gold MARGINAL must show HIGH KS_sim AND Spearman ≈ 0
    /// with the CI covering 0. Either half failing = the metric
    /// implementation is broken (a red about the metric, never about
    /// lanes). The between-concept attainment design is deliberately NOT
    /// ported — under that design this control passes by construction.
    #[test]
    fn falsifier_marginal_lane_high_ks_zero_ranking() {
        // Seeded gold from a skewed 5-level marginal; the synthetic lane
        // samples INDEPENDENTLY from the same marginal.
        let mut rng = SplitMix64(0xA4F1_CAFE);
        let marginal: [f64; 5] = [0.38, 0.27, 0.17, 0.12, 0.06];
        let cum: Vec<f64> = {
            let mut c = Vec::with_capacity(5);
            let mut acc = 0.0;
            for &m in &marginal {
                acc += m;
                c.push(acc);
            }
            c
        };
        let draw = |rng: &mut SplitMix64| -> usize {
            let u = (rng.next() >> 11) as f64 / (1u64 << 53) as f64;
            cum.iter().position(|&c| u < c).unwrap_or(4)
        };
        let n = 400;
        let golds: Vec<usize> = (0..n).map(|_| draw(&mut rng)).collect();
        let preds: Vec<usize> = (0..n).map(|_| draw(&mut rng)).collect();

        let ks = ks_sim_of(&golds, &preds, 5).unwrap();
        assert!(
            ks >= 0.9,
            "marginal lane KS_sim {ks} < 0.9 — the distributional axis is broken"
        );

        // The ranking axis: the "lane's expected label" is its picked
        // label (independent of gold) — Spearman ≈ 0, CI covering 0.
        let xs: Vec<f64> = preds.iter().map(|&p| p as f64).collect();
        let ys: Vec<f64> = golds.iter().map(|&g| g as f64).collect();
        let rho = spearman_of(&xs, &ys).unwrap();
        let (lo, hi) = bootstrap_spearman_ci(&xs, &ys, 2000, 0xB4D1);
        assert!(
            lo <= 0.0 && hi >= 0.0,
            "CI ({lo},{hi}) fails to cover 0 on independent draws"
        );
        assert!(
            rho.abs() < 0.15,
            "Spearman {rho} too far from 0 on independent draws"
        );

        // The positive control: a monotone predictor clears the ranking
        // bar the marginal lane fails (the axis separates, not saturates).
        let mono_y: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let mono_x: Vec<f64> = (0..n)
            .map(|i| i as f64 * 0.6 + ((i * 13) % 7) as f64)
            .collect();
        let rho_pos = spearman_of(&mono_x, &mono_y).unwrap();
        let (plo, phi) = bootstrap_spearman_ci(&mono_x, &mono_y, 500, 0xB4D2);
        assert!(
            rho_pos > 0.9 && plo > 0.0,
            "positive control failed: ρ {rho_pos} CI ({plo},{phi})"
        );
    }

    #[test]
    fn ordinal_of_arm_gate_and_report() {
        // Off the arm list → None (never a fabricated row).
        assert!(ordinal_of("emotion", &[(0, vec![0.5, 0.5])]).is_none());
        assert!(ordinal_of("wisesight", &[]).is_none());

        // On an ordinal suite with an empty walk → None (honest absence).
        assert!(ordinal_of("sst5", &[]).is_none());

        // A ragged probability width refuses loud.
        let ragged = vec![(0usize, vec![0.5, 0.5]), (1usize, vec![1.0, 0.0, 0.0])];
        let _ = std::panic::catch_unwind(|| ordinal_of("sst5", &ragged));

        // Happy path: every field populated, counts right.
        let rows = vec![
            (0usize, vec![0.7, 0.2, 0.1, 0.0, 0.0]),
            (1usize, vec![0.1, 0.6, 0.2, 0.1, 0.0]),
            (2usize, vec![0.0, 0.1, 0.5, 0.3, 0.1]),
            (3usize, vec![0.0, 0.0, 0.2, 0.5, 0.3]),
        ];
        let rep = ordinal_of("sst5_syn", &rows).unwrap();
        assert_eq!(rep.n_cases, 4);
        assert!(rep.ks_sim.unwrap() > 0.9); // picks track gold exactly here
        assert!(approx(rep.tv.unwrap(), 0.0, 1e-12));
        assert!(rep.spearman.unwrap() > 0.9);
        assert!(rep.spearman_ci95.unwrap().0 > 0.0);
    }
}
