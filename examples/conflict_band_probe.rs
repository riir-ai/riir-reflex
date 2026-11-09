//! Issue 110 T1 — the CONFLICT-band falsification probe (MEASUREMENT ONLY,
//! not a gate). The issue's witness (b), literal: the top-k nearest corpus
//! references SPLIT across labels while the deployed DistanceGate PASSED
//! (the question was answered, not abstained). The band a typed CONFLICT
//! status could serve is exactly: answered ∧ refs-split ∧ refs-majority ≠
//! served pick — anything else is either already gated (ScoreGate/
//! DistanceGate own the abstentions) or benign tail noise (the majority
//! agrees with the pick).
//!
//! The probe rides the SERVED posture, not a re-derivation: it joins the
//! harness's own `--dump-items` run (per-question pick + calibrated-gate
//! verdict, join key `(case_id, q_idx)`) against [`prepare_seat`]'s
//! byte-identical eval slice, and mirrors the served corpora with the run's
//! recorded `corpus_cap.effective` per suite. The kNN reference set is the
//! capped per-label corpus pool — the engine's own drafter corpora.
//!
//! Independent reader: the reference-label MAJORITY over the top-k nearest
//! corpus docs (cosine on the shared hashed-bag embedding). The served pick
//! is the corpus-centroid argmax over domains — a different functional of
//! the same corpus, so the disagreement is a real two-reader conflict, not
//! the tautology the issue rejects (top-2 of one scored question).
//!
//! Run (after a `--dump-items` harness run):
//!   cargo run --release --features modelless --example conflict_band_probe \
//!     -- <run-dir> [suite,a,b ...]
//! default run-dir `.benchmarks/116_t3` is WRONG for this probe — pass the
//! items-carrying run dir explicitly (e.g. `.benchmarks/177_issue110_conflict_band`).

#![cfg(feature = "modelless")]

use riir_reflex::embed::{EMBED_DIM, Embedder};
use riir_reflex::harness::runner::seat::prepare_seat;
use riir_reflex::harness::suites::QKind;

/// The modelless dataset suites with corpus pools (the probe's default
/// population; `typed_decisions` is excluded — its 5q/case workflow shape
/// makes the option-key/label join a different witness; disclosed absence).
const DEFAULT_SUITES: &[&str] = &[
    "ag_news",
    "banking77",
    "emotion",
    "massive_intent_en",
    "prompt_injections",
    "sst5",
    "xnli_en",
];

/// k of the top-k reference split (the issue's "top-k nearest corpus
/// references"; 5 = small enough to stay local, large enough that a
/// unanimous 5 is a grounded reading).
const K: usize = 5;

struct ItemRow {
    case_id: String,
    q_idx: usize,
    pick: usize,
    ok: bool,
    gold: usize,
    abstain_cal: Option<bool>,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let run_dir = args
        .next()
        .expect("usage: conflict_band_probe <run-dir> [suites...] — pass the --dump-items run dir");
    let suites: Vec<String> = match args.next() {
        Some(csv) if !csv.is_empty() => csv.split(',').map(str::to_string).collect(),
        _ => DEFAULT_SUITES.iter().map(|s| s.to_string()).collect(),
    };

