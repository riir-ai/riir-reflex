//! Issue 129 T4 — the collapse canaries (arXiv:2608.11746 §H.3 imports).
//!
//! The two EpiGen collapse modes become bounded, planted refusal arms for
//! any future learned-generator lane, measured TODAY as report-only rows
//! on the loaded synth artifact:
//!
//! - **Repetition ceiling** — the paper's level-only reward collapse
//!   (4-gram rep 0.47): the artifact's 4-gram repetition rate over the
//!   accepted docs must stay far below the ceiling, and the exact-dup
//!   doc share with it.
//! - **Frozen-scorer admission band** — the paper's buffer-less reward
//!   collapse (adversarial noise at frozen-scorer ppl 169.7): a scorer
//!   fitted ON THE GOLD POOL ONLY (never on synth — frozen by
//!   construction) bands the synth docs; a doc far below the gold band
//!   is noise the corpus must not admit.
//!
//! Admission law (the conjunction, load-bearing): a doc is admissible
//! only when BOTH in-band (similarity) AND not a near-duplicate
//! (novelty). Level-only admission (similarity alone) is the documented
//! NEGATIVE import — the paper showed level-only rewards collapse to
//! repetition, so similarity alone must never admit.

use std::collections::HashSet;

/// The paper's own collapse reading for the repetition canary (§H.3:
/// the repetition-collapse run measured 4-gram rep 0.47). A HEALTHY
/// artifact sits far below; the canary trips at/above the ceiling.
pub const REP_CEILING: f64 = 0.47;

/// A synth doc scoring below the gold pool's p5 band share that reads as
/// adversarial noise rather than grounded novelty (the paper's
/// buffer-less collapse was a whole-artifact pathology; a minority
/// share is the noise-trickle signature). The share needs a COUNT floor
/// beside it — a share over a tiny synth set is noise (the
/// percentile-support lesson: 1 outlier of 3 docs is not a collapse).
/// Trips when outliers ≥ max(share·n, 3).
pub const BAND_OUTLIER_SHARE: f64 = 0.10;
pub const BAND_OUTLIER_MIN_COUNT: usize = 3;

/// Exact-duplicate doc share above which the artifact is a repetition
/// collapse regardless of the n-gram rate (the cheapest forgery).
pub const DUP_SHARE_CEILING: f64 = 0.05;

/// One canary read over (gold pool, synth docs). Report-only today; the
/// refusal arms for a future generator lane are the `tripped` flags —
/// the caller refuses when one fires.
#[derive(Debug, serde::Serialize)]
pub struct CanaryRow {
    /// 4-gram repetition rate over the synth docs: 1 − unique/total
    /// over the concatenated doc 4-gram streams (the paper's metric).
    pub rep_rate_4gram: f64,
    /// Exact-duplicate synth doc share (normalized-text equality).
    pub dup_share: f64,
    /// The frozen gold-fitted scorer's band on the synth docs
    /// (p5/p50/p95 of per-doc mean log-prob per token).
    pub synth_p5: f64,
    pub synth_p50: f64,
    pub synth_p95: f64,
    /// The gold pool's own band (the scorer's calibration face).
    pub gold_p5: f64,
    pub gold_p95: f64,
    /// Share of synth docs below the gold p5 band.
    pub band_outlier_share: f64,
    /// Novelty share: synth docs whose normalized text is NOT already
    /// present in the gold pool.
    pub novelty_share: f64,
    pub rep_tripped: bool,
    pub dup_tripped: bool,
    pub band_tripped: bool,
    /// The conjunction disclosure: admissible = in-band ∧ novel.
    /// Level-only admission is the documented negative import.
    pub admissible_share: f64,
    pub verdict: String,
}

/// A frozen char-trigram membership scorer over the GOLD pool. The band
/// statistic = the share of a doc's unique char-trigrams present in the
/// fit set. GOLD DOCS ARE BANDED LEAVE-ONE-OUT (df-based: a trigram
/// found only in the scored doc itself does not count) — without LOO
/// every gold doc self-scores perfectly and the floor admits nothing
/// novel. Frozen = fitted on gold once, never updated.
pub struct FrozenBandScorer {
    trigram_df: std::collections::HashMap<u64, u32>,
}

