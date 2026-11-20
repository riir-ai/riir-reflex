//! Issue 119 — the context-INVARIAANCE probe (the runner half).
//!
//! Every transform arm answers through the SAME production decide path the
//! perm probe uses ([`crate::harness::permutation::ChoiceOracle`] over
//! `engine_request` → `decide_with` → `answer_probs_pick`) — the probe
//! measures what ships, never a parallel rendering. The pure transform
//! tables live in [`crate::harness::context_transform`].
//!
//! Pre-registration (issue 119 T3/T4, quoted from the issue):
//! - **ONE decision set.** Each slot is decided under all 8 transforms of
//!   each family (identity + 7); the K sweep (2 / 4 / 8) is a PREFIX of
//!   that set — zero extra decides. The PRIMARY cell is K=4.
//! - **Primary suite set**: the registered modelless-eligible dataset
//!   suites with n ≥ 100 at run time (the Bench-092 gate-6 rule); every
//!   exclusion named in the record. `--suites` overrides, disclosed.
//! - **Fused gate**: the identity transform's decide IS the production
//!   input — its fused-gate outcome (answered + pick, or abstained) is the
//!   fused arm everywhere.
//! - **Seat**: the strict-majority core over the K=4 prefix; empty core =
//!   the seat abstains. Unanimous disclosed beside.
//! - **G1 (hard, both families)**: the identity transform reproduces the
//!   raw single-question case's decide BYTE-identically (probs bits + the
//!   fused abstain flag). A red refuses the run verdict (harness bug).
//! - **G2a (primary)**: two paired LB95 conditions — (i) semantic −
//!   placebo seat correctness on the both-answer pairing, (ii) semantic
//!   seat − fused gate on the matched-coverage pairing — pooled across
//!   suites with suite fixed effects; LB95 = mean − 1.645·SE; the cell
//!   passes iff BOTH bounds > 0.
//! - **G2b (secondary)**: the empty-core channel arm — the fused answer
//!   withheld when the core is empty — scored on the fused-answered set
//!   with the extra abstention scoring 0 (the coverage cost paid in the
//!   same denominator), placebo twin beside; same two-bound rule.
//! - **G3 (the null path)**: G2a failing BOTH bounds on the primary cell
//!   records the NEGATIVE for this run table on THIS engine (a
//!   bag-of-hashed-words additive-feature engine), for the tested families
//!   — never for the class. The named re-open trigger rides the record.

use serde::Serialize;

use super::perm_probe::stride_indices;
use super::*;
use crate::harness::context_transform::{CoreClass, Family, agreement_of, transformed_case};
use crate::harness::permutation::{ChoiceOracle, OrderingAnswer};

/// Transforms per slot per family, INCLUDING the identity (the decision
/// set the K sweep prefixes).
pub const CTX_K_RUN: usize = 8;
/// The PRIMARY cell's K (issue 119 T3's pre-registration).
pub const DEFAULT_CTX_K: usize = 4;
/// Cases sampled per suite by deterministic stride (the perm probe's
/// default, kept for cross-probe comparability).
pub const DEFAULT_CTX_MAX_CASES: usize = 40;
/// The pre-registered suite floor (the Bench-092 gate-6 rule).
pub const PRIMARY_SUITE_FLOOR_N: usize = 100;
/// The one-sided 95% z (pre-registered; quoted in the record's rule text).
pub const Z95_ONE_SIDED: f64 = 1.644_9;

/// Knobs: the CLI's `--ctx-k` / `--ctx-max-cases`.
#[derive(Debug, Clone)]
pub struct CtxProbeOptions {
    /// The PRIMARY cell's K (2..=8; the sweep prefixes one decision set).
    pub k: usize,
    pub max_cases: usize,
}

// ─────────────────────────────────────────── the paired-statistics core ──

/// One paired-delta cell: per-case deltas on a fixed pairing set.
#[derive(Debug, Clone, Serialize)]
pub struct PairedDelta {
    pub n: usize,
    /// Mean Δ in percentage points.
    pub mean_pct: f64,
    /// SE in percentage points (sample sd / √n; 0.0 at n = 1, disclosed).
    pub se_pct: f64,
}

/// `None` when the pairing has fewer than 2 cases (an SE is owed — an
/// empty pairing is named by the caller, never folded into a pass).
#[must_use]
pub fn paired_delta(deltas: &[f64]) -> Option<PairedDelta> {
    let n = deltas.len();
    if n < 2 {
        return None;
    }
    let mean = deltas.iter().sum::<f64>() / n as f64;
    let var = deltas.iter().map(|d| (d - mean) * (d - mean)).sum::<f64>() / (n - 1) as f64;
    Some(PairedDelta {
        n,
        mean_pct: mean * 100.0,
        se_pct: var.sqrt() / (n as f64).sqrt() * 100.0,
    })
}

/// The pre-registered bound: mean − 1.645·SE (one-sided 95%).
#[must_use]
pub fn lb95(d: &PairedDelta) -> f64 {
    d.mean_pct - Z95_ONE_SIDED * d.se_pct
}

