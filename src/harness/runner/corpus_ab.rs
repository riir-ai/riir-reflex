//! Plan 426 T5 — the corpus A/B lane (the V5 gate).
//!
//! One frozen test read, two engine builds, ONE changed variable — the
//! corpus:
//! - **Arm A (gold-only):** exactly the published posture — the registry
//!   corpus pool at the registry per-label cap (gold docs first).
//! - **Arm B (+synth):** the same pool, plus the synth artifact's rows
//!   appended BEYOND the gold cap up to `extra_cap` per label (gold-first
//!   ordering is preserved inside each label, so the cap still saturates
//!   with gold everywhere gold is plentiful; thin labels — the N2
//!   targets — are where synth docs enter).
//!
//! The comparison is FORCED-PICK hard accuracy (the published convention:
//! thresholds/calibration steer abstention, never the pick — so no gate
//! fit runs here and the two arms differ ONLY in corpus content). The
//! gate is the arena's own statistic: paired LB95 of (synth − gold) over
//! per-case correctness > 0. C3: both arms are reported separately, next
//! to each other, forever. V6 rides the same read: both arms' p50/p99
//! latency — the modelless lane must stay sub-ms after corpus growth.
//!
//! Instrument aliveness (the ensemble-gate law): arm A must reproduce
//! the published massive number (0.7800) — it reads the same 300 frozen
//! rows through the same build path. A drift there means the instrument
//! is mis-built, and the verdict is void before it is read.
//!
//! G-ISO: harness-only; the serve path is untouched.

use std::collections::BTreeMap;
use std::path::Path;

use super::echo_gates::{self, AbstentionGateBlock, OodBlock, OodRungRow};
use super::synth::load_synth_corpus;
use super::{
    RunOptions, SUITES, SuiteSpec, ensemble::paired_lb95, eval_engine, git_sha, iso8601_utc,
    percentile_us, prepare, seat,
};
use crate::embed::EMBED_DIM;
use crate::engine::{DecisionEngine, ExpertSpec};
use crate::harness::suites::{QKind, SuiteCase, TrainDoc};

/// The published massive modelless row (Bench 052 / the arena re-baseline
/// — the A/B's arm-A reproduction anchor; massive-only).
const PUBLISHED_MASSIVE_A0: f64 = 0.7800;

