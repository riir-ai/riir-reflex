//! Issue 115 T2 — the Div-vs-corpus-ab correlation cell (MEASUREMENT ONLY,
//! REPORT-ONLY; not a gate). The pre-registered question: "does low-Div
//! predict low marginal corpus-ab gain?" across the existing synth pools
//! (one frozen read each — the pools' OWN recorded corpus-ab deltas are
//! the frozen reads; this probe adds the Div scalar per pool and the
//! Spearman rank correlation across pools).
//!
//! Div_P^τ(μ) per [katgpt-rs Plan 634 / arXiv:2606.01861]: 1/‖μᵀP^τ‖²_{ℓ2(1/π)}
//! over the corpus kNN-similarity graph — the CONSUMED primitive is
//! katgpt-core's `SimGraph::div_tau` (feature `diffusion_diversity`; the
//! substrate-first law: consume, never re-derive — goat-audit watches
//! this edge). The probe builds the graph (kNN over the pool's embed
//! vectors, symmetric mutual-ish edges kept simple: each node joins its
//! k nearest, edge weight = cosine), μ = uniform (the pool-spread
//! reading: how far τ-step random-walk mass spreads from every node —
//! tight clusters return high mass quickly = LOW Div).
//!
//! Pool: each `.raw/corpus_synth*/…_synth.jsonl` artifact's synth rows
//! (the text field). Pools with a recorded corpus-ab frozen read join it
//! from the run's `corpus_ab.json`.
//!
//! Run: cargo run --release --features modelless --example synth_div_probe
//! (needs a katgpt-core built with diffusion_diversity — the reflex dep
//! forwards the feature when built with it; this example reads the
//! values through the katgpt_core API directly).
#![cfg(all(feature = "modelless", feature = "synth_div"))]

use katgpt_core::SimGraph;
use riir_reflex::embed::{EMBED_DIM, Embedder};

/// The pools with a recorded frozen corpus-ab read: (artifact, delta, lb95)
/// — the deltas are the `corpus_ab.json` records' `acc_synth − acc_gold`
/// (the marginal gain), transcribed ONCE here with their source dir named;
/// a pool without a recorded read is skipped (never estimated).
const POOLS: &[(&str, &str, f64, f64)] = &[
    (
        ".raw/corpus_synth",
        ".raw/corpus_synth/massive_intent_en_synth.jsonl",
        0.0333,
        0.0110,
    ),
    (
        ".raw/corpus_synth_bekko",
        ".raw/corpus_synth_bekko/massive_intent_en_synth.jsonl",
        0.0267,
        0.0062,
    ),
    (
        ".raw/corpus_synth_bekko400m",
        ".raw/corpus_synth_bekko400m/massive_intent_en_synth.jsonl",
        0.0300,
        0.0086,
    ),
    (
        ".raw/corpus_synth_dens50",
        ".raw/corpus_synth_dens50/massive_intent_en_synth.jsonl",
        0.0200,
        0.0016,
    ),
    (
        ".raw/corpus_synth_dens75",
        ".raw/corpus_synth_dens75/massive_intent_en_synth.jsonl",
        0.0233,
        0.0062,
    ),
    (
        ".raw/corpus_synth_learn104_p50read",
        ".raw/corpus_synth_learn104/p50/massive_intent_en_synth.jsonl",
        0.0033,
        -0.0080,
    ),
    (
        ".raw/corpus_synth_learn104_p75read",
        ".raw/corpus_synth_learn104/p75/massive_intent_en_synth.jsonl",
        0.0267,
        0.0062,
    ),
];

const KNN_K: usize = 8;
const TAU: usize = 2;