/// The pooled cell: suite fixed effects — every case's delta is
/// re-centered on its suite mean before pooling, so a suite with a large
/// n cannot dominate, and the pooled mean IS the unweighted mean of the
/// suite means.
#[derive(Debug, Clone, Serialize)]
pub struct PooledDelta {
    pub n_suites: usize,
    pub n_cases: usize,
    pub mean_pct: f64,
    pub se_pct: f64,
    pub lb95_pct: f64,
}

/// Pool per-suite paired deltas as `(suite name, deltas)`. `None` when no
/// suite contributes ≥ 2 paired cases.
#[must_use]
pub fn pooled_suite_delta(cells: &[(&str, &[f64])]) -> Option<PooledDelta> {
    let mut tilde: Vec<f64> = Vec::new();
    let mut means: Vec<f64> = Vec::new();
    for (_, deltas) in cells {
        if let Some(d) = paired_delta(deltas) {
            let m = d.mean_pct / 100.0;
            means.push(m);
            for v in *deltas {
                tilde.push(v - m);
            }
        }
    }
    if means.is_empty() {
        return None;
    }
    let grand = means.iter().sum::<f64>() / means.len() as f64;
    for v in &mut tilde {
        *v += grand;
    }
    let n = tilde.len();
    let mean = tilde.iter().sum::<f64>() / n as f64;
    let var = if n > 1 {
        tilde.iter().map(|d| (d - mean) * (d - mean)).sum::<f64>() / (n - 1) as f64
    } else {
        0.0
    };
    let se = var.sqrt() / (n as f64).sqrt();
    let mean_pct = mean * 100.0;
    let se_pct = se * 100.0;
    Some(PooledDelta {
        n_suites: means.len(),
        n_cases: n,
        mean_pct,
        se_pct,
        lb95_pct: mean_pct - Z95_ONE_SIDED * se_pct,
    })
}

// ─────────────────────────────────────────────────────────── the oracle ──

/// The modelless oracle — the deployed engine's own decide path, the exact
/// `ModellessOracle` body, plus the fused-gate abstain read: after every
/// `answer()`, the scratch's last slot carries the fused gate's verdict
/// for the case just decided (the identity decide's read IS the fused arm).
pub(super) struct CtxOracle<'e, const N: usize> {
    pub(super) engine: &'e mut DecisionEngine<N, EMBED_DIM>,
    pub(super) sc: Scratch<EMBED_DIM>,
}

impl<const N: usize> CtxOracle<'_, N> {
    /// The fused gate's verdict for the LAST decided case (`answer()`
    /// leaves the slot in the scratch): `(abstained, cause)`. `None` = no
    /// slot (a decide bug).
    fn last_fused_verdict(&self) -> Option<(bool, &'static str)> {
        self.sc
            .slots
            .last()
            .map(|s| (s.abstained, s.cause.as_str()))
    }
}

impl<const N: usize> ChoiceOracle for CtxOracle<'_, N> {
    fn answer(&mut self, case: &SuiteCase) -> Result<OrderingAnswer, String> {
        let state_str = serialize_state(&case.state);
        let req = engine_request(case, &state_str)?;
        let resp = self
            .engine
            .decide_with(&req, &mut self.sc)
            .map_err(|e| format!("engine decide ({}): {e}", case.id))?;
        let ans = resp
            .answers
            .first()
            .ok_or_else(|| format!("engine decide ({}): no answers in response", case.id))?;
        let mut probs = Vec::with_capacity(ans.probabilities.len());
        let pick = answer_probs_pick(QKind::Choice, ans, &mut probs);
        Ok(OrderingAnswer { probs, pick })
    }
}

// ─────────────────────────────────────────────────────────── the record ──

/// One family's read of one slot: the 8-transform decision set and the
/// agreement structure at the K sweep's three rungs.
#[derive(Debug, Clone, Serialize)]
pub struct FamilyRow {
    pub family: String,
    /// Top pick under each transform ([0] = identity).
    pub picks: Vec<usize>,
    pub class_k2: CoreClass,
    pub class_k4: CoreClass,
    pub class_k8: CoreClass,
    /// The PRIMARY seat (the strict-majority core at K=4).
    pub seat_k4: Option<usize>,
    pub seat_votes_k4: usize,
    pub seat_correct: Option<bool>,
}

/// One slot's full record.
#[derive(Debug, Clone, Serialize)]
pub struct CtxCaseRow {
    pub case_id: String,
    pub qid: String,
    pub n_options: usize,
    pub gold_idx: usize,
    /// The fused gate (the identity decide): answered + pick + correct.
    pub fused_answered: bool,
    pub fused_pick: usize,
    pub fused_correct: bool,
    /// The fused gate's abstain cause for the slot (the Issue-060 closed
    /// taxonomy; "answered" when it did not abstain) — the smoke's
    /// posture-diagnosis read, disclosed in every record.
    pub fused_cause: String,
    pub semantic: FamilyRow,
    pub placebo: FamilyRow,
}

/// One family's agreement census over one suite (the K=4 primary + the
/// descriptive sweep rungs).
#[derive(Debug, Clone, Serialize)]
pub struct FamilyStats {
    pub family: String,
    pub n: usize,
    pub unanimous_k4: usize,
    pub majority_k4: usize,
    pub no_core_k4: usize,
    pub empty_core_rate_k4: f64,
    /// The vacuous-core disclosure across K: the empty-core rate at every
    /// rung (2 / 4 / 8) — the seat's abstention pressure by K.
    pub empty_core_rate_k2: f64,
    pub empty_core_rate_k8: f64,
}

