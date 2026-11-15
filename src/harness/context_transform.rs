//! Issue 119 — the context-INVARIAANCE probe's pure core (additive input
//! transforms, measured once with a placebo control).
//!
//! Ref: riir-reflex `.research/017` (arXiv:2611.09385 "Against All
//! Likelihood"). The paper's sketch transforms a question's context
//! (perspective/framing), re-answers under each, and keeps the invariant
//! core. On THIS engine there is no semantic channel to ride — `src/embed.rs`
//! embeds ASCII-lowercased words + bigrams as FNV-1a-hashed buckets — so a
//! persona prefix is a CONSTANT ADDITIVE bucket set, the mirror of
//! `mc_ensemble`'s bucket mask (Plan 008 / Bench 092's null verdict). The
//! probe measures exactly what it can: whether ANY additive input transform
//! produces an agreement-structure abstention signal, and the PLACEBO arm
//! separates "additive hash noise" from anything invariance-specific.
//!
//! Measurement laws:
//! - **Fixed, name-stable tables.** Both families are compile-time tables —
//!   no RNG anywhere; the placebo twins are word-count-matched to their
//!   semantic twins so the additive bucket mass is comparable.
//! - **Identity first.** `transform[0]` of each family is the identity —
//!   byte-identical passthrough, the G1 control's anchor.
//! - **The state is the only touched surface.** The identity arm
//!   preserves the state value byte-identically (G1's anchor); the
//!   transform arms rewrite the SERIALIZED text channel
//!   ([`crate::pyjson::serialize_state`] — the one text the engine
//!   embeds), so object states transform exactly like string ones. The
//!   questions, option order and gold ride along untouched — so pick
//!   indexes compare across transforms with no label-space mapping (the
//!   perm probe needs one; this probe structurally does not).
//! - **Majority is strict.** The core exists only when a pick has MORE than
//!   half the votes (strict majority, first-appearance tie order); the
//!   unanimous class is disclosed beside it. No strict majority = the empty
//!   core — the candidate third abstention channel (G2b).

use serde::Serialize;

use crate::harness::suites::SuiteCase;
use crate::pyjson::serialize_state;

/// Which family a transform belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Family {
    /// Persona/framing/formatting variants — the "invariance" arm.
    Semantic,
    /// Word-count-matched nonsense prefixes — the additive-noise control.
    Placebo,
}

impl Family {
    /// The record's family spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Family::Semantic => "semantic",
            Family::Placebo => "placebo",
        }
    }
}

/// One additive input transform: a name, its family, and the string
/// rewrite (`state -> state'`). Deterministic and total on `&str`.
#[derive(Debug, Clone, Copy)]
pub struct ContextTransform {
    pub name: &'static str,
    pub family: Family,
    pub apply: fn(&str) -> String,
}

/// Identity passthrough — `[0]` of both families (the G1 anchor).
fn identity(s: &str) -> String {
    s.to_string()
}

fn prefix_linguist(s: &str) -> String {
    format!("From the perspective of a linguist, {s}")
}

fn prefix_economist(s: &str) -> String {
    format!("From the perspective of an economist, {s}")
}

fn prefix_lawyer(s: &str) -> String {
    format!("From the perspective of a lawyer, {s}")
}

fn prefix_physician(s: &str) -> String {
    format!("From the perspective of a physician, {s}")
}

fn suffix_consider(s: &str) -> String {
    format!("{s} Consider the context carefully.")
}

fn suffix_answer(s: &str) -> String {
    format!("{s} Answer from the full context given.")
}

