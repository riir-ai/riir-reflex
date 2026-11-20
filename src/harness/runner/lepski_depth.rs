//! Issue 130 — the Lepski cascade adaptive-depth premise check + G3
//! promotion cell.
//!
//! Report-only, harness-only (G-ISO: the serve path is untouched). Forwards
//! katgpt-core `lepski_cascade` (`estimators` umbrella) — wire, never fork.
//! The pre-registration lives in `.issues/130_lepski_cascade_adaptive_depth_g3.md`
//! (constants, slice, gates, bars — written BEFORE any run); this module
//! implements it verbatim:
//!
//! - **P1** — the in-domain grid-formability census (expected inert at the
//!   served ≤64-doc per-label corpora: `implementable_j` = 0 everywhere —
//!   the structural scale finding, measured not assumed);
//! - **P-A / P-B** — the cal-slice premise gates (depth headroom over the
//!   shipped routed-whole reference + selector sanity), decided BEFORE any
//!   test read;
//! - **G3** — the gated ONE-frozen-test-read: paired LB95 of
//!   (cascade − fixed-grid-max) per-case correctness > 0, only on suites
//!   passing both gates; the reference pair (cascade − routed-whole) is
//!   reported, never gated;
//! - the abstain arm (gate ∨ cascade-first-step-reject) and the latency
//!   pricing (σ̂ + fold vs the p99 ≤ 1 ms decision budget) ride every read,
//!   report-only.
//!
//! σ̂ derivation at the surface (per the issue's σ̂ note): the per-doc
//! response vector of one option is a RANKED score vector with no
//! covariate — under the null "no depth structure" the responses are one
//! noisy level, so the spread about the median IS the noise: the bulk-MAD
//! form (`sigma_hat_bulk_mad_f32`), never the NN-MAD (differencing ranked
//! values measures order-statistic spacing and collapses τ̂). Computed over
//! the k₀-winner's top-grid-max window (the pre-registered large-slice
//! form; the primitive's two in-window selections are the only O(n) cost).
//!
//! Determinism: sequential folds, first-index ties, no RNG, one process.
//! The frozen read is the single `--lepski-depth` invocation.

use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use serde_json::Value;

use katgpt_core::compression_drafter::{CompressionDrafter, Lz4FlexDrafter};
use katgpt_core::lepski_cascade::{
    A_DEFAULT, geometric_grid_into, implementable_k0, lepski_cascade, prefix_means_into,
    sigma_hat_bulk_mad_f32, tau_hat_into,
};

use super::{
    RunOptions, SUITES, git_sha, iso8601_utc, per_label_corpus_docs, percentile_us, prepare,
};
use crate::embed::{EMBED_DIM, Embedder};
use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};

/// η — the paper's grid failure probability (pre-registered).
const ETA: f64 = 0.05;
/// The scored-window cap (pre-registered): the pool arm scores the top
/// `min(n, K_MAX_CAP)` ranked docs; the grid tops out there too. Beyond a
/// few hundred docs the slice is "everything relevant" — the depth
/// structure the cascade selects over lives below this.
const K_MAX_CAP: usize = 512;
/// P-A headroom bar, percentage points (pre-registered).
const PREMISE_HEADROOM_PP: f64 = 1.0;
/// P-B allowance below the best fixed depth, pp (pre-registered).
const PREMISE_SELECTOR_SLACK_PP: f64 = 1.0;
/// P-B abstain-rate ceiling (pre-registered).
const PREMISE_ABSTAIN_MAX: f64 = 0.5;
/// The pre-registered slice — the eight corpus-ON dataset suites, registry
/// order. `--suites` does NOT narrow this lane (the frozen read is exact);
/// the banner discloses that.
const PREMISE_SUITES: [&str; 8] = [
    "ag_news",
    "emotion",
    "sst5",
    "prompt_injections",
    "xnli_en",
    "massive_intent_en",
    "banking77",
    "typed_decisions",
];

const READ_RULE: &str = "one invocation = one frozen read: cal premise gates (P-A headroom \
>= +1.0pp over the routed-whole reference; P-B cascade >= best-fixed -1.0pp and abstain <= \
50%) decide whether the test slice is read at all; G3 = paired LB95 of (cascade - \
fixed-grid-max) per-case correctness > 0; forced-pick everywhere; case-correct = ALL its \
scorable questions correct";

// ── report shape ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct LepskiDepthReport {
    meta: Meta,
    suites: Vec<SuiteRow>,
}