/// One paired cell (per suite): the deltas' mean/SE/bound; `None` fields
/// when the pairing is empty (named, never folded into a pass).
#[derive(Debug, Clone, Serialize)]
pub struct CellRow {
    pub cell: String,
    pub n: usize,
    pub mean_pct: Option<f64>,
    pub se_pct: Option<f64>,
    pub lb95_pct: Option<f64>,
}

impl CellRow {
    fn of(cell: &str, deltas: &[f64]) -> Self {
        match paired_delta(deltas) {
            Some(d) => Self {
                cell: cell.to_string(),
                n: d.n,
                mean_pct: Some(d.mean_pct),
                se_pct: Some(d.se_pct),
                lb95_pct: Some(lb95(&d)),
            },
            None => Self {
                cell: cell.to_string(),
                n: deltas.len(),
                mean_pct: None,
                se_pct: None,
                lb95_pct: None,
            },
        }
    }
}

/// One suite's report. `deltas` is the raw paired-delta store for the
/// pooled verdict — skipped in the JSON (the cells carry the published
/// form); `g1_broken` is the suite's G1 identity-control verdict.
#[derive(Debug, Clone, Serialize)]
pub struct CtxSuiteReport {
    pub suite: String,
    pub n_slots: usize,
    /// Excluded from the PRIMARY set (n < the floor) — runs anyway when
    /// the operator named it, disclosed.
    pub below_floor: bool,
    pub g1_broken: bool,
    pub family_stats: Vec<FamilyStats>,
    /// The four paired cells: g2a_sem_plac, g2a_sem_fused, g2b_channel_sem,
    /// g2b_channel_plac.
    pub cells: Vec<CellRow>,
    pub cases: Vec<CtxCaseRow>,
    pub skipped: Vec<String>,
    #[serde(skip)]
    deltas: SuiteDeltas,
}

/// The raw per-cell delta store (transient — never serialized).
#[derive(Debug, Clone, Default)]
struct SuiteDeltas {
    g2a_sem_plac: Vec<f64>,
    g2a_sem_fused: Vec<f64>,
    g2b_channel_sem: Vec<f64>,
    g2b_channel_plac: Vec<f64>,
}

impl SuiteDeltas {
    fn cell(&self, name: &str) -> &[f64] {
        match name {
            "g2a_sem_plac" => &self.g2a_sem_plac,
            "g2a_sem_fused" => &self.g2a_sem_fused,
            "g2b_channel_sem" => &self.g2b_channel_sem,
            _ => &self.g2b_channel_plac,
        }
    }
}

