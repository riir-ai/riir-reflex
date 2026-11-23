//! Issue 121 A1 — the BM25 lexical floor scorer (report-only).
//!
//! The corpus-is-the-model engine has no STANDALONE lexical baseline: the
//! NB/ridge scorers (`nb_ridge`/`nb_scope`) are the engine's own
//! hashed-lexicon halves, so nothing asserts the engine beats plain keyword
//! search over its own corpus. This module is that floor — the WSP paper's
//! admission test for any authored/dense memory (katgpt-rs Research 679:
//! most written Transformers score BELOW plain BM25 over the writer's own
//! text; a memory scoring under BM25 is discarding the author's signal).
//!
//! Shape (the paper's `bm25/bm25.py` reference, adapted to the harness's
//! decision forms): whitespace tokenization with the engine's OWN word
//! rule ([`crate::embed::token`] — trim non-alphanumeric edges, ASCII
//! case-fold; the same normalization the hashed lexicon uses, so the
//! A-pre redundancy comparison is apples-to-apples), terms hashed to u64
//! by the house FNV-1a ([`crate::embed::fnv1a_word`]; u64 collisions are
//! possible and accepted — the floor is a baseline, not a retrieval
//! product). Ranking = the BM25 k1/b form with the standard idf
//! `ln(1 + (N − df + 0.5)/(df + 0.5))` (never negative), `k1 = 1.2`,
//! `b = 0.75` — pinned constants, disclosed in every row (no selection
//! ladder; a floor that tunes is not a floor).
//!
//! Decision rule (one query per QUESTION, retrieval → top-k train rows →
//! weighted label vote): the query is the case state text plus, on choice
//! questions, the option descriptions (the criteria VALUES — they carry
//! the discriminative words); the top-k retrieved train rows vote their
//! corpus labels with their BM25 scores. Choice questions pick the option
//! KEY with the highest vote (first-index tie — the Issue-940 tie law);
//! score questions map the vote-winning integer-label string to the level
//! index; noul questions pick the vote-winning label's universe index. A
//! question whose vote is empty picks 0 (the first option — disclosed in
//! the row as `empty_votes`; a floor that abstains by construction would
//! hide exactly the violations the lane exists to find).
//!
//! The decision rule presumes every question's option space IS the
//! suite's label universe (choice keys == labels; score levels and noul
//! pairs at the label universe's arity). [`floor_contract_violation`]
//! enforces exactly that before any row is computed (issue 125): a suite
//! outside it — typed_decisions, whose doc labels are workflow names
//! while its questions carry per-workflow schema keys, level counts and
//! noul pairs — is a typed n/a floor row, never a degenerate number and
//! never the gold-vs-arity panic the sweep first died on.
//!
//! REPORT-ONLY: no exit-code change, no promotion path (A3's kill bar
//! lives in the issue, not here). Zero new dependencies — reflex keeps
//! one foreign code-level dep (katgpt-core).

use std::collections::HashMap;

use crate::embed::{fnv1a_word, token};
use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};

/// BM25 `k1` (pinned; disclosed in every row).
pub const BM25_K1: f32 = 1.2;
/// BM25 `b` (pinned; disclosed in every row).
pub const BM25_B: f32 = 0.75;
/// The top-k retrieval cut for the label vote (pinned; disclosed).
pub const BM25_TOP_K: usize = 10;
/// The salt for the term-id hash (distinct from the embedder's salts —
/// the floor's term space is its own; sharing a salt would suggest a
/// shared vocabulary that does not exist).
const TERM_SALT: u64 = 0xb25_0000_0000_0001;

/// Whitespace tokens of `text`, normalized by the engine's word rule and
/// hashed to u64 term ids. Zero-alloc beyond the output vec.
fn terms_into(text: &str, out: &mut Vec<u64>) {
    out.clear();
    for raw in text.split(|c: char| c.is_ascii_whitespace()) {
        let t = token(raw.as_bytes());
        if t.is_empty() {
            continue;
        }
        out.push(fnv1a_word(t, TERM_SALT));
    }
}

/// The BM25 index over one suite's corpus pool (the same registry pool the
/// engine sees — [`crate::harness::suites::TrainDoc`] label/text pairs).
pub struct Bm25Index {
    /// term id → postings (doc index, in-doc term frequency).
    postings: HashMap<u64, Vec<(u32, u32)>>,
    /// Per-doc token count (the BM25 length norm).
    doc_lens: Vec<f32>,
    avgdl: f32,
    /// The corpus label of each doc (parallel to `doc_lens`).
    labels: Vec<u32>,
    /// The label universe the doc labels index into.
    label_names: Vec<String>,
}