#[derive(Serialize)]
struct Meta {
    date_utc: String,
    git_sha: String,
    datasets_dir: String,
    eta: f64,
    a: f64,
    k_max_cap: usize,
    headroom_pp: f64,
    selector_slack_pp: f64,
    abstain_max: f64,
    read_rule: &'static str,
}

#[derive(Serialize)]
struct SuiteRow {
    suite: String,
    n_pool: usize,
    domains: usize,
    fallback_labels: usize,
    k0: usize,
    grid: Vec<usize>,
    /// P1: domains whose in-domain grid forms fewer than 2 points (the
    /// in-domain adaptive-depth arm is inert there).
    p1_inert_domains: usize,
    premise: Premise,
    /// `None` = the suite did not pass both premise gates — the test slice
    /// was never read (a verdict, not a gap).
    g3: Option<G3Row>,
    latency: LatRow,
}

#[derive(Serialize)]
struct Premise {
    acc_ref: f64,
    acc_grid: Vec<f64>,
    best_fixed: usize,
    acc_best_fixed: f64,
    acc_cascade: f64,
    abstain_rate: f64,
    sigma_zero_rate: f64,
    pa_pass: bool,
    pb_pass: bool,
    gated_in: bool,
    skipped_questions: usize,
}

#[derive(Serialize)]
struct G3Row {
    n_cases: usize,
    n_questions: usize,
    acc_cascade: f64,
    acc_fixed: f64,
    lb95_cascade_vs_fixed: f64,
    acc_ref: f64,
    lb95_cascade_vs_ref: f64,
    gate: GateBlock,
}

#[derive(Serialize)]
struct GateBlock {
    threshold_p30: f64,
    gate_alone_rate: f64,
    fused_rate: f64,
    /// The cascade pick's accuracy where the fused rule SERVES.
    acc_cascade_on_served: f64,
    /// The cascade pick's accuracy where the fused rule abstains (the
    /// hard-case check: abstention should select harder questions).
    acc_cascade_on_abstained: f64,
}

#[derive(Serialize)]
struct LatRow {
    p50_pass_us: u64,
    p99_pass_us: u64,
    pass_support: usize,
    p50_ref_us: u64,
    p99_ref_us: u64,
    p50_sigma_us: f64,
    p99_sigma_us: f64,
    p50_cascade_us: f64,
    p99_cascade_us: f64,
}

/// One scored question's record (per arm-read).
struct QRec {
    gold: usize,
    ref_pick: usize,
    fixed_pick: usize,
    depth_picks: Vec<usize>,
    cascade_pick: usize,
    cascade_abstain: bool,
    gate_cos: f64,
    sigma: f64,
    pass_us: u64,
    ref_us: u64,
    sigma_ns: u64,
    cascade_ns: u64,
}

struct ArmRead {
    /// Per case, the scorable questions' records (in question order).
    recs: Vec<Vec<QRec>>,
    skipped_questions: usize,
}

// ── entry ────────────────────────────────────────────────────────────────

/// The `--lepski-depth` lane: the pre-registered frozen read over
/// [`PREMISE_SUITES`]. Writes `lepski_depth.json` + `LEPSKI_DEPTH.md` under
/// `<out_dir>/lepski_depth/` and returns the markdown.
pub fn run_lepski_depth(opts: &RunOptions, out_dir: &Path) -> Result<String, String> {
    let mut rows = Vec::with_capacity(PREMISE_SUITES.len());
    for name in PREMISE_SUITES {
        rows.push(lepski_suite(name, opts)?);
    }
    let report = LepskiDepthReport {
        meta: Meta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
            datasets_dir: opts.datasets_dir.display().to_string(),
            eta: ETA,
            a: A_DEFAULT,
            k_max_cap: K_MAX_CAP,
            headroom_pp: PREMISE_HEADROOM_PP,
            selector_slack_pp: PREMISE_SELECTOR_SLACK_PP,
            abstain_max: PREMISE_ABSTAIN_MAX,
            read_rule: READ_RULE,
        },
        suites: rows,
    };
    let dir = out_dir.join("lepski_depth");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let json_path = dir.join("lepski_depth.json");
    let md_path = dir.join("LEPSKI_DEPTH.md");
    let json = serde_json::to_string_pretty(&report).expect("lepski-depth serialize");
    let md = render_markdown(&report);
    std::fs::write(&json_path, json).map_err(|e| format!("write {}: {e}", json_path.display()))?;
    std::fs::write(&md_path, &md).map_err(|e| format!("write {}: {e}", md_path.display()))?;
    Ok(md)
}