    // The served corpora's per-suite caps, from the run's own results.json
    // (CorpusCapInfo.effective — the harness recorded what it served).
    let results_path = std::path::Path::new(&run_dir).join("results.json");
    let results: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&results_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", results_path.display())),
    )
    .unwrap_or_else(|e| panic!("parse {}: {e}", results_path.display()));
    let cap_of = |suite: &str| -> usize {
        results
            .get("suites")
            .and_then(|s| s.as_array())
            .and_then(|arr| {
                arr.iter()
                    .find(|s| s.get("name").and_then(|n| n.as_str()) == Some(suite))
            })
            .and_then(|s| s.get("modelless"))
            .and_then(|m| m.get("corpus_cap"))
            .and_then(|c| c.get("effective"))
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| {
                panic!("results.json carries no modelless corpus_cap.effective for {suite}")
            }) as usize
    };

    let embedder = Embedder;
    let mut total_band = 0usize;
    let mut total_answered = 0usize;
    println!(
        "{:<20} {:>9} {:>9} {:>9} {:>9} {:>9} {:>8}",
        "suite", "answered", "unanim", "split==", "split!=", "band%", "cap"
    );
    for suite in &suites {
        let seat = prepare_seat(suite, std::path::Path::new(".raw/datasets"))
            .unwrap_or_else(|e| panic!("seat {suite}: {e}"));
        let cap = cap_of(suite);

        // The served corpora: capped per-label docs over the seat's pool
        // (the engine's own construction — per-label take(cap), self-doc
        // fallback for an empty label).
        let mut refs: Vec<(&str, [f32; EMBED_DIM])> = Vec::new();
        for label in &seat.labels {
            let mut docs: Vec<&String> = seat
                .train
                .iter()
                .filter(|d| &d.label == label)
                .map(|d| &d.text)
                .take(cap)
                .collect();
            if docs.is_empty() {
                docs.push(label);
            }
            for text in docs {
                let mut v = [0.0f32; EMBED_DIM];
                embedder.embed_into(text.as_bytes(), &mut v);
                refs.push((label.as_str(), v));
            }
        }

        // The items dump rows for this suite's modelless lane.
        let items_path = std::path::Path::new(&run_dir)
            .join("items")
            .join(format!("{suite}.jsonl"));
        let mut rows: Vec<ItemRow> = Vec::new();
        for line in std::fs::read_to_string(&items_path)
            .unwrap_or_else(|e| {
                panic!(
                    "read {}: {e} (run the harness with --dump-items)",
                    items_path.display()
                )
            })
            .lines()
        {
            let v: serde_json::Value = serde_json::from_str(line).expect("item row parse");
            if v.get("lane").and_then(|l| l.as_str()) != Some("modelless") {
                continue;
            }
            rows.push(ItemRow {
                case_id: v
                    .get("case_id")
                    .and_then(|c| c.as_str())
                    .expect("case_id")
                    .to_string(),
                q_idx: v.get("q_idx").and_then(|q| q.as_u64()).unwrap_or(0) as usize,
                pick: v.get("pick").and_then(|p| p.as_u64()).unwrap_or(0) as usize,
                ok: v.get("ok").and_then(|o| o.as_bool()).unwrap_or(false),
                gold: v.get("gold").and_then(|g| g.as_u64()).unwrap_or(0) as usize,
                abstain_cal: v.get("abstain_cal").and_then(|a| a.as_bool()),
            });
        }
        let mut by_key = std::collections::HashMap::with_capacity(rows.len());
        for r in rows {
            by_key.insert((r.case_id.clone(), r.q_idx), r);
        }

        // Walk the eval slice in served order; kNN per question.
        let (mut answered, mut unanim, mut split_agree, mut split_conflict, mut skipped) =
            (0usize, 0usize, 0usize, 0usize, 0usize);
        let mut weak_ref = 0usize;
        // In-band gold split: who is right where the two readers disagree?
        // A band where the served pick is overwhelmingly right is READER
        // WEAKNESS (the kNN majority is a worse reader), not conflict; a
        // band split near 50/50 is the genuine two-credible-readings
        // disagreement a CONFLICT status could name.
        let (mut band_pick_right, mut band_maj_right, mut band_both_right) =
            (0usize, 0usize, 0usize);
        for (case, state_str) in seat.suite.cases.iter().zip(&seat.state_strs) {
            for (qi, q) in case.questions.iter().enumerate() {
                // noul carries no option/label correspondence (its pick
                // space is the engine's internal [yes,no]) — disclosed skip.
                if q.kind == QKind::Noul || q.kind == QKind::Score {
                    skipped += 1;
                    continue;
                }
                let Some(row) = by_key.get(&(case.id.clone(), qi)) else {
                    skipped += 1;
                    continue;
                };
                // Witness pool: the deployed gate ANSWERED (DistanceGate +
                // ScoreGate passed on the calibrated scale).
                if row.abstain_cal != Some(false) {
                    continue;
                }
                answered += 1;
                // The modelless context: state + "\n" + prompt (the wire
                // criteria are None on this lane — no criteria line).
                let mut ctx = String::with_capacity(state_str.len() + q.instructions.len() + 1);
                ctx.push_str(state_str);
                ctx.push('\n');
                ctx.push_str(&q.instructions);
                let mut qv = [0.0f32; EMBED_DIM];
                embedder.embed_into(ctx.as_bytes(), &mut qv);

                // Top-k nearest references (dot = cosine; both sides unit).
                let mut best = [(f32::NEG_INFINITY, 0usize); K];
                for (ri, (_, rv)) in refs.iter().enumerate() {
                    let mut dot = 0.0f32;
                    for i in 0..EMBED_DIM {
                        dot += qv[i] * rv[i];
                    }
                    // First slot whose score the candidate beats = its
                    // insertion position (strict >: a tie at the boundary
                    // keeps the earlier doc — deterministic).
                    let mut slot = K;
                    for (si, (bs, _)) in best.iter().enumerate() {
                        if dot > *bs {
                            slot = si;
                            break;
                        }
                    }
                    if slot < K {
                        for si in ((slot + 1)..K).rev() {
                            best[si] = best[si - 1];
                        }
                        best[slot] = (dot, ri);
                    }
                }
                // Reference-quality guard (the 256-bucket hashed bag is a
                // COARSE space): when even the nearest reference is nearly
                // orthogonal to the question (cosine < 0.05), the top-k is
                // ranking noise, not a reading. Counted separately.
                if best[0].0 < 0.05 {
                    weak_ref += 1;
                    continue;
                }
                // Label majority over the top-k, FIRST-SEEN tie rule (a
                // deterministic fold: a later label needs a STRICTLY
                // greater count to take the majority).
                let mut counts: Vec<(&str, usize)> = Vec::with_capacity(K);
                for (_, ri) in &best {
                    let label = refs[*ri].0;
                    match counts.iter_mut().find(|(l, _)| *l == label) {
                        Some((_, c)) => *c += 1,
                        None => counts.push((label, 1)),
                    }
                }
                let mut maj = counts[0];
                for c in &counts[1..] {
                    if c.1 > maj.1 {
                        maj = *c;
                    }
                }
                let maj_label = maj.0;
                let unanimous = counts.len() == 1;
                let pick_label: &str = seat
                    .labels
                    .get(row.pick)
                    .map(String::as_str)
                    .unwrap_or_else(|| {
                        panic!("{suite}: pick {} outside the label universe", row.pick)
                    });
                if unanimous {
                    unanim += 1;
                } else if maj_label == pick_label {
                    split_agree += 1;
                } else {
                    split_conflict += 1;
                    // The band's gold split (the falsification read).
                    let pick_right = row.ok;
                    let maj_right =
                        row.gold < seat.labels.len() && seat.labels[row.gold] == maj_label;
                    if pick_right {
                        band_pick_right += 1;
                    }
                    if maj_right {
                        band_maj_right += 1;
                    }
                    if pick_right && maj_right {
                        band_both_right += 1;
                    }
                }
            }
        }
        let band_pct = if answered > 0 {
            100.0 * split_conflict as f64 / answered as f64
        } else {
            0.0
        };
        total_band += split_conflict;
        total_answered += answered;
        println!(
            "{:<20} {:>9} {:>9} {:>9} {:>9} {:>8.2} {:>8}",
            suite, answered, unanim, split_agree, split_conflict, band_pct, cap
        );
        if split_conflict > 0 {
            println!(
                "    {:>20}  band gold split: pick right {}/{} ({:.1}%) · majority right {}/{} ({:.1}%) · both right {}",
                "",
                band_pick_right,
                split_conflict,
                100.0 * band_pick_right as f64 / split_conflict as f64,
                band_maj_right,
                split_conflict,
                100.0 * band_maj_right as f64 / split_conflict as f64,
                band_both_right
            );
        }
        if weak_ref > 0 {
            println!(
                "    weak-reference guard: {weak_ref}/{ans} answered questions had a near-orthogonal nearest ref (cosine < 0.05) — excluded before the split read",
                weak_ref = weak_ref,
                ans = answered + weak_ref,
            );
        }
        let _ = skipped;
    }
    let total_pct = if total_answered > 0 {
        100.0 * total_band as f64 / total_answered as f64
    } else {
        0.0
    };
    println!(
        "TOTAL answered={total_answered} band={total_band} ({total_pct:.2}% of answered) — \
         the CONFLICT band the issue's witness (b) could serve"
    );
}