/// Per-suite A/B record.
#[derive(Debug, serde::Serialize)]
pub struct CorpusAbSuite {
    pub suite: String,
    pub synth_artifact: String,
    pub synth_rows_total: usize,
    pub synth_rows_in_scope: usize,
    pub gold_corpus_docs: usize,
    pub synth_corpus_docs: usize,
    pub acc_gold: f64,
    pub acc_synth: f64,
    pub delta_lb95: f64,
    /// Per-label flip counts (the N2 attribution view).
    pub flips: Vec<CorpusAbFlip>,
    pub b_only_cases: Vec<String>,
    pub a_only_cases: Vec<String>,
    /// Issue 104 additive disclosure: the case IDs where the SYNTH arm (B)
    /// was CORRECT — pairs two corpus-ab reads synth-arm-vs-synth-arm (the
    /// learnability gate class) without a third binary. Pure disclosure;
    /// no gate reads it.
    pub synth_ok_cases: Vec<String>,
    pub lat_p50_gold_us: u64,
    pub lat_p99_gold_us: u64,
    pub lat_p50_synth_us: u64,
    pub lat_p99_synth_us: u64,
    pub determinism_ok: bool,
    /// The instrument-aliveness note (the 0.7800 reproduction).
    pub aliveness: String,
    pub verdict: String,
    /// The Issue-064 mandatory gates: abstention-entropy preservation +
    /// the OOD word-dropout ladder (computed on every run).
    pub abstention: AbstentionGateBlock,
    pub ood: OodBlock,
    /// Plan 621 T5.1 (`--probe-delta-ab`, feature `probe_delta_ab`): the
    /// paired counterfactual ΔV certification over the SAME frozen read —
    /// additive disclosure beside the V5 verdict, never a gate
    /// replacement. None when the flag is off.
    #[cfg(feature = "probe_delta_ab")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probe: Option<crate::harness::probe_delta::ProbeDeltaCert>,
    /// Issue 129 (`--corpus-ab-mix`): the three-arm mix mode + saturation
    /// pre-check + prequential growth curve + collapse canaries —
    /// additive report blocks beside the V5 verdict, never a gate
    /// replacement. None when the flag is off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mix: Option<CorpusAbMixBlock>,
    pub seconds: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct CorpusAbFlip {
    pub label: String,
    pub gained: usize,
    pub lost: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct CorpusAbMeta {
    pub date_utc: String,
    pub git_sha: String,
    pub datasets_dir: String,
    pub corpus_cap_per_label: usize,
    pub synth_extra_cap: usize,
    pub read_rule: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct CorpusAbOutput {
    pub meta: CorpusAbMeta,
    pub suites: Vec<CorpusAbSuite>,
}

/// Issue 129 T1 — one mix-mode arm row: the arm's composition, its
/// accuracy, and its paired LB95s against BOTH singles (the paper's law
/// reads mix vs each arm alone).
#[derive(Debug, serde::Serialize)]
pub struct CorpusAbArmRow {
    /// gold | synth | mix:0.25 | mix:0.50
    pub arm: String,
    pub corpus_docs: usize,
    /// synth share of the arm's corpus (0.0 gold, 1.0 synth-only).
    pub synth_share: f64,
    pub acc: f64,
    pub lb95_vs_gold: f64,
    pub lb95_vs_synth: f64,
}

/// Issue 129 T2 — the suite-saturation pre-check row (report-only, no
/// exit-code change).
#[derive(Debug, serde::Serialize)]
pub struct SaturationRow {
    /// The frozen read's dominant gold label + its share.
    pub dominant_label: String,
    pub dominant_share: f64,
    /// Always-pick-dominant baseline accuracy on the frozen read.
    pub majority_acc: f64,
    /// Arm A (gold) accuracy — the "full-suite accuracy" comparator.
    pub engine_acc: f64,
    /// true = the majority baseline MATCHES the engine — the suite is
    /// uninformative for selection/allocation methods; report says so.
    pub saturated: bool,
}

/// Issue 129 T3 — one prequential growth stage: accuracy on the frozen
/// probe (never-grown: dataset-fixed membership) with the corpus
/// appended to fraction f of the artifact (gold-first preserved — arm
/// B's composition rule).
#[derive(Debug, serde::Serialize)]
pub struct PrequentialStageRow {
    pub append_frac: f64,
    pub appended_docs: usize,
    pub acc: f64,
    /// Paired LB95 vs the gold arm (stage − gold).
    pub lb95_vs_gold: f64,
    /// Marginal paired LB95 vs the PREVIOUS stage (stage − stage-1; the
    /// first stage compares vs gold).
    pub marginal_lb95: f64,
}

/// Issue 129 — the whole mix-mode block (T1 arms + the paper-law read,
/// T2 saturation, T3 prequential curve, T4 canaries).
#[derive(Debug, serde::Serialize)]
pub struct CorpusAbMixBlock {
    pub arms: Vec<CorpusAbArmRow>,
    /// The paper-law verdict: does at least one mix arm beat BOTH
    /// singles with paired LB95 > 0? Report-only — the V5 gate above is
    /// unchanged.
    pub mix_law: String,
    pub saturation: SaturationRow,
    pub prequential: Vec<PrequentialStageRow>,
    pub canaries: crate::harness::runner::canaries::CanaryRow,
}

/// Flip tallies per gold label (gained = synth-only correct, lost =
/// gold-only correct).
#[derive(Default)]
struct Flip {
    gained: usize,
    lost: usize,
}

/// The eval case's gold LABEL string (the per-label flip attribution).
fn gold_label_of_case(case: &SuiteCase) -> String {
    let Some(q) = case.questions.first() else {
        return String::new();
    };
    let keys: Vec<String> = match q.kind {
        QKind::Choice => q
            .criteria
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
        QKind::Score => (0..q.criteria.as_array().map_or(0, Vec::len))
            .map(|i| i.to_string())
            .collect(),
        QKind::Noul => vec![],
    };
    keys.get(case.gold[0].idx).cloned().unwrap_or_default()
}

/// The A/B's per-label spec builder: gold docs first (up to `cap`), then
/// the label's synth docs (up to `extra_cap`) — the self-doc fallback law
/// mirrors `specs_from_pool` (a label with neither gold nor synth docs
/// falls back to its own label text, disclosed). When the seat posture
/// arms the count tables (`nb_sets`), each spec carries them — arm A's
/// tables read the uncapped pool ([`nb_doc_sets`]); arm B's read the same
/// pool PLUS the synth docs (the corpus extension, tables included).
#[allow(clippy::too_many_arguments)]
fn arm_specs(
    train: &[TrainDoc],
    extra: &[TrainDoc],
    labels: &[String],
    cap: usize,
    extra_cap: usize,
    nb_sets: Option<&[Vec<String>]>,
) -> (Vec<ExpertSpec>, Vec<String>, usize) {
    // Delegated to the seat's shared constructor (Plan 426 T6): the seat
    // and this lane must not be able to drift apart — the V5 measurement
    // and the served seat are the SAME build by construction now.
    super::specs_corpus_extended(train, extra, labels, cap, extra_cap, nb_sets)
}

/// Run the corpus A/B for the synth artifact's suite. `opts.suites` (when
/// non-empty) must agree with the artifact's suite — a mismatch is a loud
/// refusal, never a silent suite swap. `mix` arms the Issue-129 mode
/// (gold / synth / mix arms + saturation + prequential + canaries).
pub fn run_corpus_ab(
    opts: &RunOptions,
    synth_path: &Path,
    extra_cap: usize,
    mix: bool,
) -> Result<CorpusAbOutput, String> {
    let (header, docs, _digest) = load_synth_corpus(synth_path)?;
    let Some(spec) = SUITES.iter().find(|s| s.name == header.suite) else {
        return Err(format!(
            "artifact suite {:?} is not a registered suite",
            header.suite
        ));
    };
    if !opts.suites.is_empty() && !opts.suites.iter().any(|s| s == spec.name) {
        return Err(format!(
            "--corpus-ab artifact suite {:?} is not in --suites {:?} — pass one story",
            spec.name, opts.suites
        ));
    }
    let prepared = prepare(spec, &opts.datasets_dir)?;
    let labels = &prepared.labels;
    let n_labels = labels.len();
    let in_scope: Vec<TrainDoc> = docs
        .iter()
        .filter(|d| labels.iter().any(|l| l == &d.label))
        .cloned()
        .collect();
    let dropped = docs.len() - in_scope.len();
    if dropped > 0 {
        eprintln!(
            "  [corpus-ab {}] {} synth row(s) outside the engine's {}-label universe — \
             dropped (counted, never silent)",
            spec.name, dropped, n_labels
        );
    }

    macro_rules! dispatch {
        ($n:literal) => {
            corpus_ab_suite::<$n>(
                spec,
                &prepared,
                &in_scope,
                docs.len(),
                synth_path,
                extra_cap,
                opts,
                mix,
            )
        };
    }
    let suite_row = match n_labels {
        2 => dispatch!(2)?,
        3 => dispatch!(3)?,
        4 => dispatch!(4)?,
        5 => dispatch!(5)?,
        6 => dispatch!(6)?,
        7 => dispatch!(7)?,
        8 => dispatch!(8)?,
        59 => dispatch!(59)?,
        77 => dispatch!(77)?,
        n => {
            return Err(format!(
                "corpus-ab: no engine instantiation for {n} domains — extend the dispatch \
                 table in runner.rs (the run() law)"
            ));
        }
    };
    Ok(CorpusAbOutput {
        meta: CorpusAbMeta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".into()),
            datasets_dir: opts.datasets_dir.display().to_string(),
            corpus_cap_per_label: spec.corpus_cap_per_label,
            synth_extra_cap: extra_cap,
            read_rule: "one frozen test read; two engine builds differing ONLY in corpus \
                        content; forced-pick hard accuracy; paired LB95 (synth − gold) > 0 \
                        is the V5 gate; both arms reported (C3)",
        },
        suites: vec![suite_row],
    })
}

#[allow(clippy::too_many_arguments)]
fn corpus_ab_suite<const N: usize>(
    spec: &SuiteSpec,
    prepared: &super::Prepared,
    synth_docs: &[TrainDoc],
    synth_rows_total: usize,
    synth_path: &Path,
    extra_cap: usize,
    opts: &RunOptions,
    mix: bool,
) -> Result<CorpusAbSuite, String> {
    let t_start = std::time::Instant::now();
    let labels = &prepared.labels;
    let cap = spec.corpus_cap_per_label;

    // The seat posture (the anchor-void fix, 61d4a94d's filed repair): arm A
    // is the DEPLOYED build — the same `fit_posture` prologue the seat and
    // `run_modelless` use at the PUBLISHED posture (the four select arms on,
    // registry cap candidates; they arm only where the cal slice clears the
    // bar and are byte-identical to off elsewhere — massive arms nb_scale
    // 4.0 + the fitted fused-gate thresholds). NOT the invocation's flags:
    // the A/B's contract is the published row, and a bare `--corpus-ab`
    // invocation would otherwise fit the default posture and read 0.6100
    // against the 0.7800 anchor (bench 089's void, reproduced here before
    // the knob pin). Both arms share ONE fitted posture: the A/B varies
    // ONLY the corpus content.
    let seat = seat::prepare_seat(spec.name, &opts.datasets_dir)?;
    let posture = seat::fit_posture::<N>(
        spec.name,
        &seat,
        &seat::PostureKnobs {
            head_select: true,
            nb_select: true,
            #[cfg(feature = "option_cond")]
            oc_select: true,
            #[cfg(feature = "nb_ridge")]
            ridge_select: true,
            genome_select: false,
            genome_accept_margin: 0.0,
            // The published-posture convention (instinct server.rs:457): the
            // cap ladder is OFF — the registry cap holds. The seat's cal
            // face is the fitted fused gate, never a cap selection.
            cal_select_caps: vec![],
            // Shipped gate face (never the rate levers) — byte-identical
            // with the pre-knob lane.
            gate_fit_selection: false,
            gate_distance_only: false,
            gate_rho: None,
            // The published face's drafter lever (issue 079: PerByte,
            // `d3aeeae`) — engages only on the drafter-only path, inert on
            // this lane's by-name suites (massive); armed to keep the
            // "A/B's contract is the published row" law true as the
            // posture moves.
            drafter_fix: crate::engine::DrafterFix::PerByte,
        },
    )?;
    if posture.cfg.oc_scale > 0.0 {
        return Err(format!(
            "corpus-ab {}: the seat posture arms the option-conditioned tables — the \\
             +synth corpus arm cannot carry them; the lane refuses rather than mis-build",
            spec.name
        ));
    }
    let cfg = posture.cfg.clone();
    let effective_cap = posture.effective_cap;
    let _ = cap;

    // Arm A: the seat build at the fitted posture (gold pool, the selected
    // cap) — the deployed engine, byte-equal to run()'s.
    let (mut eng_a, fb_a) =
        seat::build_seat_engine::<N>(spec.name, &seat, effective_cap, cfg.clone())?;
    if !fb_a.is_empty() {
        eprintln!(
            "  [corpus-ab {}] gold arm: {} label(s) with NO pool docs → self-doc fallback",
            spec.name,
            fb_a.len()
        );
    }
    // Arm B: the SAME posture, corpus = gold cap + synth beyond it — the
    // retrieval specs AND the count tables (arm A's tables read the
    // uncapped pool; arm B's read that pool plus the synth docs — the
    // corpus extension is the only difference between the arms).
    #[cfg(feature = "nb_scope")]
    let nb_sets_b =
        (cfg.nb_scale > 0.0).then(|| super::nb_doc_sets(&prepared.train, labels, synth_docs));
    #[cfg(not(feature = "nb_scope"))]
    let nb_sets_b: Option<Vec<Vec<String>>> = None;
    let (specs_b, fb_b, extra_used) = arm_specs(
        &prepared.train,
        synth_docs,
        labels,
        effective_cap,
        extra_cap,
        nb_sets_b.as_deref(),
    );
    let mut eng_b = DecisionEngine::<N, EMBED_DIM>::build_specs(specs_b, cfg.clone())
        .map_err(|e| format!("engine build ({} +synth): {e}", spec.name))?;
    if !fb_b.is_empty() {
        eprintln!(
            "  [corpus-ab {}] synth arm: {} label(s) with NO docs at all → self-doc fallback",
            spec.name,
            fb_b.len()
        );
    }

    let (eval_a, lat_a) = eval_engine(
        &mut eng_a,
        &prepared.suite.cases,
        &prepared.state_strs,
        true,
    )?;
    let (eval_b, lat_b) = eval_engine(
        &mut eng_b,
        &prepared.suite.cases,
        &prepared.state_strs,
        true,
    )?;

    // Per-case forced-pick correctness (single-question suites only — the
    // synth lane's own scope law).
    let mut ok_a = Vec::with_capacity(prepared.suite.cases.len());
    let mut ok_b = Vec::with_capacity(prepared.suite.cases.len());
    let mut b_only_cases = Vec::new();
    let mut a_only_cases = Vec::new();
    let mut synth_ok_cases = Vec::with_capacity(prepared.suite.cases.len());
    let mut per_label: BTreeMap<String, Flip> = BTreeMap::new();
    for (ci, case) in prepared.suite.cases.iter().enumerate() {
        let a = eval_a.picks[ci][0] == case.gold[0].idx;
        let b = eval_b.picks[ci][0] == case.gold[0].idx;
        ok_a.push(a);
        ok_b.push(b);
        if b {
            synth_ok_cases.push(case.id.clone());
        }
        let gold_label = gold_label_of_case(case);
        match (b, a) {
            (true, false) => {
                b_only_cases.push(case.id.clone());
                per_label.entry(gold_label).or_default().gained += 1;
            }
            (false, true) => {
                a_only_cases.push(case.id.clone());
                per_label.entry(gold_label).or_default().lost += 1;
            }
            _ => {}
        }
    }
    let acc_a = ok_a.iter().filter(|&&b| b).count() as f64 / ok_a.len() as f64;
    let acc_b = ok_b.iter().filter(|&&b| b).count() as f64 / ok_b.len() as f64;
    let lb95 = paired_lb95(&ok_b, &ok_a);

    // Plan 621 T5.1: the paired counterfactual ΔV certification over the
    // SAME frozen read (report-only; the V5 verdict above is unchanged).
    #[cfg(feature = "probe_delta_ab")]
    let probe_cert = opts
        .probe_delta_ab
        .then(|| crate::harness::probe_delta::certify(&ok_a, &ok_b));

    // ── Issue-064 mandatory gates (every run; modelless reads are µs-class).

    // (1) Abstention-entropy preservation: the calibrated fused gate's
    // fitted thresholds stay valid only if the answer-entropy shape and
    // abstain rate do not drift when the corpus gains synth docs.
    let abstention = echo_gates::abstention_gate(&eval_a, &eval_b);

    // (2) OOD word-dropout ladder: re-read both engines on corrupted
    // state strings (seeded per case+rung — byte-reproducible). The
    // corrupted reads share the SAME engines and posture; only the state
    // text degrades. Determinism is already asserted on the clean read;
    // the corrupted reads are accuracy-only, so the per-arm rerun probe
    // stays off.
    let mut ood_rungs = Vec::with_capacity(echo_gates::OOD_RUNGS.len());
    for &p in echo_gates::OOD_RUNGS.iter() {
        let corrupted: Vec<String> = prepared
            .suite
            .cases
            .iter()
            .enumerate()
            .map(|(ci, c)| {
                echo_gates::corrupt_state(
                    &prepared.state_strs[ci],
                    p,
                    echo_gates::state_seed(&c.id, p),
                )
            })
            .collect();
        let (ea, _) = eval_engine(&mut eng_a, &prepared.suite.cases, &corrupted, false)?;
        let (eb, _) = eval_engine(&mut eng_b, &prepared.suite.cases, &corrupted, false)?;
        let mut ok_ca = Vec::with_capacity(prepared.suite.cases.len());
        let mut ok_cb = Vec::with_capacity(prepared.suite.cases.len());
        for (ci, case) in prepared.suite.cases.iter().enumerate() {
            ok_ca.push(ea.picks[ci][0] == case.gold[0].idx);
            ok_cb.push(eb.picks[ci][0] == case.gold[0].idx);
        }
        let racc_a = ok_ca.iter().filter(|&&b| b).count() as f64 / ok_ca.len() as f64;
        let racc_b = ok_cb.iter().filter(|&&b| b).count() as f64 / ok_cb.len() as f64;
        let rdelta = racc_b - racc_a;
        let rlb = paired_lb95(&ok_cb, &ok_ca);
        ood_rungs.push(OodRungRow {
            dropout_p: p,
            acc_gold: racc_a,
            acc_synth: racc_b,
            delta: rdelta,
            delta_lb95: rlb,
        });
    }
    let ood = echo_gates::ood_block(ood_rungs, acc_b - acc_a, lb95);

    // ── Issue 129 (`--corpus-ab-mix`): the three-arm mix mode + the
    // saturation pre-check + the prequential growth curve + the collapse
    // canaries. Report-only — the V5 verdict above is computed BEFORE
    // this block and is byte-identical with the flag off.
    let mix_block = mix
        .then(|| {
            mix_block_computed::<N>(
                spec,
                prepared,
                synth_docs,
                &cfg,
                effective_cap,
                extra_cap,
                &ok_a,
                &ok_b,
                acc_a,
            )
        })
        .transpose()?;

    // Instrument aliveness: arm A must reproduce the published massive
    // row — same 300 frozen rows, same build path. (Other suites: the
    // anchor is whatever run() publishes; no hard pin here.)
    let aliveness = if spec.name == "massive_intent_en" {
        if (acc_a - PUBLISHED_MASSIVE_A0).abs() <= 1e-6 {
            format!(
                "arm A reproduces the published {PUBLISHED_MASSIVE_A0} exactly — the \
                 instrument is alive (the seat posture, selected knobs + fitted gate)"
            )
        } else {
            format!(
                "WARN: arm A reads {acc_a:.4} vs the published {PUBLISHED_MASSIVE_A0} — the \
                 instrument is NOT reproducing the anchor; investigate before reading the \
                 verdict"
            )
        }
    } else {
        "no published anchor for this suite — arm A is its own baseline".to_string()
    };

    let verdict = if lb95 > 0.0 && acc_b > acc_a {
        "V5 PASS — synthesis lifts the modelless row (paired LB95 > 0)"
    } else {
        "V5 FAIL — synthesis does not lift the modelless row (the corpus stays gold-only; \
         the artifact stays recorded)"
    };
    let (p50a, p99a, _) = percentile_us(&lat_a.durs_us);
    let (p50b, p99b, _) = percentile_us(&lat_b.durs_us);
    let mut flips: Vec<CorpusAbFlip> = per_label
        .into_iter()
        .map(|(label, f)| CorpusAbFlip {
            label,
            gained: f.gained,
            lost: f.lost,
        })
        .collect();
    flips.sort_by(|a, b| {
        let na = a.gained as i64 - a.lost as i64;
        let nb = b.gained as i64 - b.lost as i64;
        nb.cmp(&na).then(a.label.cmp(&b.label))
    });

    eprintln!(
        "  [corpus-ab {}] gold {acc_a:.4} → synth {acc_b:.4} · LB95 {lb95:+.4} · +{} synth \
         doc(s) · verdict: {verdict}",
        spec.name, extra_used,
    );
    #[cfg(feature = "probe_delta_ab")]
    if let Some(c) = &probe_cert {
        eprintln!(
            "  [corpus-ab {}] probe ΔV: {} discordant ({}W/{}L, {} tie) · v̂ {:.3} Wilson [{:.3},{:.3}] → {}{} · {} flip(s)",
            spec.name,
            c.wins + c.losses,
            c.wins,
            c.losses,
            c.ties,
            c.v_hat,
            c.wilson_lo,
            c.wilson_hi,
            match c.verdict {
                crate::harness::probe_delta::ProbeDeltaVerdict::MovesValue => "MOVES VALUE",
                crate::harness::probe_delta::ProbeDeltaVerdict::NoValue => "NO VALUE",
                crate::harness::probe_delta::ProbeDeltaVerdict::Undetermined => "UNDETERMINED",
            },
            c.settled_at_n
                .map(|n| format!(" (settled @ n={n})"))
                .unwrap_or_default(),
            c.flips.len(),
        );
    }
    let gate_rung = ood
        .rungs
        .iter()
        .find(|r| (r.dropout_p - ood.gate_rung_p).abs() < 1e-9)
        .expect("the ladder always carries the gate rung");
    eprintln!(
        "  [corpus-ab {}] abstention gate: KL {:.4} (≤ {:.2}) · abstain {:.3} → {:.3} · {}",
        spec.name,
        abstention.entropy_kl_gold_to_synth,
        echo_gates::KL_EPS,
        abstention.abstain_rate_gold,
        abstention.abstain_rate_synth,
        if abstention.pass { "PASS" } else { "FAIL" },
    );
    eprintln!(
        "  [corpus-ab {}] OOD ladder (word-dropout): {} · gate p={:.2} Δ {:+.4} LB95 {:+.4} · \
         retention {} · {}",
        spec.name,
        ood.rungs
            .iter()
            .map(|r| format!("p={:.2} Δ {:+.4}", r.dropout_p, r.delta))
            .collect::<Vec<_>>()
            .join(" · "),
        gate_rung.dropout_p,
        gate_rung.delta,
        gate_rung.delta_lb95,
        ood.retention_at_gate
            .map(|r| format!("{r:.2}"))
            .unwrap_or_else(|| "n/a".into()),
        ood.echo_verdict,
    );
    Ok(CorpusAbSuite {
        suite: spec.name.to_string(),
        synth_artifact: synth_path.display().to_string(),
        synth_rows_total,
        synth_rows_in_scope: synth_docs.len(),
        gold_corpus_docs: prepared.train.len(),
        synth_corpus_docs: prepared.train.len() + extra_used,
        acc_gold: acc_a,
        acc_synth: acc_b,
        delta_lb95: lb95,
        flips,
        b_only_cases,
        a_only_cases,
        synth_ok_cases,
        lat_p50_gold_us: p50a,
        lat_p99_gold_us: p99a,
        lat_p50_synth_us: p50b,
        lat_p99_synth_us: p99b,
        determinism_ok: lat_a.determinism_ok.unwrap_or(false)
            && lat_b.determinism_ok.unwrap_or(false),
        aliveness,
        verdict: verdict.to_string(),
        abstention,
        ood,
        #[cfg(feature = "probe_delta_ab")]
        probe: probe_cert,
        mix: mix_block,
        seconds: t_start.elapsed().as_secs_f64(),
    })
}

/// The A/B report markdown (the bin writes CORPUS_AB.md).
pub fn render_corpus_ab_markdown(out: &CorpusAbOutput) -> String {
    let mut s = String::new();
    s.push_str("# corpus A/B (Plan 426 T5 — the V5 gate)\n\n");
    s.push_str(&format!(
        "- cap {}/label · synth extra-cap {} · datasets {} · read @ {}\n- {}\n\n",
        out.meta.corpus_cap_per_label,
        out.meta.synth_extra_cap,
        out.meta.datasets_dir,
        &out.meta.git_sha[..8.min(out.meta.git_sha.len())],
        out.meta.read_rule,
    ));
    for v in &out.suites {
        s.push_str(&format!(
            "## {} — gold {:.4} → synth {:.4} · paired LB95 {:+.4}\n\n",
            v.suite, v.acc_gold, v.acc_synth, v.delta_lb95
        ));
        s.push_str(&format!("- {}\n- {}\n", v.aliveness, v.verdict));
        #[cfg(feature = "probe_delta_ab")]
        if let Some(c) = &v.probe {
            s.push_str(&format!(
                "- probe ΔV (Plan 621 T5.1): {} discordant ({}W/{}L, {} tie) · v̂ {:.3} Wilson [{:.3},{:.3}] → **{}**{} · {} flip(s)\n",
                c.wins + c.losses,
                c.wins,
                c.losses,
                c.ties,
                c.v_hat,
                c.wilson_lo,
                c.wilson_hi,
                match c.verdict {
                    crate::harness::probe_delta::ProbeDeltaVerdict::MovesValue => "MOVES VALUE",
                    crate::harness::probe_delta::ProbeDeltaVerdict::NoValue => "NO VALUE",
                    crate::harness::probe_delta::ProbeDeltaVerdict::Undetermined => "UNDETERMINED",
                },
                c.settled_at_n
                    .map(|n| format!(", settled @ n={n}"))
                    .unwrap_or_default(),
                c.flips.len(),
            ));
        }
        s.push_str(&format!(
            "- corpus: gold {} doc(s) → synth {} doc(s) ({} in scope of {} artifact rows)\n",
            v.gold_corpus_docs, v.synth_corpus_docs, v.synth_rows_in_scope, v.synth_rows_total
        ));
        s.push_str(&format!(
            "- latency: gold p50 {} µs / p99 {} µs · synth p50 {} µs / p99 {} µs (V6: the \
             lane stays sub-ms)\n",
            v.lat_p50_gold_us, v.lat_p99_gold_us, v.lat_p50_synth_us, v.lat_p99_synth_us
        ));
        s.push_str(&format!(
            "- flips: {} synth-only wins, {} gold-only wins · determinism {}\n\n",
            v.b_only_cases.len(),
            v.a_only_cases.len(),
            v.determinism_ok
        ));
        // Issue-064 gates — always rendered beside the V5 verdict.
        s.push_str(&format!(
            "## Issue-064 gates — abstention {} · OOD {}\n\n",
            if v.abstention.pass { "PASS" } else { "FAIL" },
            v.ood.echo_verdict,
        ));
        s.push_str(&format!(
            "- abstention-entropy: KL(gold‖synth) {:.4} · reverse {:.4} · abstain {:.4} → \
             {:.4}\n",
            v.abstention.entropy_kl_gold_to_synth,
            v.abstention.entropy_kl_synth_to_gold,
            v.abstention.abstain_rate_gold,
            v.abstention.abstain_rate_synth,
        ));
        s.push_str("- OOD word-dropout ladder:\n\n| p | acc gold | acc synth | Δ | paired LB95 |\n|---|---|---|---|---|\n");
        for r in &v.ood.rungs {
            s.push_str(&format!(
                "| {:.2}{} | {:.4} | {:.4} | {:+.4} | {:+.4} |\n",
                r.dropout_p,
                if (r.dropout_p - v.ood.gate_rung_p).abs() < 1e-9 {
                    " (gate)"
                } else {
                    ""
                },
                r.acc_gold,
                r.acc_synth,
                r.delta,
                r.delta_lb95
            ));
        }
        s.push_str(&format!(
            "\n- retention @ gate {:.2} · rule: {}\n- {}\n- {}\n\n",
            v.ood
                .retention_at_gate
                .map(|r| r.to_string())
                .unwrap_or_else(|| "n/a".into()),
            v.ood.rule,
            v.abstention.rule,
            if v.ood.echo_verdict == "echo-suspect" {
                "⚠ ECHO-SUSPECT: the clean V5 win does not survive word-dropout — recorded \
                 NEGATIVE per the issue's law; the lane dies"
            } else {
                "no echo verdict fired"
            },
        ));
        if !v.flips.is_empty() {
            s.push_str("| label | gained | lost | net |\n|---|---|---|---|\n");
            for f in &v.flips {
                s.push_str(&format!(
                    "| {} | {} | {} | {:+} |\n",
                    f.label,
                    f.gained,
                    f.lost,
                    f.gained as i64 - f.lost as i64
                ));
            }
            s.push('\n');
        }
        if !v.b_only_cases.is_empty() {
            s.push_str(&format!(
                "synth-only wins: {}\n\n",
                v.b_only_cases.join(", ")
            ));
        }
        if let Some(m) = &v.mix {
            s.push_str("## Issue-129 mix mode — arms, saturation, prequential curve, canaries\n\n");
            s.push_str("| arm | docs | synth share | acc | LB95 vs gold | LB95 vs synth |\n|---|---|---|---|---|---|\n");
            for a in &m.arms {
                s.push_str(&format!(
                    "| {} | {} | {:.2} | {:.4} | {:+.4} | {:+.4} |\n",
                    a.arm, a.corpus_docs, a.synth_share, a.acc, a.lb95_vs_gold, a.lb95_vs_synth
                ));
            }
            s.push_str(&format!("\n- **{}**\n", m.mix_law));
            s.push_str(&format!(
                "- saturation (T2): dominant label '{}' share {:.3} · majority baseline {:.4} vs engine {:.4} → {}\n",
                m.saturation.dominant_label,
                m.saturation.dominant_share,
                m.saturation.majority_acc,
                m.saturation.engine_acc,
                if m.saturation.saturated {
                    "**SATURATED** — the suite is uninformative for selection/allocation methods; no teacher budget on it"
                } else {
                    "not saturated"
                }
            ));
            s.push_str("- prequential growth (T3, report-only; probe = the frozen read, never-grown by construction — dataset-fixed membership):\n\n| f | +synth docs | acc | LB95 vs gold | marginal LB95 |\n|---|---|---|---|---|\n");
            for st in &m.prequential {
                s.push_str(&format!(
                    "| {:.2} | {} | {:.4} | {:+.4} | {:+.4} |\n",
                    st.append_frac, st.appended_docs, st.acc, st.lb95_vs_gold, st.marginal_lb95
                ));
            }
            s.push_str("\n- marginal LB95 = paired LB95 vs the previous stage (the corpus-append-as-update signal for gain-weighted budget allocation; stationary-surface ban: allocation surfaces only, never the decide path).\n");
            s.push_str(&format!("- canaries (T4): {}\n", m.canaries.verdict));
            s.push_str(&format!(
                "  - rep 4-gram {:.3} (ceiling {}) · dup share {:.3} (ceiling {}) · band outliers {:.3} (p5 floor) · novelty {:.3} · admissible (in-band ∧ novel) {:.3}\n\n",
                m.canaries.rep_rate_4gram,
                crate::harness::runner::canaries::REP_CEILING,
                m.canaries.dup_share,
                crate::harness::runner::canaries::DUP_SHARE_CEILING,
                m.canaries.band_outlier_share,
                m.canaries.novelty_share,
                m.canaries.admissible_share
            ));
        }
        if !v.a_only_cases.is_empty() {
            s.push_str(&format!(
                "gold-only wins: {}\n\n",
                v.a_only_cases.join(", ")
            ));
        }
    }
    s
}

// ── Issue 129 — the mix-mode engine (`--corpus-ab-mix`) ─────────────────

/// The mix composition law (deterministic, no RNG): per label, the cap
/// is split `synth_n = (f·cap).round()` synth + the rest gold — gold
/// docs first (pool order), synth after (artifact order); a short side
/// fills from the other (the cap still saturates where docs exist — the
/// Issue-039 fallback law is untouched, `specs_from_pool` owns it).
/// The count tables read the arm's OWN pool uncapped (the same
/// single-variable rule every arm carries: its drafter corpus AND its
/// tables are the one composition being measured).
fn mix_pool(
    gold_pool: &[TrainDoc],
    synth: &[TrainDoc],
    labels: &[String],
    cap: usize,
    f: f64,
) -> Vec<TrainDoc> {
    let synth_n = ((cap as f64) * f).round() as usize;
    let gold_n = cap.saturating_sub(synth_n);
    let mut pool = Vec::new();
    for label in labels {
        let mut n_gold = 0usize;
        for d in gold_pool.iter().filter(|d| d.label == *label) {
            if n_gold == gold_n {
                break;
            }
            pool.push(d.clone());
            n_gold += 1;
        }
        // The label's synth share: synth_n, PLUS whatever gold could not
        // fill (the short-gold fill law).
        let synth_want = synth_n + (gold_n - n_gold);
        pool.extend(
            synth
                .iter()
                .filter(|d| d.label == *label)
                .take(synth_want)
                .cloned(),
        );
    }
    pool
}

/// Per-label eval helper: build from an arbitrary pool + eval the frozen
/// read, returning per-case forced-pick correctness.
fn arm_eval<const N: usize>(
    pool: &[TrainDoc],
    labels: &[String],
    cap: usize,
    cfg: &crate::engine::EngineConfig,
    cases: &[SuiteCase],
    states: &[String],
) -> Result<(Vec<bool>, f64, usize), String> {
    #[cfg(feature = "nb_scope")]
    let nb_sets = (cfg.nb_scale > 0.0).then(|| super::nb_doc_sets(pool, labels, &[]));
    #[cfg(not(feature = "nb_scope"))]
    let nb_sets: Option<Vec<Vec<String>>> = None;
    let (specs, _fb) = super::specs_from_pool(pool, labels, cap, nb_sets);
    let mut eng = crate::engine::DecisionEngine::<N, EMBED_DIM>::build_specs(specs, cfg.clone())
        .map_err(|e| format!("engine build (mix arm): {e}"))?;
    let (eval, _lat) = eval_engine(&mut eng, cases, states, true)?;
    let ok: Vec<bool> = cases
        .iter()
        .enumerate()
        .map(|(ci, case)| eval.picks[ci][0] == case.gold[0].idx)
        .collect();
    let acc = ok.iter().filter(|&&b| b).count() as f64 / ok.len() as f64;
    Ok((ok, acc, pool.len()))
}

#[allow(clippy::too_many_arguments)]
fn mix_block_computed<const N: usize>(
    spec: &SuiteSpec,
    prepared: &super::Prepared,
    synth_docs: &[TrainDoc],
    cfg: &crate::engine::EngineConfig,
    cap: usize,
    extra_cap: usize,
    ok_a: &[bool],
    ok_b: &[bool],
    acc_a: f64,
) -> Result<CorpusAbMixBlock, String> {
    let labels = &prepared.labels;
    let cases = &prepared.suite.cases;
    let states = &prepared.state_strs;

    // T1 — the synth-alone arm (the paper's second single).
    let synth_pool: Vec<TrainDoc> = synth_docs.to_vec();
    let (ok_s, acc_s, s_docs) = arm_eval::<N>(&synth_pool, labels, cap, cfg, cases, states)?;

    // T1 — the mix arms.
    let gold_pool = &prepared.train;
    let mut arms = Vec::new();
    let mut mix_accs: Vec<(f64, Vec<bool>, f64)> = Vec::new(); // (f, ok, acc)
    for f in [0.25f64, 0.50] {
        let pool = mix_pool(gold_pool, synth_docs, labels, cap, f);
        let (ok_m, acc_m, m_docs) = arm_eval::<N>(&pool, labels, cap, cfg, cases, states)?;
        arms.push(CorpusAbArmRow {
            arm: format!("mix:{f}"),
            corpus_docs: m_docs,
            synth_share: f,
            acc: acc_m,
            lb95_vs_gold: paired_lb95(&ok_m, ok_a),
            lb95_vs_synth: paired_lb95(&ok_m, &ok_s),
        });
        mix_accs.push((f, ok_m, acc_m));
    }

    // The singles' rows close the arm table (gold's LB95 vs synth).
    arms.insert(
        0,
        CorpusAbArmRow {
            arm: "gold".into(),
            corpus_docs: gold_pool.len(),
            synth_share: 0.0,
            acc: acc_a,
            lb95_vs_gold: 0.0,
            lb95_vs_synth: paired_lb95(ok_a, &ok_s),
        },
    );
    arms.push(CorpusAbArmRow {
        arm: "synth".into(),
        corpus_docs: s_docs,
        synth_share: 1.0,
        acc: acc_s,
        lb95_vs_gold: paired_lb95(&ok_s, ok_a),
        lb95_vs_synth: 0.0,
    });

    // The paper-law read: at least one mix arm beats BOTH singles with
    // paired LB95 > 0. Report-only.
    let mix_law = if mix_accs
        .iter()
        .any(|(_, ok_m, _)| paired_lb95(ok_m, ok_a) > 0.0 && paired_lb95(ok_m, &ok_s) > 0.0)
    {
        "MIX-LAW PASS — a mix arm beats both singles (paired LB95 > 0; arXiv:2608.11746 §H.3)"
            .into()
    } else if mix_accs
        .iter()
        .any(|(_, ok_m, _)| paired_lb95(ok_m, ok_a) > 0.0)
    {
        "MIX-LAW TAILS — a mix arm beats gold but not synth-alone".into()
    } else {
        "MIX-LAW FAIL — no mix arm beats the gold single".into()
    };

    // T2 — the saturation pre-check on the frozen read (report-only; no
    // exit-code change).
    let mut label_counts: BTreeMap<String, usize> = BTreeMap::new();
    for case in cases {
        *label_counts.entry(gold_label_of_case(case)).or_default() += 1;
    }
    let (dominant_label, dominant_n) = label_counts
        .iter()
        .max_by_key(|(_, n)| *n)
        .map(|(l, n)| (l.clone(), *n))
        .unwrap_or_default();
    let majority_acc = if cases.is_empty() {
        0.0
    } else {
        dominant_n as f64 / cases.len() as f64
    };
    let saturated = majority_acc >= acc_a;
    let saturation = SaturationRow {
        dominant_label,
        dominant_share: majority_acc,
        majority_acc,
        engine_acc: acc_a,
        saturated,
    };
    if saturated {
        eprintln!(
            "  [corpus-ab {}] SATURATED — the dominant-label baseline ({majority_acc:.4}) \
             matches the engine ({acc_a:.4}); the suite is uninformative for selection/\
             allocation methods — no teacher budget on it",
            spec.name
        );
    }

    // T3 — the prequential growth curve: corpus append IS the update;
    // the probe is the frozen read (never-grown: dataset-fixed
    // membership, disclosed in the report). Stages scale the PER-LABEL
    // extra cap (arm B's `take(extra_cap)` per-label rule) so f = 1.0
    // IS arm B byte-equal (its eval is reused, one build saved). The
    // stage's doc list is the full in-scope artifact; the cap picks the
    // same per-label prefix arm B reads at f = 1.0.
    let mut prequential = Vec::new();
    let mut prev_ok: Vec<bool> = ok_a.to_vec();
    let mut prev_label = "gold".to_string();
    for &f in [0.25f64, 0.5, 0.75, 1.0].iter() {
        let extra_cap_stage = (((extra_cap as f64) * f).ceil()) as usize;
        let ok_stage: Vec<bool>;
        let acc_stage: f64;
        let appended: usize = synth_docs
            .iter()
            .filter(|d| labels.iter().any(|l| l == &d.label))
            .fold(BTreeMap::<&str, usize>::new(), |mut m, d| {
                let e = m.entry(d.label.as_str()).or_default();
                if *e < extra_cap_stage {
                    *e += 1;
                }
                m
            })
            .into_values()
            .sum();
        if f >= 1.0 {
            // Byte-equal reuse: the full-cap stage IS arm B.
            ok_stage = ok_b.to_vec();
            acc_stage = ok_b.iter().filter(|&&b| b).count() as f64 / ok_b.len() as f64;
        } else {
            let (mut specs, _fb, _used) = super::specs_corpus_extended(
                gold_pool,
                synth_docs,
                labels,
                cap,
                extra_cap_stage,
                None,
            );
            // Arm B's count-table rule, at the stage's own fraction: the
            // tables read train ∪ the full artifact (the capped take
            // happens in the spec builder; the tables stay the uncapped
            // union — the same rule arm B's tables read, so the stage's
            // ONLY variable is the drafter-corpus cap).
            #[cfg(feature = "nb_scope")]
            if cfg.nb_scale > 0.0 {
                let sets = super::nb_doc_sets(gold_pool, labels, synth_docs);
                for (idx, sp) in specs.iter_mut().enumerate() {
                    *sp = sp.clone().with_nb_docs(sets[idx].clone());
                }
            }
            #[cfg(not(feature = "nb_scope"))]
            let _ = &specs;
            let mut eng =
                crate::engine::DecisionEngine::<N, EMBED_DIM>::build_specs(specs, cfg.clone())
                    .map_err(|e| format!("engine build (prequential {f}): {e}"))?;
            let (eval, _) = eval_engine(&mut eng, cases, states, true)?;
            ok_stage = cases
                .iter()
                .enumerate()
                .map(|(ci, case)| eval.picks[ci][0] == case.gold[0].idx)
                .collect();
            acc_stage = ok_stage.iter().filter(|&&b| b).count() as f64 / ok_stage.len() as f64;
        }
        let lb_vs_gold = paired_lb95(&ok_stage, ok_a);
        let marginal = paired_lb95(&ok_stage, &prev_ok);
        eprintln!(
            "  [corpus-ab {}] prequential f={f} (extra-cap {extra_cap_stage}, +{appended} synth): \
             acc {acc_stage:.4} · LB95 vs {prev_label} {marginal:+.4}",
            spec.name
        );
        prequential.push(PrequentialStageRow {
            append_frac: f,
            appended_docs: appended,
            acc: acc_stage,
            lb95_vs_gold: lb_vs_gold,
            marginal_lb95: marginal,
        });
        prev_ok = ok_stage;
        prev_label = format!("stage f={f}");
    }

    // T4 — the collapse canaries over the loaded artifact (report rows;
    // the refusal arms are the flags a future generator lane reads).
    let gold_texts: Vec<String> = gold_pool.iter().map(|d| d.text.clone()).collect();
    let synth_texts: Vec<String> = synth_docs.iter().map(|d| d.text.clone()).collect();
    let canary_row = crate::harness::runner::canaries::canaries(&gold_texts, &synth_texts);
    eprintln!(
        "  [corpus-ab {}] canaries: {}",
        spec.name, canary_row.verdict
    );

    eprintln!(
        "  [corpus-ab {}] mix law: {mix_law} · gold {acc_a:.4} · synth {acc_s:.4} · \
         mix25 {:.4} · mix50 {:.4} · saturation: dominant '{}' {:.3} (majority {:.4} vs engine {:.4}){}",
        spec.name,
        mix_accs[0].2,
        mix_accs[1].2,
        saturation.dominant_label,
        saturation.dominant_share,
        majority_acc,
        acc_a,
        if saturated { " — SATURATED" } else { "" },
    );

    Ok(CorpusAbMixBlock {
        arms,
        mix_law,
        saturation,
        prequential,
        canaries: canary_row,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn td(label: &str, text: &str) -> TrainDoc {
        TrainDoc {
            label: label.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn arm_specs_gold_first_then_synth_beyond_the_cap() {
        let train = vec![
            td("L1", "gold a1"),
            td("L1", "gold a2"),
            td("L1", "gold a3"),
            td("L2", "gold b1"),
        ];
        let extra = vec![
            td("L1", "synth a4"),
            td("L1", "synth a5"),
            td("L1", "synth a6"),
            td("L9", "synth z9"),
        ];
        let labels = vec!["L1".to_string(), "L2".to_string()];
        let (specs, fb, used) = arm_specs(&train, &extra, &labels, 2, 2, None);
        assert_eq!(specs.len(), 2);
        assert_eq!(
            used, 2,
            "L1 takes 2 of its 3 extras (extra cap); L2/L9 take none"
        );
        // Inspect the built specs through the public name field.
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["L1", "L2"]);
        assert!(fb.is_empty());
    }

    #[test]
    fn arm_specs_fallback_law_matches_specs_from_pool() {
        let train = vec![td("L1", "gold a1")];
        let labels = vec!["L1".to_string(), "L_empty".to_string()];
        let (specs, fb, _used) = arm_specs(&train, &[], &labels, 48, 128, None);
        assert_eq!(fb, vec!["L_empty".to_string()], "the empty label discloses");
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn gold_label_reads_the_presented_keys() {
        let case = SuiteCase {
            id: "t".into(),
            state: serde_json::json!({"utterance": "x"}),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "intent".into(),
                kind: QKind::Choice,
                instructions: "i".into(),
                criteria: {
                    let mut m = serde_json::Map::new();
                    m.insert("beta".to_string(), serde_json::Value::String("b".into()));
                    m.insert("alpha".to_string(), serde_json::Value::String("a".into()));
                    serde_json::Value::Object(m)
                },
            }],
            gold: vec![crate::harness::suites::GoldAnswer {
                idx: 0,
                soft: vec![0.0; 2],
                gold_score: None,
            }],
        };
        // serde_json::Map preserves INSERTION order here (preserve_order):
        // beta inserted first → keys [beta, alpha] → idx 0 = "beta". The
        // veto path is order-agnostic (key-string accept), and gold_pos is
        // computed from the same map the lane builds (sorted insertion).
        assert_eq!(gold_label_of_case(&case), "beta");
    }

    #[test]
    fn mix_pool_splits_the_cap_gold_first_deterministically() {
        let train = vec![
            td("L1", "gold a1"),
            td("L1", "gold a2"),
            td("L1", "gold a3"),
            td("L1", "gold a4"),
            td("L2", "gold b1"),
            td("L2", "gold b2"),
        ];
        let synth = vec![
            td("L1", "synth a5"),
            td("L1", "synth a6"),
            td("L2", "synth b3"),
        ];
        let labels = vec!["L1".to_string(), "L2".to_string()];
        // cap 4, f = 0.5 → 2 synth + 2 gold per label.
        let pool = mix_pool(&train, &synth, &labels, 4, 0.5);
        let l1: Vec<&str> = pool
            .iter()
            .filter(|d| d.label == "L1")
            .map(|d| d.text.as_str())
            .collect();
        assert_eq!(l1, vec!["gold a1", "gold a2", "synth a5", "synth a6"]);
        let l2: Vec<&str> = pool
            .iter()
            .filter(|d| d.label == "L2")
            .map(|d| d.text.as_str())
            .collect();
        // L2 has 2 synth docs and asks for 2: gold 2 first then synth 2 →
        // 4 docs (gold_n = cap − synth_n = 2, synth_n = 2 — both sides
        // fill exactly).
        assert_eq!(l2, vec!["gold b1", "gold b2", "synth b3"]);
        assert_eq!(
            l2.len(),
            3,
            "synth is short for L2 — the pool takes what exists"
        );
    }

    #[test]
    fn mix_pool_short_gold_fills_from_synth() {
        let train = vec![td("L1", "gold a1")];
        let synth = vec![
            td("L1", "synth a2"),
            td("L1", "synth a3"),
            td("L1", "synth a4"),
        ];
        let labels = vec!["L1".to_string()];
        // cap 4, f = 0.25 → synth 1 + gold 3; gold has 1, so synth fills
        // 1 + 2 = 3 (the short-gold fill law) → 4 docs, 3 of them synth.
        let pool = mix_pool(&train, &synth, &labels, 4, 0.25);
        assert_eq!(pool.len(), 4);
        let synth_n = pool.iter().filter(|d| d.text.starts_with("synth")).count();
        assert_eq!(synth_n, 3);
    }

    #[test]
    fn mix_pool_zero_fraction_is_the_gold_pool() {
        let train = vec![td("L1", "gold a1"), td("L1", "gold a2")];
        let synth = vec![td("L1", "synth a3")];
        let labels = vec!["L1".to_string()];
        let pool = mix_pool(&train, &synth, &labels, 2, 0.0);
        let texts: Vec<&str> = pool.iter().map(|d| d.text.as_str()).collect();
        assert_eq!(texts, vec!["gold a1", "gold a2"]);
    }
}