/// The pooled verdict block (the G2a/G2b rules' actual arithmetic).
#[derive(Debug, Clone, Serialize)]
pub struct CtxPooled {
    pub g2a_sem_plac: Option<PooledDelta>,
    pub g2a_sem_fused: Option<PooledDelta>,
    pub g2b_channel_sem: Option<PooledDelta>,
    pub g2b_channel_plac: Option<PooledDelta>,
    /// G2a passes iff BOTH pooled bounds > 0; G2b the same on its pair.
    pub g2a: String,
    pub g2b: String,
    /// G3 fired (the null path) when G2a fails BOTH bounds — the scoped
    /// negative's trigger, quoted in the record.
    pub g3_fired: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CtxProbeMeta {
    pub date_utc: String,
    pub git_sha: Option<String>,
    pub host: String,
    pub datasets_dir: String,
    pub k_primary: usize,
    pub k_run: usize,
    pub max_cases: usize,
    pub suite_set_rule: String,
    pub seat_rule: String,
    pub fused_rule: String,
    pub verdict_rule: String,
    pub route_binding: String,
    pub box_start: crate::harness::box_state::BoxState,
    pub box_end: crate::harness::box_state::BoxState,
}

#[derive(Debug, Clone, Serialize)]
pub struct CtxProbeOutput {
    pub meta: CtxProbeMeta,
    /// The G1 identity control held in every suite (a false value refuses
    /// the run verdict in the bin).
    pub g1_identity_holds: bool,
    pub suites: Vec<CtxSuiteReport>,
    pub pooled: CtxPooled,
}

// ─────────────────────────────────────────────────────────── the engine ──

/// The facts one slot contributes to the cells (the record's row source).
struct CtxCaseFacts {
    case_id: String,
    qid: String,
    n_options: usize,
    gold_idx: usize,
    fused_answered: bool,
    fused_pick: usize,
    fused_correct: bool,
    fused_cause: &'static str,
    semantic: FamilyRow,
    placebo: FamilyRow,
}

/// One slot's decided record: the fused read (the raw decide + the G1
/// identity anchor for both families) + the 8-transform decision set per
/// family. Returns `Ok(None)` when the state is not a string (the skip is
/// named, never a fabricated row).
fn decide_slot<const N: usize>(
    oracle: &mut CtxOracle<'_, N>,
    case: &SuiteCase,
    qi: usize,
    copts: &CtxProbeOptions,
    skips: &mut Vec<String>,
) -> Result<Option<(CtxCaseFacts, bool)>, String> {
    let q = &case.questions[qi];
    let n_options = q
        .criteria
        .as_object()
        .map(serde_json::Map::len)
        .unwrap_or(0);
    let gold_idx = case.gold.get(qi).map(|g| g.idx).unwrap_or(0);
    // The single-question reduction (the perm probe's wrapping — one
    // question per decide, gold rides along).
    let raw = SuiteCase {
        id: format!("{}#{}", case.id, q.qid),
        state: case.state.clone(),
        questions: vec![q.clone()],
        gold: vec![case.gold[qi].clone()],
    };
    // The fused arm: the RAW decide (the production input).
    let raw_ans = oracle.answer(&raw)?;
    let (fused_abstained, fused_cause) = oracle
        .last_fused_verdict()
        .ok_or_else(|| format!("{}: no engine slot", raw.id))?;
    let fused_pick = raw_ans.pick;
    let fused_bits: Vec<u64> = raw_ans.probs.iter().map(|p| p.to_bits()).collect();
    // The G1 anchor: the identity transform must reproduce the raw decide
    // byte-identically (probs bits + pick + the fused abstain flag) in
    // BOTH families.
    let mut g1_ok = true;
    for family in [Family::Semantic, Family::Placebo] {
        let id_case = transformed_case(&raw, family, 0).map_err(|e| {
            skips.push(format!("{}: {e}", raw.id));
            e
        })?;
        let id_ans = oracle.answer(&id_case)?;
        let id_abstained = oracle
            .last_fused_verdict()
            .ok_or_else(|| format!("{}: no engine slot", raw.id))?
            .0;
        if id_ans.probs.iter().map(|p| p.to_bits()).collect::<Vec<_>>() != fused_bits
            || id_ans.pick != fused_pick
            || id_abstained != fused_abstained
        {
            g1_ok = false;
        }
    }
    let fused_correct = !fused_abstained && fused_pick == gold_idx;
    // The decision set: 8 transforms per family.
    let mut rows: Vec<(Family, FamilyRow)> = Vec::with_capacity(2);
    for family in [Family::Semantic, Family::Placebo] {
        let mut picks = Vec::with_capacity(CTX_K_RUN);
        for rep in 0..CTX_K_RUN {
            let tc = transformed_case(&raw, family, rep)?;
            let a = oracle.answer(&tc)?;
            picks.push(a.pick);
        }
        let a4 = agreement_of(&picks, copts.k);
        let seat_correct = a4.seat.map(|s| s == gold_idx);
        let class_k2 = agreement_of(&picks, 2).class;
        let class_k8 = agreement_of(&picks, CTX_K_RUN).class;
        rows.push((
            family,
            FamilyRow {
                family: family.as_str().to_string(),
                picks,
                class_k2,
                class_k4: a4.class,
                class_k8,
                seat_k4: a4.seat,
                seat_votes_k4: a4.seat_votes,
                seat_correct,
            },
        ));
    }
    let sem = rows
        .iter()
        .find(|(f, _)| *f == Family::Semantic)
        .map(|(_, r)| r.clone())
        .ok_or("semantic row missing")?;
    let plac = rows
        .iter()
        .find(|(f, _)| *f == Family::Placebo)
        .map(|(_, r)| r.clone())
        .ok_or("placebo row missing")?;
    Ok(Some((
        CtxCaseFacts {
            case_id: raw.id,
            qid: q.qid.clone(),
            n_options,
            gold_idx,
            fused_answered: !fused_abstained,
            fused_pick,
            fused_correct,
            fused_cause,
            semantic: sem,
            placebo: plac,
        },
        g1_ok,
    )))
}

/// One suite's full ctx cell: the engine build (the perm probe's exact
/// plain posture), the slot loop, the family censuses, the four paired
/// cells. Const-generic over the label count.
#[allow(clippy::too_many_lines)]
fn ctx_suite<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    slots: &[(usize, usize)],
    copts: &CtxProbeOptions,
    kn_route: bool,
) -> Result<CtxSuiteReport, String> {
    let inp = ModellessInput {
        spec,
        cascade_worthiness: false,
        gate_fit_selection: false,
        gate_distance_only: false,
        gate_fit_calibrated: true,
        gate_rho: None,
        gate_risk_target: None,
        esc_delta_fit: None,
        kn_route,
        drafter_fix: crate::engine::DrafterFix::Off,
        pick_latch: None,
        route_census: false,
        bm25_floor: false,
        suite: &prepared.suite,
        train: &prepared.train,
        state_strs: &prepared.state_strs,
        cal_cases: &prepared.cal_cases,
        cal_state_strs: &prepared.cal_state_strs,
        labels: &prepared.labels,
        want_by_type: false,
        corpus_cap_per_label: spec.corpus_cap_per_label,
        head_scale: 0.0,
        head_select: false,
        nb_select: false,
        oc_select: false,
        ridge_select: false,
        genome_select: false,
        genome_accept_margin: 0.0,
        cap_source_base: "registry",
        cal_select_caps: &[],
        pool_rows: &prepared.pool_rows,
        pair_head_ab: false,
        nli_feature_ab: false,
        nli_m1: false,
        #[cfg(feature = "mc_ensemble")]
        mc_ab: None,
        density_gate: false,
        set_refine: None,
        gate_fit_cases_override: None,
        leak_flags: None,
    };
    let FittedPosture {
        effective_cap,
        default_cfg,
        score_threshold,
        distance_threshold,
        ..
    } = fit_posture_inner::<N>(&inp)?;
    #[cfg(feature = "option_cond")]
    if default_cfg.oc_scale > 0.0 {
        return Err(format!(
            "the fitted config arms option-conditioned tables (oc_scale {}) — \
             the probe pins the plain posture",
            default_cfg.oc_scale
        ));
    }
    #[cfg(feature = "nb_scope")]
    if default_cfg.nb_scale > 0.0 {
        return Err(format!(
            "the fitted config arms count tables (nb_scale {}) — the probe \
             pins the plain posture",
            default_cfg.nb_scale
        ));
    }
    let cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        ..default_cfg
    };
    let (mut engine, fallbacks) = build_engine_with::<N>(
        spec.name,
        &prepared.train,
        &prepared.labels,
        effective_cap,
        cfg,
        &[],
    )?;
    if !fallbacks.is_empty() {
        eprintln!(
            "  [issue-039 corpus guard] {}: {} option label(s) with NO train docs \
             → self-doc fallback: {}",
            spec.name,
            fallbacks.len(),
            fallbacks.join(", ")
        );
    }
    let mut sc = Scratch::<EMBED_DIM>::new();
    sc.prepare(1);
    let mut oracle = CtxOracle {
        engine: &mut engine,
        sc,
    };
    let mut skipped: Vec<String> = Vec::new();
    let mut facts: Vec<CtxCaseFacts> = Vec::with_capacity(slots.len());
    let mut g1_ok = true;
    for &(ci, qi) in slots {
        let case = &prepared.suite.cases[ci];
        match decide_slot(&mut oracle, case, qi, copts, &mut skipped) {
            Ok(Some((row, slot_g1))) => {
                g1_ok &= slot_g1;
                facts.push(row);
            }
            Ok(None) => {} // the state-shape skip — named by decide_slot
            Err(e) => return Err(e),
        }
    }
    drop(oracle);

    // Family censuses (the K=4 primary + the descriptive sweep rungs).
    let mut family_stats = Vec::with_capacity(2);
    for family in [Family::Semantic, Family::Placebo] {
        let rows: Vec<&FamilyRow> = facts
            .iter()
            .map(|f| {
                if family == Family::Semantic {
                    &f.semantic
                } else {
                    &f.placebo
                }
            })
            .collect();
        let n = rows.len();
        let cnt_at = |class: CoreClass, k: usize| {
            rows.iter()
                .filter(|r| agreement_of(&r.picks, k).class == class)
                .count()
        };
        let rate_at = |k: usize| {
            if n == 0 {
                0.0
            } else {
                cnt_at(CoreClass::NoCore, k) as f64 / n as f64
            }
        };
        family_stats.push(FamilyStats {
            family: family.as_str().to_string(),
            n,
            unanimous_k4: cnt_at(CoreClass::Unanimous, 4),
            majority_k4: cnt_at(CoreClass::Majority, 4),
            no_core_k4: cnt_at(CoreClass::NoCore, 4),
            empty_core_rate_k4: rate_at(4),
            empty_core_rate_k2: rate_at(2),
            empty_core_rate_k8: rate_at(CTX_K_RUN),
        });
    }

    // The four paired cells (the pre-registered pairings, K=4 primary).
    let mut d = SuiteDeltas::default();
    for f in &facts {
        let sem_seat = f.semantic.seat_correct;
        let plac_seat = f.placebo.seat_correct;
        // G2a(i): both seats answer — the placebo-adjusted seat delta.
        if let (Some(s), Some(p)) = (sem_seat, plac_seat) {
            d.g2a_sem_plac.push((i8::from(s) - i8::from(p)) as f64);
        }
        // G2a(ii): the seat vs the fused gate at matched coverage — both
        // must answer.
        if let (Some(s), true) = (sem_seat, f.fused_answered) {
            d.g2a_sem_fused
                .push((i8::from(s) - i8::from(f.fused_correct)) as f64);
        }
        // G2b: the channel arm on the fused-answered set — the fused pick
        // withheld (scored 0) when the core is empty; the coverage cost
        // paid in the same denominator.
        if f.fused_answered {
            let channel = |core_empty: bool| {
                if core_empty {
                    0.0
                } else {
                    i8::from(f.fused_correct) as f64
                }
            };
            d.g2b_channel_sem.push(
                channel(f.semantic.class_k4 == CoreClass::NoCore)
                    - i8::from(f.fused_correct) as f64,
            );
            d.g2b_channel_plac.push(
                channel(f.placebo.class_k4 == CoreClass::NoCore) - i8::from(f.fused_correct) as f64,
            );
        }
    }
    let cells = vec![
        CellRow::of("g2a_sem_plac", &d.g2a_sem_plac),
        CellRow::of("g2a_sem_fused", &d.g2a_sem_fused),
        CellRow::of("g2b_channel_sem", &d.g2b_channel_sem),
        CellRow::of("g2b_channel_plac", &d.g2b_channel_plac),
    ];
    eprintln!(
        "    ctx[{}]: {} slot(s) · empty-core@K4 sem {:.3} / plac {:.3} · G1 {}",
        spec.name,
        facts.len(),
        family_stats[0].empty_core_rate_k4,
        family_stats[1].empty_core_rate_k4,
        if g1_ok { "HOLDS" } else { "BROKEN" },
    );
    Ok(CtxSuiteReport {
        suite: spec.name.to_string(),
        n_slots: facts.len(),
        below_floor: false,
        g1_broken: !g1_ok,
        family_stats,
        cells,
        cases: facts
            .into_iter()
            .map(|f| CtxCaseRow {
                case_id: f.case_id,
                qid: f.qid,
                n_options: f.n_options,
                gold_idx: f.gold_idx,
                fused_answered: f.fused_answered,
                fused_pick: f.fused_pick,
                fused_correct: f.fused_correct,
                fused_cause: f.fused_cause.to_string(),
                semantic: f.semantic,
                placebo: f.placebo,
            })
            .collect(),
        skipped,
        deltas: d,
    })
}