impl Bm25Index {
    /// Build over the corpus pool. `label_names` must contain every label
    /// string the docs carry (a doc label not in the universe is SKIPPED
    /// with a count — never silently dropped into a wrong bucket).
    pub fn build(docs: &[crate::harness::suites::TrainDoc], label_names: &[String]) -> Self {
        let label_idx: HashMap<&str, u32> = label_names
            .iter()
            .enumerate()
            .map(|(i, l)| (l.as_str(), i as u32))
            .collect();
        let mut postings: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
        let mut doc_lens = Vec::with_capacity(docs.len());
        let mut labels = Vec::with_capacity(docs.len());
        let mut skipped = 0usize;
        let mut terms = Vec::new();
        for doc in docs {
            let Some(&li) = label_idx.get(doc.label.as_str()) else {
                skipped += 1;
                continue;
            };
            terms_into(&doc.text, &mut terms);
            let di = doc_lens.len() as u32;
            doc_lens.push(terms.len() as f32);
            labels.push(li);
            // In-doc term frequencies.
            let mut tf: HashMap<u64, u32> = HashMap::new();
            for t in &terms {
                *tf.entry(*t).or_insert(0) += 1;
            }
            for (t, f) in tf {
                postings.entry(t).or_default().push((di, f));
            }
        }
        if skipped > 0 {
            eprintln!("    bm25-floor: {skipped} doc(s) skipped — labels outside the universe");
        }
        let n = doc_lens.len();
        let avgdl = if n == 0 {
            0.0
        } else {
            doc_lens.iter().sum::<f32>() / n as f32
        };
        Self {
            postings,
            doc_lens,
            avgdl,
            labels,
            label_names: label_names.to_vec(),
        }
    }

    /// The BM25-scored top-k docs for one query text.
    #[must_use]
    pub fn search(&self, query: &str, k: usize) -> Vec<(u32, f32)> {
        let n_docs = self.doc_lens.len();
        if n_docs == 0 || self.avgdl <= 0.0 {
            return Vec::new();
        }
        let mut q_terms = Vec::new();
        terms_into(query, &mut q_terms);
        // Unique query terms (repeated query words count once — the
        // standard BM25 query form; the paper's own scorer does the same).
        q_terms.sort_unstable();
        q_terms.dedup();
        let mut scores: HashMap<u32, f32> = HashMap::new();
        for t in &q_terms {
            let Some(postings) = self.postings.get(t) else {
                continue;
            };
            let df = postings.len();
            // The standard idf: ln(1 + (N - df + 0.5)/(df + 0.5)) — always
            // positive (the plain form goes negative above df = N/2, which
            // would let a COMMON term veto a doc; the +1 log keeps it a
            // weight).
            let idf = ((n_docs as f32 - df as f32 + 0.5) / (df as f32 + 0.5) + 1.0).ln();
            for &(di, tf) in postings {
                let dl = self.doc_lens[di as usize];
                let norm = tf as f32 * (BM25_K1 + 1.0)
                    / (tf as f32 + BM25_K1 * (1.0 - BM25_B + BM25_B * dl / self.avgdl));
                *scores.entry(di).or_insert(0.0) += idf * norm;
            }
        }
        let mut rows: Vec<(u32, f32)> = scores.into_iter().collect();
        // Strict > so the earliest doc keeps ties — the deterministic
        // first-index tie law (the Issue-940 family; sort_unstable_by is
        // fine because the comparator is a total order on (score DESC,
        // doc ASC)).
        rows.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        rows.truncate(k);
        rows
    }

    /// The weighted label vote over the top-k docs for one query:
    /// `(label index, summed BM25 score)` pairs, sorted by weight DESC,
    /// label index ASC (the deterministic tie law).
    #[must_use]
    pub fn vote(&self, query: &str, k: usize) -> Vec<(u32, f32)> {
        let mut votes: HashMap<u32, f32> = HashMap::new();
        for (di, s) in self.search(query, k) {
            *votes.entry(self.labels[di as usize]).or_insert(0.0) += s;
        }
        let mut rows: Vec<(u32, f32)> = votes.into_iter().collect();
        rows.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        rows
    }

    /// The floor's pick for one question: the option index. `query` is the
    /// state text (+ choice-option descriptions, appended by the caller —
    /// this fn stays question-shape-blind); `options` names the allowed
    /// answers IN PICK-INDEX ORDER (choice: the criteria keys; score/noul:
    /// the label universe in index order). Empty vote → 0 (disclosed
    /// upstream, never hidden).
    #[must_use]
    pub fn pick(&self, query: &str, options: &[&str]) -> usize {
        let votes = self.vote(query, BM25_TOP_K);
        if votes.is_empty() {
            return 0;
        }
        let mut best: Option<(usize, f32)> = None;
        for (oi, name) in options.iter().enumerate() {
            let Some(w) = votes
                .iter()
                .find(|(li, _)| self.label_names[*li as usize] == **name)
                .map(|(_, s)| *s)
            else {
                continue;
            };
            // Strict > keeps the earliest option on ties (first-index law).
            if best.is_none_or(|(_, bw)| w > bw) {
                best = Some((oi, w));
            }
        }
        best.map_or(0, |(oi, _)| oi)
    }
}