fn main() {
    let embedder = Embedder;
    println!(
        "{:<34} {:>6} {:>10} {:>10} {:>10}",
        "pool", "rows", "Div(tau)", "delta", "lb95"
    );
    let mut divs: Vec<(String, f64, f64, f64)> = Vec::new();
    for (name, artifact, delta, lb95) in POOLS {
        let text = match std::fs::read_to_string(artifact) {
            Ok(t) => t,
            Err(e) => {
                println!("{name}: artifact unreadable ({e}) — skipped, never estimated");
                continue;
            }
        };
        let mut texts: Vec<String> = Vec::new();
        for line in text.lines() {
            let v: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(t) = v.get("text").and_then(|t| t.as_str()) {
                texts.push(t.to_string());
            }
        }
        if texts.len() < KNN_K + 1 {
            println!("{name}: pool too small ({}) — skipped", texts.len());
            continue;
        }
        // Embed the pool once.
        let mut vecs: Vec<[f32; EMBED_DIM]> = Vec::with_capacity(texts.len());
        for t in &texts {
            let mut v = [0.0f32; EMBED_DIM];
            embedder.embed_into(t.as_bytes(), &mut v);
            vecs.push(v);
        }
        // kNN edges: each node joins its k nearest (symmetrized by
        // SimGraph::build's duplicate-summing contract — one-sided edges
        // refuse, so both directions are inserted).
        let n = vecs.len();
        let mut edges: Vec<(usize, usize, f32)> = Vec::new();
        for (i, vi) in vecs.iter().enumerate() {
            let mut best: Vec<(f32, usize)> = Vec::with_capacity(KNN_K + 1);
            for (j, vj) in vecs.iter().enumerate() {
                if i == j {
                    continue;
                }
                let d = dot(vi, vj);
                best.push((d, j));
                best.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                best.truncate(KNN_K);
            }
            for (d, j) in best {
                edges.push((i, j, d.max(0.0)));
                edges.push((j, i, d.max(0.0)));
            }
        }
        let g = SimGraph::build(n, &edges);
        // Uniform μ — the pool-spread reading.
        let mu = vec![1.0f32 / n as f32; n];
        let mut q = vec![0.0f32; n];
        let mut r = vec![0.0f32; n];
        let div = g.div_tau(&mu, TAU, &mut q, &mut r);
        println!(
            "{:<34} {:>6} {:>10.4} {:>10.4} {:>10.4}",
            name, n, div, delta, lb95
        );
        divs.push((name.to_string(), f64::from(div), *delta, *lb95));
    }
    if divs.len() >= 4 {
        let spearman_delta = spearman(divs.iter().map(|(_, d, delta, _)| (*d, *delta)));
        let spearman_lb = spearman(divs.iter().map(|(_, d, _, lb)| (*d, *lb)));
        println!(
            "Spearman rank corr: Div↔delta = {spearman_delta:+.3} · Div↔lb95 = {spearman_lb:+.3} \
             over {} pools (the pre-registered floor for T3 promotion: YES-direction ≥ 0.7 \
             — a YES means LOW Div predicts LOW gain, i.e. NEGATIVE corr)",
            divs.len()
        );
    } else {
        println!("fewer than 4 pools measured — no correlation emitted (never a thin-n claim)");
    }
}

fn dot(a: &[f32; EMBED_DIM], b: &[f32; EMBED_DIM]) -> f32 {
    let mut s = 0.0f32;
    for i in 0..EMBED_DIM {
        s += a[i] * b[i];
    }
    s
}

/// Spearman rank correlation with average ranks for ties.
fn spearman(pairs: impl ExactSizeIterator<Item = (f64, f64)>) -> f64 {
    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    for (x, y) in pairs {
        xs.push(x);
        ys.push(y);
    }
    let n = xs.len();
    if n < 3 {
        return f64::NAN;
    }
    let rx = ranks(&mut xs);
    let ry = ranks(&mut ys);
    let mx = rx.iter().sum::<f64>() / n as f64;
    let my = ry.iter().sum::<f64>() / n as f64;
    let mut num = 0.0;
    let mut dx2 = 0.0;
    let mut dy2 = 0.0;
    for i in 0..n {
        let dx = rx[i] - mx;
        let dy = ry[i] - my;
        num += dx * dy;
        dx2 += dx * dx;
        dy2 += dy * dy;
    }
    if dx2 <= 0.0 || dy2 <= 0.0 {
        return f64::NAN;
    }
    num / (dx2.sqrt() * dy2.sqrt())
}

fn ranks(v: &mut [f64]) -> Vec<f64> {
    let n = v.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && v[idx[j + 1]] == v[idx[i]] {
            j += 1;
        }
        let avg = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j {
            out[idx[k]] = avg;
        }
        i = j + 1;
    }
    out
}