/// Run the probe over the requested suites. `Err` = a loud refusal
/// (missing datasets, a posture drift); per-suite preparation failures
/// are named exclusions in the record.
pub fn run_ctx_probe(opts: &RunOptions, copts: &CtxProbeOptions) -> Result<CtxProbeOutput, String> {
    if !opts.datasets_dir.exists() {
        return Err(format!(
            "ctx probe: datasets dir {} is absent — the lane is datasets-required \
             (SKIP loud, never a fabricated number)",
            opts.datasets_dir.display()
        ));
    }
    let box_start = crate::harness::box_state::capture();
    let mut suites: Vec<CtxSuiteReport> = Vec::new();
    let mut excluded: Vec<(String, String)> = Vec::new();
    for spec in SUITES {
        let named = !opts.suites.is_empty() && opts.suites.iter().any(|s| s == spec.name);
        if !opts.suites.is_empty() {
            if !named {
                continue;
            }
        } else if spec.named_only {
            continue;
        }
        let prepared = match prepare(spec, &opts.datasets_dir) {
            Ok(p) => p,
            Err(e) => {
                excluded.push((spec.name.to_string(), format!("prepare: {e}")));
                continue;
            }
        };
        let n = prepared.suite.cases.len();
        let below_floor = n < PRIMARY_SUITE_FLOOR_N;
        if below_floor && !named {
            excluded.push((
                spec.name.to_string(),
                format!("n={n} < {PRIMARY_SUITE_FLOOR_N} — outside the pre-registered set"),
            ));
            continue;
        }
        // Choice slots (the perm probe's enumeration).
        let choice_cases: Vec<usize> = prepared
            .suite
            .cases
            .iter()
            .enumerate()
            .filter(|(_, c)| c.questions.iter().any(|q| q.kind == QKind::Choice))
            .map(|(i, _)| i)
            .collect();
        if choice_cases.is_empty() {
            excluded.push((
                spec.name.to_string(),
                "no choice questions — the probe is choice-only".to_string(),
            ));
            continue;
        }
        let sampled = stride_indices(choice_cases.len(), copts.max_cases);
        let slots: Vec<(usize, usize)> = sampled
            .iter()
            .flat_map(|&si| {
                let ci = choice_cases[si];
                let case = &prepared.suite.cases[ci];
                (0..case.questions.len()).filter_map(move |qi| {
                    (case.questions[qi].kind == QKind::Choice).then_some((ci, qi))
                })
            })
            .collect();
        eprintln!(
            "═══ suite {} (ctx) ═══ n={} · {} choice slot(s) over {} sampled case(s) · \
             K primary {} of the {}-transform set{}",
            spec.name,
            n,
            slots.len(),
            sampled.len(),
            copts.k,
            CTX_K_RUN,
            if below_floor {
                " · OPERATOR OVERRIDE (below the floor)"
            } else {
                ""
            },
        );
        macro_rules! dispatch {
            ($n_lit:literal) => {
                ctx_suite::<$n_lit>(spec, &prepared, &slots, copts, opts.kn_route)
            };
        }
        let report = match prepared.labels.len() {
            2 => dispatch!(2),
            3 => dispatch!(3),
            4 => dispatch!(4),
            5 => dispatch!(5),
            6 => dispatch!(6),
            7 => dispatch!(7),
            8 => dispatch!(8),
            10 => dispatch!(10),
            15 => dispatch!(15),
            59 => dispatch!(59),
            77 => dispatch!(77),
            842 => dispatch!(842),
            n => Err(format!(
                "no engine instantiation for {n} domains — extend the dispatch \
                 table in ctx_probe.rs"
            )),
        };
        match report {
            Ok(mut r) => {
                r.below_floor = below_floor;
                suites.push(r);
            }
            Err(e) => excluded.push((spec.name.to_string(), format!("engine: {e}"))),
        }
    }
    if suites.is_empty() {
        let mut why = format!(
            "ctx probe: no suite measured ({} request line(s), {} exclusion(s) \
             named)",
            opts.suites.len(),
            excluded.len()
        );
        for (suite, reason) in &excluded {
            why.push_str(&format!("\n  - {suite}: {reason}"));
        }
        return Err(why);
    }
    let pooled = build_pooled(&suites);
    Ok(CtxProbeOutput {
        meta: CtxProbeMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha(),
            host: hostname_refusing_unknown(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            k_primary: copts.k,
            k_run: CTX_K_RUN,
            max_cases: copts.max_cases,
            suite_set_rule: format!(
                "the registered modelless-eligible dataset suites with n >= \
                 {PRIMARY_SUITE_FLOOR_N} at run time (the Bench-092 gate-6 rule); \
                 every exclusion named; --suites overrides (disclosed)"
            ),
            seat_rule: "the strict-majority core over the K-primary prefix \
                        (first-appearance tie order); empty core = the seat \
                        abstains; unanimous disclosed beside; the K sweep (2/8) \
                        is a PREFIX of one 8-transform decision set — zero extra \
                        decides"
                .to_string(),
            fused_rule: "the fused gate IS the identity transform's decide (the \
                         production input): answered+pick or abstained, read from \
                         the engine slot verdicts"
                .to_string(),
            verdict_rule: format!(
                "G2a passes iff BOTH pooled paired LB95s > 0 (semantic-placebo on \
                 the both-answer pairing; semantic-fused at matched coverage); \
                 G2b the same on its pair, scored on the fused-answered set with \
                 the extra abstention scoring 0; LB95 = mean - {Z95_ONE_SIDED}·SE, \
                 pooled with suite fixed effects; G3 (the null path) fires when \
                 G2a fails BOTH bounds — the negative is scoped to this run \
                 table, THIS engine (bag-of-hashed-words additive features), the \
                 tested families"
            ),
            route_binding: if opts.kn_route {
                "by-name + index-anchored (issue 079)".to_string()
            } else {
                "strict by-name (--no-kn-route)".to_string()
            },
            box_start,
            box_end: crate::harness::box_state::capture(),
        },
        g1_identity_holds: suites.iter().all(|s| !s.g1_broken),
        suites,
        pooled,
    })
}