// ── per-suite ────────────────────────────────────────────────────────────

fn lepski_suite(name: &str, opts: &RunOptions) -> Result<SuiteRow, String> {
    let spec = SUITES
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("lepski-depth: suite {name} is not in the registry"))?;
    if spec.synthetic.is_some() {
        return Err(format!(
            "lepski-depth: {name} is a synthetic suite (no corpus pool)"
        ));
    }
    let prepared = prepare(spec, &opts.datasets_dir)?;
    // The engine's own drafter corpora (the wire-faithful doc sets):
    // per-label pool docs, capped, the Issue-039 self-doc fallback.
    let (per_label, fallback_labels) =
        per_label_corpus_docs(&prepared.train, &prepared.labels, spec.corpus_cap_per_label);
    let mut pool: Vec<&str> = Vec::new();
    let mut domain_ranges: Vec<(usize, usize)> = Vec::with_capacity(per_label.len());
    for docs in &per_label {
        let start = pool.len();
        pool.extend(docs.iter().map(String::as_str));
        domain_ranges.push((start, pool.len()));
    }
    let n_pool = pool.len();
    if n_pool == 0 {
        return Err(format!("lepski-depth: {name} has an empty corpus pool"));
    }

    // The pool grid (the only formable one — see P1).
    let k0 = implementable_k0(n_pool, ETA);
    let mut grid_arr = [0usize; 32];
    let grid_w = geometric_grid_into(k0, K_MAX_CAP.min(n_pool), &mut grid_arr);
    if grid_w < 1 {
        return Err(format!(
            "lepski-depth: {name} forms an empty grid (pool {n_pool})"
        ));
    }
    let grid: Vec<usize> = grid_arr[..grid_w].to_vec();

    // P1 — the in-domain formability census (the served per-label corpora).
    let mut p1_inert = 0usize;
    for (s, e) in &domain_ranges {
        let n_d = e - s;
        let k0_d = implementable_k0(n_d, ETA);
        let mut g = [0usize; 8];
        let w = geometric_grid_into(k0_d, n_d, &mut g);
        if w < 2 {
            p1_inert += 1;
        }
    }

    // Embed the pool once; domain centroids mirror `DomainExpert::from_embedded`
    // (mean then unit — the route direction).
    let embedder = Embedder;
    let mut doc_rows: Vec<[f32; EMBED_DIM]> = Vec::with_capacity(n_pool);
    for d in &pool {
        let mut v = [0.0f32; EMBED_DIM];
        embedder.embed_into(d.as_bytes(), &mut v);
        doc_rows.push(v);
    }
    let mut dirs: Vec<[f32; EMBED_DIM]> = Vec::with_capacity(per_label.len());
    for (s, e) in &domain_ranges {
        let k = (e - s) as f32;
        let mut acc = [0.0f32; EMBED_DIM];
        for r in &doc_rows[*s..*e] {
            for (a, x) in acc.iter_mut().zip(r.iter()) {
                *a += x;
            }
        }
        for x in acc.iter_mut() {
            *x /= k;
        }
        let nrm = acc.iter().map(|x| x * x).sum::<f32>().sqrt();
        if nrm > 0.0 {
            for x in acc.iter_mut() {
                *x /= nrm;
            }
        }
        dirs.push(acc);
    }

    // Per-doc drafters (the pool arm) + the shipped-form reference drafters
    // (`docs.join("\n")` — exactly `DomainExpert::new`'s corpus).
    let mut pool_drafters: Vec<Lz4FlexDrafter> = pool
        .iter()
        .map(|d| Lz4FlexDrafter::new(d.as_bytes().to_vec()))
        .collect();
    let mut domain_drafters: Vec<Lz4FlexDrafter> = per_label
        .iter()
        .map(|docs| Lz4FlexDrafter::new(docs.join("\n").into_bytes()))
        .collect();

    // CAL — the premise read.
    let cal = read_arm(
        &prepared.cal_cases,
        &prepared.cal_state_strs,
        &embedder,
        &doc_rows,
        &dirs,
        &mut pool_drafters,
        &mut domain_drafters,
        &grid,
        n_pool,
    )?;
    let prem = premise_of(&cal, &grid);

    // Gate threshold: the cal 30th percentile of the routed-domain max-cos
    // (nearest-rank; the shipped gate's cal-fitted convention). Fitted on
    // CAL only, applied to whichever slice is read.
    let mut cos: Vec<f64> = cal.recs.iter().flatten().map(|q| q.gate_cos).collect();
    cos.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let gate_p30 = nearest_rank(&cos, 0.30);

    // The premise gates decide whether the test slice is read AT ALL — the
    // gated frozen read (one invocation, one test read, no re-run on a miss).
    let g3 = if prem.pa_pass && prem.pb_pass {
        let test = read_arm(
            &prepared.suite.cases,
            &prepared.state_strs,
            &embedder,
            &doc_rows,
            &dirs,
            &mut pool_drafters,
            &mut domain_drafters,
            &grid,
            n_pool,
        )?;
        Some(g3_row(&test, gate_p30))
    } else {
        None
    };

    let latency = latency_row(&cal);
    Ok(SuiteRow {
        suite: name.to_string(),
        n_pool,
        domains: per_label.len(),
        fallback_labels: fallback_labels.len(),
        k0,
        grid,
        p1_inert_domains: p1_inert,
        premise: prem,
        g3,
        latency,
    })
}

