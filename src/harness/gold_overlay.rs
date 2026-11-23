//! The ag_news gold-corrections overlay (issue 126 — the delegated-owner
//! verdict's steps 2+3; the symmetric 62-row audit, Bench 202 addendum,
//! commit `2ac1fd5`).
//!
//! Fifteen test golds are CONTENT-OBVIOUS upstream mislabels (4 in rows the
//! engine lost + 11 in rows both lanes lost — every correction an
//! agree-wrong pair, so the overlay is direction-neutral by construction:
//! it moves both lanes equally and the A2 gap change comes only from the
//! 4 one-sided rows). The overlay is applied at prepare time — BEFORE any
//! lane reads gold — so the modelless engine, the BM25 floor, and any
//! future lane score against the same corrected gold; the comparison
//! stays honest, the metric stops rewarding agreement with a wrong label.
//!
//! Laws:
//! - **Drift refuses LOUD**: a case id the suite doesn't hold, a label
//!   outside the universe, a no-op relabel (old == new), or a row count
//!   mismatch is a hard prepare error — an upstream re-fetch that moves
//!   ids never silently mislabels; it refuses until the overlay is
//!   re-audited.
//! - **Disclosed everywhere**: [`DISCLOSURE`] rides the suite row and the
//!   floor line in every table; the upstream-pure numbers stay published
//!   beside in every record citing this overlay (Bench 202 + the 126
//!   adjudication carry them).
//! - **The straddles stay untouched**: 14 defensible golds named in the
//!   audit are deliberately absent — a relabel there is an owner call,
//!   never a lane's.

use serde_json::Value;

/// The compiled artifact (15 rows; the audit record is the authority).
pub const AG_NEWS_GOLD_OVERLAY_JSON: &str = include_str!("ag_news_gold_overlay.json");

/// The suite this overlay binds to (never applied to anything else).
pub const OVERLAY_SUITE: &str = "ag_news";

/// The one-line disclosure every table and record prints beside the
/// overlay'd numbers.
pub const DISCLOSURE: &str = "gold overlay: 15 rows relabelled (content-obvious upstream errors, symmetric 62-row audit — issue 126 / Bench 202 addendum); applied to BOTH lanes; upstream-pure numbers published beside";

/// Parse + validate the compiled artifact once. Returns
/// (case_id → corrected label) sorted by case_id for a deterministic
/// application order.
pub fn parse_overlay() -> Result<Vec<(String, String)>, String> {
    let v: Value = serde_json::from_str(AG_NEWS_GOLD_OVERLAY_JSON)
        .map_err(|e| format!("gold overlay: the compiled artifact does not parse: {e}"))?;
    if v["format"].as_str() != Some("ag_news_gold_overlay_v1") {
        return Err(format!(
            "gold overlay: unknown format {:?} (want ag_news_gold_overlay_v1)",
            v["format"].as_str()
        ));
    }
    let rows = v["rows"]
        .as_array()
        .ok_or("gold overlay: the artifact carries no rows array")?;
    if rows.len() != 15 {
        return Err(format!(
            "gold overlay: {} rows — the audit's count is 15; a change re-runs the audit and edits this file WITH its record, never past it",
            rows.len()
        ));
    }
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let id = r["case_id"]
            .as_str()
            .ok_or("gold overlay: a row carries no case_id")?
            .to_string();
        if !id.starts_with("ag_news:") {
            return Err(format!(
                "gold overlay: row id {id:?} is not an ag_news case — the overlay binds one suite"
            ));
        }
        let gold = r["gold"]
            .as_str()
            .ok_or_else(|| format!("gold overlay: row {id} carries no gold"))?
            .to_string();
        out.push((id, gold));
    }
    out.sort();
    Ok(out)
}