/// The pooled verdict across the run suites' raw deltas.
fn build_pooled(suites: &[CtxSuiteReport]) -> CtxPooled {
    let pick = |name: &'static str| {
        let cells: Vec<(&str, &[f64])> = suites
            .iter()
            .map(|s| (s.suite.as_str(), s.deltas.cell(name)))
            .collect();
        pooled_suite_delta(&cells)
    };
    let g2a_sem_plac = pick("g2a_sem_plac");
    let g2a_sem_fused = pick("g2a_sem_fused");
    let g2b_channel_sem = pick("g2b_channel_sem");
    let g2b_channel_plac = pick("g2b_channel_plac");
    let both_positive = |a: &Option<PooledDelta>, b: &Option<PooledDelta>| matches!((a, b), (Some(x), Some(y)) if x.lb95_pct > 0.0 && y.lb95_pct > 0.0);
    let any_positive = |a: &Option<PooledDelta>, b: &Option<PooledDelta>| {
        a.as_ref().is_some_and(|x| x.lb95_pct > 0.0) || b.as_ref().is_some_and(|y| y.lb95_pct > 0.0)
    };
    let g2a_pass = both_positive(&g2a_sem_plac, &g2a_sem_fused);
    let g2a_fail = !any_positive(&g2a_sem_plac, &g2a_sem_fused);
    let g2b_pass = both_positive(&g2b_channel_sem, &g2b_channel_plac);
    CtxPooled {
        g2a_sem_plac,
        g2a_sem_fused,
        g2b_channel_sem,
        g2b_channel_plac,
        g2a: if g2a_pass {
            "PASS — both pooled bounds > 0 (the G4 promotion path opens: an \
             opt-in `ctx_seat` plan)"
                .to_string()
        } else if g2a_fail {
            "NEGATIVE (G3) — both pooled bounds <= 0: the additive-transform \
             class is bounded on this engine, this run table, these families"
                .to_string()
        } else {
            "MIXED — one bound positive, one not: disclosed, no promotion, \
             no scoped close"
                .to_string()
        },
        g2b: if g2b_pass {
            "PASS — both pooled bounds > 0 (feeds the fused-gate extension \
             plan instead of a seat)"
                .to_string()
        } else {
            "NEGATIVE — the empty-core channel shows no placebo-separated lift".to_string()
        },
        g3_fired: g2a_fail,
    }
}