// ── premise + G3 aggregation ─────────────────────────────────────────────

fn premise_of(read: &ArmRead, grid: &[usize]) -> Premise {
    let (acc_ref, _, _) = acc_of(read, |q| q.ref_pick);
    let mut acc_grid = Vec::with_capacity(grid.len());
    for j in 0..grid.len() {
        acc_grid.push(acc_of(read, |q| q.depth_picks[j]).0);
    }
    let (best_fixed, acc_best_fixed) = {
        // First-index tie rule (the pre-registered law — never max_by's
        // last-on-tie).
        let mut bi = 0usize;
        let mut bv = f64::NEG_INFINITY;
        for (i, &v) in acc_grid.iter().enumerate() {
            if v > bv {
                bv = v;
                bi = i;
            }
        }
        (bi, bv)
    };
    let (acc_cascade, nq, _) = acc_of(read, |q| q.cascade_pick);
    let abstain = read
        .recs
        .iter()
        .flatten()
        .filter(|q| q.cascade_abstain)
        .count();
    let sigma_zero = read
        .recs
        .iter()
        .flatten()
        .filter(|q| q.sigma <= 0.0)
        .count();
    let abstain_rate = pct(abstain, nq);
    let sigma_zero_rate = pct(sigma_zero, nq);
    let max_grid = acc_grid.iter().copied().fold(0.0_f64, f64::max);
    let pa_pass = max_grid >= acc_ref + PREMISE_HEADROOM_PP / 100.0;
    let pb_pass = acc_cascade + PREMISE_SELECTOR_SLACK_PP / 100.0 >= acc_best_fixed
        && abstain_rate <= PREMISE_ABSTAIN_MAX;
    Premise {
        acc_ref,
        acc_grid,
        best_fixed,
        acc_best_fixed,
        acc_cascade,
        abstain_rate,
        sigma_zero_rate,
        pa_pass,
        pb_pass,
        gated_in: pa_pass && pb_pass,
        skipped_questions: read.skipped_questions,
    }
}

fn g3_row(test: &ArmRead, gate_p30: f64) -> G3Row {
    let ok_cas = case_ok(test, |q| q.cascade_pick);
    let ok_fixed = case_ok(test, |q| q.fixed_pick);
    let ok_ref = case_ok(test, |q| q.ref_pick);
    let (acc_cascade, nq, _) = acc_of(test, |q| q.cascade_pick);
    let (acc_fixed, _, _) = acc_of(test, |q| q.fixed_pick);
    let (acc_ref, _, _) = acc_of(test, |q| q.ref_pick);
    let mut gate_alone = 0usize;
    let mut fused = 0usize;
    let mut served_n = 0usize;
    let mut served_ok = 0usize;
    let mut abst_n = 0usize;
    let mut abst_ok = 0usize;
    for q in test.recs.iter().flatten() {
        let gate_fires = q.gate_cos < gate_p30;
        let abstains = gate_fires || q.cascade_abstain;
        let ok = q.cascade_pick == q.gold;
        if gate_fires {
            gate_alone += 1;
        }
        if abstains {
            fused += 1;
            abst_n += 1;
            abst_ok += usize::from(ok);
        } else {
            served_n += 1;
            served_ok += usize::from(ok);
        }
    }
    G3Row {
        n_cases: test.recs.len(),
        n_questions: nq,
        acc_cascade,
        acc_fixed,
        lb95_cascade_vs_fixed: super::ensemble::paired_lb95(&ok_cas, &ok_fixed),
        acc_ref,
        lb95_cascade_vs_ref: super::ensemble::paired_lb95(&ok_cas, &ok_ref),
        gate: GateBlock {
            threshold_p30: gate_p30,
            gate_alone_rate: pct(gate_alone, nq),
            fused_rate: pct(fused, nq),
            acc_cascade_on_served: pct(served_ok, served_n),
            acc_cascade_on_abstained: pct(abst_ok, abst_n),
        },
    }
}

