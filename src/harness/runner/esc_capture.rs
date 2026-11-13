//! Issue 117 T1 — the ESC-capture harness arm.
//!
//! Replays the ESC-armed cascade's records over a suite's CAL slice and
//! writes the VoI outcome join: one `(case id, split tag, cheap pick, think
//! pick, gold, gate flag)` triple per cal question, plus the divergence
//! stats and the captured-id list the T3 frozen read asserts its
//! disjointness against.
//!
//! SPLIT LAW (the issue's blocking clause, enforced here fail-loud):
//! mining reads ONLY the cal side of the suite split — the capture walks
//! `Prepared::cal_cases` (the stratified cal front), never the test cases;
//! `massive_intent_en` is the ONLY allowed mining source (the Bench-089
//! shadow-PASS clause; banking77 rows are forbidden outright); and every
//! captured id is asserted disjoint from the suite's test-split ids (the
//! id set is read solely inside this assert — no test content enters the
//! triples). A future T3 must re-assert disjointness of its own frozen
//! read against `captured_ids.txt`.
//!
//! The capture RUN (a serve lane feeding the escalate lane) is deferred —
//! this arm is the harness-side writer the run will call.

use std::io::Write as _;
use std::path::PathBuf;

use serde::Serialize;

use crate::harness::cascade::{CalGateRecords, LayaQuestions, WorthinessInput};
use super::{Prepared, SuiteSpec};

/// The only suite the initial mining posture may capture (issue 117 T1:
/// `massive_intent_en` ONLY — the Bench-089 shadow-read PASS clause;
/// banking77 carries the shadow-MIXED caveat and is forbidden until a
/// separate caveat-cleared lane).
pub const ESC_CAPTURE_SUITES: &[&str] = &["massive_intent_en"];

/// One `(cheap, think, gold)` observation: the VoI join's row.
#[derive(Debug, Clone, Serialize)]
pub struct EscTriple {
    /// The cal case's suite-qualified id (`massive_intent_en:<pos>`).
    pub id: String,
    /// The split tag — always `"cal"` here (the capture never reads the
    /// test side; the field exists so downstream joins can grep the tag).
    pub split: &'static str,
    /// The modelless (cheap) lane's pick, label index.
    pub cheap_pick: usize,
    /// The escalator's (think) pick when it served this case, else `null`
    /// (the ANE bucket skip / the question cap removed it from serving).
    pub think_pick: Option<usize>,
    /// The gold answer's label index.
    pub gold: usize,
    /// The shipped escalation semantics for this question: the calibrated
    /// gate abstained AND the escalator served AND the worthiness verdict
    /// left the escalation armed — the fired gate is the only row where
    /// the think pick would be served in production.
    pub gate_fired: bool,
}

/// The run's divergence summary (stdout + `esc_capture_stats.json`).
#[derive(Debug, Clone, Serialize)]
pub struct EscCaptureStats {
    pub suite: String,
    pub cal_cases: usize,
    pub cal_questions: usize,
    /// Questions where the ESC gate FIRED (an escalation would happen).
    pub gate_fired: usize,
    /// Fired questions where the escalator actually served the case.
    pub gate_fired_served: usize,
    /// Fired+served questions where cheap and think disagree.
    pub disagreements: usize,
    /// Among fired+served: the think lane's correct count.
    pub think_correct: usize,
    /// Among fired+served: the cheap lane's correct count.
    pub cheap_correct: usize,
    /// Among fired+served: think right where cheap was wrong (the ESC win).
    pub think_wins: usize,
    /// Among fired+served: cheap right where think was wrong (the ESC loss).
    pub cheap_wins: usize,
    /// The worthiness verdict the run armed under (mirrored for the record).
    pub armed: bool,
    pub probe_n: usize,
    pub probe_delta: f64,
    /// Rows written to `triples.jsonl`.
    pub triples_written: usize,
}

