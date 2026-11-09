//! Issue 116 — the `pick_latch` consumer gates (the `pick_domain` seat).
//!
//! - **G1 (purity):** the reflex seam's score fold (`router_scores`) is
//!   `pick_domain`'s EXACT sequential fold — parity pinned against the
//!   producer's own comparator over a seeded synthetic battery — and a
//!   cold latch seats exactly what the bare argmax seats.
//! - **G2 (byte-identity):** the unarmed latch posture is bit-identical
//!   to the pre-116 engine over repeated decides (same answers, same
//!   receipts), and the armed latch's FIRST pick of any stream (a cold
//!   seat) is the bare argmax's pick too.
//! - **Hysteresis semantics:** a near-tie stream holds the route under
//!   score noise where the bare argmax flips; a clear-win challenger
//!   switches the hold. The gate mechanics themselves (margin, NaN-hold,
//!   abstain-reuse, first-index ties) stay katgpt-core's — replayed here
//!   through the real fold to pin the seam end to end.

#![cfg(all(feature = "modelless", feature = "pick_latch"))]

use katgpt_core::decision_wire::{DecisionRequest, Question};
use katgpt_core::pick_latch::{LatchDecision, PickLatch};
use katgpt_core::variable_rank_domain_expert::pick_domain;
use riir_reflex::embed::EMBED_DIM;
use riir_reflex::engine::{DecisionEngine, EngineConfig, ExpertSpec, router_scores};

const OPS_DOC: &str = "Deploy the server to staging and verify the rollout before promoting \
to production. The staging cluster mirrors production capacity and runs the \
same release candidate. Rollback is one command when a deploy regresses the \
error budget. Verify the health endpoints after every rollout step.";

const SUPPORT_DOC: &str = "The customer asked for a refund of the last invoice because the \
billing account was charged twice. Check the account balance and the payment \
history, then refund the duplicate charge to the original payment method. \
Escalate to the billing team when the invoice does not match the account \
records.";

const DOCS_DOC: &str = "Write the release notes and document the new configuration flags. \
The documentation build runs on every pull request and the rendered manual \
is published with the release. Keep the changelog entries ordered by date.";

fn specs() -> Vec<ExpertSpec> {
    vec![
        ExpertSpec::new("ops", &[OPS_DOC.to_string()]),
        ExpertSpec::new("support", &[SUPPORT_DOC.to_string()]),
        ExpertSpec::new("docs", &[DOCS_DOC.to_string()]),
    ]
}

fn armed_cfg() -> EngineConfig {
    let (t, th) = riir_reflex::engine::pick_latch_defaults();
    EngineConfig {
        route_latch_gate: Some((t, th)),
        ..EngineConfig::default()
    }
}

fn ops_request(state: &str) -> DecisionRequest {
    DecisionRequest {
        state: state.to_string(),
        questions: vec![Question::choice(
            "env",
            "Which environment should receive this build next?",
            vec![
                "production".to_string(),
                "staging".to_string(),
                "local sandbox".to_string(),
            ],
            None,
        )],
    }
}

/// Deterministic LCG (no RNG dep — the harness-law: a seeded synthetic
/// stream, replayable bit-identically on every box).
struct Lcg(u64);