fn hash_word(w: &str) -> u64 {
    // FNV-1a 64 — the workspace's tripwire hash (deterministic, dep-free).
    let mut h: u64 = 0xcbf29ce484222325;
    for b in w.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn words(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect()
}

fn norm_text(t: &str) -> String {
    words(t).join(" ").to_lowercase()
}

fn doc_trigrams(doc: &str) -> HashSet<u64> {
    let t = norm_text(doc);
    let bytes = t.as_bytes();
    if bytes.len() < 3 {
        return HashSet::new();
    }
    (0..=bytes.len() - 3)
        .map(|i| hash_word(&t[i..i + 3]))
        .collect()
}

impl FrozenBandScorer {
    /// Fit on the GOLD pool only (trigram doc-frequency map).
    pub fn fit(gold: &[String]) -> Self {
        let mut trigram_df: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
        for doc in gold {
            for g in doc_trigrams(doc) {
                *trigram_df.entry(g).or_default() += 1;
            }
        }
        Self { trigram_df }
    }

    /// Membership ratio under the frozen fit. `leave_one_out` scores a
    /// GOLD doc against the fit MINUS its own trigrams (df ≥ 2 only).
    pub fn score(&self, doc: &str, leave_one_out: bool) -> f64 {
        let tri = doc_trigrams(doc);
        if tri.is_empty() {
            return 0.0;
        }
        let min_df = if leave_one_out { 2 } else { 1 };
        let matched = tri
            .iter()
            .filter(|g| self.trigram_df.get(g).is_some_and(|d| *d >= min_df))
            .count();
        matched as f64 / tri.len() as f64
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (((sorted.len() as f64) * p) as usize).min(sorted.len() - 1);
    sorted[idx]
}

/// The canary read. `gold` = the gold pool docs (the scorer's fit set +
/// the novelty reference), `synth` = the artifact's accepted docs.
pub fn canaries(gold: &[String], synth: &[String]) -> CanaryRow {
    // 4-gram repetition over the synth stream.
    let mut total_4g = 0usize;
    let mut uniq_4g: HashSet<u64> = HashSet::new();
    for doc in synth {
        let ws = words(doc);
        if ws.len() < 4 {
            continue;
        }
        for w in ws.windows(4) {
            total_4g += 1;
            uniq_4g.insert(hash_word(&w.join(" ")));
        }
    }
    let rep_rate = if total_4g == 0 {
        0.0
    } else {
        1.0 - (uniq_4g.len() as f64 / total_4g as f64)
    };

    // Exact-dup + novelty shares (normalized text).
    let mut seen: HashSet<String> = HashSet::new();
    let mut dups = 0usize;
    for doc in synth {
        let n = norm_text(doc);
        if !seen.insert(n) {
            dups += 1;
        }
    }
    let dup_share = if synth.is_empty() {
        0.0
    } else {
        dups as f64 / synth.len() as f64
    };
    let gold_norm: HashSet<String> = gold.iter().map(|d| norm_text(d)).collect();
    let novel = synth
        .iter()
        .filter(|d| !gold_norm.contains(&norm_text(d)))
        .count();
    let novelty_share = if synth.is_empty() {
        0.0
    } else {
        novel as f64 / synth.len() as f64
    };

    // Frozen-scorer band: fit on gold; gold docs banded LEAVE-ONE-OUT
    // (df-based) so the floor reflects held-out gold, not self-hits.
    let scorer = FrozenBandScorer::fit(gold);
    let mut gold_scores: Vec<f64> = gold.iter().map(|d| scorer.score(d, true)).collect();
    gold_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let gold_p5 = percentile(&gold_scores, 0.05);
    let gold_p95 = percentile(&gold_scores, 0.95);
    let mut synth_scores: Vec<f64> = synth.iter().map(|d| scorer.score(d, false)).collect();
    synth_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let (synth_p5, synth_p50, synth_p95) = (
        percentile(&synth_scores, 0.05),
        percentile(&synth_scores, 0.5),
        percentile(&synth_scores, 0.95),
    );
    let outliers = synth_scores.iter().filter(|&&s| s < gold_p5).count();
    let band_outlier_share = if synth_scores.is_empty() {
        0.0
    } else {
        outliers as f64 / synth_scores.len() as f64
    };
    let admissible = synth
        .iter()
        .zip(synth_scores.iter())
        .filter(|(d, s)| **s >= gold_p5 && !gold_norm.contains(&norm_text(d)))
        .count();
    let admissible_share = if synth.is_empty() {
        0.0
    } else {
        admissible as f64 / synth.len() as f64
    };

    let rep_tripped = rep_rate >= REP_CEILING;
    let dup_tripped = dup_share >= DUP_SHARE_CEILING;
    let outlier_count = outliers;
    let band_tripped =
        band_outlier_share >= BAND_OUTLIER_SHARE && outlier_count >= BAND_OUTLIER_MIN_COUNT;
    let verdict = if rep_tripped || dup_tripped {
        format!(
            "COLLAPSE — repetition canary fired (rep {rep_rate:.3} ceiling {REP_CEILING}, dup {dup_share:.3})"
        )
    } else if band_tripped {
        format!(
            "NOISE — frozen-scorer band canary fired ({:.1}% of synth docs below the gold p5 band)",
            band_outlier_share * 100.0
        )
    } else {
        format!(
            "CLEAN — rep {rep_rate:.3} · dup {dup_share:.3} · band outliers {:.1}% · admissible {admissible_share:.3} (in-band ∧ novel)",
            band_outlier_share * 100.0
        )
    };

    CanaryRow {
        rep_rate_4gram: rep_rate,
        dup_share,
        synth_p5,
        synth_p50,
        synth_p95,
        gold_p5,
        gold_p95,
        band_outlier_share,
        novelty_share,
        rep_tripped,
        dup_tripped,
        band_tripped,
        admissible_share,
        verdict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_gold() -> Vec<String> {
        vec![
            "transfer money to my checking account please".into(),
            "set an alarm for seven thirty in the morning".into(),
            "what is the weather forecast for tomorrow".into(),
            "play some jazz music from the eighties".into(),
            "add milk and bread to the shopping list".into(),
            "remind me to call the dentist at noon".into(),
            "find a italian restaurant near my hotel".into(),
            "turn up the volume on the living room speaker".into(),
        ]
    }

    #[test]
    fn healthy_artifact_reads_clean() {
        let gold = healthy_gold();
        let synth = vec![
            "move five hundred dollars from savings into checking".into(),
            "wake me up at six with the radio alarm".into(),
            "is it going to rain during my walk later".into(),
        ];
        let c = canaries(&gold, &synth);
        assert!(!c.rep_tripped);
        assert!(!c.dup_tripped);
        assert!(!c.band_tripped);
        assert!(c.verdict.starts_with("CLEAN"));
        // Grounded-but-novel docs are admissible under the conjunction.
        assert!(c.admissible_share >= 0.99);
        assert!(c.novelty_share >= 0.99);
    }

    #[test]
    fn repetition_collapse_trips_the_rep_canary() {
        let gold = healthy_gold();
        // The paper's level-only collapse: near-identical outputs.
        let one = "the the the the the the the the alarm is set alarm is set";
        let synth: Vec<String> = std::iter::repeat_n(one.to_string(), 12).collect();
        let c = canaries(&gold, &synth);
        assert!(
            c.dup_tripped,
            "exact dups must trip: dup_share={}",
            c.dup_share
        );
        assert!(
            c.rep_tripped,
            "repeated 4-grams must trip: rep={}",
            c.rep_rate_4gram
        );
        assert!(c.verdict.starts_with("COLLAPSE"));
    }

    #[test]
    fn adversarial_noise_trips_the_band_canary() {
        let gold = healthy_gold();
        // The paper's buffer-less collapse: grounded-looking noise the
        // frozen gold scorer reads far below the gold band.
        let synth: Vec<String> = (0..20)
            .map(|i| format!("qxz{j} vibzla morplik {i} zqwtvbk flunxap qyzjtvk", j = i))
            .collect();
        let c = canaries(&gold, &synth);
        assert!(
            c.band_tripped,
            "noise must fall below the gold band: outliers={}",
            c.band_outlier_share
        );
        assert!(c.verdict.starts_with("NOISE"));
        // The conjunction: noise is out-of-band, so nothing is admissible
        // even though the docs are novel.
        assert!(c.admissible_share < 0.01);
    }

    #[test]
    fn similarity_alone_never_admits_a_duplicate() {
        // The load-bearing conjunction: a gold duplicate is IN-BAND but
        // NOT novel — level-only admission would take it; the
        // conjunction must refuse it.
        let gold = healthy_gold();
        let synth = vec!["set an alarm for seven thirty in the morning".into()];
        let c = canaries(&gold, &synth);
        assert!(
            !c.band_tripped,
            "in-band doc: outliers={}",
            c.band_outlier_share
        );
        assert!(
            c.admissible_share < 0.01,
            "duplicate must not be admissible"
        );
        assert!(c.novelty_share < 0.01);
    }

    #[test]
    fn scorer_is_frozen_to_gold_fit() {
        // The scorer's ordering is fixed by the gold fit: a grounded
        // English string shares the gold trigram structure; keyboard
        // noise does not.
        let gold = healthy_gold();
        let scorer = FrozenBandScorer::fit(&gold);
        let grounded = scorer.score("move money into my account this afternoon", false);
        let noise = scorer.score("qxz vibzla morplik zqwtvbk flunxap", false);
        assert!(
            grounded > noise,
            "grounded {grounded:.3} vs noise {noise:.3}"
        );
    }
}