/// The query text for one case-question pair: the state text plus, on
/// choice questions, the criteria VALUES (the option descriptions carry
/// the discriminative words; the KEYS are the vote targets, named by
/// [`question_options`]).
#[must_use]
pub fn question_query(case: &SuiteCase, q: &SuiteQuestion) -> String {
    let mut text = crate::pyjson::serialize_state(&case.state);
    if let Some(m) = q.criteria.as_object() {
        for v in m.values() {
            if let Some(s) = v.as_str() {
                text.push('\n');
                text.push_str(s);
            }
        }
    } else if let Some(a) = q.criteria.as_array() {
        for v in a {
            if let Some(s) = v.as_str() {
                text.push('\n');
                text.push_str(s);
            }
        }
    }
    text
}

/// The vote targets for one question IN PICK-INDEX ORDER: choice → the
/// criteria keys; score/noul → the label universe (the pick index is the
/// option/level index, which for the classification suites IS the label
/// index — the runner's own pick↔domain law). `None` = malformed criteria
/// (the caller skips the question, counted upstream).
///
/// The Score|Noul substitution is NOT unconditional: it is honest only
/// under [`floor_contract_violation`] — the question's true option space
/// must have the label universe's arity (a score question's criteria
/// levels ARE the label universe; a noul pair fits a 2-label universe),
/// or the substitution silently answers a question in the wrong space.
pub fn question_options<'a>(q: &'a SuiteQuestion, labels: &'a [String]) -> Option<Vec<&'a str>> {
    match q.kind {
        QKind::Choice => q
            .criteria
            .as_object()
            .map(|m| m.keys().map(String::as_str).collect()),
        QKind::Score | QKind::Noul => Some(labels.iter().map(String::as_str).collect()),
    }
}

/// The floor's contract over one suite, checked BEFORE any row is
/// computed: every question's option space must BE the suite's label
/// universe — the same arity, every choice key spelled as a doc label,
/// and every gold index inside. This is the only shape the label-vote
/// decision rule can answer ([`Bm25Index::pick`] maps option spellings
/// to doc-label votes), so a suite whose questions carry per-workflow
/// option spaces outside the doc-label universe — typed_decisions: 4
/// workflow labels vs per-schema choice keys, level counts and noul
/// pairs — would silently degenerate to pick-0 rows and, on a score gold
/// at/above the label count, panic inside `hard_metrics` (the Issue-125
/// sweep kill). Returns `Some(reason)` naming the first violation;
/// `None` = the floor may run, the Score|Noul substitution in
/// [`question_options`] is then the level/noul universe by arity, and
/// the classification-suite identity case is byte-identical to before.
#[must_use]
pub fn floor_contract_violation(cases: &[SuiteCase], labels: &[String]) -> Option<String> {
    let n = labels.len();
    if n == 0 {
        return Some("empty label universe".to_string());
    }
    for case in cases {
        for (qi, q) in case.questions.iter().enumerate() {
            let Some(gold) = case.gold.get(qi) else {
                return Some(format!(
                    "case '{}' question {qi} ({:?}): questions/gold desynced — no gold row",
                    case.id, q.kind
                ));
            };
            // The question's TRUE option-space arity — the engine's own
            // wire derivation (choice = criteria keys, score = criteria
            // levels, noul = the fixed [p_no, p_yes] pair).
            let arity = match q.kind {
                QKind::Choice => q.criteria.as_object().map(|m| m.len()),
                QKind::Score => q.criteria.as_array().map(|a| a.len()),
                QKind::Noul => Some(2),
            };
            let Some(arity) = arity else {
                return Some(format!(
                    "case '{}' question '{}' ({:?}): malformed criteria — no option space to \
                     contract against",
                    case.id, q.qid, q.kind
                ));
            };
            if arity != n {
                return Some(format!(
                    "case '{}' question '{}' ({:?}): option space of {arity} vs the doc-label \
                     universe of {n} — per-question option spaces outside the label universe \
                     are outside the label-vote contract",
                    case.id, q.qid, q.kind
                ));
            }
            if q.kind == QKind::Choice {
                let keys = q
                    .criteria
                    .as_object()
                    .expect("arity check proved the object")
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>();
                if let Some(k) = keys.iter().find(|k| !labels.iter().any(|l| l == *k)) {
                    return Some(format!(
                        "case '{}' question '{}' (choice): option key {k:?} is not a doc label \
                         — votes cannot decide it (picks would degenerate to 0)",
                        case.id, q.qid
                    ));
                }
            }
            if gold.idx >= n {
                return Some(format!(
                    "case '{}' question '{}' ({:?}): gold index {} outside the {n}-entry \
                     option space",
                    case.id, q.qid, q.kind, gold.idx
                ));
            }
        }
    }
    None
}

impl Bm25Index {
    /// The number of docs indexed (disclosure field).
    #[must_use]
    pub fn n_docs(&self) -> usize {
        self.doc_lens.len()
    }

    /// Whether the top-k vote for `query` is empty (no doc shares any
    /// query term) — the disclosed empty-vote class counts on it.
    #[must_use]
    pub fn vote_is_empty(&self, query: &str, k: usize) -> bool {
        self.vote(query, k).is_empty()
    }
}