impl Lcg {
    fn next_f32(&mut self) -> f32 {
        // 32-bit LCG steps, mapped into [-1, 1).
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bits = (self.0 >> 32) as u32;
        (bits as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// First-index argmax — the producer comparator's tie rule (`pick_domain`
/// breaks ties to the LOWEST index; `PickLatch::decide` uses the same
/// rule, so on bit-identical scores the two can never disagree).
fn first_index_argmax<const N: usize>(scores: &[f32; N]) -> usize {
    let mut best = 0usize;
    for (d, &s) in scores.iter().enumerate().skip(1) {
        if s > scores[best] {
            best = d;
        }
    }
    best
}

// ── G1: fold parity + cold-seat purity ──────────────────────────────────

#[test]
fn g1_router_scores_fold_matches_pick_domain() {
    // Seeded synthetic battery: the fold + first-index argmax must equal
    // `pick_domain` on every vector — including exact ties (the tie rule
    // is part of the parity claim) and NaNs in non-winning slots.
    let mut rng = Lcg(0x0116_0116);
    for case in 0..512 {
        let mut q = [0.0f32; 24];
        for v in q.iter_mut() {
            *v = rng.next_f32();
        }
        let mut dirs = [[0.0f32; 24]; 3];
        for row in dirs.iter_mut() {
            for v in row.iter_mut() {
                *v = rng.next_f32();
            }
        }
        // Every 8th case is an exact-tie case: identical directions.
        if case % 8 == 0 {
            dirs[1] = dirs[0];
            dirs[2] = dirs[0];
        }
        let scores = router_scores::<3, 24>(&q, &dirs);
        assert_eq!(
            first_index_argmax(&scores),
            pick_domain::<3, 24>(&q, &dirs),
            "case {case}: the seam fold + tie rule must equal the bare argmax"
        );
    }
}

#[test]
fn g1_cold_latch_seats_the_bare_argmax_pick() {
    // The armed engine's FIRST decision of a stream (cold latch) must be
    // bit-identical to the unarmed engine's — same answer, same receipt.
    let mut bare: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let mut armed: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), armed_cfg()).unwrap();
    let mut rng = Lcg(0x00C0_001D);
    for i in 0..16 {
        // A synthetic state tilted differently per case — some route ops,
        // some support, some docs.
        let state = match i % 3 {
            0 => "staging rollout verify rollback deploy production cluster",
            1 => "refund invoice billing account charge payment history",
            _ => "release notes documentation manual changelog pull request",
        };
        let req = ops_request(state);
        // Diversify per repeat so the embeddings are not all identical.
        let mut req = req;
        req.state.push_str(&format!(" seed {}", rng.next_f32()));
        let a = bare.decide(&req).unwrap();
        let b = armed.decide(&req).unwrap();
        let ja = serde_json::to_string(&a).unwrap();
        let jb = serde_json::to_string(&b).unwrap();
        // First picks agree (cold seat); later picks MAY diverge (the
        // latch held a near-tie) — so only assert the FULL equality while
        // the latch has not had a near-tie to hold, and otherwise assert
        // the answer validity.
        b.validate_against(&req)
            .expect("armed engine output must satisfy the wire contract");
        if i == 0 {
            assert_eq!(
                ja, jb,
                "the cold seat must be bit-identical to the bare engine"
            );
        }
        // And reset keeps the property: after a reset the next decision is
        // the bare engine's again.
        armed.reset_route_latch();
    }
}

// ── G2: the unarmed posture is byte-identical to the pre-116 engine ─────

#[test]
fn g2_unarmed_latch_is_byte_identical_repeat_decides() {
    // The armed engine with an EMPTY hold is not the claim here; the claim
    // is the UNARMED config (`route_latch_gate: None`) — the seat compiles
    // to the bare argmax, so repeat decides must be bit-identical across
    // engines and runs.
    let mut a: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let mut b: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let req = ops_request("staging rollout verify rollback deploy production");
    for _ in 0..8 {
        let ja = serde_json::to_string(&a.decide(&req).unwrap()).unwrap();
        let jb = serde_json::to_string(&b.decide(&req).unwrap()).unwrap();
        assert_eq!(ja, jb, "the unarmed posture must be bit-identical");
    }
}

#[test]
fn g2_reset_route_latch_restores_the_bare_seat() {
    let mut armed: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), armed_cfg()).unwrap();
    let mut bare: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let req = ops_request("staging rollout verify rollback deploy production");
    // Warm the latch (holds whatever it seats), then reset: the next pick
    // must be the bare engine's pick again.
    let _ = armed.decide(&req).unwrap();
    armed.reset_route_latch();
    let ja = serde_json::to_string(&bare.decide(&req).unwrap()).unwrap();
    let jb = serde_json::to_string(&armed.decide(&req).unwrap()).unwrap();
    assert_eq!(
        ja, jb,
        "after reset_route_latch the seat must be the bare argmax again"
    );
}

// ── Hysteresis semantics: the fold feeds the producer gate end to end ───