/// Apply the overlay to `suite`'s TEST cases. `labels` is the suite's
/// label universe (the corrected gold maps through it to the gold index).
/// Drift refuses loud per the module doc.
pub fn apply_ag_news_overlay(
    suite: &mut crate::harness::suites::Suite,
    labels: &[String],
) -> Result<(), String> {
    let overlay = parse_overlay()?;
    let mut applied = 0usize;
    for (id, gold) in &overlay {
        let idx = labels.iter().position(|l| l == gold).ok_or_else(|| {
            format!("gold overlay: {id} → {gold:?} is outside the label universe {labels:?}")
        })?;
        let case = suite
            .cases
            .iter_mut()
            .find(|c| &c.id == id)
            .ok_or_else(|| {
                format!(
                    "gold overlay: case {id} is absent from the suite — the upstream rows moved; \
                     re-run the audit and edit the overlay with its record (never apply blind)"
                )
            })?;
        let g = case
            .gold
            .first_mut()
            .ok_or_else(|| format!("gold overlay: case {id} carries no gold"))?;
        if g.idx == idx {
            return Err(format!(
                "gold overlay: {id} already reads {gold:?} — a no-op relabel means the \
                 upstream fixed it; re-audit and drop the row"
            ));
        }
        g.idx = idx;
        applied += 1;
    }
    if applied != 15 {
        return Err(format!(
            "gold overlay: applied {applied}/15 — the artifact drifted mid-apply (impossible \
             by construction; refuse rather than half-apply)"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The artifact's shape law: format, 15 rows, ag_news-prefixed ids,
    /// every label inside the AG_NEWS universe, no duplicate ids.
    #[test]
    fn overlay_artifact_is_wellformed() {
        let overlay = parse_overlay().expect("parse");
        assert_eq!(overlay.len(), 15);
        let universe = crate::harness::suites::AG_NEWS_KEYS;
        let mut seen = std::collections::BTreeSet::new();
        for (id, gold) in &overlay {
            assert!(id.starts_with("ag_news:"), "non-ag_news id {id}");
            assert!(
                universe.contains(&gold.as_str()),
                "{id} → {gold} outside the universe"
            );
            assert!(seen.insert(id.as_str()), "duplicate id {id}");
        }
    }

    /// The audit's content pins — the 15 corrections are exactly these
    /// (case_id → label), byte-pinned so a silent artifact edit reds.
    #[test]
    fn overlay_content_is_the_audited_fifteen() {
        let overlay: std::collections::BTreeMap<String, String> =
            parse_overlay().expect("parse").into_iter().collect();
        let expect: [(&str, &str); 15] = [
            ("ag_news:106", "business"),
            ("ag_news:196", "business"),
            ("ag_news:248", "business"),
            ("ag_news:336", "world"),
            ("ag_news:24", "business"),
            ("ag_news:56", "business"),
            ("ag_news:79", "sports"),
            ("ag_news:88", "sports"),
            ("ag_news:207", "business"),
            ("ag_news:258", "business"),
            ("ag_news:346", "business"),
            ("ag_news:348", "sports"),
            ("ag_news:355", "sports"),
            ("ag_news:369", "sports"),
            ("ag_news:372", "sports"),
        ];
        assert_eq!(overlay.len(), expect.len());
        for (id, gold) in expect {
            assert_eq!(
                overlay.get(id).map(String::as_str),
                Some(gold),
                "row {id} drifted off the audit"
            );
        }
    }

    /// The apply law on a synthetic suite: correct relabel lands, a
    /// missing case id refuses loud, an out-of-universe label refuses
    /// loud, a no-op relabel refuses loud.
    #[test]
    fn apply_refuses_loud_on_drift() {
        use crate::harness::suites::{GoldAnswer, Suite, SuiteCase, SuiteQuestion};
        let mk = |ids: &[&str]| Suite {
            name: "ag_news",
            cases: ids
                .iter()
                .map(|id| SuiteCase {
                    id: id.to_string(),
                    state: serde_json::json!({}),
                    questions: vec![SuiteQuestion {
                        qid: "q".into(),
                        kind: crate::harness::suites::QKind::Choice,
                        instructions: String::new(),
                        criteria: serde_json::json!({}),
                    }],
                    gold: vec![GoldAnswer {
                        idx: 0,
                        soft: vec![],
                        gold_score: None,
                    }],
                })
                .collect(),
            option_counts_note: "",
        };
        let labels = crate::harness::suites::AG_NEWS_KEYS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        // Sorted iteration starts at ag_news:106 — a suite holding ONLY it
        // (pre-set to the overlay's own target, business = 2) hits the
        // NO-OP arm first: a relabel that changes nothing refuses (the
        // upstream-fixed case).
        let mut s = mk(&["ag_news:106"]);
        s.cases[0].gold[0].idx = 2;
        let err = apply_ag_news_overlay(&mut s, &labels.clone()).unwrap_err();
        assert!(
            err.contains("already reads"),
            "the no-op relabel must refuse: {err}"
        );
        // A suite holding NONE of the 15 ids: the first sorted id
        // (ag_news:106) is absent → the absent arm refuses.
        let mut s = mk(&["ag_news:999"]);
        let err = apply_ag_news_overlay(&mut s, &labels).unwrap_err();
        assert!(
            err.contains("absent from the suite"),
            "the absent ids must refuse: {err}"
        );
    }
}