/// The floor's answers over one suite: one forced pick per question, the
/// same `(gold, probs)` row shape [`crate::harness::metrics::hard_metrics`]
/// consumes (probs = 1.0 at the pick — the floor carries no calibration).
/// The questions and gold lists are parallel by [`SuiteCase`]'s own
/// contract, so the walk enumerates positions.
pub fn floor_rows(
    index: &Bm25Index,
    cases: &[SuiteCase],
    labels: &[String],
) -> Vec<(usize, Vec<f64>)> {
    let mut rows = Vec::new();
    for case in cases {
        for (qi, q) in case.questions.iter().enumerate() {
            let Some(options) = question_options(q, labels) else {
                continue;
            };
            if options.is_empty() {
                continue;
            }
            let query = question_query(case, q);
            let pick = index.pick(&query, &options);
            let arity = options.len();
            let mut probs = vec![0.0; arity];
            probs[pick.min(arity - 1)] = 1.0;
            rows.push((case.gold[qi].idx, probs));
        }
    }
    rows
}

// ── Issue 121 A2 — the small-n parity clause (issue 126's verdict) ─────
//
// A2's closing rule is strict `engine ≥ bm25`. On a small suite that
// rule is decided by a handful of rows: at n = 102 the paired 95% interval
// on (engine − bm25) spans ±9 pp, so a one-row deficit is indistinguishable
// from a one-row surplus (McNemar p = 1.0). The clause closes such a suite
// as FLOOR-PARITY — it retires the CORPUS-DEFECT finding and never counts
// as a PASS for any promotion or GOAT claim.

/// The parity clause applies only BELOW this many scored questions; at or
/// above it, strict `engine ≥ bm25` applies unchanged.
pub const PARITY_MAX_N: usize = 400;
/// The exact two-sided McNemar level a parity verdict must not fall under.
pub const PARITY_ALPHA: f64 = 0.05;
/// The largest point deficit (bm25 − engine, percentage points) a parity
/// verdict tolerates. The cap fires on its own: `p ≥ α` is absence of
/// evidence, and alone it would let an engine sit well under the floor.
pub const PARITY_MAX_DEFICIT_PP: f64 = 2.0;

/// A suite's A2 verdict on one two-run record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum FloorVerdict {
    /// `engine ≥ bm25` — the strict A2 rule.
    Pass,
    /// `engine < bm25`, inside the small-n parity clause: the CORPUS-DEFECT
    /// finding is retired; never a PASS.
    FloorParity,
    /// `engine < bm25` and the parity clause does not hold.
    CorpusDefect,
}

impl FloorVerdict {
    /// The label's kind (pinned by the gate test — a rename is a wire
    /// change on every floor row).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::FloorParity => "FLOOR-PARITY",
            Self::CorpusDefect => "CORPUS-DEFECT",
        }
    }

    /// The row label. A parity verdict carries its n: the claim is scoped to
    /// the instrument's resolution, never to the suite.
    #[must_use]
    pub fn label(self, n: usize) -> String {
        if self == Self::FloorParity {
            format!("{}@{n}", self.as_str())
        } else {
            self.as_str().to_string()
        }
    }
}

/// The paired engine-vs-floor read behind one A2 verdict.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct FloorParity {
    /// Scored questions (both lanes' rows, joined by position).
    pub n: usize,
    /// `b` — bm25 right, engine wrong.
    pub bm25_right_engine_wrong: usize,
    /// `c` — engine right, bm25 wrong.
    pub engine_right_bm25_wrong: usize,
    pub both_right: usize,
    pub both_wrong: usize,
    /// Exact two-sided McNemar: `min(1, 2·P(X ≤ min(b, c)))`,
    /// `X ~ Bin(b + c, ½)`; 1.0 when there is no discordant pair.
    pub mcnemar_p: f64,
    /// `engine − bm25` in percentage points (rows × 100 / n).
    pub delta_pp: f64,
    /// Wald 95% interval on `delta_pp`:
    /// `±1.96·√(b + c − (b − c)²/n)/n`.
    pub ci95_pp: [f64; 2],
    pub verdict: FloorVerdict,
}

/// The exact two-sided McNemar p-value on `b` vs `c` discordant pairs.
///
/// The conditional binomial is summed in log space — `C(m, i)` overflows
/// f64 once `m` reaches the hundreds, so the tail probability is a
/// log-sum-exp over a running `ln C` recurrence, never a ratio of
/// factorials.
#[must_use]
pub fn mcnemar_exact_two_sided(b: usize, c: usize) -> f64 {
    let m = b + c;
    let k = b.min(c);
    if m == 0 {
        return 1.0;
    }
    let ln2 = std::f64::consts::LN_2;
    let mut log_sum = f64::NEG_INFINITY; // log Σ_{i≤k} C(m, i)
    let mut ln_c = 0.0f64; // ln C(m, 0)
    for i in 0..=k {
        if i > 0 {
            // ln C(m, i) = ln C(m, i−1) + ln(m − i + 1) − ln i
            ln_c += ((m - i + 1) as f64).ln() - (i as f64).ln();
        }
        let term = ln_c - (m as f64) * ln2;
        // log-add: log(e^a + e^b) without underflow.
        log_sum = if log_sum == f64::NEG_INFINITY {
            term
        } else {
            let (hi, lo) = (log_sum.max(term), log_sum.min(term));
            hi + (lo - hi).exp().ln_1p()
        };
    }
    (2.0 * log_sum.exp()).min(1.0)
}