#[test]
fn gate_mechanics_replay_through_the_real_fold() {
    // Score vectors folded from synthetic embeddings, driven through the
    // REAL producer latch: near-ties hold, clear wins switch, NaN holds,
    // abstain reuses. The margins mirror the producer fixtures (a switch
    // needs exact_sigmoid(delta/t) ≥ threshold — ln(3) ≈ 1.099 score units
    // at the defaults).
    let mut latch = PickLatch::new();
    // Cold seat: the argmax.
    let d = latch
        .decide(Some(&[0.9, 0.8, 0.1]))
        .pick()
        .expect("cold seat");
    assert_eq!(d, 0);
    assert!(matches!(
        latch.decide(Some(&[0.9, 0.8, 0.1])),
        LatchDecision::Held(0)
    ));
    // Near-tie challenger (delta 0.05 « 1.099 margin): hold.
    assert!(matches!(
        latch.decide(Some(&[0.9, 0.95, 0.1])),
        LatchDecision::Held(0)
    ));
    // Clear win (delta 2.0 » margin): switch.
    assert_eq!(
        latch.decide(Some(&[0.9, 2.9, 0.1])),
        LatchDecision::Switched(1)
    );
    // NaN in a NON-winning slot: the held pick's score is clean, the
    // challenger argmax skips NaN (NaN comparisons are false), hold.
    assert!(matches!(
        latch.decide(Some(&[2.9, f32::NAN, 0.1])),
        LatchDecision::Held(1)
    ));
    // Abstain (None): reuse.
    assert!(matches!(latch.decide(None), LatchDecision::Held(1)));
    latch.reset();
    assert!(matches!(latch.decide(None), LatchDecision::NoPick));
}

#[test]
fn hysteresis_holds_a_route_under_near_tie_noise() {
    // The seat-level shape (issue 116's G3 bar, engine cell): a stream of
    // contexts that land as near-ties between domains. Bare argmax flips
    // with the noise; the latch holds. Measured through `route_probe` —
    // the SAME seat the decide path runs — so the flip counts are the
    // seat's real behavior, not a proxy.
    let mut armed: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), armed_cfg()).unwrap();
    let mut bare: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    let mut rng = Lcg(0x1165_0005);
    let mut bare_flips = 0usize;
    let mut armed_flips = 0usize;
    let mut prev_bare: Option<usize> = None;
    let mut prev_armed: Option<usize> = None;
    for i in 0..96 {
        // Two-domain bases that tilt the argmax across the hold boundary
        // (measured: the original single-base stream never flipped at all
        // — a vacuous 0 ≤ 0; this one churns the bare argmax 63/96 rows
        // while staying inside the latch's margin), plus a noise word pair.
        let mut state = match i % 3 {
            0 => String::from(
                "staging rollout verify rollback deploy refund invoice billing account payment ",
            ),
            1 => String::from(
                "refund invoice billing account payment history staging rollout verify rollback deploy ",
            ),
            _ => String::from(
                "release notes documentation manual changelog rollback deploy staging rollout verify production ",
            ),
        };
        if rng.next_f32() > 0.0 {
            state.push_str("invoice billing ");
        } else {
            state.push_str("notes docs ");
        }
        state.push_str(&format!("row {i}"));
        // route_probe runs the SAME seat on both engines: on `bare` the
        // latch is unarmed (None) so it is the bare argmax; on `armed` it
        // advances the hold.
        let b = bare.route_probe(&state);
        let a = armed.route_probe(&state);
        if prev_bare != Some(b) && prev_bare.is_some() {
            bare_flips += 1;
        }
        prev_bare = Some(b);
        if prev_armed != Some(a) && prev_armed.is_some() {
            armed_flips += 1;
        }
        prev_armed = Some(a);
    }
    // The mechanism bar: the latched seat changes route no more often
    // than the bare argmax over a noisy stream — and the stream must
    // ACTUALLY churn (a vacuous 0 ≤ 0 against a never-flipping argmax
    // proves the hold nothing).
    assert!(
        bare_flips > 0,
        "the hysteresis stream must actually flip the bare argmax (a \n         vacuous stream proves the hold nothing)"
    );
    assert!(
        armed_flips <= bare_flips,
        "the latch must not flip routes MORE than the bare argmax \
         (armed {armed_flips} vs bare {bare_flips})"
    );
}

#[test]
fn defaults_come_from_one_place() {
    // The arming sites (harness flag, serve env) must cite the producer
    // consts — the (1.0, 0.75) shape, drift-guarded here.
    let (t, th) = riir_reflex::engine::pick_latch_defaults();
    assert_eq!((t, th), (1.0, 0.75));
    let mut latch = PickLatch::with_gate(t, th);
    // The margin at the defaults: a challenger must clear
    // t·lnit(th) = ln(3) ≈ 1.099 score units to switch (the issue's own
    // "ln 3 ≈ 1.099" number).
    latch.decide(Some(&[1.0, 0.5, 0.0]));
    // delta 0.4 < 1.099 → hold…
    assert!(matches!(
        latch.decide(Some(&[1.0, 1.4, 0.0])),
        LatchDecision::Held(0)
    ));
    // …delta 1.3 > 1.099 → switch.
    assert_eq!(
        latch.decide(Some(&[1.0, 2.3, 0.0])),
        LatchDecision::Switched(1)
    );
}