fn acc_of(read: &ArmRead, pick: impl Fn(&QRec) -> usize) -> (f64, usize, usize) {
    let mut nq = 0usize;
    let mut ok = 0usize;
    for q in read.recs.iter().flatten() {
        nq += 1;
        ok += usize::from(pick(q) == q.gold);
    }
    (pct(ok, nq), nq, ok)
}

fn case_ok(read: &ArmRead, pick: impl Fn(&QRec) -> usize) -> Vec<bool> {
    read.recs
        .iter()
        .map(|qs| qs.iter().all(|q| pick(q) == q.gold))
        .collect()
}

fn pct(ok: usize, n: usize) -> f64 {
    if n == 0 { 0.0 } else { ok as f64 / n as f64 }
}

/// Nearest-rank percentile over an ASCENDING-sorted slice: the value at or
/// below which `p` of the mass lies (`idx = ceil(p·n) − 1`, clamped ≥ 0).
fn nearest_rank(sorted_asc: &[f64], p: f64) -> f64 {
    if sorted_asc.is_empty() {
        return 0.0;
    }
    let idx = ((p * sorted_asc.len() as f64).ceil() as usize).saturating_sub(1);
    sorted_asc[idx.min(sorted_asc.len() - 1)]
}

fn latency_row(read: &ArmRead) -> LatRow {
    let pass: Vec<u64> = read.recs.iter().flatten().map(|q| q.pass_us).collect();
    let refr: Vec<u64> = read.recs.iter().flatten().map(|q| q.ref_us).collect();
    let sigma_ns: Vec<u64> = read.recs.iter().flatten().map(|q| q.sigma_ns).collect();
    let cascade_ns: Vec<u64> = read.recs.iter().flatten().map(|q| q.cascade_ns).collect();
    let (p50_pass, p99_pass, support) = percentile_us(&pass);
    let (p50_ref, p99_ref, _) = percentile_us(&refr);
    let (s50, s99) = ns_percentiles_us(&sigma_ns);
    let (c50, c99) = ns_percentiles_us(&cascade_ns);
    LatRow {
        p50_pass_us: p50_pass,
        p99_pass_us: p99_pass,
        pass_support: support,
        p50_ref_us: p50_ref,
        p99_ref_us: p99_ref,
        p50_sigma_us: s50,
        p99_sigma_us: s99,
        p50_cascade_us: c50,
        p99_cascade_us: c99,
    }
}

/// Nearest-rank p50/p99 over ns samples, reported in µs (the same
/// convention as the harness's `percentile_us`).
fn ns_percentiles_us(ns: &[u64]) -> (f64, f64) {
    if ns.is_empty() {
        return (0.0, 0.0);
    }
    let mut d = ns.to_vec();
    d.sort_unstable();
    let n = d.len();
    let p50 = d[n / 2] as f64 / 1000.0;
    let idx99 = (n * 99).div_ceil(100) - 1;
    (p50, d[idx99] as f64 / 1000.0)
}

// ── the read ─────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn read_arm(
    cases: &[SuiteCase],
    state_strs: &[String],
    embedder: &Embedder,
    doc_rows: &[[f32; EMBED_DIM]],
    dirs: &[[f32; EMBED_DIM]],
    pool_drafters: &mut [Lz4FlexDrafter],
    domain_drafters: &mut [Lz4FlexDrafter],
    grid: &[usize],
    n_pool: usize,
) -> Result<ArmRead, String> {
    let mut recs = Vec::with_capacity(cases.len());
    let mut skipped_questions = 0usize;
    for (ci, case) in cases.iter().enumerate() {
        let state = state_strs
            .get(ci)
            .ok_or_else(|| format!("lepski-depth: case {} has no state string", case.id))?;
        let mut case_recs = Vec::with_capacity(case.questions.len());
        for (q, gold) in case.questions.iter().zip(case.gold.iter()) {
            let Ok(options) = options_of(q) else {
                skipped_questions += 1;
                continue;
            };
            if options.is_empty() || gold.idx >= options.len() {
                skipped_questions += 1;
                continue;
            }
            case_recs.push(score_question(
                q,
                state,
                gold.idx,
                &options,
                embedder,
                doc_rows,
                dirs,
                pool_drafters,
                domain_drafters,
                grid,
                n_pool,
            )?);
        }
        if !case_recs.is_empty() {
            recs.push(case_recs);
        }
    }
    Ok(ArmRead {
        recs,
        skipped_questions,
    })
}