/// Join the engine's forced rows with the floor's rows (the SAME
/// `(gold, probs)` shape and question order [`floor_rows`] and
/// `Eval::forced_rows` produce) into the paired read + the A2 verdict.
///
/// The two lists must be the same length — a mis-joined read is refused
/// loud, never counted. Picks follow the first-index argmax of
/// `hard_metrics` ([`crate::harness::metrics::argmax_first`]), so `b`, `c`
/// and the two agreement cells always reconcile with the two lanes' own
/// accuracies.
pub fn floor_parity(
    engine_rows: &[(usize, Vec<f64>)],
    floor_rows: &[(usize, Vec<f64>)],
) -> Result<FloorParity, String> {
    if engine_rows.len() != floor_rows.len() {
        return Err(format!(
            "floor parity: engine {} row(s) vs floor {} row(s) — the lanes' question \
             orders diverged; refusing the join",
            engine_rows.len(),
            floor_rows.len()
        ));
    }
    let n = engine_rows.len();
    let mut b = 0usize;
    let mut c = 0usize;
    let mut both_right = 0usize;
    let mut both_wrong = 0usize;
    for ((eg, ep), (fg, fp)) in engine_rows.iter().zip(floor_rows) {
        if *eg != *fg {
            return Err(format!(
                "floor parity: gold {eg} vs {fg} at one position — the lanes scored \
                 different questions; refusing the join"
            ));
        }
        let e_ok = crate::harness::metrics::argmax_first(ep).0 == *eg;
        let f_ok = crate::harness::metrics::argmax_first(fp).0 == *fg;
        match (e_ok, f_ok) {
            (true, false) => c += 1,
            (false, true) => b += 1,
            (true, true) => both_right += 1,
            (false, false) => both_wrong += 1,
        }
    }
    if n == 0 {
        return Err("floor parity: no scored questions".to_string());
    }
    let p = mcnemar_exact_two_sided(b, c);
    let delta_pp = ((c as f64) - (b as f64)) * 100.0 / (n as f64);
    let half =
        1.96 * ((b + c) as f64 - ((c as f64 - b as f64).powi(2)) / (n as f64)).sqrt() * 100.0
            / (n as f64);
    let deficit_pp = -delta_pp;
    let verdict = if delta_pp >= 0.0 {
        FloorVerdict::Pass
    } else if n < PARITY_MAX_N && p >= PARITY_ALPHA && deficit_pp <= PARITY_MAX_DEFICIT_PP {
        FloorVerdict::FloorParity
    } else {
        FloorVerdict::CorpusDefect
    };
    Ok(FloorParity {
        n,
        bm25_right_engine_wrong: b,
        engine_right_bm25_wrong: c,
        both_right,
        both_wrong,
        mcnemar_p: p,
        delta_pp,
        ci95_pp: [delta_pp - half, delta_pp + half],
        verdict,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::suites::TrainDoc;

    fn docs() -> Vec<TrainDoc> {
        vec![
            TrainDoc {
                label: "billing".into(),
                text: "refund the charged invoice card payment duplicate".into(),
            },
            TrainDoc {
                label: "billing".into(),
                text: "charge appears twice on my card statement refund please".into(),
            },
            TrainDoc {
                label: "login".into(),
                text: "cannot sign in password reset email never arrives".into(),
            },
            TrainDoc {
                label: "login".into(),
                text: "account locked after too many password attempts".into(),
            },
        ]
    }

    #[test]
    fn votes_the_matching_label_above_the_other() {
        let labels = vec!["billing".to_string(), "login".to_string()];
        let idx = Bm25Index::build(&docs(), &labels);
        let votes = idx.vote("duplicate charge on my card refund", BM25_TOP_K);
        assert_eq!(votes.first().map(|(li, _)| li), Some(&0));
        let votes2 = idx.vote("password reset email never arrived", BM25_TOP_K);
        assert_eq!(votes2.first().map(|(li, _)| li), Some(&1));
    }

    #[test]
    fn pick_maps_choice_keys_and_keeps_first_index_ties() {
        let labels = vec!["billing".to_string(), "login".to_string()];
        let idx = Bm25Index::build(&docs(), &labels);
        let pick = idx.pick("duplicate charge refund", &["login", "billing"]);
        assert_eq!(pick, 1);
        // Zero-hit query → the disclosed 0 pick (never a hidden abstain).
        assert_eq!(idx.pick("zzzqqq", &["login", "billing"]), 0);
    }

    #[test]
    fn empty_index_is_a_clean_zero() {
        let labels = vec!["a".to_string()];
        let idx = Bm25Index::build(&[], &labels);
        assert!(idx.search("anything", 5).is_empty());
        assert_eq!(idx.pick("anything", &["a"]), 0);
    }

    #[test]
    fn determinism_same_query_same_vote() {
        let labels = vec!["billing".to_string(), "login".to_string()];
        let a = Bm25Index::build(&docs(), &labels);
        let b = Bm25Index::build(&docs(), &labels);
        assert_eq!(a.vote("card refund", 10), b.vote("card refund", 10));
    }

    // ── issue 125: the floor-contract check ────────────────────────────

    use crate::harness::suites::{GoldAnswer, SuiteCase, SuiteQuestion};
    use serde_json::json;

    fn q(qid: &str, kind: QKind, criteria: serde_json::Value) -> SuiteQuestion {
        SuiteQuestion {
            qid: qid.to_string(),
            kind,
            instructions: "answer".to_string(),
            criteria,
        }
    }

    fn g(idx: usize) -> GoldAnswer {
        GoldAnswer {
            idx,
            soft: Vec::new(),
            gold_score: None,
        }
    }

    fn one_case(qs: Vec<SuiteQuestion>, golds: Vec<GoldAnswer>) -> Vec<SuiteCase> {
        vec![SuiteCase {
            id: "c0".to_string(),
            state: json!({"text": "state"}),
            questions: qs,
            gold: golds,
        }]
    }

    #[test]
    fn contract_accepts_the_classification_shape() {
        // The standing classification suites: choice keys == the label
        // universe, gold inside — the floor's byte-identical case.
        let labels: Vec<String> = ["billing", "login"].iter().map(|s| s.to_string()).collect();
        let cases = one_case(
            vec![q(
                "topic",
                QKind::Choice,
                json!({"billing": serde_json::Value::Null, "login": serde_json::Value::Null}),
            )],
            vec![g(1)],
        );
        assert_eq!(floor_contract_violation(&cases, &labels), None);
    }

    #[test]
    fn contract_keeps_score_and_noul_at_the_universe_arity() {
        // sst5's shape: score levels ARE the (stringified) label universe;
        // s1mb_noul's shape: the noul pair fits the 2-label universe.
        let sst5_labels: Vec<String> = (0..5).map(|i| i.to_string()).collect();
        let cases = one_case(
            vec![q("s", QKind::Score, json!(["l0", "l1", "l2", "l3", "l4"]))],
            vec![g(3)],
        );
        assert_eq!(floor_contract_violation(&cases, &sst5_labels), None);

        let noul_labels: Vec<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
        let cases = one_case(
            vec![q("n", QKind::Noul, serde_json::Value::Null)],
            vec![g(1)],
        );
        assert_eq!(floor_contract_violation(&cases, &noul_labels), None);
    }

    #[test]
    fn contract_rejects_the_typed_decisions_choice_shape() {
        // Issue 125's actual kill: workflow doc labels vs per-workflow
        // schema choice keys — votes cannot decide them.
        let labels: Vec<String> = ["wf_a", "wf_b", "wf_c", "wf_d"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let cases = one_case(
            vec![q(
                "action",
                QKind::Choice,
                json!({"escalate": serde_json::Value::Null, "resolve": serde_json::Value::Null}),
            )],
            vec![g(0)],
        );
        let reason = floor_contract_violation(&cases, &labels)
            .expect("typed_decisions-shaped choice must violate");
        // The arity arm fires first here (2 keys vs 4 workflow labels);
        // the spelling arm is the same violation class at equal arity.
        assert!(
            reason.contains("option space of 2") || reason.contains("not a doc label"),
            "reason: {reason}"
        );
    }

    #[test]
    fn contract_rejects_choice_keys_outside_the_universe_at_equal_arity() {
        // The spelling arm alone: same key count as the universe, but a
        // key the doc labels never carry — votes cannot decide it.
        let labels: Vec<String> = ["wf_a", "wf_b", "wf_c", "wf_d"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let cases = one_case(
            vec![q(
                "action",
                QKind::Choice,
                json!({
                    "wf_a": serde_json::Value::Null,
                    "wf_b": serde_json::Value::Null,
                    "wf_c": serde_json::Value::Null,
                    "escalate": serde_json::Value::Null,
                }),
            )],
            vec![g(0)],
        );
        let reason =
            floor_contract_violation(&cases, &labels).expect("a non-label choice key must violate");
        assert!(reason.contains("not a doc label"), "reason: {reason}");
    }

    #[test]
    fn contract_rejects_an_option_space_outside_the_universe() {
        // The panic class: a 5-level score question over a 4-workflow
        // universe — gold 4 against 4 workflow probs.
        let labels: Vec<String> = ["wf_a", "wf_b", "wf_c", "wf_d"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let cases = one_case(
            vec![q(
                "severity",
                QKind::Score,
                json!(["none", "low", "medium", "high", "critical"]),
            )],
            vec![g(4)],
        );
        let reason = floor_contract_violation(&cases, &labels)
            .expect("5-level score over a 4-label universe must violate");
        assert!(reason.contains("option space of 5"), "reason: {reason}");
    }

    #[test]
    fn contract_rejects_a_gold_outside_the_space() {
        let labels: Vec<String> = (0..5).map(|i| i.to_string()).collect();
        let cases = one_case(
            vec![q("s", QKind::Score, json!(["l0", "l1", "l2", "l3", "l4"]))],
            vec![g(7)],
        );
        let reason = floor_contract_violation(&cases, &labels)
            .expect("gold 7 over a 5-entry space must violate");
        assert!(reason.contains("gold index 7"), "reason: {reason}");
    }

    #[test]
    fn contract_rejects_a_questions_gold_desync() {
        let labels: Vec<String> = ["billing", "login"].iter().map(|s| s.to_string()).collect();
        // Two questions, one gold row — the parallel-vector contract is
        // broken; the walk refuses loud instead of indexing blind.
        let cases = vec![SuiteCase {
            id: "c0".to_string(),
            state: json!({"text": "state"}),
            questions: vec![
                q(
                    "a",
                    QKind::Choice,
                    json!({"billing": serde_json::Value::Null, "login": serde_json::Value::Null}),
                ),
                q("b", QKind::Noul, serde_json::Value::Null),
            ],
            gold: vec![g(0)],
        }];
        let reason = floor_contract_violation(&cases, &labels).expect("desynced gold must violate");
        assert!(reason.contains("desynced"), "reason: {reason}");
    }

    // ── issue 121 A2: the small-n parity clause ─────────────────────────
    //
    // The vectors are the delegated verdict's own (session 060599b8,
    // round 2) — each one proves a different arm of the clause.

    /// The two lanes' rows for a contingency table: `b` rows only the floor
    /// gets right, `c` rows only the engine gets right, `bw` rows both get
    /// wrong, and the rest both get right.
    type Rows = Vec<(usize, Vec<f64>)>;
    fn rows_for(b: usize, c: usize, bw: usize, n: usize) -> (Rows, Rows) {
        assert!(b + c + bw <= n, "the table needs b + c + bw ≤ n");
        let ok = (0usize, vec![1.0, 0.0]);
        let bad = (0usize, vec![0.0, 1.0]);
        let mut engine = Vec::with_capacity(n);
        let mut floor = Vec::with_capacity(n);
        for i in 0..n {
            // Row i: [0, b) bm25-only-right, [b, b+c) engine-only-right,
            // [b+c, b+c+bw) both wrong, then both right.
            engine.push(if i < b || (b + c..b + c + bw).contains(&i) {
                bad.clone()
            } else {
                ok.clone()
            });
            floor.push(
                if (b..b + c).contains(&i) || (b + c..b + c + bw).contains(&i) {
                    bad.clone()
                } else {
                    ok.clone()
                },
            );
        }
        (engine, floor)
    }

    fn close(a: f64, want: f64, tol: f64) -> bool {
        (a - want).abs() <= tol
    }

    #[test]
    fn parity_the_closed_case_is_floor_parity_at_102() {
        // semantic_defects, sweep c80f8d28: engine 58/102, bm25 59/102 —
        // the full contingency table (48 both-right, 33 both-wrong).
        let (engine, floor) = rows_for(11, 10, 33, 102);
        let p = floor_parity(&engine, &floor).expect("same length, same golds");
        assert_eq!(p.verdict, FloorVerdict::FloorParity);
        assert_eq!(p.verdict.label(p.n), "FLOOR-PARITY@102");
        assert_eq!(
            (p.n, p.bm25_right_engine_wrong, p.engine_right_bm25_wrong),
            (102, 11, 10)
        );
        assert_eq!((p.both_right, p.both_wrong), (48, 33));
        assert!(close(p.mcnemar_p, 1.0, 1e-12), "p {}", p.mcnemar_p);
        assert!(close(p.delta_pp, -0.98, 0.01), "delta {}", p.delta_pp);
        // engine − bm25: the interval mirrors the floor's [−7.8, +9.8].
        assert!(close(p.ci95_pp[0], -9.8, 0.06), "lo {:?}", p.ci95_pp);
        assert!(close(p.ci95_pp[1], 7.8, 0.06), "hi {:?}", p.ci95_pp);
        // The table reconciles with the two lanes' own accuracies.
        let e_right = p.engine_right_bm25_wrong + p.both_right;
        let f_right = p.bm25_right_engine_wrong + p.both_right;
        assert_eq!((e_right, f_right), (58, 59));
        assert_eq!(e_right + p.bm25_right_engine_wrong + p.both_wrong, p.n);
    }

    #[test]
    fn parity_no_discordant_pair_is_p_one_and_nan_free() {
        // Both lanes identical on every row — the CI must not divide by zero.
        let (engine, floor) = rows_for(0, 0, 0, 102);
        let p = floor_parity(&engine, &floor).expect("same length, same golds");
        assert_eq!(mcnemar_exact_two_sided(0, 0), 1.0);
        assert_eq!(p.verdict, FloorVerdict::Pass);
        assert_eq!(p.ci95_pp, [0.0, 0.0]);
        // Identical and WRONG everywhere is still engine ≥ bm25 (a tie).
        let both_bad = vec![(0usize, vec![0.0, 1.0]); 102];
        let tie = floor_parity(&both_bad, &both_bad).expect("same length, same golds");
        assert_eq!(tie.verdict, FloorVerdict::Pass);
        assert_eq!((tie.both_right, tie.both_wrong), (0, 102));
    }

    #[test]
    fn parity_a_significant_deficit_is_a_corpus_defect() {
        // p ≈ 0.0215 < 0.05: the significance arm refuses on its own.
        let (engine, floor) = rows_for(9, 1, 33, 102);
        let p = floor_parity(&engine, &floor).expect("same length, same golds");
        assert_eq!(p.verdict, FloorVerdict::CorpusDefect);
        assert!(close(p.mcnemar_p, 0.0215, 0.0005), "p {}", p.mcnemar_p);
    }

    #[test]
    fn parity_the_deficit_cap_fires_on_its_own() {
        // p = 0.25 ≥ 0.05, but the point deficit 3/102 = 2.94 pp > 2.0 pp:
        // the cap is the arm that refuses.
        let (engine, floor) = rows_for(3, 0, 33, 102);
        let p = floor_parity(&engine, &floor).expect("same length, same golds");
        assert!(close(p.mcnemar_p, 0.25, 1e-12), "p {}", p.mcnemar_p);
        assert!(close(p.delta_pp, -2.94, 0.01), "delta {}", p.delta_pp);
        assert_eq!(p.verdict, FloorVerdict::CorpusDefect);
    }

    #[test]
    fn parity_the_clause_does_not_apply_at_or_above_400() {
        // The same (11, 10) split at n = 400 and just above it: strict A2
        // applies — engine < bm25 is a CORPUS-DEFECT finding.
        for n in [400usize, 409] {
            let (engine, floor) = rows_for(11, 10, 0, n);
            let p = floor_parity(&engine, &floor).expect("same length, same golds");
            assert_eq!(p.n, n);
            assert_eq!(p.verdict, FloorVerdict::CorpusDefect, "n = {n}");
            assert_eq!(p.verdict.label(p.n), "CORPUS-DEFECT");
        }
    }

    #[test]
    fn parity_engine_above_the_floor_passes_whatever_p_is() {
        // The mirror of the significant deficit (9, 1): engine 9, bm25 1 —
        // the same p, the opposite sign, the strict PASS.
        let (engine, floor) = rows_for(1, 9, 33, 102);
        let p = floor_parity(&engine, &floor).expect("same length, same golds");
        assert!(close(p.mcnemar_p, 0.0215, 0.0005), "p {}", p.mcnemar_p);
        assert!(close(p.delta_pp, 7.84, 0.01), "delta {}", p.delta_pp);
        assert_eq!(p.verdict, FloorVerdict::Pass);
        assert_eq!(p.verdict.label(p.n), "PASS");
    }

    #[test]
    fn parity_refuses_a_misaligned_join_loud() {
        let (engine, floor) = rows_for(11, 10, 33, 102);
        let short = floor[..101].to_vec();
        let err = floor_parity(&engine, &short).expect_err("a length mismatch must refuse");
        assert!(err.contains("101"), "{err}");
        let mut shifted_gold = floor.clone();
        shifted_gold[50].0 = 1;
        let err = floor_parity(&engine, &shifted_gold).expect_err("a gold mismatch must refuse");
        assert!(err.contains("gold"), "{err}");
        let err = floor_parity(&[], &[]).expect_err("an empty join must refuse");
        assert!(err.contains("no scored questions"), "{err}");
    }

    #[test]
    fn parity_the_exact_tail_survives_large_discordant_counts() {
        // 300 discordant pairs: C(300, 150) ≈ 1e88 overflows nothing in log
        // space, and the split is even, so p is 1.
        assert!(close(mcnemar_exact_two_sided(150, 150), 1.0, 1e-12));
        // A lopsided 300: the tail is tiny but finite and ordered.
        let lopsided = mcnemar_exact_two_sided(240, 60);
        assert!((0.0..1.0).contains(&lopsided), "p {lopsided}");
        // Known small values: Bin(4, ½) tails.
        assert!(close(mcnemar_exact_two_sided(3, 1), 0.625, 1e-12));
        assert!(close(mcnemar_exact_two_sided(4, 0), 0.125, 1e-12));
        assert!(close(mcnemar_exact_two_sided(1, 1), 1.0, 1e-12));
    }

    #[test]
    fn parity_labels_are_pinned() {
        // A rename or reorder of the verdict strings is a wire change on
        // every floor row — it must be a deliberate edit here.
        assert_eq!(FloorVerdict::Pass.as_str(), "PASS");
        assert_eq!(FloorVerdict::FloorParity.as_str(), "FLOOR-PARITY");
        assert_eq!(FloorVerdict::CorpusDefect.as_str(), "CORPUS-DEFECT");
        assert_eq!(FloorVerdict::Pass.label(102), "PASS");
        assert_eq!(FloorVerdict::CorpusDefect.label(102), "CORPUS-DEFECT");
        assert_eq!(
            (PARITY_MAX_N, PARITY_ALPHA, PARITY_MAX_DEFICIT_PP),
            (400, 0.05, 2.0)
        );
    }
}