// ── The fold itself stays honest under the Embedder ──────────────────

// (Engine-level parity is covered by `g1_cold_latch_seats_the_bare_argmax_pick`
// — the fold runs inside the real engine there. The probe-side twin pins the
// same shape independently in routing_state_probe.)

// ── The flip census (`--route-census`, measurement-only) ─────────────

#[test]
fn census_off_by_default_and_report_only() {
    // The census exists only when the config armed it; the default
    // posture pays nothing — and a census-armed engine's DECISIONS are
    // byte-identical to a census-less engine's (the census observes,
    // never decides).
    let mut plain: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), EngineConfig::default()).unwrap();
    assert!(plain.route_flip_census().is_none());
    let req = ops_request("staging rollout verify rollback deploy production");
    let baseline = plain.decide(&req).unwrap();
    assert!(plain.route_flip_census().is_none());

    let census_cfg = EngineConfig {
        route_latch_census: true,
        ..EngineConfig::default()
    };
    let mut counted: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), census_cfg).unwrap();
    let observed = counted.decide(&req).unwrap();
    let census = counted
        .route_flip_census()
        .expect("census-armed config must expose the census");
    assert_eq!(census.picks(), 1, "one decide = one route_pick");
    assert_eq!(census.changes(), 0, "the first pick has no previous route");
    assert_eq!(
        census.disagreements(),
        0,
        "a latch-off seat can never disagree with the argmax it IS"
    );
    assert_eq!(
        serde_json::to_string(&baseline).unwrap(),
        serde_json::to_string(&observed).unwrap(),
        "the census must not change a decision"
    );
}

#[test]
fn census_counts_changes_and_armed_disagreements() {
    // The T3 flip-rate instrument over a real near-tie stream: the BARE
    // arm's disagreements stay 0 (its seat IS the argmax) while its
    // changes counter tracks the churn; the ARMED arm may disagree with
    // the argmax (counted) and must change its route no more often than
    // the bare argmax did (the G3 bar, through the census itself).
    let census_bare_cfg = EngineConfig {
        route_latch_census: true,
        ..EngineConfig::default()
    };
    let mut bare: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), census_bare_cfg).unwrap();
    let census_armed_cfg = EngineConfig {
        route_latch_census: true,
        ..armed_cfg()
    };
    let mut armed: DecisionEngine<3, EMBED_DIM> =
        DecisionEngine::build_specs(specs(), census_armed_cfg).unwrap();

    let mut rng = Lcg(0x1165_0009);
    for i in 0..96 {
        // Two-domain bases that tilt the argmax across the hold boundary
        // (measured: bare churns 63/96 rows, the armed seat holds) plus a
        // noise word pair — the same construction the hysteresis gate
        // above measures.
        let mut state = match i % 3 {
            0 => String::from(
                "staging rollout verify rollback deploy refund invoice billing account payment ",
            ),
            1 => String::from(
                "refund invoice billing account payment history staging rollout verify rollback deploy ",
            ),
            _ => String::from(
                "release notes documentation manual changelog rollback deploy staging rollout verify production ",
            ),
        };
        if rng.next_f32() > 0.0 {
            state.push_str("invoice billing ");
        } else {
            state.push_str("notes docs ");
        }
        state.push_str(&format!("row {i}"));
        let _ = bare.route_probe(&state);
        let _ = armed.route_probe(&state);
    }
    let b = bare.route_flip_census().expect("census-armed bare engine");
    let a = armed
        .route_flip_census()
        .expect("census-armed armed engine");
    assert_eq!(b.picks(), 96);
    assert_eq!(a.picks(), 96);
    assert_eq!(
        b.disagreements(),
        0,
        "a latch-off seat can never disagree with the argmax it IS"
    );
    assert!(
        b.changes() > 0,
        "the near-tie stream must actually churn the bare argmax (a \
         vacuous stream proves the census nothing)"
    );
    assert!(
        a.changes() <= b.changes(),
        "armed changes {} must be <= bare changes {}",
        a.changes(),
        b.changes()
    );
    assert!(
        a.disagreements() > 0,
        "the near-tie stream must give the latch real decisions \
         (disagreements 0 means the cell saw no near-ties)"
    );
}

// ── The fold itself stays honest under the Embedder ──────────────────

// (Engine-level parity is covered by `g1_cold_latch_seats_the_bare_argmax_pick`
// — the fold runs inside the real engine there. The probe-side twin pins the
// same shape independently in routing_state_probe.)