/// The neutral formatting variant: collapse runs of whitespace to one
/// space and trim (no added tokens — a presentation-only variant).
fn normalize_space(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

// The placebo twins: word-count-matched nonsense prefixes/suffixes over a
// fixed lowercase-ASCII word table (every word hashes to buckets exactly
// like any English word — that IS the additive-noise mechanism under test).
// Counts: rows 2–5 = 6 words each (the persona twins), row 6 = 4 words, row
// 7 = 5 words (the suffix twins), row 8 = the formatting twin.

fn prefix_noise_a(s: &str) -> String {
    format!("zorble blicket wug dax fep korbl, {s}")
}

fn prefix_noise_b(s: &str) -> String {
    format!("Snorp vlim plonk trube gwark mestin, {s}")
}

fn prefix_noise_c(s: &str) -> String {
    format!("Oble rifkin quamp heldor snav eckb, {s}")
}

fn prefix_noise_d(s: &str) -> String {
    format!("Pralt vusk crindle bost nork flimm, {s}")
}

fn suffix_noise_a(s: &str) -> String {
    format!("{s} Zorble wug dax korbl.")
}

fn suffix_noise_b(s: &str) -> String {
    format!("{s} Snorp vlim trube gwark mestin orvex.")
}

/// The SEMANTIC family: `[0]` identity + 7 transforms (the K≤8 sweep fits
/// without extra decides — the K sweep is a prefix of ONE decision set).
pub const SEMANTIC_FAMILY: &[ContextTransform] = &[
    ContextTransform { name: "identity", family: Family::Semantic, apply: identity },
    ContextTransform { name: "persona_linguist", family: Family::Semantic, apply: prefix_linguist },
    ContextTransform { name: "persona_economist", family: Family::Semantic, apply: prefix_economist },
    ContextTransform { name: "persona_lawyer", family: Family::Semantic, apply: prefix_lawyer },
    ContextTransform {
        name: "persona_physician",
        family: Family::Semantic,
        apply: prefix_physician,
    },
    ContextTransform { name: "suffix_consider", family: Family::Semantic, apply: suffix_consider },
    ContextTransform { name: "suffix_answer", family: Family::Semantic, apply: suffix_answer },
    ContextTransform { name: "normalize_space", family: Family::Semantic, apply: normalize_space },
];

/// The PLACEBO family: index-aligned noise twins of [`SEMANTIC_FAMILY`]
/// (same word counts, zero semantic content — the additive-hash-noise
/// control the G2a verdict adjusts by).
pub const PLACEBO_FAMILY: &[ContextTransform] = &[
    ContextTransform { name: "identity", family: Family::Placebo, apply: identity },
    ContextTransform { name: "noise_prefix_a", family: Family::Placebo, apply: prefix_noise_a },
    ContextTransform { name: "noise_prefix_b", family: Family::Placebo, apply: prefix_noise_b },
    ContextTransform { name: "noise_prefix_c", family: Family::Placebo, apply: prefix_noise_c },
    ContextTransform { name: "noise_prefix_d", family: Family::Placebo, apply: prefix_noise_d },
    ContextTransform { name: "noise_suffix_a", family: Family::Placebo, apply: suffix_noise_a },
    ContextTransform { name: "noise_suffix_b", family: Family::Placebo, apply: suffix_noise_b },
    ContextTransform {
        name: "normalize_space",
        family: Family::Placebo,
        apply: normalize_space,
    },
];

/// Apply transform `rep` of `family` to a case. The identity arm (`[0]`,
/// the G1 anchor) preserves the state VALUE byte-identically — any JSON
/// type rides untouched. The transform arms rewrite the SERIALIZED text
/// channel (`crate::pyjson::serialize_state` — the one text the engine
/// embeds: identity on strings, `py_json` on everything else), so object
/// states transform exactly like string ones. Questions, option order and
/// gold ride untouched. The case id names the family transform so every
/// record row is traceable.
pub fn transformed_case(
    case: &SuiteCase,
    family: Family,
    rep: usize,
) -> Result<SuiteCase, String> {
    let table = match family {
        Family::Semantic => SEMANTIC_FAMILY,
        Family::Placebo => PLACEBO_FAMILY,
    };
    let t = table
        .get(rep)
        .ok_or_else(|| format!("rep {rep} out of the {} family table", family.as_str()))?;
    let state = if rep == 0 {
        // The identity arm: the state value untouched (G1 is exact).
        case.state.clone()
    } else {
        serde_json::Value::String((t.apply)(&serialize_state(&case.state)))
    };
    Ok(SuiteCase {
        id: format!("{}#ctx{}", case.id, t.name),
        state,
        questions: case.questions.clone(),
        gold: case.gold.clone(),
    })
}

/// The agreement structure over one case's K picks (option indexes —
/// compares directly: the transforms never permute options).
#[derive(Debug, Clone, Serialize)]
pub struct CaseAgreement {
    /// Top pick under each transform (index-aligned with the family table
    /// prefix; `[0]` is the identity's).
    pub picks: Vec<usize>,
    /// The seat: the strict-majority label index (`Some` under unanimous
    /// AND majority classes, `None` when the core is empty).
    pub seat: Option<usize>,
    /// The class: unanimous (all K agree), majority (a strict majority
    /// that is not unanimous), no_core (no label cleared K/2).
    pub class: CoreClass,
    /// Votes for the seat (`K` when unanimous).
    pub seat_votes: usize,
}

/// The core class over K picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreClass {
    Unanimous,
    Majority,
    NoCore,
}