/// What `capture` wrote, for the caller's summary line.
#[derive(Debug, Clone)]
pub struct EscCaptureOutput {
    pub stats: EscCaptureStats,
    pub triples_path: PathBuf,
    pub ids_path: PathBuf,
    pub stats_path: PathBuf,
}

/// The split-law + shape validation, then the triples/stats/id-list write.
///
/// `worthiness` is the SAME `WorthinessInput` the cascade's compose step
/// consumes (deterministic — the verdict recomputed here is byte-faithful
/// with the lane's arming decision; the capture never re-derives a delta
/// of its own).
pub(crate) fn capture(
    spec: &SuiteSpec,
    prepared: &Prepared,
    gate: Option<&CalGateRecords>,
    escalator: Option<&LayaQuestions>,
    worthiness: Option<&WorthinessInput<'_>>,
    out_dir: &std::path::Path,
) -> Result<EscCaptureOutput, String> {
    // SPLIT LAW, clause 1: the mining-source allowlist (massive_intent_en
    // only; banking77 forbidden outright).
    if !ESC_CAPTURE_SUITES.contains(&spec.name) {
        return Err(format!(
            "esc-capture: suite {} is not an approved mining source (allowed: \
             {:?}); banking77 is forbidden outright (the Bench-089 shadow-MIXED \
             caveat) — refuse, never mine",
            spec.name, ESC_CAPTURE_SUITES
        ));
    }

    // The worthiness verdict: the same inputs the cascade's compose consumes,
    // so `armed` mirrors the shipped arming decision byte-for-byte.
    let verdict = match worthiness {
        Some(w) => Some(w.probe()?),
        None => None,
    };
    let armed = verdict.as_ref().is_none_or(|v| !v.disarmed);

    // SPLIT LAW, clause 2 (shape): the gate must cover EVERY cal case — a
    // short record would fabricate rows exactly like a misaligned test
    // record would fabricate accuracy.
    let Some(gate) = gate else {
        return Err(
            "esc-capture: the modelless cal-gate records are absent — pass \
             --cascade-worthiness so the cal slice is evaluated (the capture \
             reads the cal side only, never the test rows)"
                .to_string(),
        );
    };
    if gate.picks.len() != prepared.cal_cases.len()
        || gate.abstained.len() != prepared.cal_cases.len()
    {
        return Err(format!(
            "esc-capture: gate shape drift — {} cal cases vs {} pick records",
            prepared.cal_cases.len(),
            gate.picks.len()
        ));
    }

    // The escalator's cal serving map: cal-case index → served position.
    let served: std::collections::HashMap<usize, usize> = escalator
        .map(|e| {
            e.served_orig
                .iter()
                .enumerate()
                .map(|(j, &orig)| (orig, j))
                .collect()
        })
        .unwrap_or_default();

    // SPLIT LAW, clause 3 (disjointness): every emitted id must be ABSENT
    // from the suite's test-split ids. The test ids are read ONLY inside
    // this assert — the triples carry cal rows exclusively.
    let test_ids: std::collections::HashSet<&str> = prepared
        .suite
        .cases
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    let mut triples = Vec::new();
    let mut ids = Vec::new();
    let mut stats = EscCaptureStats {
        suite: spec.name.to_string(),
        cal_cases: prepared.cal_cases.len(),
        cal_questions: 0,
        gate_fired: 0,
        gate_fired_served: 0,
        disagreements: 0,
        think_correct: 0,
        cheap_correct: 0,
        think_wins: 0,
        cheap_wins: 0,
        armed,
        probe_n: verdict.as_ref().map_or(0, |v| v.probe_n),
        probe_delta: verdict.as_ref().map_or(0.0, |v| v.delta),
        triples_written: 0,
    };
    for (ci, case) in prepared.cal_cases.iter().enumerate() {
        if test_ids.contains(case.id.as_str()) {
            return Err(format!(
                "esc-capture: SPLIT-LAW VIOLATION — cal case id {} also exists \
                 in the test split; mining it would fold the frozen read into \
                 the corpus. Refusing the capture",
                case.id
            ));
        }
        ids.push(case.id.clone());
        for (qi, gold) in case.gold.iter().enumerate() {
            stats.cal_questions += 1;
            let cheap_pick = gate.picks[ci][qi];
            let abstained = gate.abstained[ci][qi];
            let served_j = served.get(&ci).copied();
            let think_pick = served_j.map(|j| escalator.expect("served map implies records").picks[j][qi]);
            let gate_fired = abstained && armed && served_j.is_some();
            if gate_fired {
                stats.gate_fired += 1;
                stats.gate_fired_served += 1;
                let think = escalator.expect("served map implies records").picks[served_j.expect("checked")][qi];
                if think == gold.idx {
                    stats.think_correct += 1;
                }
                if cheap_pick == gold.idx {
                    stats.cheap_correct += 1;
                }
                if think != cheap_pick {
                    stats.disagreements += 1;
                }
                if think == gold.idx && cheap_pick != gold.idx {
                    stats.think_wins += 1;
                }
                if cheap_pick == gold.idx && think != gold.idx {
                    stats.cheap_wins += 1;
                }
            }
            triples.push(EscTriple {
                id: case.id.clone(),
                split: "cal",
                cheap_pick,
                think_pick,
                gold: gold.idx,
                gate_fired,
            });
        }
    }
    stats.triples_written = triples.len();

    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("esc-capture: create {}: {e}", out_dir.display()))?;

    let triples_path = out_dir.join("triples.jsonl");
    let mut f = std::fs::File::create(&triples_path)
        .map_err(|e| format!("esc-capture: create {}: {e}", triples_path.display()))?;
    for t in &triples {
        let line = serde_json::to_string(t).expect("esc triple serializes");
        writeln!(f, "{line}").map_err(|e| format!("esc-capture: write {line}: {e}"))?;
    }

    let ids_path = out_dir.join("captured_ids.txt");
    let mut f = std::fs::File::create(&ids_path)
        .map_err(|e| format!("esc-capture: create {}: {e}", ids_path.display()))?;
    for id in &ids {
        writeln!(f, "{id}").map_err(|e| format!("esc-capture: write {id}: {e}"))?;
    }

    let stats_path = out_dir.join("esc_capture_stats.json");
    std::fs::write(
        &stats_path,
        serde_json::to_string_pretty(&stats).expect("esc stats serializes"),
    )
    .map_err(|e| format!("esc-capture: write {}: {e}", stats_path.display()))?;

    Ok(EscCaptureOutput {
        stats,
        triples_path,
        ids_path,
        stats_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cal_case(id: &str, gold: usize) -> crate::harness::suites::SuiteCase {
        crate::harness::suites::SuiteCase {
            id: id.to_string(),
            state: serde_json::json!({ "utterance": "x" }),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "intent".to_string(),
                kind: crate::harness::suites::QKind::Choice,
                instructions: String::new(),
                criteria: serde_json::json!({}),
            }],
            gold: vec![crate::harness::suites::GoldAnswer {
                idx: gold,
                soft: vec![],
                gold_score: None,
            }],
        }
    }

    fn gate_all(picks: &[usize], abstained: bool) -> CalGateRecords {
        CalGateRecords {
            picks: picks.iter().map(|&p| vec![p]).collect(),
            abstained: picks.iter().map(|_| vec![abstained]).collect(),
        }
    }

    fn escalator_all(origs: &[usize], picks: &[usize]) -> LayaQuestions {
        LayaQuestions {
            served_orig: origs.to_vec(),
            picks: picks.iter().map(|&p| vec![p]).collect(),
        }
    }

    fn spec(name: &str) -> SuiteSpec {
        SuiteSpec {
            name: Box::leak(name.to_string().into_boxed_str()),
            dataset_dir: "x",
            test_cap: 0,
            cal_cap: 0,
            corpus_cap_per_label: 0,
            eval_split: "test",
            named_only: true,
            coverage_audit: false,
            build: |_, _| {
                unreachable!("the capture never builds a suite — it reads the cal slice");
            },
            synthetic: None,
            modelless_lane: true,
        }
    }

    fn prepared(
        spec: &SuiteSpec,
        cal: Vec<crate::harness::suites::SuiteCase>,
    ) -> Prepared {
        Prepared {
            suite: crate::harness::suites::Suite {
                name: spec.name,
                cases: vec![cal_case("massive_intent_en:TEST", 0)],
                option_counts_note: "",
            },
            train: Vec::new(),
            state_strs: Vec::new(),
            cal_cases: cal,
            cal_state_strs: Vec::new(),
            labels: Vec::new(),
            pool_rows: serde_json::Value::Null,
            slices: None,
        }
    }

    #[test]
    fn writes_triples_ids_and_stats() {
        let sp = spec("massive_intent_en");
        // cal ids distinct from the (single) test id the fake suite carries.
        let prep = prepared(
            &sp,
            vec![
                cal_case("massive_intent_en:CAL0", 1),
                cal_case("massive_intent_en:CAL1", 0),
            ],
        );
        let gate = gate_all(&[0, 2], true);
        let esc = escalator_all(&[0, 1], &[1, 2]);
        let dir = std::env::temp_dir().join(format!("esc-cap-test-{}", std::process::id()));
        let out =
            capture(&sp, &prep, Some(&gate), Some(&esc), None, &dir).expect("capture succeeds");
        assert_eq!(out.stats.cal_questions, 2);
        // Both abstained + served + armed (no worthiness input = armed):
        // both rows fire the gate; think (1) beats cheap (0) on CAL0 and
        // both miss on CAL1.
        assert_eq!(out.stats.gate_fired, 2);
        assert_eq!(out.stats.disagreements, 1);
        assert_eq!(out.stats.think_wins, 1);
        assert_eq!(out.stats.cheap_wins, 0);
        assert_eq!(out.stats.triples_written, 2);
        let triples = std::fs::read_to_string(&out.triples_path).expect("triples exist");
        assert!(triples.contains("\"split\":\"cal\""));
        assert!(triples.contains("\"think_pick\":1"));
        let ids = std::fs::read_to_string(&out.ids_path).expect("ids exist");
        assert!(ids.contains("massive_intent_en:CAL0"));
        assert!(!ids.contains(":TEST"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn refuses_foreign_suite_and_test_overlap() {
        let bank = spec("banking77");
        let prep = prepared(&bank, vec![cal_case("banking77:0", 0)]);
        let err = capture(
            &bank,
            &prep,
            Some(&gate_all(&[0], true)),
            None,
            None,
            &std::env::temp_dir(),
        )
        .expect_err("banking77 must refuse");
        assert!(err.contains("not an approved mining source"), "{err}");

        let sp = spec("massive_intent_en");
        // The cal case id COLLIDES with the fake suite's test id: loud refuse.
        let prep = prepared(&sp, vec![cal_case("massive_intent_en:TEST", 0)]);
        let err = capture(
            &sp,
            &prep,
            Some(&gate_all(&[0], true)),
            None,
            None,
            &std::env::temp_dir(),
        )
        .expect_err("test-overlap must refuse");
        assert!(err.contains("SPLIT-LAW VIOLATION"), "{err}");
    }

    #[test]
    fn unserved_rows_record_null_think_and_no_fire() {
        let sp = spec("massive_intent_en");
        let prep = prepared(&sp, vec![cal_case("massive_intent_en:CAL0", 0)]);
        let gate = gate_all(&[0], true);
        // The escalator served NOTHING: think_pick must be null, no gate fire.
        let esc = escalator_all(&[], &[]);
        let dir = std::env::temp_dir().join(format!("esc-cap-null-{}", std::process::id()));
        let out =
            capture(&sp, &prep, Some(&gate), Some(&esc), None, &dir).expect("capture succeeds");
        assert_eq!(out.stats.gate_fired, 0);
        let triples = std::fs::read_to_string(&out.triples_path).expect("triples exist");
        assert!(triples.contains("\"think_pick\":null"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