// ─────────────────────────────────────────────────────────── the render ──

/// The markdown rendering (the bench record's table block).
#[must_use]
pub fn render_ctx_probe_markdown(out: &CtxProbeOutput) -> String {
    let m = &out.meta;
    let mut s = String::new();
    s.push_str("# Ctx probe — additive context-transform agreement (issue 119)\n\n");
    s.push_str(&format!(
        "- date {} · sha {} · host {}\n- datasets {}\n- K primary {} of the \
         {}-transform set × ≤ {} case(s)/suite\n",
        m.date_utc,
        m.git_sha.as_deref().unwrap_or("unknown"),
        m.host,
        m.datasets_dir,
        m.k_primary,
        m.k_run,
        m.max_cases,
    ));
    for (name, rule) in [
        ("suite set", &m.suite_set_rule),
        ("seat", &m.seat_rule),
        ("fused", &m.fused_rule),
        ("verdict", &m.verdict_rule),
        ("route", &m.route_binding),
    ] {
        s.push_str(&format!("- {name}: {rule}\n"));
    }
    s.push_str(&format!(
        "\n**G1 identity control: {}**\n\n",
        if out.g1_identity_holds {
            "HOLDS"
        } else {
            "BROKEN — the run verdict is refused"
        },
    ));
    s.push_str("## Family agreement census (K=4 primary; the sweep rungs beside)\n\n");
    s.push_str(
        "| suite | family | n | unanimous | majority | no-core | no-core@K2 | no-core@K8 |\n",
    );
    s.push_str("|---|---|---|---|---|---|---|---|\n");
    for suite in &out.suites {
        for f in &suite.family_stats {
            s.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {:.3} | {:.3} |\n",
                suite.suite,
                f.family,
                f.n,
                f.unanimous_k4,
                f.majority_k4,
                f.no_core_k4,
                f.empty_core_rate_k2,
                f.empty_core_rate_k8,
            ));
        }
    }
    s.push_str("\n## Paired cells (per suite, percentage points)\n\n");
    s.push_str("| suite | cell | n | mean | SE | LB95 |\n");
    s.push_str("|---|---|---|---|---|---|\n");
    for suite in &out.suites {
        for c in &suite.cells {
            s.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                suite.suite,
                c.cell,
                c.n,
                c.mean_pct
                    .map(|v| format!("{v:.2}"))
                    .unwrap_or_else(|| "—".into()),
                c.se_pct
                    .map(|v| format!("{v:.2}"))
                    .unwrap_or_else(|| "—".into()),
                c.lb95_pct
                    .map(|v| format!("{v:.2}"))
                    .unwrap_or_else(|| "—".into()),
            ));
        }
    }
    let p = &out.pooled;
    s.push_str("\n## Pooled verdict (suite fixed effects)\n\n");
    let row = |name: &str, d: &Option<PooledDelta>| match d {
        Some(d) => format!(
            "| {name} | {} | {} | {:.2} | {:.2} | {:.2} |\n",
            d.n_suites, d.n_cases, d.mean_pct, d.se_pct, d.lb95_pct
        ),
        None => format!("| {name} | 0 | 0 | — | — | — |\n"),
    };
    s.push_str("| cell | suites | cases | mean | SE | LB95 |\n");
    s.push_str("|---|---|---|---|---|---|\n");
    s.push_str(&row("g2a_sem_plac", &p.g2a_sem_plac));
    s.push_str(&row("g2a_sem_fused", &p.g2a_sem_fused));
    s.push_str(&row("g2b_channel_sem", &p.g2b_channel_sem));
    s.push_str(&row("g2b_channel_plac", &p.g2b_channel_plac));
    s.push_str(&format!("\n**G2a:** {}\n\n**G2b:** {}\n\n", p.g2a, p.g2b));
    s.push_str(&format!(
        "**G3 (null path): {}**\n",
        if p.g3_fired {
            "FIRED — the scoped negative is the record's verdict"
        } else {
            "not fired"
        }
    ));
    if out.suites.iter().any(|s| !s.skipped.is_empty()) {
        s.push_str("\n## Skipped (loud)\n\n");
        for suite in &out.suites {
            for e in &suite.skipped {
                s.push_str(&format!("- {}: {e}\n", suite.suite));
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::{lb95, paired_delta, pooled_suite_delta};

    #[test]
    fn paired_delta_needs_two_cases_and_reads_mean_se() {
        assert!(paired_delta(&[]).is_none());
        assert!(paired_delta(&[0.5]).is_none());
        let d = paired_delta(&[0.0, 0.0, 0.0, 1.0]).expect("n=4");
        assert_eq!(d.n, 4);
        assert!((d.mean_pct - 25.0).abs() < 1e-9);
        // sample sd of {0,0,0,1} = 0.5; SE = 0.25 → pct 25.
        assert!((d.se_pct - 25.0).abs() < 1e-9);
        // LB95 = 25 − 1.6449×25 < 0 — a 25% edge over 4 cases cannot clear.
        assert!(lb95(&d) < 0.0);
        // A unanimous win carries the bound over zero.
        let all = paired_delta(&[1.0; 9]).expect("n=9");
        assert!(lb95(&all) > 0.0);
    }

    #[test]
    fn pooled_suite_delta_recenters_on_suite_means() {
        // Suite A: 3 cases at +1; suite B: 1 case — B contributes nothing
        // (n < 2), A pools alone.
        let b = [1.0_f64];
        let a = [1.0_f64, 1.0, 1.0];
        let p = pooled_suite_delta(&[("A", &a), ("B", &b)]).expect("A pools");
        assert_eq!(p.n_suites, 1);
        assert_eq!(p.n_cases, 3);
        assert!((p.mean_pct - 100.0).abs() < 1e-9);
        // Two suites, opposite effects → the fixed-effects pool cancels.
        let x = [1.0_f64, 1.0, 1.0, 1.0];
        let y = [-1.0_f64, -1.0, -1.0, -1.0];
        let p2 = pooled_suite_delta(&[("X", &x), ("Y", &y)]).expect("two suites");
        assert_eq!(p2.n_suites, 2);
        assert!(p2.mean_pct.abs() < 1e-9, "opposite suite effects cancel");
        // Empty pool → None.
        let c = [1.0_f64];
        assert!(pooled_suite_delta(&[("C", &c)]).is_none());
    }
}