/// Compute the agreement over the first `k` entries of `picks` (the K
/// sweep is a prefix of one decision set — no re-decides). Empty input
/// yields the empty core (the seat abstains — never a guess).
#[must_use]
pub fn agreement_of(picks: &[usize], k: usize) -> CaseAgreement {
    let k = k.min(picks.len());
    if k == 0 {
        return CaseAgreement { picks: Vec::new(), seat: None, class: CoreClass::NoCore, seat_votes: 0 };
    }
    let used = &picks[..k];
    let mut counts: Vec<(usize, usize)> = Vec::new();
    for &p in used {
        match counts.iter_mut().find(|(label, _)| *label == p) {
            Some((_, c)) => *c += 1,
            None => counts.push((p, 1)),
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let top = counts[0];
    let (seat, class) = if top.1 == k {
        (Some(top.0), CoreClass::Unanimous)
    } else if top.1 * 2 > k {
        (Some(top.0), CoreClass::Majority)
    } else {
        (None, CoreClass::NoCore)
    };
    CaseAgreement { picks: used.to_vec(), seat, class, seat_votes: top.1 }
}

/// The G1 anchor: the identity transform reproduces the input case
/// byte-identically (state included). A red here is a harness bug.
///
/// # Errors
/// Same as [`transformed_case`] (a non-string state).
pub fn identity_is_passthrough(case: &SuiteCase, family: Family) -> Result<bool, String> {
    let t = transformed_case(case, family, 0)?;
    Ok(t.state == case.state && t.questions == case.questions && t.gold == case.gold)
}

#[cfg(test)]
mod tests {
    use crate::harness::suites::{GoldAnswer, QKind, SuiteQuestion};
    use super::*;
    use serde_json::{Map, Value};

    fn text_case() -> SuiteCase {
        SuiteCase {
            id: "c1".to_string(),
            state: Value::String("  the  client  wants  a refund.".to_string()),
            questions: vec![SuiteQuestion {
                qid: "q".to_string(),
                kind: QKind::Choice,
                instructions: "pick".to_string(),
                criteria: Value::Object(Map::new()),
            }],
            gold: vec![GoldAnswer { idx: 1, soft: vec![], gold_score: None }],
        }
    }

    #[test]
    fn identity_is_a_byte_passthrough_both_families() {
        let c = text_case();
        for family in [Family::Semantic, Family::Placebo] {
            assert!(
                identity_is_passthrough(&c, family).expect("identity"),
                "{family:?} [0] must be the identity"
            );
        }
    }

    #[test]
    fn transforms_touch_only_the_state_and_name_the_rep() {
        let c = text_case();
        let t = transformed_case(&c, Family::Semantic, 1).expect("persona");
        assert_eq!(
            t.state.as_str().expect("string"),
            "From the perspective of a linguist,   the  client  wants  a refund."
        );
        assert_eq!(t.questions, c.questions, "questions untouched");
        assert_eq!(t.gold, c.gold, "gold untouched");
        assert_eq!(t.id, "c1#ctxpersona_linguist");
        // The original is never mutated.
        assert_eq!(
            c.state.as_str().expect("string"),
            "  the  client  wants  a refund."
        );
    }

    #[test]
    fn placebo_twins_match_semantic_word_counts() {
        // rows 1..=4: 6-word prefixes; row 5: 4-word suffix; row 6:
        // 5-word suffix; row 7: 0 added tokens.
        let s = "x";
        for rep in 1..SEMANTIC_FAMILY.len() {
            let sem_words = (SEMANTIC_FAMILY[rep].apply)(s).split_whitespace().count();
            let plac_words = (PLACEBO_FAMILY[rep].apply)(s).split_whitespace().count();
            assert_eq!(
                sem_words, plac_words,
                "row {rep}: {} vs {} word counts must match (the additive-mass pairing)",
                SEMANTIC_FAMILY[rep].name,
                PLACEBO_FAMILY[rep].name
            );
        }
    }

    #[test]
    fn object_states_transform_through_the_serialized_channel() {
        // Dataset states are OBJECTS (ag_news et al.): the identity arm
        // preserves the value; the transform arms rewrite the serialized
        // text — the one text the engine embeds.
        let mut c = text_case();
        c.state = serde_json::json!({"text": "the client wants a refund"});
        let id = transformed_case(&c, Family::Semantic, 0).expect("identity");
        assert_eq!(id.state, c.state, "identity preserves the object state");
        let t = transformed_case(&c, Family::Semantic, 1).expect("persona");
        let s = t.state.as_str().expect("string");
        assert!(s.starts_with("From the perspective of a linguist, "));
        assert!(s.contains("the client wants a refund"), "the serialized body rides: {s}");
    }

    #[test]
    fn agreement_reads_unanimous_majority_and_empty_core() {
        let a = agreement_of(&[2, 2, 2, 2], 4);
        assert_eq!(a.class, CoreClass::Unanimous);
        assert_eq!(a.seat, Some(2));
        assert_eq!(a.seat_votes, 4);
        // Strict majority: 3 of 4 (not unanimous).
        let m = agreement_of(&[0, 0, 0, 1], 4);
        assert_eq!(m.class, CoreClass::Majority);
        assert_eq!(m.seat, Some(0));
        assert_eq!(m.seat_votes, 3);
        // No core at K=4: 2-2 split.
        let n = agreement_of(&[1, 1, 2, 2], 4);
        assert_eq!(n.class, CoreClass::NoCore);
        assert_eq!(n.seat, None);
        // The K sweep is a PREFIX: K=2 of an 8-pick row reads picks[0..2].
        let p = agreement_of(&[3, 3, 1, 1, 1, 1, 1, 1], 2);
        assert_eq!(p.class, CoreClass::Unanimous);
        assert_eq!(p.seat, Some(3));
        // First-appearance tie order at equal counts (1-1 at K=2).
        let t = agreement_of(&[4, 5], 2);
        assert_eq!(t.class, CoreClass::NoCore);
        // k clamps to the picks' length and to >= 1.
        assert_eq!(agreement_of(&[1, 2, 3], 8).picks.len(), 3);
        assert_eq!(agreement_of(&[], 4).picks.len(), 0);
    }

    #[test]
    fn families_are_index_aligned() {
        assert_eq!(SEMANTIC_FAMILY.len(), PLACEBO_FAMILY.len());
        for (s, p) in SEMANTIC_FAMILY.iter().zip(PLACEBO_FAMILY.iter()) {
            assert_eq!(s.name, p.name, "identity rows share the name");
            assert_eq!((s.apply)("x"), (p.apply)("x"), "identity rows share the rewrite");
            break;
        }
    }
}