/// The question's option bytes, the engine's own spellings (`fresh_question`
/// mirrored): choice = criteria keys in map order, score = the level
/// strings, noul = the internal `[yes, no]` pair.
fn options_of(q: &SuiteQuestion) -> Result<Vec<String>, String> {
    match q.kind {
        QKind::Choice => {
            let keys = q
                .criteria
                .as_object()
                .ok_or_else(|| format!("q {}: choice criteria must be an object", q.qid))?;
            Ok(keys.keys().cloned().collect())
        }
        QKind::Score => {
            let arr = q
                .criteria
                .as_array()
                .ok_or_else(|| format!("q {}: score criteria must be an array", q.qid))?;
            Ok(arr
                .iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect())
        }
        QKind::Noul => Ok(vec!["yes".to_string(), "no".to_string()]),
    }
}

/// Internal → label-space pick: noul's internal order is `[yes, no]`, the
/// label space is `[no, yes]` (`gold_answer`'s `usize::from(is_true)`).
fn to_label(kind: QKind, internal: usize) -> usize {
    if kind == QKind::Noul {
        1 - internal.min(1)
    } else {
        internal
    }
}

#[allow(clippy::too_many_arguments)]
fn score_question(
    q: &SuiteQuestion,
    state: &str,
    gold: usize,
    options: &[String],
    embedder: &Embedder,
    doc_rows: &[[f32; EMBED_DIM]],
    dirs: &[[f32; EMBED_DIM]],
    pool_drafters: &mut [Lz4FlexDrafter],
    domain_drafters: &mut [Lz4FlexDrafter],
    grid: &[usize],
    n_pool: usize,
) -> Result<QRec, String> {
    // The FULL adaptive block (embed, route, retrieval, scoring, prefix
    // folds, picks, σ̂, cascade) is one timed region — the per-decision
    // cost this form would pay. The reference block below re-runs its own
    // embed+route so both rows are self-contained per-decision costs (the
    // shipped engine pays embed+route per decision too).
    let t_pass = Instant::now();
    // ctx = state + '\n' + instructions (the wire question carries no
    // criteria — the engine's exact bytes).
    let mut ctx = Vec::with_capacity(state.len() + q.instructions.len() + 2);
    ctx.extend_from_slice(state.as_bytes());
    ctx.push(b'\n');
    ctx.extend_from_slice(q.instructions.as_bytes());

    // One embed — the query vector drives route + retrieval + gate.
    let mut qv = [0.0f32; EMBED_DIM];
    embedder.embed_into(&ctx, &mut qv);

    // Route: the routed dot (first-index argmax of the sequential fold —
    // the engine's pick law) IS the gate's corpus-distance value. The
    // routed DOMAIN itself is re-derived inside the reference block below.
    let mut best_dot = f32::NEG_INFINITY;
    for dir in dirs.iter() {
        let dot: f32 = qv.iter().zip(dir.iter()).map(|(a, b)| a * b).sum();
        if dot > best_dot {
            best_dot = dot;
        }
    }
    let gate_cos = f64::from(best_dot);

    // ONE retrieval at grid-max over the pool: stable sort, ties keep pool
    // order (the pre-registered tie rule).
    let mut ranked: Vec<(f32, u32)> = doc_rows
        .iter()
        .enumerate()
        .map(|(i, r)| (qv.iter().zip(r.iter()).map(|(a, b)| a * b).sum(), i as u32))
        .collect();
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // The scoring pass: per-doc compressed-length deltas per option, the
    // ranked windows kept for the prefix folds. The window is the grid max
    // (the deepest prefix the folds consume).
    let grid_len = grid.len();
    let grid_max = grid[grid_len - 1];
    let mut windows: Vec<Vec<f64>> = Vec::with_capacity(options.len());
    let mut resp = vec![0.0f64; grid_max];
    for opt in options {
        let cand = opt.as_bytes();
        for (r, slot) in resp.iter_mut().enumerate() {
            let doc = ranked[r].1 as usize;
            *slot = f64::from(pool_drafters[doc].score(&ctx, cand));
        }
        windows.push(resp.clone());
    }

    // Prefix means per option (the pre-registered fold), then the per-depth
    // first-index winners + the estimate series.
    let mut pm: Vec<Vec<f64>> = Vec::with_capacity(options.len());
    let mut pm_buf = vec![0.0f64; grid_len];
    for win in &windows {
        let written = prefix_means_into(win, grid, &mut pm_buf);
        if written != grid_len {
            return Err(format!(
                "lepski-depth: prefix fold wrote {written}/{grid_len} estimates"
            ));
        }
        pm.push(pm_buf.clone());
    }
    let mut depth_picks = vec![0usize; grid_len];
    let mut estimates = vec![0.0f64; grid_len];
    for (j, col_j) in estimates.iter_mut().enumerate() {
        let mut w = 0usize;
        let mut bv = f64::NEG_INFINITY;
        for (oi, col) in pm.iter().enumerate() {
            if col[j] > bv {
                bv = col[j];
                w = oi;
            }
        }
        depth_picks[j] = w;
        *col_j = bv;
    }

    // σ̂: bulk-MAD over the k₀-winner's ranked window (in-place selection on
    // a throwaway copy; the primitive permutes the buffer).
    let t_sigma = Instant::now();
    let winner0 = depth_picks[0];
    let mut win32: Vec<f32> = windows[winner0].iter().map(|&v| v as f32).collect();
    let sigma = sigma_hat_bulk_mad_f32(&mut win32).unwrap_or(0.0);
    let sigma_ns = t_sigma.elapsed().as_nanos() as u64;

    // τ̂ over the grid + the cascade (A = A_DEFAULT, pre-registered).
    let t_cas = Instant::now();
    let mut tau = vec![0.0f64; grid_len];
    tau_hat_into(sigma, n_pool, grid, &mut tau);
    let sel = lepski_cascade(&estimates, &tau, A_DEFAULT)
        .ok_or_else(|| "lepski-depth: empty grid reached the cascade".to_string())?;
    let cascade_abstain = sel.rejected_at == Some(1);
    let pass_us = t_pass.elapsed().as_micros() as u64;
    let cascade_ns = t_cas.elapsed().as_nanos() as u64;

    // The reference arm: the shipped form's drafter term — the routed
    // domain's WHOLE-corpus drafter, raw delta, first-index argmax. Its
    // own embed+route re-run keeps the row a self-contained per-decision
    // cost (deterministically identical values).
    let t_ref = Instant::now();
    let mut qv2 = [0.0f32; EMBED_DIM];
    embedder.embed_into(&ctx, &mut qv2);
    let mut ref_d = 0usize;
    let mut ref_dot = f32::NEG_INFINITY;
    for (d, dir) in dirs.iter().enumerate() {
        let dot: f32 = qv2.iter().zip(dir.iter()).map(|(a, b)| a * b).sum();
        if dot > ref_dot {
            ref_dot = dot;
            ref_d = d;
        }
    }
    let mut ref_pick = 0usize;
    let mut best_ref = i32::MIN;
    for (oi, opt) in options.iter().enumerate() {
        let s = domain_drafters[ref_d].score(&ctx, opt.as_bytes());
        if s > best_ref {
            best_ref = s;
            ref_pick = oi;
        }
    }
    let ref_us = t_ref.elapsed().as_micros() as u64;

    Ok(QRec {
        gold,
        ref_pick: to_label(q.kind, ref_pick),
        fixed_pick: to_label(q.kind, depth_picks[grid_len - 1]),
        depth_picks: depth_picks.iter().map(|&p| to_label(q.kind, p)).collect(),
        cascade_pick: to_label(q.kind, depth_picks[sel.index]),
        cascade_abstain,
        gate_cos,
        sigma,
        pass_us,
        ref_us,
        sigma_ns,
        cascade_ns,
    })
}

// ── markdown ─────────────────────────────────────────────────────────────

fn render_markdown(r: &LepskiDepthReport) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(8192);
    let _ = writeln!(
        out,
        "# Lepski cascade adaptive depth — the i130 frozen read\n"
    );
    let _ = writeln!(
        out,
        "| field | value |\n|---|---|\n| date | {} |\n| git | {} |\n| datasets | {} |\n| eta | {} |\n| A | {} |\n| k_max cap | {} |\n| read rule | {} |",
        r.meta.date_utc,
        r.meta.git_sha,
        r.meta.datasets_dir,
        r.meta.eta,
        r.meta.a,
        r.meta.k_max_cap,
        r.meta.read_rule
    );
    for s in &r.suites {
        let _ = writeln!(out, "\n## {}\n", s.suite);
        let _ = writeln!(
            out,
            "- pool {n} docs · {d} domains ({f} fallback) · k0 = {k0} · grid {grid:?} · P1 inert in-domain grids: {p1}/{d2}",
            n = s.n_pool,
            d = s.domains,
            f = s.fallback_labels,
            k0 = s.k0,
            grid = s.grid,
            p1 = s.p1_inert_domains,
            d2 = s.domains
        );
        let p = &s.premise;
        let _ = writeln!(
            out,
            "- CAL premise: ref(routed-whole) {acc_ref:.4} · grid [{}] · best-fixed {bf} {acc_bf:.4} · cascade {acc_c:.4} · abstain {ar:.3} · σ̂=0 rate {sz:.3}",
            p.acc_grid
                .iter()
                .map(|v| format!("{v:.4}"))
                .collect::<Vec<_>>()
                .join(", "),
            acc_ref = p.acc_ref,
            bf = p.best_fixed,
            acc_bf = p.acc_best_fixed,
            acc_c = p.acc_cascade,
            ar = p.abstain_rate,
            sz = p.sigma_zero_rate
        );
        let _ = writeln!(
            out,
            "- gates: P-A(headroom ≥ +{h}pp) {} · P-B(selector ≥ best−{sl}pp, abstain ≤ {am}) {} → {}",
            if p.pa_pass { "PASS" } else { "FAIL" },
            if p.pb_pass { "PASS" } else { "FAIL" },
            if p.gated_in {
                "GATED IN — test read"
            } else {
                "GATED OUT — test slice NOT read"
            },
            h = r.meta.headroom_pp,
            sl = r.meta.selector_slack_pp,
            am = r.meta.abstain_max
        );
        match &s.g3 {
            Some(g) => {
                let verdict = if g.lb95_cascade_vs_fixed > 0.0 {
                    "G3 PASS (LB95 > 0)"
                } else {
                    "G3 FAIL (LB95 ≤ 0)"
                };
                let _ = writeln!(
                    out,
                    "- TEST: {nc} cases / {nq} questions · cascade {acc:.4} vs fixed-grid-max {accf:.4} · LB95(cas−fix) {lb:.4} — {v}",
                    nc = g.n_cases,
                    nq = g.n_questions,
                    acc = g.acc_cascade,
                    accf = g.acc_fixed,
                    lb = g.lb95_cascade_vs_fixed,
                    v = verdict
                );
                let _ = writeln!(
                    out,
                    "- reference pair: routed-whole {accr:.4} · LB95(cas−ref) {lbr:.4} (reported, never gated)",
                    accr = g.acc_ref,
                    lbr = g.lb95_cascade_vs_ref
                );
                let _ = writeln!(
                    out,
                    "- abstain arm: gate p30 {thr:.4} · gate-alone rate {ga:.3} · fused rate {fr:.3} · cascade acc on served {sv:.4} · on abstained {ab:.4}",
                    thr = g.gate.threshold_p30,
                    ga = g.gate.gate_alone_rate,
                    fr = g.gate.fused_rate,
                    sv = g.gate.acc_cascade_on_served,
                    ab = g.gate.acc_cascade_on_abstained
                );
            }
            None => {
                let _ = writeln!(out, "- TEST: NOT READ (the premise gates decided it)");
            }
        }
        let l = &s.latency;
        let _ = writeln!(
            out,
            "- latency (cal questions): adaptive pass p50 {pp}µs / p99 {p9}µs (support {sup}) · routed-whole pass p50 {rp}µs / p99 {r9}µs · σ̂ p50 {s50:.3}µs / p99 {s99:.3}µs · cascade fold p50 {c50:.3}µs / p99 {c99:.3}µs",
            pp = l.p50_pass_us,
            p9 = l.p99_pass_us,
            sup = l.pass_support,
            rp = l.p50_ref_us,
            r9 = l.p99_ref_us,
            s50 = l.p50_sigma_us,
            s99 = l.p99_sigma_us,
            c50 = l.p50_cascade_us,
            c99 = l.p99_cascade_us
        );
    }
    out
}
