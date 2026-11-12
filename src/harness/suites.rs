//! Suite builders from HF datasets-server `/rows` JSON row files
//! (`.docs/02_protocols/laya_bench_protocols.md` §3, Plan 603 T1.5). Pure: takes the
//! parsed envelope `{"features": [...], "rows": [{"row_idx": N, "row": {...}}]}`
//! and returns cases; no fetch, no I/O, no engine/laya imports.
//!
//! Conventions shared by every builder:
//! - **Sampling is FIRST-N in dataset order** (`max_rows`, 0 = all), so the
//!   positional index of a row inside the sampled slice equals the
//!   envelope's `row_idx` (envelopes enumerate from 0).
//! - Rows missing required fields are SKIPPED (the Python reference would
//!   crash; the runner's count floors are the loud guard against silent
//!   shrinkage).
//! - Gold soft targets carry per-key zeros where the dataset has no soft
//!   distribution — `soft_metrics` returns None on the zero-sum target, so
//!   the runner can call it unconditionally.
//! - Noul option order is ALWAYS `[false, true]`: gold idx 0 = false, 1 =
//!   true (lane-layer rendering fact; criteria stay Null here).

use serde::Serialize;
use serde_json::{Map, Value, json};
use std::collections::HashMap;

/// Question type (§1.1 `QTYPES`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QKind {
    Choice,
    Score,
    Noul,
}

impl QKind {
    /// The reference's string spelling (`QTYPE_NAMES`), used in result
    /// records and sequence heads.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            QKind::Choice => "choice",
            QKind::Score => "score",
            QKind::Noul => "noul",
        }
    }
}

/// One question definition. `criteria` is a JSON OBJECT for choice (whose
/// insertion order IS the label order — serde_json `preserve_order`), an
/// ARRAY for score (index = level), and Null for noul (options render
/// `[false, true]` with the fixed default phrases).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SuiteQuestion {
    pub qid: String,
    pub kind: QKind,
    pub instructions: String,
    pub criteria: Value,
}

/// Gold for one question.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GoldAnswer {
    pub idx: usize,
    pub soft: Vec<f64>,
    pub gold_score: Option<f64>,
}

/// One case: a state plus its parallel question/gold lists (same length,
/// same order — built pairwise).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SuiteCase {
    pub id: String,
    pub state: Value,
    pub questions: Vec<SuiteQuestion>,
    pub gold: Vec<GoldAnswer>,
}

impl AsRef<SuiteCase> for SuiteCase {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// A built suite. `option_counts_note` documents the option-count policy
/// (fixed vs sampled, and the seed rule).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Suite {
    pub name: &'static str,
    pub cases: Vec<SuiteCase>,
    pub option_counts_note: &'static str,
}

// ── envelope helpers ────────────────────────────────────────────────────────

/// Each `.rows[i].row` of the /rows envelope (empty when the envelope is
/// malformed — callers' count floors catch that).
fn rows_of(file: &Value) -> Vec<&Value> {
    file.get("rows")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(|r| r.get("row")).collect())
        .unwrap_or_default()
}

/// FIRST-N in dataset order; 0 = all.
fn sampled<'a>(rows: &'a [&'a Value], max_rows: usize) -> &'a [&'a Value] {
    if max_rows == 0 {
        rows
    } else {
        let n = rows.len().min(max_rows);
        &rows[..n]
    }
}

fn row_str<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

/// Python `int()`-style tolerant integer read: JSON number or numeric string.
fn value_as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn row_i64(row: &Value, key: &str) -> Option<i64> {
    row.get(key).and_then(value_as_i64)
}

fn value_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// Python `str()` for the JSON values gold labels arrive as.
/// 4. py_str: JSON-escape non-string values as the reference would — CPython
/// json.dumps with ensure_ascii=False, default separators (", ", ": ") — for
/// str values this is identity, for bools/ints short literals.
fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::Null => "None".to_string(),
        other => other.to_string(),
    }
}

/// Build the choice question; `criteria` object insertion order = label
/// order (`preserve_order` keeps it).
fn choice_q(
    qid: &str,
    instructions: &str,
    keys: &[String],
    descs: Option<Vec<Option<String>>>,
) -> SuiteQuestion {
    let mut crit = Map::new();
    for (i, k) in keys.iter().enumerate() {
        let desc = descs.as_ref().and_then(|d| d.get(i)).cloned().flatten();
        crit.insert(k.clone(), desc.map_or(Value::Null, Value::String));
    }
    SuiteQuestion {
        qid: qid.to_string(),
        kind: QKind::Choice,
        instructions: instructions.to_string(),
        criteria: Value::Object(crit),
    }
}

/// Criteria KEY order for a choice question: object → keys in order; array →
/// elements as strings (the §1.2 `to_internal` list normalization).
fn choice_keys(crit: &Value) -> Option<Vec<String>> {
    match crit {
        Value::Object(m) => Some(m.keys().cloned().collect()),
        Value::Array(a) => Some(a.iter().map(py_str).collect()),
        _ => None,
    }
}

// ── builders ────────────────────────────────────────────────────────────────

/// §3.7 Banking77 (the `bench_apps` variant) — `mteb/banking77`, whose `label`
/// is an int64 (NOT a ClassLabel) and which carries `label_text`. Option keys
/// derive from the data exactly like the reference: `labels = sorted(set(label_text))`,
/// each key underscore-stripped, criteria value None; gold = the key position.
#[must_use]
pub fn build_banking77_mteb(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    // Option universe = sorted unique label_text over ALL rows on disk (the
    // reference derives it from the whole split); the CASES come from the
    // first-`max_rows` slice. The eval slice may exercise a subset of the
    // universe (the mirror's first rows are label-clustered) — the option
    // list is 77-wide regardless, matching bench_apps.
    let mut labels: Vec<String> = Vec::new();
    for row in &all {
        if let Some(t) = row_str(row, "label_text")
            && !labels.iter().any(|l| l == t)
        {
            labels.push(t.to_string());
        }
    }
    labels.sort();
    // NOTE: no universe-size assert here — the builder also runs over TRAIN
    // rows for the calibration slice, where a partial intent coverage is
    // normal. The runner asserts the TEST suite presents all 77 options.
    let keys: Vec<String> = labels.iter().map(|n| n.replace('_', " ")).collect();
    let descs: Vec<Option<String>> = vec![None; keys.len()];
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label_text)) = (row_str(row, "text"), row_str(row, "label_text"))
        else {
            continue;
        };
        let idx = keys
            .iter()
            .position(|k| *k == label_text.replace('_', " "))
            .unwrap_or_else(|| {
                panic!("banking77 mteb: row label_text {label_text:?} not in key set")
            });
        cases.push(SuiteCase {
            id: format!("banking77:{pos}"),
            state: json!({ "message": text }),
            questions: vec![choice_q(
                "intent",
                "Which banking intent does `message` express?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "banking77",
        cases,
        option_counts_note: "77 options derived from sorted unique label_text (bench_apps variant)",
    }
}

/// Issue 114 T2 — CLINC150 (`clinc/clinc_oos`, config `plus`): 151-way intent
/// choice (150 intents + `oos` at its ClassLabel position, the board row's
/// "150-intent (+ OOS)"). The class-label order is FIXED and pinned by
/// [`CLINC150_KEYS`]; the builder VERIFY-AT-PORTS the fetched envelope's
/// features (the ag_news law — a drifted ClassLabel order fails loud, never
/// silently rebinds 151 corpora). The columns are the banking77 shape with
/// the ClassLabel-names half present: `text` (string) + `intent` (ClassLabel).
#[must_use]
pub fn build_clinc150(rows_file: &Value, max_rows: usize) -> Suite {
    let names = label_names(rows_file, "intent").unwrap_or_else(|| {
        panic!(
            "clinc150: envelope features carry no intent ClassLabel names — \
             the /rows fetch must include the features block (verify-at-port)"
        )
    });
    let spelled: Vec<String> = names.iter().map(|n| n.replace('_', " ")).collect();
    assert_eq!(
        spelled, CLINC150_KEYS,
        "clinc150: the fetched ClassLabel order drifted from CLINC150_KEYS — \
         the engine domains would silently rebind (the ag_news verify-at-port law)"
    );
    let keys = spelled;
    let descs: Vec<Option<String>> = vec![None; keys.len()];
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(intent)) = (row_str(row, "text"), row_i64(row, "intent")) else {
            continue;
        };
        if intent < 0 || intent as usize >= keys.len() {
            continue; // out-of-range label — fails the rule, the row drops
        }
        cases.push(SuiteCase {
            id: format!("clinc150:{pos}"),
            state: json!({ "message": text }),
            questions: vec![choice_q(
                "intent",
                "Which intent does `message` express?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: intent as usize,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "clinc150",
        cases,
        option_counts_note: "151 fixed options in ClassLabel order (150 intents + oos; underscores stripped)",
    }
}

/// Issue 077: the fixed-criteria classification suites' option keys, as
/// MODULE consts so the builders AND [`train_row_label`] share ONE
/// spelling. `train_row_label` returning the KEY (not the integer class
/// label) names the engine's domains by option content — the by-name
/// option→domain resolution then binds by CONTENT instead of the legacy
/// `k == N` index alignment, which bound option-at-position-i to domain-i
/// and made the ranking position-driven under any non-canonical option
/// order (the Bench-128 finding). Canonical-order outputs are
/// byte-identical (union[i] ≡ old domain i by the parity assert the
/// prepare path kept since the fetch law).
///
/// Issue 114 T3: the When2Call category universe rides the same law — the
/// four fixed category names, in the `answers` object's key order (uniform
/// across the split, verified at port), which is ALSO the gold-index order
/// (the official eval's `answer_names.index(correct_answer)` law).
pub(crate) const AG_NEWS_KEYS: [&str; 4] = ["world", "sports", "business", "sci_tech"];
pub(crate) const EMOTION_KEYS: [&str; 6] = ["sadness", "joy", "love", "anger", "fear", "surprise"];
pub(crate) const XNLI_KEYS: [&str; 3] = ["entailment", "neutral", "contradiction"];
pub(crate) const WISESIGHT_KEYS: [&str; 4] = ["pos", "neu", "neg", "q"];

/// Issue 114 T3 — the When2Call category universe: the four fixed category
/// names, in the `answers` object's key order (uniform across the whole mcq
/// split — verified at port; this order is ALSO the gold-index order, the
/// official eval's `answer_names.index(correct_answer)` law). `direct` is
/// presented as an option on every row but is NEVER gold in the mcq split
/// (measured 0/3,652) — the option universe deliberately exceeds the gold
/// space, the s1mb coverage posture (`coverage_audit: false`).
pub(crate) const WHEN2CALL_KEYS: [&str; 4] =
    ["direct", "tool_call", "request_for_info", "cannot_answer"];

/// Issue 114 T3 — When2Call (`nvidia/When2Call`, config `test`, split `mcq`):
/// the tool-calling decision benchmark's MCQ surface, the JDI ladder's
/// tools-rung row (the abstention-first shape — the wire's own contract).
/// TASK-FAITHFUL rendering, the official lm-eval MCQ form: each row's four
/// CHOICES are the row's own response candidates — the `answers` object's
/// values, keyed by the fixed category names in the object's (uniform)
/// insertion order — and the gold is `correct_answer`'s position in that
/// order, exactly the official `get_choices_and_index`. Fixed option-KEY
/// universe, per-row criteria VALUES (the mmlupro breadth shape at fixed
/// width 4). The state is the row's own task input: the query + the
/// tool-schema strings AS STORED (the no-silent-transforms gate; 258 rows
/// carry an empty tools list — the benchmark's own no-tools tier, kept
/// verbatim).
///
/// CORPUS-OPTIONAL: no lawful train pool exists (issue 114 T3's three
/// measured refusals — llm_judge is a documented mcq subset, train splits
/// carry no class label, the answers objects are test rows) — the modelless
/// lane is skipped loud and only the LLM lanes answer
/// (`modelless_lane: false` in the registry).
#[must_use]
pub fn build_when2call(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let keys: Vec<String> = WHEN2CALL_KEYS.iter().map(|k| (*k).to_string()).collect();
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(question), Some(answers), Some(correct)) = (
            row_str(row, "question"),
            row.get("answers").and_then(Value::as_object),
            row_str(row, "correct_answer"),
        ) else {
            continue; // malformed row — fails the rule, the row drops
        };
        // Verify-at-port: the answers object's key order IS the option order
        // (uniform [direct, tool_call, request_for_info, cannot_answer]). A
        // drifted order would silently rebind every gold index (the gold is
        // the POSITION in this order), so the order is pinned per row — the
        // clinc150 ClassLabel law at the answers-object level.
        let key_spellings: Vec<&str> = answers.keys().map(String::as_str).collect();
        assert_eq!(
            key_spellings, WHEN2CALL_KEYS,
            "when2call: the answers-object key order drifted from WHEN2CALL_KEYS — \
             the gold index is the position in this order, so a reorder would \
             silently rebind every row"
        );
        let gold_idx = key_spellings
            .iter()
            .position(|k| *k == correct)
            .unwrap_or_else(|| {
                panic!("when2call: correct_answer {correct:?} not in the answers key set")
            });
        let descs: Vec<Option<String>> = WHEN2CALL_KEYS
            .iter()
            .map(|k| answers.get(*k).and_then(Value::as_str).map(str::to_string))
            .collect();
        let tools = row
            .get("tools")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        cases.push(SuiteCase {
            id: format!("when2call:{pos}"),
            state: json!({ "query": question, "tools": tools }),
            questions: vec![choice_q(
                "response",
                "Which of the four candidate responses is the correct one for `query`, \
                 given the available `tools`?",
                &keys,
                Some(descs),
            )],
            gold: vec![GoldAnswer {
                idx: gold_idx,
                soft: vec![0.0; WHEN2CALL_KEYS.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "when2call",
        cases,
        option_counts_note: "4 options per row: the row's own response candidates keyed by the fixed category names (the official MCQ form)",
    }
}

/// Issue 114 T7 — the RAGTruth option universe: the binary hallucination
/// decision the board's row scores (chance row 59: "Always `hallucinated`
/// · F1"). Index 1 is the POSITIVE side — [`binary_f1_of`] is pinned to
/// it (the board metric is the single-class F1, never the two-class
/// macro-F1).
pub(crate) const RAGTRUTH_KEYS: [&str; 2] = ["grounded", "hallucinated"];

/// Issue 114 T7 — RAGTruth (board id 59, added edition 0.3; rethink 060
/// R3a): the JDI language rung's response-level hallucination decision —
/// "does the response contain unsupported or contradictory claims relative
/// to its context?" The eval population is ALL 2,700 upstream test
/// responses (the chance arithmetic pins the population: 2·943/(943+2700)
/// = 0.51771 = the board's 0.5177 always-hallucinated F1 — no quality
/// filtering, the truncation/incorrect-refusal rows stay), fetched from
/// the parquet-backed flowaicom/RAGTruth_test mirror (response-level
/// binary labels "as described in the paper"; upstream
/// ParticleMedia/RAGTruth LICENSE verified MIT, rethink 060 R1c residue
/// 2026-11-12) across its three task-type splits (qa / summarization /
/// data2txt, 900 rows each — the comma-list merge). Gold = the mirror's
/// `score` column (1 = ≥1 annotated hallucination span, 0 = none).
/// Corpus-ON: the lawful pool is the upstream TRAIN split (15,090
/// response rows, fetched from the wandb/RAGTruth-processed mirror — the
/// 060 R1c scan proved wandb-train ≡ upstream-train at the response
/// level); the corpus row text is the RESPONSE alone (the discriminative
/// side — the context is shared per source article and would drown the
/// per-label signal in article noise), labelled through
/// `hallucination_labels_processed` (any count > 0 = "hallucinated"). The
/// jev_* prediction columns are never read (the 060 record's
/// provider-terms caveat).
#[must_use]
pub fn build_ragtruth(rows_file: &Value, max_rows: usize) -> Suite {
    let keys: Vec<String> = RAGTRUTH_KEYS.iter().map(|k| (*k).to_string()).collect();
    let all = rows_of(rows_file);
    // Two row schemas reach this ONE builder: the EVAL rows
    // ({response, source_info, score} — the upstream-direct transform) and
    // the CAL/POLLY front rows ({output, context, hallucination_labels
    // _processed} — the same train rows the corpus pool reads, the cal
    // slice being the stratified front of the train split). Project both
    // to the same case shape: the eval fields win; a pool row reads its
    // own columns.
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (response, context, gold) = if let Some(score) = eval_gold(row) {
            match (row_str(row, "response"), row_str(row, "source_info")) {
                (Some(r), Some(c)) => (r, c, score),
                _ => continue, // malformed eval row — fails the rule, drops
            }
        } else if let Some(label) = pool_gold(row) {
            match (row_str(row, "output"), row_str(row, "context")) {
                (Some(r), Some(c)) => (r, c, label),
                _ => continue, // malformed pool row — fails the rule, drops
            }
        } else {
            continue; // neither schema — fails the rule, the row drops
        };
        let source = row_str(row, "source").unwrap_or("");
        let task = row_str(row, "task_type").unwrap_or("");
        cases.push(SuiteCase {
            id: format!("ragtruth:{pos}"),
            state: json!({
                "context": context,
                "response": response,
                "source": source,
                "task_type": task,
            }),
            questions: vec![choice_q(
                "grounding",
                "Given `context` and the model's `response`, decide whether the response is \"grounded\" \
                 (every factual claim is supported by or directly follows from the context) or \
                 \"hallucinated\" (it contains at least one claim that contradicts the context or is \
                 not supported by it).",
                &keys,
                None,
            )],
            gold: vec![GoldAnswer {
                idx: gold,
                soft: vec![0.0; RAGTRUTH_KEYS.len()],
                gold_score: None,
            }],
        });
    }
    // The 2,700 population pin binds the EVAL posture only: the cal front
    // (a train-schema slice) and any future partial read are exempt — the
    // caller signals eval by carrying the `score` column on every row. A
    // drifted EVAL fetch still reds here (the eval rows all carry score;
    // 900-class partials fail this assert exactly as before).
    let eval_rows = all.iter().filter(|r| eval_gold(r).is_some()).count();
    let built_eval = all
        .iter()
        .take(cases.len())
        .filter(|r| eval_gold(r).is_some())
        .count();
    if eval_rows > 0 {
        assert_eq!(
            eval_rows, 2_700,
            "ragtruth: the fetched population drifted from the board's 2,700 test \
             responses (the chance arithmetic 2·943/(943+2700)=0.51771 pins it) — \
             a drifted mirror reds here, never silently"
        );
        assert_eq!(
            cases.len(),
            eval_rows,
            "ragtruth: rows that carry the eval schema must all build (skipped \
             rows would silently shrink the scored population)"
        );
        let _ = built_eval;
    } else {
        assert!(
            !cases.is_empty(),
            "ragtruth: the cal/pool front produced no cases — the train-schema \
             rows failed their parse; refusing a silent empty cal slice"
        );
    }
    Suite {
        name: "ragtruth",
        cases,
        option_counts_note: "2 fixed options (grounded / hallucinated); the board metric is the hallucinated-class F1 (HardMetrics.positive_f1)",
    }
}

/// The EVAL-schema gold: `score` ∈ {0, 1} names the class directly
/// (1 = ≥1 annotated span). None = not an eval-schema row.
fn eval_gold(row: &Value) -> Option<usize> {
    match row_i64(row, "score") {
        Some(0) => Some(0),
        Some(1) => Some(1),
        _ => None,
    }
}

/// The POOL/CAL-schema gold: the processed hallucination-label counts (any
/// > 0 = the "hallucinated" class). None = not a pool-schema row.
fn pool_gold(row: &Value) -> Option<usize> {
    let counts = row
        .get("hallucination_labels_processed")
        .and_then(Value::as_object)?;
    let any_span = counts.values().any(|v| v.as_i64().is_some_and(|n| n > 0));
    Some(usize::from(any_span))
}

/// Issue 114 T8 — the FinEntity entity-given sentiment universe
/// (sorted unique upstream labels, the CLINC150 mechanical-from-fetched
/// law; neither authoritative surface — the authors' readme nor the HF
/// card — pins a key order). One label drifted off this set fails loud in
/// the builder.
pub(crate) const FINENTITY_KEYS: [&str; 3] = ["Negative", "Neutral", "Positive"];

/// Issue 114 T8 — FinEntity (board id 39; the language rung's second
/// suite, the first probe to pass the wave-3 residue: the authors' own
/// readme at the board's pinned sha states "FinEntity is licensed under
/// ODC-BY" and the authors' own HF card tags `license:odc-by` — two
/// authoritative surfaces agreeing; the StartLux NC precedent does not
/// bite an attribution licence). The board row is B_mechanical:
/// "entity-given sentiment — the entity span is PROVIDED, the model
/// judges the sentiment expressed toward it" (macro-F1, board chance
/// 0.3201 Monte Carlo; our analytic random-choice macro-F1 over the same
/// population is 0.32016 — Δ 3e-5, MC noise ✓). Each upstream document
/// (Tang et al., EMNLP 2023 main.956) carries `content` + `annotations`
/// (entity spans with `value`/`start`/`end`/`label`/`tag`; label ≡ tag on
/// every row). The faithful case = ONE question per annotated SPAN: state
/// = {text, entity, start, end} (the span provided, the offsets carried),
/// options = the 3 sentiment keys, gold = the span's label position. The
/// metric frame this pins is span-level macro-F1 — the frame the board's
/// own chance arithmetic implies (a per-document 979-request frame does
/// NOT reproduce 0.3201 under any selection rule: first-annotation reads
/// 0.3178, plurality 0.3158 — both >10× the MC noise; the catalog's
/// `requests=979` bookkeeping is the board's own, disclosed here, never
/// inherited).
///
/// THE QUARANTINE (the board's own disclosed rule, census 2026-11-12):
/// within one document the same exact (start, end) span carries TWO
/// labels exactly once — doc 299's "S&P 500" (Neutral + Positive), an
/// unresolved annotation conflict. BOTH rows drop (never a guessed
/// pick), giving 2,131 built − 2 = **2,129 cases = the board's own
/// `fields` count** — the reconciliation that pins the shape. Same-label
/// exact duplicates STAY (two pairs — BlackRock, Microsoft — the board
/// kept them; `fields` counts annotations; the third duplicated span IS
/// the quarantined conflict). The census asserts (2,131 seen / exactly
/// one conflicting span / 2,129 built) are the drift guard — a partial
/// or revised fetch reds at build, never silently.
///
/// CORPUS-OPTIONAL (the T3 decision tree): the source is
/// evaluation-only unsplit (the board's own disclosure — "not an
/// invented official test split"; the authors' HF dataloader carries the
/// same 979 documents as one `train`-named split), so the spans and
/// their labels ARE the test rows' content (grammar-not-answers) —
/// `modelless_lane: false`.
#[must_use]
pub fn build_finentity(rows_file: &Value, max_rows: usize) -> Suite {
    let keys: Vec<String> = FINENTITY_KEYS.iter().map(|k| (*k).to_string()).collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    let mut seen_annotations = 0usize;
    let mut conflicting_spans = 0usize;
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(content), Some(annotations)) = (
            row_str(row, "content"),
            row.get("annotations").and_then(Value::as_array),
        ) else {
            continue; // malformed doc — fails the rule, the doc drops
        };
        // The conflicting-exact-span quarantine: within one document, a
        // (start, end) span labelled two ways is an unresolved annotation
        // conflict — BOTH its rows drop (never a guessed pick). Census
        // 2026-11-12 pins exactly ONE such span in the fetched population.
        let mut labels_by_span: HashMap<(i64, i64), &str> = HashMap::new();
        let mut conflicting: Vec<(i64, i64)> = Vec::new();
        for ann in annotations {
            let (Some(start), Some(end), Some(label)) = (
                row_i64(ann, "start"),
                row_i64(ann, "end"),
                row_str(ann, "label"),
            ) else {
                continue;
            };
            match labels_by_span.get(&(start, end)) {
                Some(prev) if *prev != label => {
                    if !conflicting.contains(&(start, end)) {
                        conflicting.push((start, end));
                        conflicting_spans += 1;
                    }
                }
                _ => {
                    labels_by_span.insert((start, end), label);
                }
            }
        }
        for (i, ann) in annotations.iter().enumerate() {
            let (Some(start), Some(end), Some(label)) = (
                row_i64(ann, "start"),
                row_i64(ann, "end"),
                row_str(ann, "label"),
            ) else {
                continue; // malformed annotation — fails the rule, drops
            };
            seen_annotations += 1;
            if conflicting.contains(&(start, end)) {
                continue; // the quarantined conflict — both rows drop
            }
            let Some(gold_idx) = keys.iter().position(|k| k == label) else {
                panic!(
                    "finentity: label {label:?} (doc {pos} span {i}) is outside \
                     FINENTITY_KEYS — the fetched universe drifted from the \
                     pinned three-label sentiment set"
                );
            };
            let entity = ann.get("value").and_then(Value::as_str).unwrap_or("");
            cases.push(SuiteCase {
                id: format!("finentity:{pos}:{i}"),
                state: json!({
                    "text": content,
                    "entity": entity,
                    "start": start,
                    "end": end,
                }),
                questions: vec![choice_q(
                    "sentiment",
                    "What sentiment does the document in `text` express toward the \
                     entity named in `entity` (its span is characters `start`..`end` \
                     of `text`)?",
                    &keys,
                    None,
                )],
                gold: vec![GoldAnswer {
                    idx: gold_idx,
                    soft: vec![0.0; FINENTITY_KEYS.len()],
                    gold_score: None,
                }],
            });
        }
    }
    // The census asserts bind the FULL fetched population (the eval
    // posture loads every document; `max_rows` is 0 in the eval path and
    // the prepare pin rests on these). A partial test envelope carries a
    // nonzero cap and skips them.
    if max_rows == 0 {
        assert_eq!(
            seen_annotations, 2_131,
            "finentity: the fetched annotation census drifted from the pinned \
             2,131 upstream spans — a partial or revised fetch; refetch via \
             scripts/fetch_datasets.sh and re-pin the manifest digest"
        );
        assert_eq!(
            conflicting_spans, 1,
            "finentity: {} conflicting exact spans (pinned exactly ONE — doc \
             299's S&P 500, Neutral + Positive); a changed upstream census \
             must be re-pinned here with the drift disclosed",
            conflicting_spans
        );
        assert_eq!(
            cases.len(),
            2_129,
            "finentity: the built population drifted from the board's fields \
             count (2,131 spans − 2 quarantined conflict rows = 2,129) — a \
             drifted fetch or quarantine rule"
        );
    }
    Suite {
        name: "finentity",
        cases,
        option_counts_note: "3 fixed sentiment options (Negative / Neutral / Positive, \
                             sorted unique upstream labels); one case per annotated \
                             entity span (2,131 − 2 quarantined conflict rows = 2,129); \
                             the board metric is span-level macro-F1 (chance 0.3201 \
                             Monte Carlo, analytic 0.32016); corpus-optional (eval-only \
                             unsplit source — grammar-not-answers)",
    }
}

/// Issue 114 T9 — the ContractNLI entailment universe (sorted unique
/// upstream `choice` spellings — the CLINC150 mechanical-from-fetched law;
/// the board's panel words supported/contradicted/unstated are DISPLAY
/// names, never the wire keys).
pub(crate) const CONTRACTNLI_KEYS: [&str; 3] = ["Contradiction", "Entailment", "NotMentioned"];

/// Issue 114 T9 — ContractNLI (board id 11; the language rung's third
/// suite; the T8-note's licence conflict RESOLVED upstream 2026-11-12:
/// the authors' canonical repo `stanfordnlp/contract-nli` (gh-pages) —
/// the T8-note's "theainswer" spelling 404s — carries the full CC-BY-4.0
/// LICENSE, the bundled TERMS grants rights "in accordance with the terms
/// and conditions of the Creative Commons Attribution 4.0 International
/// Public License", and the dataset site's §License names it; the
/// kiddothe2b HF mirror's cc-by-nc-sa-4.0 tag is a third-party curator's
/// CONTRADICTION of the upstream and is never a source — the
/// flowaicom-witness pattern).
///
/// The task (Koreeda & Manning, EMNLI 2021): contract + hypothesis →
/// the inference relation. A_direct on the board ("original hypotheses
/// and NLI labels"). The upstream test.json shape: 123 documents, each
/// carrying ONE annotation set of 17 per-hypothesis annotations keyed
/// `nda-N` — 2,091 (document, hypothesis) pairs = the board's own
/// `fields` count (the reconciliation, the finentity pattern). Choices
/// upstream: Entailment 968 / NotMentioned 903 / Contradiction 220. The
/// board metric is pair-level macro-F1 — the frame its own chance
/// arithmetic pins (analytic random-choice macro-F1 over this
/// distribution = 0.30793 vs the catalog's Monte Carlo 0.30852, Δ 6e-4
/// = MC noise at the 4.6/43.2/10.5% class split; the catalog's
/// `requests=123` documents bookkeeping repeats the finentity note —
/// disclosed, never inherited).
///
/// THE FETCH TRANSFORM (the recorded one — the digest pins the OUTPUT
/// envelopes): the authors' zip (`resources/contract-nli.zip` at the
/// board's pinned sha `eced652`) carries nested per-document JSON; the
/// fetch flattens BOTH splits to the pair rows the builder reads:
/// `{contract, hypothesis, choice, split}` (test 2,091 rows; train 423
/// documents × 17 = 7,191 rows). ONE row schema reaches this builder —
/// the `split` column tells eval from pool (the ragtruth schema
/// dispatch, replaced by an explicit column; the eval population assert
/// keys on every row carrying split=test).
///
/// CORPUS-ON (the wave-3 posture): the upstream TRAIN split is the
/// task's own released training data (the SemEval-2024 paper documents
/// systems fine-tuning on it — the interventions were withheld from
/// training only, and the interventions are NOT part of the board's
/// track: the track is the original 2,091 test pairs). The corpus doc
/// text = contract + hypothesis (the contract is exactly what flips the
/// label — 17 hypotheses repeat across documents, so the hypothesis
/// alone cannot carry the signal; the ragtruth response-alone rule does
/// NOT transfer, disclosed here). corpus_cap_per_label 40 → ≤120 docs
/// (~1–2 MB of contract text), cal_cap 200 at the clinc150 ratio.
#[must_use]
pub fn build_contractnli(rows_file: &Value, max_rows: usize) -> Suite {
    let keys: Vec<String> = CONTRACTNLI_KEYS.iter().map(|k| (*k).to_string()).collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(contract), Some(hypothesis), Some(choice)) = (
            row_str(row, "contract"),
            row_str(row, "hypothesis"),
            row_str(row, "choice"),
        ) else {
            continue; // malformed row — fails the rule, the row drops
        };
        let Some(gold_idx) = keys.iter().position(|k| k == choice) else {
            panic!(
                "contractnli: choice {choice:?} (row {pos}) is outside \
                 CONTRACTNLI_KEYS — the fetched universe drifted from the \
                 pinned three-label entailment set"
            );
        };
        cases.push(SuiteCase {
            id: format!("contractnli:{pos}"),
            state: json!({
                "contract": contract,
                "hypothesis": hypothesis,
            }),
            questions: vec![choice_q(
                "inference",
                "Does the NDA in `contract` entail, contradict, or not mention the \
                 claim in `hypothesis`?",
                &keys,
                None,
            )],
            gold: vec![GoldAnswer {
                idx: gold_idx,
                soft: vec![0.0; CONTRACTNLI_KEYS.len()],
                gold_score: None,
            }],
        });
    }
    // The eval population assert binds the FULL fetched test envelope
    // (the eval posture loads every pair; `max_rows` is 0 in the eval
    // path and the prepare pin rests on this). A partial test envelope
    // (the cal front reads the TRAIN envelope) carries a nonzero cap or
    // train rows and skips the census — the split column is the eval
    // discriminator (the ragtruth schema-dispatch law, explicit form).
    let eval_rows = all
        .iter()
        .filter(|r| row_str(r, "split") == Some("test"))
        .count();
    if max_rows == 0 && eval_rows == all.len() && eval_rows > 0 {
        assert_eq!(
            cases.len(),
            2_091,
            "contractnli: the fetched test population drifted from the board's \
             fields count (123 documents × 17 hypotheses = 2,091 pairs) — a \
             partial or revised fetch; refetch via scripts/fetch_datasets.sh \
             and re-pin the manifest digest"
        );
    }
    Suite {
        name: "contractnli",
        cases,
        option_counts_note: "3 fixed entailment options (Contradiction / Entailment / \
                             NotMentioned, sorted unique upstream choices); one case per \
                             (document, hypothesis) pair (123 × 17 = 2,091 = the board's \
                             fields count); the board metric is pair-level macro-F1 (chance \
                             0.3085 Monte Carlo, analytic 0.30793); corpus-ON (the upstream \
                             train split is the task's own released training data)",
    }
}

/// Issue 114 T10 — the iSarcasmEval binary universe (sorted upstream
/// spellings — the mechanical-from-fetched law). The board's index metric
/// is track-A-English sarcastic-class F1, and `hard_metrics::positive_f1`
/// keys the LAST option (index 1) for all-2-option rows — so the sorted
/// order ["non-sarcastic", "sarcastic"] lands the sarcastic class exactly
/// where the metric reads it.
pub(crate) const ISARCASM_KEYS: [&str; 2] = ["non-sarcastic", "sarcastic"];

/// Issue 114 T10 — iSarcasmEval (board id 40; the language rung's fourth
/// suite). The board's suite.md pins `iabufarha/iSarcasmEval @dfc708b,
/// test/task_{A,B,C}_{En,Ar}_test.csv`, "All prepared cases retained",
/// cases 4,600 — the five test tracks' combined count. LICENCE (read live
/// 2026-11-12 at the board's pinned tree sha `dfc708b…`): the authors'
/// canonical repo carries the full MIT LICENSE (1,074 bytes, "Copyright
/// (c) 2022 Ibrahim Abu Farha") and GitHub's API independently reports
/// spdx_id=MIT; the recursive tree shows no separate data-terms file —
/// the CSV data dirs ride the repo licence (the FinEntity two-surface
/// PASS shape).
///
/// THE SCOPE (the triage's sanctioned partial, narrowed by the metric
/// frame): this suite carries TRACK A ENGLISH ONLY — the board row's own
/// index metric and chance (0.2227) are defined on exactly that track
/// ("track A English: F1 of random guessing on the sarcastic class"), so
/// the suite's positive_f1 (binary, index 1) IS the board number, read
/// straight off the aggregate. The population pin is the fetched A_En
/// census (1,400 tweets: 200 sarcastic / 1,200 non-sarcastic — the 1-in-7
/// share the board's chance arithmetic pins), NOT the catalog's 4,600:
/// that count books all five test tracks (A_En 1,400 + A_Ar + B_En +
/// C_En + C_Ar ≈ 3,200 more) — the contractnli/finentity reconciliation
/// law (the catalog's bookkeeping is disclosed, never inherited). THE
/// RE-OPENS, recorded loud: track C (pair pick — shape-fine plain choice,
/// but its second option universe would break the one-suite-one-universe
/// router law and dilute the binary positive_f1 the board scores), track
/// B (the six-label multilabel — needs the wave-2 multi-question
/// envelope), Arabic tracks (the triage's English-first posture
/// decision).
///
/// THE FETCH TRANSFORM (the recorded one): the task-A CSVs at the pinned
/// sha — test `text,sarcastic`; train the 10-column file whose tweet +
/// sarcastic columns are read — project to `{text, sarcastic, split}`
/// rows (the fetch's csv DictReader program; embedded commas/newlines
/// handled by the csv module). Upstream test census: 1,400 rows / 200
/// sarcastic. Upstream train census: 3,468 rows, of which ONE (row 1062)
/// carries an empty tweet — the converter's drop rule fails it; the
/// fetched train envelope is 3,467 (867 sarcastic / 2,600 non).
///
/// CORPUS-ON (the wave-3 posture): the upstream TRAIN split is the
/// task's own released training data (the shared task ran on it). Corpus
/// doc text = the tweet alone (the label IS the tweet's own author label
/// — the ragtruth response-alone rule transfers: nothing else carries
/// the signal). corpus_cap_per_label 200 (the pool wants both classes
/// well-heat — 867/2,600 upstream), cal_cap 200, named_only like every
/// ladder suite.
#[must_use]
pub fn build_isarcasmeval(rows_file: &Value, max_rows: usize) -> Suite {
    let keys: Vec<String> = ISARCASM_KEYS.iter().map(|k| (*k).to_string()).collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(sarcastic)) = (row_str(row, "text"), row_str(row, "sarcastic"))
        else {
            continue; // malformed row — fails the rule, the row drops
        };
        let gold_idx = match sarcastic {
            "0" => 0,
            "1" => 1,
            other => panic!(
                "isarcasmeval: sarcastic label {other:?} (row {pos}) is outside \
                 ISARCASM_KEYS — the fetched universe drifted from the \
                 pinned binary track-A set",
            ),
        };
        cases.push(SuiteCase {
            id: format!("isarcasmeval:{pos}"),
            state: json!({
                "tweet": text,
            }),
            questions: vec![choice_q(
                "sarcasm",
                "Was the tweet in `tweet` intended to be sarcastic by its author?",
                &keys,
                None,
            )],
            gold: vec![GoldAnswer {
                idx: gold_idx,
                soft: vec![0.0; ISARCASM_KEYS.len()],
                gold_score: None,
            }],
        });
    }
    // The population pin binds the FULL fetched test envelope (the eval
    // posture loads every tweet; `max_rows` is 0 there). A partial test
    // envelope (the cal front reads the TRAIN envelope) skips the census —
    // the split column is the eval discriminator (the contractnli form).
    let eval_rows = all
        .iter()
        .filter(|r| row_str(r, "split") == Some("test"))
        .count();
    if max_rows == 0 && eval_rows == all.len() && eval_rows > 0 {
        assert_eq!(
            cases.len(),
            1_400,
            "isarcasmeval: the fetched track-A-English test population drifted \
             from the pinned census (1,400 tweets: 200 sarcastic / 1,200 \
             non-sarcastic — the 1-in-7 share the board's chance 0.2227 pins) \
             — a partial or revised fetch; refetch via scripts/fetch_datasets.sh \
             and re-pin the manifest digest"
        );
    }
    Suite {
        name: "isarcasmeval",
        cases,
        option_counts_note: "2 fixed track-A options (non-sarcastic / sarcastic, sorted \
                             upstream 0/1 spellings); one case per test tweet (1,400 = \
                             the fetched track-A-English census: 200 sarcastic / 1,200 \
                             non; the board catalog's 4,600 books all five test tracks — \
                             disclosed, never inherited); the board metric is \
                             sarcastic-class F1 (chance 0.2227 Monte Carlo, analytic \
                             random-guess 0.2222), read straight off positive_f1 (index 1); \
                             corpus-ON (the upstream train split is the task's own \
                             released training data)",
    }
}

/// Issue 114 T11 — the JDI ladder wave-2 machinery's first consumer (board
/// row 9, the home-appliance simulator; MIT, board-generated, seed
/// 2026091807). One case per kept household request (88 — the release-v2.1
/// subset's pinned kept set; the fetch lane verifies the pin and the
/// regeneration census), each carrying ~18 choice questions (1,590 total)
/// whose option KEYS are opaque `k_<hash>` tokens and whose option TEXTS
/// ride the criteria values (the engine renders `{key}: {value}` —
/// render_options; the texts are the signal). The board's index metric is
/// CASE EXACT ACCURACY (a request passes iff every question's answer is
/// exact; chance 2.1e-07 exact expectation) — read off the runner's
/// `case_exact` column ([`crate::harness::metrics::case_exact_metrics`]),
/// never the per-question accuracy `hard` carries.
///
/// The gold is the criteria KEY the board's `expected` names — mapped to
/// its position in the criteria object's insertion order (serde_json
/// `preserve_order`; python json round-trips dict order — the pair IS the
/// label order the engine renders and scores). CORPUS-OPTIONAL
/// (`modelless_lane: false`): the board publishes no train/dev pool for
/// the track (the dev split is the DEDUPE reference, never a corpus —
/// grammar-not-answers), so only the LLM lanes answer.
#[must_use]
pub fn build_home_appliance(rows_file: &Value, max_rows: usize) -> Suite {
    const EXPECTED_CASES: usize = 88;
    const EXPECTED_QUESTIONS: usize = 1_590;
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    let mut total_questions = 0usize;
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(id), Some(state), Some(questions), Some(expected)) = (
            row_str(row, "id"),
            row.get("state"),
            row.get("questions"),
            row.get("expected"),
        ) else {
            continue; // malformed row — fails the rule, the row drops
        };
        let Some(qmap) = questions.as_object() else {
            panic!(
                "home_appliance: row {pos} ({id}) carries non-object questions — \
                 the fetched envelope drifted from the board's row shape"
            );
        };
        let Some(emap) = expected.as_object() else {
            panic!(
                "home_appliance: row {pos} ({id}) carries non-object expected — \
                 the fetched envelope drifted from the board's row shape"
            );
        };
        if qmap.is_empty() {
            continue;
        }
        let mut qs = Vec::with_capacity(qmap.len());
        let mut gold = Vec::with_capacity(qmap.len());
        for (qid, qdef) in qmap {
            let kind = match qdef.get("type").and_then(Value::as_str) {
                Some("choice") => QKind::Choice,
                Some("noul") => QKind::Noul,
                other => panic!(
                    "home_appliance: row {pos} ({id}) question {qid} carries type \
                     {other:?} — the board track is choice/noul only (score was \
                     never adapted); the envelope drifted"
                ),
            };
            let Some(instructions) = qdef.get("instructions").and_then(Value::as_str) else {
                panic!(
                    "home_appliance: row {pos} ({id}) question {qid} carries no \
                     instructions — the envelope drifted"
                );
            };
            let criteria = qdef.get("criteria").cloned().unwrap_or(Value::Null);
            let idx = match kind {
                QKind::Choice => {
                    let Some(ck) = choice_keys(&criteria) else {
                        panic!(
                            "home_appliance: row {pos} ({id}) question {qid} carries \
                             non-object criteria — the envelope drifted"
                        );
                    };
                    let g = emap.get(qid).and_then(Value::as_str);
                    match g.and_then(|g| ck.iter().position(|k| k == g)) {
                        Some(i) => i,
                        _ => panic!(
                            "home_appliance: row {pos} ({id}) question {qid} gold \
                             {g:?} is not a criteria key — the expected/criteria \
                             join drifted"
                        ),
                    }
                }
                QKind::Noul => match emap.get(qid).and_then(Value::as_bool) {
                    Some(b) => usize::from(b), // the [false, true] law
                    _ => panic!(
                        "home_appliance: row {pos} ({id}) question {qid} noul gold \
                         is not a bool — the envelope drifted"
                    ),
                },
                QKind::Score => unreachable!("refused above"),
            };
            let k = crate::harness::runner::question_option_count(&SuiteQuestion {
                qid: qid.clone(),
                kind,
                instructions: instructions.to_string(),
                criteria: criteria.clone(),
            });
            qs.push(SuiteQuestion {
                qid: qid.clone(),
                kind,
                instructions: instructions.to_string(),
                criteria,
            });
            gold.push(GoldAnswer {
                idx,
                soft: vec![0.0; k],
                gold_score: None,
            });
        }
        total_questions += qs.len();
        cases.push(SuiteCase {
            id: format!("home_appliance:{id}"),
            state: state.clone(),
            questions: qs,
            gold,
        });
    }
    // The population pin binds the FULL fetched envelope (max_rows 0 — the
    // eval posture). A partial fetch reds loud instead of misreporting the
    // board row (the when2call form).
    if max_rows == 0 {
        assert_eq!(
            cases.len(),
            EXPECTED_CASES,
            "home_appliance: the fetched population drifted from the pinned \
             kept set (88 requests — the release-v2.1 subset's run_ids pin \
             f76d0ef2) — a partial or revised fetch; refetch via \
             scripts/fetch_datasets.sh and re-pin the manifest digest"
        );
        assert_eq!(
            total_questions, EXPECTED_QUESTIONS,
            "home_appliance: the fetched question census drifted from the \
             pinned 1,590 — the kept rows' fields changed; refetch and \
             re-pin"
        );
    }
    Suite {
        name: "home_appliance",
        cases,
        option_counts_note: "per-case variable option universes (the resolution question \
                             carries k=9, the target-member questions k=2, the outcome/\
                             state/post questions k=3-6; the option KEYS are opaque \
                             k_<hash> tokens, the TEXTS ride the criteria values and are \
                             what the engine renders); one case per kept request (88 = \
                             the release-v2.1 subset's pinned kept set, 1,590 questions); \
                             the board metric is CASE EXACT ACCURACY (chance 2.103e-07 \
                             exact expectation) read off the case_exact column; \
                             corpus-optional (the board publishes no train pool — the dev \
                             split is the dedupe reference, never a corpus)",
    }
}

/// Issue 114 T4 — the BFCL abstain option, APPENDED LAST on the
/// irrelevance-tier rows only (the board row's "or irrelevance-detect";
/// the pick tier never carries it — the official pick eval always has a
/// gold function, and adding a never-gold distractor would be an invented
/// transform). The option universe deliberately exceeds any fixed set —
/// per-row candidate-function names (the s1mb variable-breadth posture,
/// `coverage_audit: false`).
pub(crate) const BFCL_ABSTAIN_KEY: &str = "no_appropriate_function";

/// Issue 114 T4 — the BFCL pick tier's gold resolution law, the official
/// ast evals' own name-matching semantics: the gold name from the
/// possible_answer file matches a candidate by FULL name first, else by
/// LEAF name (the segment after the last `.` — simple_363's gold
/// `find_closest` vs the candidate's module-qualified
/// `restaurant_search.find_closest`, the one such row in the fetched
/// population, census 2026-11-10). A leaf match must be UNAMBIGUOUS (one
/// candidate) — anything else is a fetch drift and fails loud, naming the
/// row id.
fn bfcl_gold_index(candidates: &[&str], gold: &str, row_id: &str) -> usize {
    if let Some(pos) = candidates.iter().position(|c| *c == gold) {
        return pos;
    }
    let gold_leaf = gold.rsplit('.').next().unwrap_or(gold);
    let mut hits = candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| c.rsplit('.').next().unwrap_or(c) == gold_leaf)
        .map(|(p, _)| p);
    match (hits.next(), hits.next()) {
        (Some(p), None) => p,
        _ => panic!(
            "bfcl: gold {gold:?} for {row_id} matches no candidate by full or \
             unique leaf name — the possible_answer join drifted (refetch via \
             scripts/fetch_datasets.sh; the census pins this to zero besides \
             the pinned simple_363 leaf row)"
        ),
    }
}

/// Issue 114 T4 — BFCL (`gorilla-llm/Berkeley-Function-Calling-Leaderboard`,
/// the wave-1 scope): the board's tools-rung pick row — "given a query +
/// tool schemas, pick the function to call (or irrelevance-detect)". The
/// eval surface is the CONCATENATION of the six fetched files (the runner's
/// comma-list split order: simple, multiple, live_simple, live_multiple,
/// then the two irrelevance tiers); rows are task-faithful as stored:
/// state = {query: the official `question` structure verbatim, tools: the
/// official `function` spec array verbatim}. PICK rows (id prefix
/// simple/multiple/live_simple/live_multiple): options = the row's own
/// candidate-function names, criteria = each spec's `description` string;
/// gold = the joined `ground_truth` entry's function name via
/// [`bfcl_gold_index`]. IRRELEVANCE rows (id prefix irrelevance/
/// live_irrelevance): the same candidates PLUS the fixed abstain option
/// appended LAST ([`BFCL_ABSTAIN_KEY`]) — gold is that option; rows whose
/// candidate list is EMPTY are skipped (a one-option choice is no choice —
/// 4 such rows in the live_irrelevance tier, census 2026-11-10, recorded
/// in the manifest row).
///
/// CORPUS-OPTIONAL: no train split exists in the mirror and the candidate
/// functions ARE the test rows' content (grammar-not-answers) — the
/// modelless lane is skipped loud (`modelless_lane: false` in the
/// registry); the corpus re-open is the rethink-side synthesis route.
#[must_use]
pub fn build_bfcl(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(id), Some(question), Some(functions)) = (
            row_str(row, "id"),
            row.get("question"),
            row.get("function").and_then(Value::as_array),
        ) else {
            continue; // malformed row — fails the rule, the row drops
        };
        let is_irrelevance = id.starts_with("irrelevance_") || id.starts_with("live_irrelevance_");
        let mut names: Vec<&str> = Vec::with_capacity(functions.len());
        let mut descs: Vec<Option<String>> = Vec::with_capacity(functions.len());
        for f in functions {
            let Some(name) = f.get("name").and_then(Value::as_str) else {
                continue;
            };
            names.push(name);
            descs.push(
                f.get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            );
        }
        if names.is_empty() {
            // The irrelevance tier's 4 pinned empty-candidate rows (census
            // 2026-11-10) and any malformed pick row drop here; the
            // population + tier pins in `prepare` are the loud guard.
            continue;
        }
        let gold_idx = if is_irrelevance {
            names.len() // the abstain option, appended last below
        } else {
            let ground_truth = row
                .get("ground_truth")
                .and_then(Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "bfcl: pick row {id} carries no joined ground_truth — \
                            refetch via scripts/fetch_datasets.sh (the gold join \
                            is all-or-nothing)"
                    )
                });
            if ground_truth.len() != 1 {
                panic!(
                    "bfcl: pick row {id} carries {} ground_truth entries (the \
                     fetched census pins exactly 1 per row)",
                    ground_truth.len()
                );
            }
            let entry = &ground_truth[0];
            let gold = entry
                .as_object()
                .and_then(|o| o.keys().next())
                .unwrap_or_else(|| {
                    panic!("bfcl: pick row {id} carries an empty ground_truth entry")
                });
            bfcl_gold_index(&names, gold, id)
        };
        let mut keys: Vec<String> = names.iter().map(|n| (*n).to_string()).collect();
        if is_irrelevance {
            keys.push(BFCL_ABSTAIN_KEY.to_string());
            descs.push(Some(
                "None of the available functions is appropriate for this query".to_string(),
            ));
        }
        cases.push(SuiteCase {
            id: format!("bfcl:{pos}"),
            state: json!({ "query": question, "tools": functions }),
            questions: vec![choice_q(
                "function",
                "Which function should be called for `query`, given the available \
                 `tools`?",
                &keys,
                Some(descs),
            )],
            gold: vec![GoldAnswer {
                idx: gold_idx,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "bfcl",
        cases,
        option_counts_note: "per-row candidate-function names (the pick tier's option \
                             universe; up to 37 candidates on live_multiple); the \
                             irrelevance tier appends the fixed abstain option LAST and \
                             golds it; 4 empty-candidate live_irrelevance rows skipped \
                             (census 2026-11-10); corpus-optional (no lawful train \
                             pool — grammar-not-answers)",
    }
}

/// Issue 114 T5 — the API-Bank which-API tier
/// (`liminghao1630/API-Bank`, test-data/level-1-api.json + level-2-api.json,
/// fetched whole): "API-call selection in a dialogue" — the board row's
/// which-API half (multi-turn state carry refused wave 1; the level-3 and
/// response files are out of scope). Each row's `instruction` embeds the
/// dialogue context and a `API descriptions:` block of newline-separated
/// JSON specs — the row's own candidate set; the gold API name rides the
/// row's `expected_output` (`API-Request: [Name(...)` prefix, parsed per
/// row — the recorded transform). ROW-LEVEL SCOREABLE-PARTIAL (the issue's
/// R2b law, census 2026-11-10): every level-1 row's gold IS among its own
/// candidates (399/399 — 1–4 candidates each); the level-2 rows are
/// ToolSearcher-gated (the dialogue's next step is the tool search) and
/// present exactly ONE candidate — the 45 rows whose expected_output names
/// that candidate are choice-shaped and kept, the 90 whose expected_output
/// names the DOWNSTREAM API the search would return are NOT answerable as
/// a choice and drop (the official eval is generation-based there — never
/// forced into the pick wire). Options = the parsed candidate API names,
/// criteria = each spec's `description`; state = {instruction: the row's
/// instruction VERBATIM (dialogue + candidate specs AS STORED)}. A row
/// whose gold parses but matches no candidate drops (the prepare census
/// pin is the loud guard: 399 + 45 = 444).
///
/// CORPUS-OPTIONAL (the T3 decision tree): the candidate API descriptions
/// ARE the test rows' content (grammar-not-answers); the authors' own
/// -api-train files overlap the same catalog and are never fetched —
/// `modelless_lane: false`.
#[must_use]
pub fn build_api_bank(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(instruction), Some(expected)) =
            (row_str(row, "instruction"), row_str(row, "expected_output"))
        else {
            continue; // malformed row — fails the rule, the row drops
        };
        // Gold: the API name in the row's own expected_output — the
        // recorded parse ("API-Request: [Name(...)"). A row without the
        // prefix drops (census pins 534/534 carrying it).
        let Some(gold) = expected
            .strip_prefix("API-Request: [")
            .and_then(|rest| rest.split('(').next())
            .filter(|name| !name.is_empty())
        else {
            continue;
        };
        // Candidates: the JSON spec lines after the instruction's
        // "API descriptions:" marker (the recorded parse — the specs are
        // newline-separated JSON objects there).
        let Some(spec_block) = instruction.split("API descriptions:").nth(1) else {
            continue;
        };
        let mut names: Vec<String> = Vec::new();
        let mut descs: Vec<Option<String>> = Vec::new();
        for line in spec_block.lines() {
            let line = line.trim();
            if !line.starts_with("{\"name\"") {
                continue;
            }
            let Ok(spec) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            let Some(name) = spec.get("name").and_then(Value::as_str) else {
                continue;
            };
            names.push(name.to_string());
            descs.push(
                spec.get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            );
        }
        if names.is_empty() {
            continue; // no candidates parsed — no honest choice to present
        }
        let Some(gold_idx) = names.iter().position(|n| n == gold) else {
            // The row-level scoreable-partial drop: the level-2
            // ToolSearcher-gated rows whose expected_output names the
            // DOWNSTREAM API are not answerable as a choice from the
            // presented options (the official eval is generation-based
            // there). The prepare census pin (444) is the loud guard.
            continue;
        };
        let keys: Vec<String> = names.iter().map(|n| (*n).to_string()).collect();
        cases.push(SuiteCase {
            id: format!("api_bank:{pos}"),
            state: json!({ "instruction": instruction }),
            questions: vec![choice_q(
                "api",
                "Which API should be called next in the dialogue described by \
                 `instruction`?",
                &keys,
                Some(descs),
            )],
            gold: vec![GoldAnswer {
                idx: gold_idx,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "api_bank",
        cases,
        option_counts_note: "per-row candidate API names parsed from the instruction's \
                             `API descriptions:` block (1-4 on level-1; exactly \
                             ToolSearcher on the kept level-2 rows); the gold rides the \
                             row's own expected_output; 90 level-2 rows dropped \
                             (downstream-API golds — the generation-shaped slice, the \
                             R2b scoreable-partial law); corpus-optional (the candidate \
                             catalog IS the test content — grammar-not-answers)",
    }
}

/// Issue 114 T2 — the CLINC150 intent vocabulary (151: 150 intents + `oos`,
/// ClassLabel order 0..=150, identical across every clinc/clinc_oos config —
/// verified against the card's dataset_info blocks 2026-11-09; the builder
/// re-verifies at port from the fetched envelope's features). Underscores
/// stripped to the house option-key spelling (the ag_news law). `oos` rides
/// at its ClassLabel position — the board row is "150-intent (+ OOS)", the
/// abstention-flavoured gold is a REGULAR option here, never a wire special.
pub(crate) const CLINC150_KEYS: [&str; 151] = [
    "restaurant reviews",
    "nutrition info",
    "account blocked",
    "oil change how",
    "time",
    "weather",
    "redeem rewards",
    "interest rate",
    "gas type",
    "accept reservations",
    "smart home",
    "user name",
    "report lost card",
    "repeat",
    "whisper mode",
    "what are your hobbies",
    "order",
    "jump start",
    "schedule meeting",
    "meeting schedule",
    "freeze account",
    "what song",
    "meaning of life",
    "restaurant reservation",
    "traffic",
    "make call",
    "text",
    "bill balance",
    "improve credit score",
    "change language",
    "no",
    "measurement conversion",
    "timer",
    "flip coin",
    "do you have pets",
    "balance",
    "tell joke",
    "last maintenance",
    "exchange rate",
    "uber",
    "car rental",
    "credit limit",
    "oos",
    "shopping list",
    "expiration date",
    "routing",
    "meal suggestion",
    "tire change",
    "todo list",
    "card declined",
    "rewards balance",
    "change accent",
    "vaccines",
    "reminder update",
    "food last",
    "change ai name",
    "bill due",
    "who do you work for",
    "share location",
    "international visa",
    "calendar",
    "translate",
    "carry on",
    "book flight",
    "insurance change",
    "todo list update",
    "timezone",
    "cancel reservation",
    "transactions",
    "credit score",
    "report fraud",
    "spending history",
    "directions",
    "spelling",
    "insurance",
    "what is your name",
    "reminder",
    "where are you from",
    "distance",
    "payday",
    "flight status",
    "find phone",
    "greeting",
    "alarm",
    "order status",
    "confirm reservation",
    "cook time",
    "damaged card",
    "reset settings",
    "pin change",
    "replacement card duration",
    "new card",
    "roll dice",
    "income",
    "taxes",
    "date",
    "who made you",
    "pto request",
    "tire pressure",
    "how old are you",
    "rollover 401k",
    "pto request status",
    "how busy",
    "application status",
    "recipe",
    "calendar update",
    "play music",
    "yes",
    "direct deposit",
    "credit limit change",
    "gas",
    "pay bill",
    "ingredients list",
    "lost luggage",
    "goodbye",
    "what can i ask you",
    "book hotel",
    "are you a bot",
    "next song",
    "change speed",
    "plug type",
    "maybe",
    "w2",
    "oil change when",
    "thank you",
    "shopping list update",
    "pto balance",
    "order checks",
    "travel alert",
    "fun fact",
    "sync device",
    "schedule maintenance",
    "apr",
    "transfer",
    "ingredient substitution",
    "calories",
    "current location",
    "international fees",
    "calculator",
    "definition",
    "next holiday",
    "update playlist",
    "mpg",
    "min payment",
    "change user name",
    "restaurant suggestion",
    "travel notification",
    "cancel",
    "pto used",
    "travel suggestion",
    "change volume",
];

/// §3.8 AG News — choice, 4 FIXED options (never shuffled); the criteria
/// key order IS the dataset ClassLabel order (VERIFY-AT-PORT in the fetch
/// layer; the keys themselves live in [`AG_NEWS_KEYS`]).
#[must_use]
pub fn build_ag_news(rows_file: &Value, max_rows: usize) -> Suite {
    const CRIT: [(&str, &str); 4] = [
        ("world", "world news and international politics"),
        ("sports", "sports"),
        ("business", "business and economy"),
        ("sci_tech", "science and technology"),
    ];
    debug_assert_eq!(
        CRIT.len(),
        AG_NEWS_KEYS.len(),
        "ag_news key consts must stay in sync"
    );
    let keys: Vec<String> = CRIT.iter().map(|(k, _)| (*k).to_string()).collect();
    let descs: Vec<Option<String>> = CRIT.iter().map(|(_, d)| Some((*d).to_string())).collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("ag_news:{pos}"),
            state: json!({ "article": text }),
            questions: vec![choice_q(
                "topic",
                "What is the topic of `article`?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 4],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "ag_news",
        cases,
        option_counts_note: "4 fixed options, never shuffled",
    }
}

/// §3.5 DAIR Emotion (config "split") — choice, 6 fixed options; the names
/// order must equal the dataset ClassLabel order (VERIFY-AT-PORT).
#[must_use]
pub fn build_emotion(rows_file: &Value, max_rows: usize) -> Suite {
    build_emotion_as("emotion", rows_file, max_rows)
}

/// `emotion_syn` — the first-party synthetic emotion suite (rethink Research
/// 002 §Execution step 1, the licence-clean re-entry for the barred dair-ai
/// niche): the SAME six-key wire and ClassLabel order as [`build_emotion`],
/// rows generated by `scripts/synth_suite.py` (train = Bonsai-27B
/// over prompt family A, test = Qwen3.5-4B over the disjoint family B — the
/// outside-teacher/outside-prompt-family holdout; every row dual-model
/// label-confirmed). Apache-2.0, ours.
#[must_use]
pub fn build_emotion_syn(rows_file: &Value, max_rows: usize) -> Suite {
    build_emotion_as("emotion_syn", rows_file, max_rows)
}

fn build_emotion_as(name: &'static str, rows_file: &Value, max_rows: usize) -> Suite {
    const NAMES: [&str; 6] = ["sadness", "joy", "love", "anger", "fear", "surprise"];
    debug_assert_eq!(NAMES, EMOTION_KEYS, "emotion key consts must stay in sync");
    let keys: Vec<String> = NAMES.iter().map(|n| (*n).to_string()).collect();
    let descs: Vec<Option<String>> = vec![None; 6];
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("{name}:{pos}"),
            state: json!({ "text": text }),
            questions: vec![choice_q(
                "emotion",
                "Which emotion is most strongly expressed in `text`?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 6],
                gold_score: None,
            }],
        });
    }
    Suite {
        name,
        cases,
        option_counts_note: "6 fixed options, never shuffled",
    }
}

/// §3.4 SST-5 — score, 5 levels; gold score = the label as f64; no soft in
/// the dataset (zeros).
#[must_use]
pub fn build_sst5(rows_file: &Value, max_rows: usize) -> Suite {
    build_sst5_as("sst5", rows_file, max_rows)
}

/// `sst5_syn` — the first-party synthetic SST-5 suite (rethink Research 002
/// §Execution step 2): the SAME five score levels and int-label wire as
/// [`build_sst5`], rows generated by `scripts/synth_suite.py --suite
/// sst5_syn` (train = Bonsai-27B over review family A, test = Qwen3.5-4B
/// over the disjoint family B; every row dual-model label-confirmed, the
/// judge answering digits 1..5). Apache-2.0, ours.
#[must_use]
pub fn build_sst5_syn(rows_file: &Value, max_rows: usize) -> Suite {
    build_sst5_as("sst5_syn", rows_file, max_rows)
}

fn build_sst5_as(name: &'static str, rows_file: &Value, max_rows: usize) -> Suite {
    const LEVELS: [&str; 5] = [
        "very negative",
        "negative",
        "neutral",
        "positive",
        "very positive",
    ];
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        let criteria = Value::Array(LEVELS.iter().map(|l| json!(l)).collect());
        cases.push(SuiteCase {
            id: format!("{name}:{pos}"),
            state: json!({ "text": text }),
            questions: vec![SuiteQuestion {
                qid: "sentiment".to_string(),
                kind: QKind::Score,
                instructions: "How positive is the sentiment of `text`?".to_string(),
                criteria,
            }],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 5],
                gold_score: Some(label as f64),
            }],
        });
    }
    Suite {
        name,
        cases,
        option_counts_note: "5 fixed score levels (index = level)",
    }
}

/// Extract `{"name": "label", "type": {"names": [...]}}` ClassLabel names
/// from the envelope's features array. All-or-nothing: a non-string entry
/// invalidates the whole list.
fn label_names(file: &Value, col: &str) -> Option<Vec<String>> {
    let features = file.get("features")?.as_array()?;
    for f in features {
        if f.get("name").and_then(Value::as_str) != Some(col) {
            continue;
        }
        let names = f.get("type")?.get("names")?.as_array()?;
        let mut out = Vec::with_capacity(names.len());
        for v in names {
            out.push(v.as_str()?.to_string());
        }
        return Some(out);
    }
    None
}

/// §3.7 Banking77 (the Colab variant) — choice over the dataset's ClassLabel
/// names, underscores stripped. Loudly refuses an envelope without label
/// names (the criteria set cannot be derived from rows alone here).
///
/// # Panics
/// When the envelope's `features` carry no `label` ClassLabel `names`.
#[must_use]
pub fn build_banking77(rows_file: &Value, max_rows: usize) -> Suite {
    let names = label_names(rows_file, "label").unwrap_or_else(|| {
        panic!(
            "banking77: envelope features carry no label ClassLabel names — \
             the /rows fetch must include {{\"name\":\"label\",\"type\":{{\"names\":[...]}}}}"
        )
    });
    let keys: Vec<String> = names.iter().map(|n| n.replace('_', " ")).collect();
    let descs: Vec<Option<String>> = vec![None; keys.len()];
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("banking77:{pos}"),
            state: json!({ "message": text }),
            questions: vec![choice_q(
                "intent",
                "Which banking intent does `message` express?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "banking77",
        cases,
        option_counts_note: "77 fixed options in ClassLabel order (underscores stripped)",
    }
}

/// §3.6 Prompt injections — noul; gold idx 1 = injection. The dataset README
/// semantics (0 benign / 1 injection) are VERIFY-AT-PORT in the fetch layer.
#[must_use]
pub fn build_prompt_injections(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "text"), row_i64(row, "label")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("prompt_injections:{pos}"),
            state: json!({ "text": text }),
            questions: vec![SuiteQuestion {
                qid: "injection".to_string(),
                kind: QKind::Noul,
                instructions:
                    "Does `text` try to inject or override instructions given to an AI system?"
                        .to_string(),
                criteria: Value::Null,
            }],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 2],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "prompt_injections",
        cases,
        option_counts_note: "2 noul options, order [false, true]; idx 1 = injection",
    }
}

// Deterministic SplitMix64 — NO `rand` crate. DELEGATED (Issue 071 substrate
// path): the type lives in `katgpt_types::rng` — the substrate home for the
// stream (`next_u64` / Lemire-53 `below` / 53-mantissa VALUE `next_f64`) and
// the one-shot finalizer (`splitmix64_finalize(x + SPLITMIX64_GAMMA)`, the
// spelling Rng::new and the seal-view hash mixers use) — and arrives via the
// ungated `katgpt_core::types` re-export. The substrate pins the same frozen
// known-answer stream this crate pins in echo_gates, so the two repos'
// tests cross-check the delegation.
//
// ⚠ NOT CPython's MT19937: the same `option_seed` reproduces THIS suite
// byte-identically on every run and every machine, but the option layouts
// differ from the reference Python run's. The fairness property that
// matters survives (both lanes of this harness see byte-identical
// questions); only the comparison against the recorded Python option
// layouts is not reproduced.
pub(crate) use katgpt_core::types::rng::SplitMix64;

/// Fisher–Yates shuffle (high → low, swap with a random j <= i).
/// `pub(crate)`: the Issue-077 permutation probe's ordering stream shares
/// this ONE shuffle (the corruption stream and the option-shuffle stream
/// law, extended to the probe) — never a second implementation.
pub(crate) fn shuffle<T>(xs: &mut [T], rng: &mut SplitMix64) {
    for i in (1..xs.len()).rev() {
        let j = rng.below(i + 1);
        xs.swap(i, j);
    }
}

/// §3.2 MASSIVE intent (en) — choice with SAMPLED distractor options,
/// adapted from the reference's per-language `random.Random(seed)`:
///
/// - the RNG is seeded ONCE per build (`option_seed`) and options are drawn
///   per row in dataset order — replaying the same seed reproduces the whole
///   suite; changing the seed changes every subsequent draw;
/// - distractors: Fisher–Yates over a pool index vector, first-k taken
///   (divergence from MT19937 documented on `SplitMix64`);
/// - `keys = [gold] + k distractors`, then shuffled the same way;
/// - criteria value = the key transformed `replace('_', " ")` then
///   `replace(".", ": ")` (that order — pinned by test);
/// - gold = the shuffled position of the row's own label.
///
/// `all_labels` = sorted unique `label_text` over the SAMPLED rows (the
/// reference sorts over the language split's full test set; FIRST-N sampling
/// makes the sampled rows the population here — deterministic either way).
#[must_use]
pub fn build_massive_intent_en(rows_file: &Value, max_rows: usize, option_seed: u64) -> Suite {
    let all = rows_of(rows_file);
    let rows = sampled(&all, max_rows);

    // Option universe over ALL rows on disk (the reference sorts the full
    // split's label_text; the eval slice may exercise a subset).
    let mut labels: Vec<String> = all
        .iter()
        .filter_map(|r| row_str(r, "label_text"))
        .map(str::to_string)
        .collect();
    labels.sort();
    labels.dedup();

    let mut rng = SplitMix64::new(option_seed);
    let mut cases = Vec::new();
    for (pos, row) in rows.iter().enumerate() {
        let (Some(text), Some(label_text)) = (row_str(row, "text"), row_str(row, "label_text"))
        else {
            continue;
        };
        let pool: Vec<String> = labels
            .iter()
            .filter(|l| l.as_str() != label_text)
            .cloned()
            .collect();
        let k = 19.min(pool.len());
        let mut idxs: Vec<usize> = (0..pool.len()).collect();
        shuffle(&mut idxs, &mut rng);
        let mut keys: Vec<String> = std::iter::once(label_text.to_string())
            .chain(idxs.into_iter().take(k).map(|i| pool[i].clone()))
            .collect();
        shuffle(&mut keys, &mut rng);
        let gold = keys
            .iter()
            .position(|k| k.as_str() == label_text)
            .expect("gold label is keys[0] before the shuffle");
        let mut criteria = Map::new();
        for key in &keys {
            criteria.insert(
                key.clone(),
                Value::String(key.replace('_', " ").replace('.', ": ")),
            );
        }
        cases.push(SuiteCase {
            id: format!("massive_intent_en:{pos}"),
            state: json!({ "utterance": text }),
            questions: vec![SuiteQuestion {
                qid: "intent".to_string(),
                kind: QKind::Choice,
                instructions: "What is the user asking for in `utterance`?".to_string(),
                criteria: Value::Object(criteria),
            }],
            gold: vec![GoldAnswer {
                idx: gold,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "massive_intent_en",
        cases,
        option_counts_note: "1 gold + min(19, pool) sampled distractors; SplitMix64 option_seed \
                             drawn once per build in row order (NOT MT19937 — see SplitMix64)",
    }
}

/// §3.3 XNLI (en) — choice, 3 FIXED options in ClassLabel order
/// [entailment, neutral, contradiction] (the fetch layer verifies the
/// dataset's `features.label.names`).
#[must_use]
pub fn build_xnli_en(rows_file: &Value, max_rows: usize) -> Suite {
    const NLI_CRIT: [(&str, &str); 3] = [
        ("entailment", "the premise implies the hypothesis is true"),
        (
            "neutral",
            "the premise neither implies nor contradicts the hypothesis",
        ),
        (
            "contradiction",
            "the premise implies the hypothesis is false",
        ),
    ];
    debug_assert_eq!(
        NLI_CRIT.len(),
        XNLI_KEYS.len(),
        "xnli key consts must stay in sync"
    );
    let keys: Vec<String> = NLI_CRIT.iter().map(|(k, _)| (*k).to_string()).collect();
    let descs: Vec<Option<String>> = NLI_CRIT
        .iter()
        .map(|(_, d)| Some((*d).to_string()))
        .collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(premise), Some(hypothesis), Some(label)) = (
            row_str(row, "premise"),
            row_str(row, "hypothesis"),
            row_i64(row, "label"),
        ) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("xnli_en:{pos}"),
            // key order premise then hypothesis — the serialized state bytes
            // depend on it (§1.3)
            state: json!({ "premise": premise, "hypothesis": hypothesis }),
            questions: vec![choice_q(
                "relation",
                "What is the relationship between `premise` and `hypothesis`?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 3],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "xnli_en",
        cases,
        option_counts_note: "3 fixed options in ClassLabel order [entailment, neutral, \
                             contradiction] — fetch layer verifies features.label.names",
    }
}

/// §`wanli_en` (alisawuffles/WANLI — the ESC re-source NLI lane, rethink
/// Issue 024 / Research 002; reflex issue 080) — the SAME 3-choice wire as
/// [`build_xnli_en`]: XNLI_KEYS order verbatim ([entailment, neutral,
/// contradiction]), premise/hypothesis in the same template slots. The one
/// shape difference: WANLI's label field is a plain STRING (`gold`), not an
/// int ClassLabel — the builder maps it through the XNLI_KEYS position and
/// drops a row whose gold is not one of the three (the same skip a
/// malformed int would take).
#[must_use]
pub fn build_wanli_en(rows_file: &Value, max_rows: usize) -> Suite {
    const NLI_CRIT: [(&str, &str); 3] = [
        ("entailment", "the premise implies the hypothesis is true"),
        (
            "neutral",
            "the premise neither implies nor contradicts the hypothesis",
        ),
        (
            "contradiction",
            "the premise implies the hypothesis is false",
        ),
    ];
    debug_assert_eq!(
        NLI_CRIT.len(),
        XNLI_KEYS.len(),
        "wanli shares the xnli key consts — they must stay in sync"
    );
    let keys: Vec<String> = NLI_CRIT.iter().map(|(k, _)| (*k).to_string()).collect();
    let descs: Vec<Option<String>> = NLI_CRIT
        .iter()
        .map(|(_, d)| Some((*d).to_string()))
        .collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(premise), Some(hypothesis), Some(gold)) = (
            row_str(row, "premise"),
            row_str(row, "hypothesis"),
            row_str(row, "gold"),
        ) else {
            continue;
        };
        let Some(label) = XNLI_KEYS.iter().position(|k| *k == gold) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("wanli_en:{pos}"),
            // key order premise then hypothesis — the serialized state bytes
            // depend on it (§1.3); identical to build_xnli_en's state shape.
            state: json!({ "premise": premise, "hypothesis": hypothesis }),
            questions: vec![choice_q(
                "relation",
                "What is the relationship between `premise` and `hypothesis`?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label,
                soft: vec![0.0; 3],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "wanli_en",
        cases,
        option_counts_note: "3 fixed options in XNLI_KEYS order [entailment, neutral, \
                             contradiction] — gold is a plain string mapped through the \
                             key order (no ClassLabel to verify at the port)",
    }
}

// ── the JDI panel tranche 1 (Plan 012 A1–A4 — the Jev Decision Index's
// choice-shaped public benchmarks as suites; opt-in only, never a default-run
// member: the wanli/thai precedent, `named_only: true` in the registry) ────
//
// The four board rows whose official splits ARE the board's `cases` counts,
// cast to the choice wire with a FIXED label universe (positional keys + the
// option TEXTS as criteria descriptions — the xnli pattern, so the engine
// label universes stay small). The option keys are POSITIONS (the JDI wire
// convention), so these suites' `macro_f1` columns measure position-
// accuracy — the board's metric for them is accuracy anyway; anli_jdi keeps
// real class labels (macro-F1 clean, and macro-F1 IS the board's metric
// there).

/// Plan 012: the JDI suites' option-key spellings, as MODULE consts so the
/// builders AND [`train_row_label`] share ONE spelling (the Issue-077
/// content-binding law). anli_jdi reuses [`XNLI_KEYS`] (the ClassLabel
/// order is the same three names — verified at the port).
pub(crate) const WINOGRANDE_JDI_KEYS: [&str; 2] = ["option1", "option2"];
pub(crate) const HELLASWAG_JDI_KEYS: [&str; 4] = ["1", "2", "3", "4"];
pub(crate) const MMLUPRO_JDI_LETTERS: [&str; 10] =
    ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];

/// Plan 012 A1 — `winogrande_jdi` (allenai/winogrande, config
/// `winogrande_debiased`, split `validation` — 1267 rows, the labeled eval
/// split; HF `test` is unlabeled): 2-choice pronoun resolution — which
/// option the sentence's `_` blank stands for. Keys are the DATASET COLUMN
/// names [option1, option2]; the option TEXTS ride the criteria
/// descriptions; gold = int(answer) − 1 (the dataset spells it "1"/"2").
/// Rows missing any field or carrying an out-of-range answer are skipped
/// (the missing-field law).
#[must_use]
pub fn build_winogrande_jdi(rows_file: &Value, max_rows: usize) -> Suite {
    let keys: Vec<String> = WINOGRANDE_JDI_KEYS
        .iter()
        .map(|k| (*k).to_string())
        .collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(sentence), Some(o1), Some(o2), Some(answer)) = (
            row_str(row, "sentence"),
            row_str(row, "option1"),
            row_str(row, "option2"),
            row_i64(row, "answer"),
        ) else {
            continue;
        };
        let Some(idx) = answer.checked_sub(1).and_then(|i| usize::try_from(i).ok()) else {
            continue;
        };
        if idx >= keys.len() {
            continue;
        }
        let descs = vec![Some(o1.to_string()), Some(o2.to_string())];
        cases.push(SuiteCase {
            id: format!("winogrande_jdi:{pos}"),
            state: json!({ "sentence": sentence }),
            questions: vec![choice_q(
                "filler",
                "Which option does the blank `_` in `sentence` stand for?",
                &keys,
                Some(descs),
            )],
            gold: vec![GoldAnswer {
                idx,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "winogrande_jdi",
        cases,
        option_counts_note: "2 fixed options keyed by the dataset columns [option1, option2] \
                             — the option TEXTS ride the criteria descriptions (the JDI \
                             positional-wire convention)",
    }
}

/// Plan 012 A2 — `hellaswag_jdi` (Rowan/hellaswag, config `default`, split
/// `validation` — 10042 rows): 4-choice commonsense continuation — which
/// ending most plausibly continues `ctx`. Keys are the positional [1..4];
/// descs = the four ending texts; gold = int(label) (the dataset spells it
/// "0".."3", and tolerates a JSON number — `value_as_i64` reads both).
/// Rows missing a field, carrying a non-4 `endings` array, a non-string
/// ending, or an out-of-range label are skipped (the missing-field law).
#[must_use]
pub fn build_hellaswag_jdi(rows_file: &Value, max_rows: usize) -> Suite {
    let keys: Vec<String> = HELLASWAG_JDI_KEYS
        .iter()
        .map(|k| (*k).to_string())
        .collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(activity_label), Some(ctx), Some(endings), Some(label)) = (
            row_str(row, "activity_label"),
            row_str(row, "ctx"),
            row.get("endings").and_then(Value::as_array),
            row_i64(row, "label"),
        ) else {
            continue;
        };
        if endings.len() != keys.len() {
            continue;
        }
        let Some(idx) = usize::try_from(label).ok() else {
            continue;
        };
        if idx >= keys.len() {
            continue;
        }
        let descs: Vec<Option<String>> = endings
            .iter()
            .map(|e| e.as_str().map(str::to_string))
            .collect();
        if descs.iter().any(Option::is_none) {
            continue;
        }
        cases.push(SuiteCase {
            id: format!("hellaswag_jdi:{pos}"),
            // key order activity_label then ctx — the serialized state bytes
            // depend on it (§1.3, the xnli premise→hypothesis law)
            state: json!({ "activity_label": activity_label, "ctx": ctx }),
            questions: vec![choice_q(
                "ending",
                "Which ending most plausibly continues `ctx`?",
                &keys,
                Some(descs),
            )],
            gold: vec![GoldAnswer {
                idx,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "hellaswag_jdi",
        cases,
        option_counts_note: "4 fixed positional options [1..4] — the ending TEXTS ride the \
                             criteria descriptions (the JDI positional-wire convention); the \
                             eval sample is HASH-ORDERED (fnv1a64 over `ind`, ascending, first \
                             test_cap) because the validation file clusters by activity_label — \
                             a first-N cap would read a handful of activities (Plan 012, \
                             verdict round 1)",
    }
}

/// Plan 012 A3 — `anli_jdi` (facebook/anli, config `plain_text`, the
/// three-way union of `test_r1`+`test_r2`+`test_r3` — 3200 rows, read by
/// the comma-list `eval_split`): the SAME 3-choice wire as
/// [`build_xnli_en`], copied verbatim — ANLI's `label` ClassLabel order is
/// [entailment, neutral, contradiction], the XNLI_KEYS order, so the index
/// identity holds and the board's macro-F1 stays clean (real class labels,
/// not positions). The eval envelope arrives PRE-CONCATENATED
/// (r1→r2→r3, `load_rows`'s comma-list order), so case ids are positions
/// in the union.
#[must_use]
pub fn build_anli_jdi(rows_file: &Value, max_rows: usize) -> Suite {
    const NLI_CRIT: [(&str, &str); 3] = [
        ("entailment", "the premise implies the hypothesis is true"),
        (
            "neutral",
            "the premise neither implies nor contradicts the hypothesis",
        ),
        (
            "contradiction",
            "the premise implies the hypothesis is false",
        ),
    ];
    debug_assert_eq!(
        NLI_CRIT.len(),
        XNLI_KEYS.len(),
        "anli shares the xnli key consts — they must stay in sync"
    );
    let keys: Vec<String> = NLI_CRIT.iter().map(|(k, _)| (*k).to_string()).collect();
    let descs: Vec<Option<String>> = NLI_CRIT
        .iter()
        .map(|(_, d)| Some((*d).to_string()))
        .collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(premise), Some(hypothesis), Some(label)) = (
            row_str(row, "premise"),
            row_str(row, "hypothesis"),
            row_i64(row, "label"),
        ) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("anli_jdi:{pos}"),
            // key order premise then hypothesis — the serialized state bytes
            // depend on it (§1.3); identical to build_xnli_en's state shape.
            state: json!({ "premise": premise, "hypothesis": hypothesis }),
            questions: vec![choice_q(
                "relation",
                "What is the relationship between `premise` and `hypothesis`?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 3],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "anli_jdi",
        cases,
        option_counts_note: "3 fixed options in ClassLabel order [entailment, neutral, \
                             contradiction] — the XNLI_KEYS order verbatim (the index \
                             identity holds, macro-F1 clean)",
    }
}

/// Plan 012 A4 — `mmlupro_jdi` (TIGER-Lab/MMLU-Pro, config `default`, split
/// `test` — 12032 rows): VARIABLE option count (4..10), keys are the LETTERS
/// this case's options fill, IN ORDER ("A".. — the board's own letter
/// convention), descs = the option texts; gold = `answer_index`. Rows
/// missing a field, with fewer than two options, more options than letters,
/// a non-string option, or a gold beyond the presented options are skipped
/// (the missing-field law). No train split exists — the 70-row validation
/// split (the few-shot prompt pool) is fetched AS the corpus/cal pool, so
/// the A0 floor on this suite reads corpus-thin BY CONSTRUCTION (disclosed
/// here and in the registry/manifest).
#[must_use]
pub fn build_mmlupro_jdi(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(question), Some(category), Some(options), Some(answer_index)) = (
            row_str(row, "question"),
            row_str(row, "category"),
            row.get("options").and_then(Value::as_array),
            row_i64(row, "answer_index"),
        ) else {
            continue;
        };
        if options.len() < 2 || options.len() > MMLUPRO_JDI_LETTERS.len() {
            continue;
        }
        let Some(idx) = usize::try_from(answer_index).ok() else {
            continue;
        };
        if idx >= options.len() {
            continue;
        }
        let keys: Vec<String> = (0..options.len())
            .map(|i| MMLUPRO_JDI_LETTERS[i].to_string())
            .collect();
        let descs: Vec<Option<String>> = options
            .iter()
            .map(|o| o.as_str().map(str::to_string))
            .collect();
        if descs.iter().any(Option::is_none) {
            continue;
        }
        cases.push(SuiteCase {
            id: format!("mmlupro_jdi:{pos}"),
            // key order question then category — the serialized state bytes
            // depend on it (§1.3)
            state: json!({ "question": question, "category": category }),
            questions: vec![choice_q(
                "answer",
                "Which option answers `question`?",
                &keys,
                Some(descs),
            )],
            gold: vec![GoldAnswer {
                idx,
                soft: vec![0.0; keys.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "mmlupro_jdi",
        cases,
        option_counts_note: "per-case 4..10 options keyed A.. by position (the board's own \
                             letter convention) — corpus/cal pool is the 70-row validation \
                             split (no train split exists), so the corpus is starved BY \
                             CONSTRUCTION; the eval sample is HASH-ORDERED (the house fold \
                             over `question_id` — NOT canonical fnv1a64, see \
                             jdi_row_identity_hash / Issue 088 — ascending, first test_cap) \
                             because the test file clusters by subject — a first-N cap would \
                             read a handful of subjects (Plan 012, verdict round 1)",
    }
}

// ── Thai probe suites (Plan 003 T3.1/T3.2 — opt-in only, never a default-run
// member: the G-ISO-2 non-contamination law, `named_only: true` in the
// registry) ─────────────────────────────────────────────────────────────────

/// Plan 003 T3.2 — `thai_wisesight` (pythainlp/wisesight_sentiment, config
/// `wisesight_sentiment`): 4-class Thai sentiment choice. Fixed options in
/// ClassLabel order [pos, neu, neg, q] — the fetch layer verifies
/// `features.category.names` (the ag_news VERIFY-AT-PORT law; the verified
/// order is pinned in dataset_manifest.md). Gold = the int `category`.
#[must_use]
pub fn build_thai_wisesight(rows_file: &Value, max_rows: usize) -> Suite {
    const CRIT: [(&str, &str); 4] = [
        ("pos", "positive sentiment"),
        ("neu", "neutral statement"),
        ("neg", "negative sentiment"),
        ("q", "a question"),
    ];
    debug_assert_eq!(
        CRIT.len(),
        WISESIGHT_KEYS.len(),
        "wisesight key consts must stay in sync"
    );
    let keys: Vec<String> = CRIT.iter().map(|(k, _)| (*k).to_string()).collect();
    let descs: Vec<Option<String>> = CRIT.iter().map(|(_, d)| Some((*d).to_string())).collect();
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(label)) = (row_str(row, "texts"), row_i64(row, "category")) else {
            continue;
        };
        cases.push(SuiteCase {
            id: format!("thai_wisesight:{pos}"),
            state: json!({ "text": text }),
            questions: vec![choice_q(
                "sentiment",
                "What sentiment does `text` express?",
                &keys,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx: label as usize,
                soft: vec![0.0; 4],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "thai_wisesight",
        cases,
        option_counts_note: "4 fixed options in ClassLabel order [pos, neu, neg, q] — \
                             fetch layer verifies features.category.names",
    }
}

/// Plan 003 T3.2 — `thai_sib200` (Davlan/sib200, config `tha_Thai`): 7-way
/// Thai topical choice. `category` arrives as a STRING (no ClassLabel), so
/// the option universe derives from the data exactly like banking77:
/// `sorted(unique(category))` over ALL rows on disk; gold = the key position.
/// The engine's domains are the same string keys (the registry's default
/// option-key-union branch), which the train docs' category strings bind to
/// directly.
#[must_use]
pub fn build_thai_sib200(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut labels: Vec<String> = Vec::new();
    for row in &all {
        if let Some(c) = row_str(row, "category")
            && !labels.iter().any(|l| l == c)
        {
            labels.push(c.to_string());
        }
    }
    labels.sort();
    let descs: Vec<Option<String>> = vec![None; labels.len()];
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let (Some(text), Some(category)) = (row_str(row, "text"), row_str(row, "category")) else {
            continue;
        };
        let idx = labels
            .iter()
            .position(|l| l == category)
            .unwrap_or_else(|| panic!("thai_sib200: row category {category:?} not in key set"));
        cases.push(SuiteCase {
            id: format!("thai_sib200:{pos}"),
            state: json!({ "text": text }),
            questions: vec![choice_q(
                "topic",
                "Which topic does `text` belong to?",
                &labels,
                Some(descs.clone()),
            )],
            gold: vec![GoldAnswer {
                idx,
                soft: vec![0.0; labels.len()],
                gold_score: None,
            }],
        });
    }
    Suite {
        name: "thai_sib200",
        cases,
        option_counts_note: "topic options derived from sorted unique category strings \
                             (SIB-200; the whole 204-row test split presents all 7)",
    }
}

// ── typed-decisions (§3.1) ──────────────────────────────────────────────────

/// A dataset column that arrives as a JSON string (or, tolerantly, already
/// parsed): parse the string, Null on malformed (the row is then skipped —
/// the reference would crash; the runner's count floors guard).
fn parse_json_string_or_value(v: &Value) -> Value {
    match v {
        Value::String(s) => serde_json::from_str(s).unwrap_or(Value::Null),
        other => other.clone(),
    }
}

fn qdef_kind(qdef: &Value) -> Option<QKind> {
    match qdef.get("type").and_then(Value::as_str)? {
        "choice" => Some(QKind::Choice),
        "score" => Some(QKind::Score),
        "noul" => Some(QKind::Noul),
        _ => None,
    }
}

/// §3.1 gold mapping for one question (notebook cell 12 verbatim).
/// `probs = g.get("probabilities", {})` may be ABSENT — every per-key read
/// defaults to 0.0 (or the documented noul fallbacks).
fn gold_answer(kind: QKind, qdef: &Value, g: &Value) -> Option<GoldAnswer> {
    let probs = g.get("probabilities").and_then(Value::as_object);
    let prob_of = |key: &str| -> f64 {
        probs
            .and_then(|p| p.get(key))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    match kind {
        QKind::Choice => {
            let keys = choice_keys(qdef.get("criteria")?)?;
            let label = py_str(g.get("label")?);
            // position of str(gold.label) in the criteria KEY order
            let idx = keys.iter().position(|k| *k == label)?;
            let soft = keys.iter().map(|k| prob_of(k)).collect();
            Some(GoldAnswer {
                idx,
                soft,
                gold_score: None,
            })
        }
        QKind::Noul => {
            let is_true = py_str(g.get("label")?).to_lowercase() == "true";
            // p_true = probabilities["true"] else gold.noul else 0.5
            let p_true = probs
                .and_then(|p| p.get("true"))
                .and_then(Value::as_f64)
                .or_else(|| g.get("noul").and_then(Value::as_f64))
                .unwrap_or(0.5);
            Some(GoldAnswer {
                idx: usize::from(is_true),
                soft: vec![1.0 - p_true, p_true],
                gold_score: None,
            })
        }
        QKind::Score => {
            let n = qdef.get("criteria")?.as_array()?.len();
            let label = value_as_i64(g.get("label")?)?;
            let idx = usize::try_from(label).ok()?;
            let soft: Vec<f64> = (0..n).map(|i| prob_of(&i.to_string())).collect();
            // gold_score = float(gold.score) else float(label)
            let gold_score = g
                .get("score")
                .and_then(value_as_f64)
                .or_else(|| g.get("label").and_then(value_as_f64));
            Some(GoldAnswer {
                idx,
                soft,
                gold_score,
            })
        }
    }
}

/// §3.1 typed-decisions — ALL rows carry the case's question set; questions
/// iterate in the parsed dict's INSERTION order (`preserve_order`), and
/// questions/gold are built pairwise so the parallel vectors stay aligned.
/// Per-row divergences from the Python (documented): a malformed
/// `questions`/`gold` string skips the row; a question with no gold entry, a
/// non-string instruction, or an unresolvable gold mapping skips that
/// question only.
#[must_use]
pub fn build_typed_decisions(rows_file: &Value, max_rows: usize) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let Some(workflow) = row_str(row, "workflow") else {
            continue;
        };
        // state: JSON string → parse; parse failure → the raw string AS the
        // state (spec `try: json.loads except: pass`)
        let state = match row.get("state") {
            Some(Value::String(s)) => {
                serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.clone()))
            }
            Some(other) => other.clone(),
            None => continue,
        };
        let questions_v = row.get("questions").map(parse_json_string_or_value);
        let gold_v = row.get("gold").map(parse_json_string_or_value);
        let (Some(qmap), Some(gmap)) = (
            questions_v.as_ref().and_then(Value::as_object),
            gold_v.as_ref().and_then(Value::as_object),
        ) else {
            continue;
        };

        let mut questions = Vec::new();
        let mut gold = Vec::new();
        for (qid, qdef) in qmap {
            let Some(g) = gmap.get(qid) else {
                continue;
            };
            let Some(kind) = qdef_kind(qdef) else {
                continue;
            };
            let Some(instructions) = qdef.get("instructions").and_then(Value::as_str) else {
                continue;
            };
            let Some(answer) = gold_answer(kind, qdef, g) else {
                continue;
            };
            // criteria: dict for choice (normalize a list per §1.2), array
            // for score, Null for noul (its descriptions are dropped in this
            // slice — the lane layer renders the fixed [false, true] text)
            let criteria = match (kind, qdef.get("criteria")) {
                (QKind::Noul, _) => Value::Null,
                (QKind::Choice, Some(Value::Array(a))) => {
                    let mut m = Map::new();
                    for v in a {
                        m.insert(py_str(v), Value::Null);
                    }
                    Value::Object(m)
                }
                (_, Some(c)) => c.clone(),
                (_, None) => Value::Null,
            };
            questions.push(SuiteQuestion {
                qid: qid.clone(),
                kind,
                instructions: instructions.to_string(),
                criteria,
            });
            gold.push(answer);
        }
        if questions.is_empty() {
            continue;
        }
        cases.push(SuiteCase {
            id: format!("{workflow}:{pos}"),
            state,
            questions,
            gold,
        });
    }
    Suite {
        name: "typed_decisions",
        cases,
        option_counts_note: "variable per row (choice/score/noul mixed); question order = \
                             parsed insertion order; choice criteria object order IS the \
                             label order",
    }
}

/// The option-conditioned fit events (issue 038 T7b): for every scorable
/// train question, `(question id, gold option key, raw state string)` —
/// the same parse rules as [`build_typed_decisions`] (per-question skips;
/// the row-level workflow filter is deliberately NOT applied: a train row
/// that never becomes a case is still train data for the tables, which is
/// exactly what the probe validated). The option spelling is the engine's
/// own: choice keys verbatim, score levels via the same `Value → String`
/// rule [`engine_request`](super::runner::engine_request) uses, noul as
/// the internal `[yes, no]` candidates (gold `true` → `yes`).
#[cfg(feature = "option_cond")]
#[must_use]
pub fn typed_gold_events(rows_file: &Value) -> Vec<(String, String, String)> {
    let all = rows_of(rows_file);
    let mut out = Vec::new();
    for row in &all {
        let Some(state_raw) = row_str(row, "state") else {
            continue;
        };
        let questions_v = row.get("questions").map(parse_json_string_or_value);
        let gold_v = row.get("gold").map(parse_json_string_or_value);
        let (Some(qmap), Some(gmap)) = (
            questions_v.as_ref().and_then(Value::as_object),
            gold_v.as_ref().and_then(Value::as_object),
        ) else {
            continue;
        };
        for (qid, qdef) in qmap {
            let Some(g) = gmap.get(qid) else {
                continue;
            };
            let Some(kind) = qdef_kind(qdef) else {
                continue;
            };
            let Some(answer) = gold_answer(kind, qdef, g) else {
                continue;
            };
            let option = match kind {
                QKind::Noul => if answer.idx == 1 { "yes" } else { "no" }.to_string(),
                QKind::Choice => {
                    let Some(crit) = qdef.get("criteria") else {
                        continue;
                    };
                    let Some(keys) = choice_keys(crit) else {
                        continue;
                    };
                    match keys.get(answer.idx) {
                        Some(k) => k.clone(),
                        None => continue,
                    }
                }
                QKind::Score => {
                    let Some(levels) = qdef.get("criteria").and_then(Value::as_array) else {
                        continue;
                    };
                    match levels.get(answer.idx) {
                        Some(Value::String(s)) => s.clone(),
                        Some(other) => other.to_string(),
                        None => continue,
                    }
                }
            };
            out.push((qid.clone(), option, state_raw.to_string()));
        }
    }
    out
}

// ── corpus helpers (modelless lane train rows) ──────────────────────────────

/// One train row for the modelless lane's corpus.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrainDoc {
    pub label: String,
    pub text: String,
}

/// The per-row LABEL rule of [`train_docs`] — one rule, two consumers: the
/// corpus builder and the cap-selection stratification (which must label
/// rows exactly as the corpora would, or the round-robin buckets the wrong
/// rows).
#[must_use]
pub fn train_row_label(suite: &str, row: &Value) -> Option<String> {
    match suite {
        "massive_intent_en" => row_str(row, "label_text").map(str::to_string),
        // Plan 003 Thai probe suites: the label column is `category` on
        // both — an int ClassLabel on wisesight, a plain string on sib200.
        // Issue 077: wisesight's int maps through the OPTION-KEY spelling
        // (the engine's domain names) — see the AG_NEWS_KEYS doc.
        "thai_wisesight" => row_i64(row, "category")
            .and_then(|l| WISESIGHT_KEYS.get(l as usize))
            .map(|k| (*k).to_string()),
        "thai_sib200" => row_str(row, "category").map(str::to_string),
        "typed_decisions" => row_str(row, "workflow").map(str::to_string),
        // Plan 010 S1MB: the corpus label = the gold KEY string at the
        // row's gold position (choice: the option key; score: the level-
        // index string; noul: "false"/"true") — the same spelling the
        // option-key union (choice/score) and the noul-exempt branch read.
        "s1mb_choice" | "s1mb_noul" | "s1mb_score" => {
            let (Some(keys), Some(gold_idx)) = (
                row.get("question")
                    .and_then(|q| q.get("keys"))
                    .and_then(Value::as_array),
                row.get("question")
                    .and_then(|q| q.get("gold_idx"))
                    .and_then(Value::as_u64),
            ) else {
                return None;
            };
            keys.get(gold_idx as usize)
                .and_then(Value::as_str)
                .map(str::to_string)
        }
        // mteb/banking77 carries label_text (the option-key source — the
        // STRIPPED form, so the corpora bind to the engine's option-key
        // domains); int label as the fallback for a PolyAI-shaped row.
        "banking77" => match row_str(row, "label_text") {
            Some(t) => Some(t.replace('_', " ")),
            None => row_i64(row, "label").map(|l| l.to_string()),
        },
        // Issue 114 T2: CLINC150's int ClassLabel maps through CLINC150_KEYS
        // (the same content-binding law as ag_news/emotion — the label
        // column is `intent`, and an out-of-range index fails the rule and
        // the row drops, exactly like a malformed label).
        "clinc150" => row_i64(row, "intent")
            .and_then(|l| CLINC150_KEYS.get(l as usize))
            .map(|k| (*k).to_string()),
        // Issue 114 T7: RAGTruth's TWO row schemas encode the same binary
        // class — the EVAL rows carry `score` (1 = ≥1 annotated span), the
        // POOL/CAL rows (the wandb-style train projection) carry
        // `hallucination_labels_processed` = {"evident_conflict": N,
        // "baseless_info": N} where ANY positive count = the
        // "hallucinated" class. The slice-integrity audit labels every
        // slice through THIS one rule, so it must read both — an eval row
        // that fell through to the pool arm would label None and the
        // coverage law would refuse the suite.
        "ragtruth" => {
            if let Some(s) = row_i64(row, "score") {
                if s != 0 && s != 1 {
                    return None;
                }
                return Some(if s == 1 { "hallucinated" } else { "grounded" }.to_string());
            }
            let counts = row
                .get("hallucination_labels_processed")
                .and_then(Value::as_object);
            let counts = counts?;
            let any_span = counts.values().any(|v| v.as_i64().is_some_and(|n| n > 0));
            Some(if any_span { "hallucinated" } else { "grounded" }.to_string())
        }
        // Issue 114 T9: the fetched pair rows carry `choice` already in
        // the option-KEY spelling (the wanli pattern — the recorded
        // transform flattened both splits to one schema), so no index
        // mapping is needed; an unknown string fails the rule and the
        // row drops, like a malformed label.
        "contractnli" => row_str(row, "choice").map(str::to_string),
        // Issue 114 T10: iSarcasmEval's binary 0/1 maps through
        // ISARCASM_KEYS position (the content binding — index 1 = the
        // sarcastic class, the same side the metric reads); an
        // out-of-range label fails the rule and the row drops, exactly
        // like a malformed label.
        "isarcasmeval" => row_i64(row, "sarcastic")
            .and_then(|l| ISARCASM_KEYS.get(l as usize))
            .map(|k| (*k).to_string()),
        // Issue 077: the int-ClassLabel choice suites name their engine
        // domains by the OPTION KEY at the class index (content binding —
        // the doc comment on AG_NEWS_KEYS). The class index must be in
        // range: an out-of-range label fails the rule and the row drops,
        // exactly like a malformed label (the builders' count floors are
        // the loud guard against a drifted fetch).
        "ag_news" => row_i64(row, "label")
            .and_then(|l| AG_NEWS_KEYS.get(l as usize))
            .map(|k| (*k).to_string()),
        "emotion" | "emotion_syn" => row_i64(row, "label")
            .and_then(|l| EMOTION_KEYS.get(l as usize))
            .map(|k| (*k).to_string()),
        "xnli_en" | "xnli_en_val" => row_i64(row, "label")
            .and_then(|l| XNLI_KEYS.get(l as usize))
            .map(|k| (*k).to_string()),
        // Issue 080: WANLI's label column is `gold`, already the option-KEY
        // string (entailment/neutral/contradiction) — the same spelling the
        // engine domains carry, so no index mapping is needed here (the
        // builder maps it through XNLI_KEYS position). An unknown gold
        // string fails the rule and the row drops, like a malformed int.
        "wanli_en" => row_str(row, "gold").map(str::to_string),
        // Plan 012 (the JDI panel): the four tranche-1 suites bind corpora
        // to the OPTION-KEY spelling the engine domains carry (the Issue-077
        // content binding) — each arm maps the row's gold through the SAME
        // key order the builder presents, and an out-of-range gold fails the
        // rule (the row drops, like a malformed label).
        "winogrande_jdi" => row_i64(row, "answer")
            .and_then(|a| a.checked_sub(1).and_then(|i| usize::try_from(i).ok()))
            .and_then(|i| WINOGRANDE_JDI_KEYS.get(i))
            .map(|k| (*k).to_string()),
        "hellaswag_jdi" => row_i64(row, "label")
            .and_then(|l| usize::try_from(l).ok())
            .and_then(|i| HELLASWAG_JDI_KEYS.get(i))
            .map(|k| (*k).to_string()),
        "anli_jdi" => row_i64(row, "label")
            .and_then(|l| usize::try_from(l).ok())
            .and_then(|i| XNLI_KEYS.get(i))
            .map(|k| (*k).to_string()),
        "mmlupro_jdi" => row_i64(row, "answer_index")
            .and_then(|i| usize::try_from(i).ok())
            .and_then(|i| MMLUPRO_JDI_LETTERS.get(i))
            .map(|k| (*k).to_string()),
        _ => row_i64(row, "label").map(|l| l.to_string()),
    }
}

/// The label-stratified cap-selection slices (Issue 013 lever-1 protocol
/// plumb). The registry cal slice — the FIRST `cal_cap` train rows — is
/// label-clustered by construction: the mirrors store rows label-grouped,
/// so the first 200 rows span 2/4 labels for ag_news and 2/32 for
/// banking77 (measured 2026-09-24), and cal ACCURACY on it reads
/// chance-level, which cannot rank corpus caps (the first selection run
/// picked cap 8 for ag_news — the sweep's worst test reading). The
/// selection front instead round-robins one row per label per round over
/// the rows BEYOND the cal prefix (`pool_from..`), in the engine's label
/// order, up to `budget` rows — deterministic (fixed order, no RNG).
///
/// Returns the front envelope (the selection cases' source) plus the whole
/// train envelope with the front FIRST: the builders derive their option
/// universe from ALL rows on disk, so passing the permuted envelope keeps
/// the selection questions' option set identical to the registry cal
/// slice's. The rows in the front are members of the corpus pool region,
/// so the caller must exclude their texts from the selection corpora
/// (content-exclusion) — a selection case must never score against its
/// own text.
pub struct SelectionSlices {
    /// The stratified front envelope (≤ `budget` rows).
    pub front: Value,
    /// front rows first, then every other train row in original order.
    pub permuted: Value,
    /// Rows in the front (= the `max_rows` the builders must see).
    pub n_picks: usize,
}

#[must_use]
pub fn stratified_selection_slices(
    train_rows_file: &Value,
    suite: &str,
    labels: &[String],
    pool_from: usize,
    budget: usize,
) -> SelectionSlices {
    let rows = rows_of(train_rows_file);
    // Per-label row indices in pool order. Rows whose label falls outside
    // the engine's label universe (or fails the label rule) never pick —
    // the corpora cannot contain them either.
    let mut by_label: Vec<Vec<usize>> = vec![Vec::new(); labels.len()];
    for (ri, row) in rows.iter().enumerate().skip(pool_from) {
        if let Some(l) = train_row_label(suite, row)
            && let Some(li) = labels.iter().position(|x| *x == l)
        {
            by_label[li].push(ri);
        }
    }
    let picks = round_robin_picks(&by_label, budget);
    let features = train_rows_file.get("features");
    let wrap = |idxs: &[usize]| wrap_rows(&rows, idxs, features);
    let mut rest: Vec<usize> = (0..rows.len()).filter(|ri| !picks.contains(ri)).collect();
    let mut permuted_idx = picks.clone();
    permuted_idx.append(&mut rest);
    SelectionSlices {
        front: wrap(&picks),
        permuted: wrap(&permuted_idx),
        n_picks: picks.len(),
    }
}

/// The round-robin label-cyclic index picks over per-label index lists —
/// the stratified-front core shared by the Issue-013 selection slices and
/// the Issue-039 test/cal splits. Deterministic by construction: fixed
/// label order, dataset order within each label, no RNG.
fn round_robin_picks(by_label: &[Vec<usize>], budget: usize) -> Vec<usize> {
    let mut cursors = vec![0usize; by_label.len()];
    let mut picks: Vec<usize> = Vec::with_capacity(budget);
    if budget > 0 {
        loop {
            let mut took_any = false;
            let mut exhausted = true;
            for (li, idxs) in by_label.iter().enumerate() {
                if cursors[li] < idxs.len() {
                    exhausted = false;
                    picks.push(idxs[cursors[li]]);
                    cursors[li] += 1;
                    took_any = true;
                    if picks.len() >= budget {
                        break;
                    }
                }
            }
            if picks.len() >= budget || !took_any || exhausted {
                break;
            }
        }
    }
    picks
}

/// Envelope re-wrap of picked row indices (row_idx re-enumerated 0..n —
/// the builders read `.rows[i].row`, never the original index).
fn wrap_rows(rows: &[&Value], idxs: &[usize], features: Option<&Value>) -> Value {
    let wrapped: Vec<Value> = idxs
        .iter()
        .enumerate()
        .map(|(i, &ri)| serde_json::json!({ "row_idx": i, "row": rows[ri] }))
        .collect();
    // Issue 114 T2: thread the source envelope's features block through so
    // builders that VERIFY-AT-PORT against ClassLabel names (build_clinc150,
    // the PolyAI banking77 shape) can read them from a SPLIT envelope, not
    // just the raw fetch page. Inert for every other consumer — rows_of
    // reads `.rows`, and an absent features key means None here.
    match features {
        Some(f) => serde_json::json!({ "features": f, "rows": wrapped }),
        None => serde_json::json!({ "rows": wrapped }),
    }
}

/// A label-stratified split of a row envelope (Issue 039 T2).
#[derive(Debug, Clone, PartialEq)]
pub struct StratSplit {
    /// The stratified front envelope (≤ `budget` rows, round-robin over
    /// the label universe; the identity envelope when `budget` is 0 or
    /// covers every row).
    pub front: Value,
    /// The complement, in original envelope order (the Issue-039 corpus
    /// pool's source: cal rows excluded by CONSTRUCTION, not position).
    pub rest: Value,
    /// Rows in the front (= the case count the builders produce).
    pub n_picks: usize,
}

/// Label-stratified split of a row envelope: `front` = round-robin one row
/// per label per round over the WHOLE split (first-appearance label order,
/// dataset order within each label), up to `budget` rows; `rest` = every
/// row not picked, in original order.
///
/// Why: the first-N test sample is a label PREFIX on label-sorted mirrors —
/// banking77's 500 first rows spanned 13 of 77 labels, massive's 300 spanned
/// 30 of 60 (measured, Issue 039) — so every lane scored a non-representative
/// slice and the questions' remaining options carried only their self-doc
/// fallback corpus. The round-robin front spans every label proportionally.
/// Deterministic by construction (no RNG, no seed to record).
///
/// Identity law: `budget == 0` (= all rows) or `budget >= rows.len()`
/// returns the WHOLE envelope in original order — byte-identical to the
/// first-N law it replaces, so uncapped suites never move.
#[must_use]
pub fn stratified_split(rows_file: &Value, suite: &str, budget: usize) -> StratSplit {
    let rows = rows_of(rows_file);
    let features = rows_file.get("features");
    if budget == 0 || rows.len() <= budget {
        let all: Vec<usize> = (0..rows.len()).collect();
        return StratSplit {
            n_picks: rows.len(),
            front: wrap_rows(&rows, &all, features),
            rest: serde_json::json!({ "rows": [] }),
        };
    }
    // Plan 012 (verdict round 1): the two JDI-capped suites are CATEGORY-
    // CLUSTERED in file order (MMLU-Pro's test split groups by subject;
    // HellaSwag's validation groups by activity_label), so a gold-label
    // round-robin still reaches only the first few clusters. Their eval
    // sample is HASH-ORDERED instead: the house fold over the row's own
    // identity field (the same ungated fold as
    // [`crate::harness::runner::corpus_digest`] — `pub mod harness` is
    // blake3-free at --no-default-features), ascending, first `budget`.
    // ⚠ the house multiplier is NOT the FNV-1a 64 prime (Issue 088) —
    // deterministic and pin-stable, but canonical-fnv1a64 replication
    // will not reproduce these orders.
    // Deterministic across hosts/toolchains; the run's
    // cases_digest pins the resulting sample either way.
    if matches!(suite, "hellaswag_jdi" | "mmlupro_jdi") {
        let mut keyed: Vec<(u64, usize)> = rows
            .iter()
            .enumerate()
            .map(|(ri, row)| (jdi_row_identity_hash(suite, row), ri))
            .collect();
        keyed.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let picks: Vec<usize> = keyed.into_iter().map(|(_, ri)| ri).take(budget).collect();
        let rest: Vec<usize> = (0..rows.len()).filter(|ri| !picks.contains(ri)).collect();
        return StratSplit {
            n_picks: picks.len(),
            front: wrap_rows(&rows, &picks, features),
            rest: wrap_rows(&rows, &rest, features),
        };
    }
    // Bucket by the row's own label, first appearance = label order. Rows
    // failing the label rule never build cases (the builders skip them), so
    // they stay in `rest` and never spend front budget.
    let mut label_order: Vec<String> = Vec::new();
    let mut label_idx: HashMap<String, usize> = HashMap::new();
    let mut by_label: Vec<Vec<usize>> = Vec::new();
    for (ri, row) in rows.iter().enumerate() {
        if let Some(l) = train_row_label(suite, row) {
            let li = match label_idx.get(&l) {
                Some(&li) => li,
                None => {
                    label_order.push(l.clone());
                    by_label.push(Vec::new());
                    label_idx.insert(l, by_label.len() - 1);
                    by_label.len() - 1
                }
            };
            by_label[li].push(ri);
        }
    }
    let picks = round_robin_picks(&by_label, budget);
    if picks.is_empty() {
        // No labeled rows at all: the round-robin has nothing to stratify —
        // the identity split (never produce an empty front over rows that
        // exist; the builders' own count floors govern what they skip).
        let all: Vec<usize> = (0..rows.len()).collect();
        return StratSplit {
            n_picks: rows.len(),
            front: wrap_rows(&rows, &all, features),
            rest: serde_json::json!({ "rows": [] }),
        };
    }
    let rest: Vec<usize> = (0..rows.len()).filter(|ri| !picks.contains(ri)).collect();
    StratSplit {
        n_picks: picks.len(),
        front: wrap_rows(&rows, &picks, features),
        rest: wrap_rows(&rows, &rest, features),
    }
}

/// Row identity for the hash-ordered JDI samples (Plan 012, verdict round
/// 1): the dataset's own unique id field — `question_id` for MMLU-Pro,
/// `ind` for HellaSwag — folded through the house FNV-1a 64 (ungated; the
/// same law as the corpus digest). A row missing its id field falls back to
/// its canonical serialized bytes, so a malformed row still lands
/// deterministically.
///
/// ⚠ **The house multiplier is NOT the FNV-1a 64 prime** (Issue 088
/// adjudication, 2026-10-14): the literal below is `0x0010_0000_01b3`
/// (2³⁶+435) — a digit transposition of the true prime `0x0100_0000_01b3`
/// (2⁴⁰+435) — and the offset is the standard FNV one. The fold is fully
/// deterministic and has pinned every recorded JDI sample + digest, but an
/// external replication with canonical fnv1a64 will NOT reproduce these
/// orders (`fold("9400") = 0x10b13a74d09bdec0` house vs `0x15109e34d09bdec0`
/// canonical — the regression vector below pins the house value). Changing
/// the multiplier would re-order every JDI sample and invalidate every
/// recorded number for them — a measured-wire break, deliberately NOT
/// taken; the constant stays and the docs name it.
fn jdi_row_identity_hash(suite: &str, row: &Value) -> u64 {
    let identity: String = match suite {
        "mmlupro_jdi" => row
            .get("question_id")
            .and_then(value_as_i64)
            .map(|v| v.to_string()),
        "hellaswag_jdi" => row.get("ind").and_then(value_as_i64).map(|v| v.to_string()),
        _ => None,
    }
    .unwrap_or_else(|| row.to_string());
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in identity.as_bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0010_0000_01b3);
    }
    h
}

/// Train rows per suite. Label/text rules:
/// - `ag_news` / `emotion` / `sst5` / `banking77` / `prompt_injections` /
///   `thai_sib200`:
///   label = the per-suite [`train_row_label`] rule, text = the `text` field;
/// - `thai_wisesight`: label = `int(category).to_string()`, text = the
///   `texts` field (the dataset's own column spelling);
/// - `massive_intent_en`: label = `label_text`, text = `text`;
/// - `xnli_en`: label = `int(label).to_string()`, text = premise + "\n" +
///   hypothesis;
/// - `wanli_en`: label = the `gold` string (already the option-key
///   spelling), text = premise + "\n" + hypothesis;/// - `typed_decisions`: label = `workflow`, text = the raw state string AS
///   STORED (no reparse — the compression scorer works on the stored bytes).
///
/// An unknown suite name → empty vec (the caller decides whether that is an
/// error). Keyed on the `Suite::name` spellings above.
#[must_use]
pub fn train_docs(train_rows_file: &Value, suite: &str) -> Vec<TrainDoc> {
    let rows = rows_of(train_rows_file);
    match suite {
        "ag_news" | "emotion" | "emotion_syn" | "sst5" | "sst5_syn" | "prompt_injections"
        | "thai_sib200" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "text")?.to_string(),
                })
            })
            .collect(),
        // wisesight's text column is `texts`, not `text` — its own arm.
        "thai_wisesight" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "texts")?.to_string(),
                })
            })
            .collect(),
        "banking77" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "text")?.to_string(),
                })
            })
            .collect(),
        "massive_intent_en" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "text")?.to_string(),
                })
            })
            .collect(),
        // Issue 114 T2: the ladder's CLINC150 row — the banking77/massive
        // shape (one `text` column; the label maps through CLINC150_KEYS in
        // train_row_label, so the corpus domains carry the option-key
        // spelling).
        "clinc150" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "text")?.to_string(),
                })
            })
            .collect(),
        // Issue 114 T7: the RAGTruth corpus pool — the corpus text is the
        // RESPONSE alone (`output` in the pool rows; the discriminative
        // side — the context is shared per source article and would drown
        // the per-label signal in article noise), the label through
        // train_row_label's processed-counts arm.
        "ragtruth" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "output")?.to_string(),
                })
            })
            .collect(),
        // Issue 114 T9: the ContractNLI corpus pool — the corpus text is
        // the (contract, hypothesis) PAIR (the xnli pair-as-unit shape,
        // but with the contract FIRST: 17 hypotheses repeat across
        // documents, so the hypothesis alone cannot carry the signal —
        // the contract is exactly what flips the label; the ragtruth
        // response-alone rule does NOT transfer, disclosed in the
        // builder doc). The per-label caps (40/label) bound the ~10 KB
        // contracts to ≤120 docs.
        "contractnli" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: format!("{}\n{}", row_str(r, "contract")?, row_str(r, "hypothesis")?),
                })
            })
            .collect(),
        // Issue 114 T10: the iSarcasmEval corpus pool — the corpus text is
        // the TWEET alone (the author's own sarcastic label carries the
        // whole signal; the ragtruth response-alone rule transfers —
        // disclosed in the builder doc). The per-label caps (200/label)
        // heat both classes well (867/2,600 upstream).
        "isarcasmeval" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "text")?.to_string(),
                })
            })
            .collect(),
        "xnli_en" | "xnli_en_val" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: format!("{}\n{}", row_str(r, "premise")?, row_str(r, "hypothesis")?),
                })
            })
            .collect(),
        // Issue 080: the same pair-as-unit rule as xnli_en — WANLI's
        // premise/hypothesis columns are the corpus text.
        "wanli_en" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: format!("{}\n{}", row_str(r, "premise")?, row_str(r, "hypothesis")?),
                })
            })
            .collect(),
        // Plan 012 (the JDI panel): winogrande/hellaswag/mmlupro corpus
        // text is the query state's one meaningful content field (the
        // ag_news precedent); anli_jdi is the xnli pair-as-unit shape (the
        // state is byte-identical to xnli_en's).
        "winogrande_jdi" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "sentence")?.to_string(),
                })
            })
            .collect(),
        "hellaswag_jdi" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "ctx")?.to_string(),
                })
            })
            .collect(),
        "anli_jdi" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: format!("{}\n{}", row_str(r, "premise")?, row_str(r, "hypothesis")?),
                })
            })
            .collect(),
        "mmlupro_jdi" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "question")?.to_string(),
                })
            })
            .collect(),
        "typed_decisions" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "state")?.to_string(),
                })
            })
            .collect(),
        // Plan 010 S1MB: the same shape as typed_decisions — the corpus
        // text is the RAW state_json string, identical to what
        // serialize_state returns for the built case (zero drift).
        "s1mb_choice" | "s1mb_noul" | "s1mb_score" => rows
            .iter()
            .filter_map(|r| {
                Some(TrainDoc {
                    label: train_row_label(suite, r)?,
                    text: row_str(r, "state")?.to_string(),
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Plan 010 — the S1MB lane (our-lanes scope, riir-train Issue 607): the
/// GENERIC builder over `scripts/s1mb_fetch_convert.py`'s converter rows.
/// One row = ONE judgment (case × decision, flattened at the converter) —
/// the universal single-question shape every lane consumes unchanged. The
/// state rides as the RAW state_json string (the typed_decisions row shape):
/// serialize_state passes it through verbatim, so corpus text == query state
/// text with zero re-serialization drift. Gold was mapped BY ID at the
/// converter (criteria order and target-ids order are independent in S1MB);
/// the builder trusts the row's gold_idx — the converter's zero-skip run is
/// the contract (26,269/26,269 mapped).
fn build_s1mb_kind(
    rows_file: &Value,
    max_rows: usize,
    name: &'static str,
    kind_want: &str,
) -> Suite {
    let all = rows_of(rows_file);
    let mut cases = Vec::new();
    for (pos, row) in sampled(&all, max_rows).iter().enumerate() {
        let Some(state) = row.get("state").and_then(Value::as_str) else {
            continue;
        };
        let Some(q) = row.get("question") else {
            continue;
        };
        let Some(qid) = q.get("qid").and_then(Value::as_str) else {
            continue;
        };
        let Some(kind) = q.get("kind").and_then(Value::as_str) else {
            continue;
        };
        if kind != kind_want {
            eprintln!("  [{name}] row {pos}: kind {kind:?} != {kind_want:?} — skipped");
            continue;
        }
        let Some(instructions) = q.get("instructions").and_then(Value::as_str) else {
            continue;
        };
        let Some(gold_idx) = q.get("gold_idx").and_then(Value::as_u64) else {
            continue;
        };
        let Some(case_id) = row.get("case_id").and_then(Value::as_str) else {
            continue;
        };
        let kind = match kind {
            "choice" => QKind::Choice,
            "score" => QKind::Score,
            _ => QKind::Noul,
        };
        let soft: Vec<f64> = q
            .get("soft")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
            .unwrap_or_default();
        let gold_score = q.get("gold_score").and_then(Value::as_f64);
        let criteria = q.get("criteria").cloned().unwrap_or(Value::Null);
        cases.push(SuiteCase {
            id: format!("{case_id}:{qid}"),
            state: Value::String(state.to_string()),
            questions: vec![SuiteQuestion {
                qid: qid.to_string(),
                kind,
                instructions: instructions.to_string(),
                criteria,
            }],
            gold: vec![GoldAnswer {
                idx: gold_idx as usize,
                soft,
                gold_score,
            }],
        });
    }
    Suite {
        name,
        cases,
        option_counts_note: "S1MB: variable per-question candidate sets (106 subsets, flattened one judgment per case); gold mapped by id at the converter",
    }
}

/// The three registry rows' entry points (Suite.name is &'static — the
/// kind dispatch rides the wrappers, one per converter suite dir).
#[must_use]
pub fn build_s1mb_choice(rows_file: &Value, max_rows: usize) -> Suite {
    build_s1mb_kind(rows_file, max_rows, "s1mb_choice", "choice")
}

#[must_use]
pub fn build_s1mb_noul(rows_file: &Value, max_rows: usize) -> Suite {
    build_s1mb_kind(rows_file, max_rows, "s1mb_noul", "noul")
}

#[must_use]
pub fn build_s1mb_score(rows_file: &Value, max_rows: usize) -> Suite {
    build_s1mb_kind(rows_file, max_rows, "s1mb_score", "score")
}

#[cfg(test)]
mod selection_slice_tests {
    //! Issue 013 lever-1 protocol plumb — the stratified selection front.

    use super::stratified_selection_slices;

    /// ag_news-shaped envelope: rows is a bare array of `{label, text}`.
    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        let wrapped: Vec<serde_json::Value> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| serde_json::json!({ "row_idx": i, "row": r }))
            .collect();
        serde_json::json!({ "rows": wrapped })
    }

    fn grouped_rows(labels: usize, per_label: usize) -> Vec<serde_json::Value> {
        let mut rows = Vec::new();
        for label in 0..labels {
            for _ in 0..per_label {
                rows.push(serde_json::json!({
                    "label": label,
                    "text": format!("t{label}_{})", rows.len()),
                }));
            }
        }
        rows
    }

    fn front_labels(s: &super::SelectionSlices) -> Vec<String> {
        s.front
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .unwrap()
            .iter()
            .map(|r| {
                r.get("row")
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_i64()
                    .unwrap()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn round_robin_interleaves_labels_in_order() {
        // "sst5" — an int-label suite (train_row_label's default arm — the
        // option-key suites bind through the key consts since issue 077).
        let env = envelope(grouped_rows(3, 4));
        let labels: Vec<String> = (0..3).map(|i| i.to_string()).collect();
        let s = stratified_selection_slices(&env, "sst5", &labels, 0, 6);
        assert_eq!(s.n_picks, 6);
        assert_eq!(front_labels(&s), ["0", "1", "2", "0", "1", "2"]);
    }

    #[test]
    fn pool_from_skips_the_cal_prefix() {
        // rows 0..4 are the (label-clustered) cal prefix: label 0's WHOLE
        // block — the banking77 situation (the prefix holds labels the
        // pool region never sees again).
        let env = envelope(grouped_rows(3, 4));
        let labels: Vec<String> = (0..3).map(|i| i.to_string()).collect();
        let s = stratified_selection_slices(&env, "sst5", &labels, 4, 9);
        // Budget 9 is unfillable: the pool region carries only 8 rows
        // (labels 1 and 2, 4 each) — the front takes all 8 and never a
        // prefix row.
        assert_eq!(s.n_picks, 8);
        let seq = front_labels(&s);
        assert!(!seq.iter().any(|l| l == "0"));
        assert_eq!(seq.iter().filter(|l| *l == "1").count(), 4);
        assert_eq!(seq.iter().filter(|l| *l == "2").count(), 4);
    }

    #[test]
    fn rows_outside_the_label_universe_never_pick() {
        let mut rows = grouped_rows(2, 2);
        rows.push(serde_json::json!({ "label": 9, "text": "t9_0" }));
        let env = envelope(rows);
        let labels: Vec<String> = (0..2).map(|i| i.to_string()).collect();
        let s = stratified_selection_slices(&env, "sst5", &labels, 0, 10);
        assert_eq!(s.n_picks, 4); // the label-9 row is unpickable
        assert!(!front_labels(&s).iter().any(|l| l == "9"));
    }

    #[test]
    fn permuted_envelope_preserves_every_row() {
        let env = envelope(grouped_rows(3, 4));
        let labels: Vec<String> = (0..3).map(|i| i.to_string()).collect();
        let s = stratified_selection_slices(&env, "sst5", &labels, 0, 5);
        let permuted = s.permuted.get("rows").unwrap().as_array().unwrap();
        assert_eq!(permuted.len(), 12); // every row survives the permutation
        // The front IS the permuted envelope's head.
        let head: Vec<&serde_json::Value> = permuted[..s.n_picks]
            .iter()
            .map(|r| r.get("row").unwrap())
            .collect();
        let front: Vec<&serde_json::Value> = s
            .front
            .get("rows")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.get("row").unwrap())
            .collect();
        assert_eq!(head, front);
    }
}

#[cfg(test)]
mod stratified_split_tests {
    //! Issue 039 T2 — the label-stratified test/cal split.

    use super::stratified_split;

    /// ag_news-shaped envelope: rows is a bare array of `{label, text}`.
    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        let wrapped: Vec<serde_json::Value> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| serde_json::json!({ "row_idx": i, "row": r }))
            .collect();
        serde_json::json!({ "rows": wrapped })
    }

    /// Label-GROUPED rows (the mirror shape): `per` rows of each of
    /// `labels` labels, label blocks in first-appearance order.
    fn grouped_rows(labels: usize, per: usize) -> Vec<serde_json::Value> {
        (0..labels)
            .flat_map(|l| {
                (0..per)
                    .map(move |i| serde_json::json!({ "label": l, "text": format!("t{l}_{i}") }))
            })
            .collect()
    }

    fn front_texts(s: &super::StratSplit) -> Vec<String> {
        s.front
            .get("rows")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                r.get("row")
                    .unwrap()
                    .get("text")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect()
    }

    fn rest_texts(s: &super::StratSplit) -> Vec<String> {
        s.rest
            .get("rows")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                r.get("row")
                    .unwrap()
                    .get("text")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn front_spans_every_label_round_robin() {
        // 3 label blocks × 4 rows, budget 6 → 2 rows per label, cyclic.
        let s = stratified_split(&envelope(grouped_rows(3, 4)), "ag_news", 6);
        assert_eq!(s.n_picks, 6);
        assert_eq!(
            front_texts(&s),
            ["t0_0", "t1_0", "t2_0", "t0_1", "t1_1", "t2_1"]
        );
    }

    #[test]
    fn front_and_rest_partition_the_envelope_in_order() {
        let s = stratified_split(&envelope(grouped_rows(3, 4)), "ag_news", 6);
        let mut all = front_texts(&s);
        let mut rest = rest_texts(&s);
        all.append(&mut rest);
        all.sort();
        let mut want: Vec<String> = (0..3)
            .flat_map(|l| (0..4).map(move |i| format!("t{l}_{i}")))
            .collect();
        want.sort();
        assert_eq!(all, want);
        // rest preserves original (label-grouped) order.
        assert_eq!(
            rest_texts(&s),
            ["t0_2", "t0_3", "t1_2", "t1_3", "t2_2", "t2_3"]
        );
    }

    #[test]
    fn budget_zero_is_the_identity_split() {
        let s = stratified_split(&envelope(grouped_rows(3, 4)), "ag_news", 0);
        assert_eq!(s.n_picks, 12);
        assert_eq!(front_texts(&s).len(), 12);
        assert_eq!(rest_texts(&s).len(), 0);
    }

    #[test]
    fn budget_ge_rows_is_the_identity_split() {
        let s = stratified_split(&envelope(grouped_rows(3, 4)), "ag_news", 12);
        assert_eq!(s.n_picks, 12);
        // original order, not round-robin — the first-N law preserved.
        assert_eq!(front_texts(&s)[0], "t0_0");
        assert_eq!(front_texts(&s)[1], "t0_1");
    }

    #[test]
    fn unbalanced_labels_get_proportional_picks() {
        // banking77 shape: many labels, budget spanning all with a remainder
        // — every label picks at least once, no label more than one extra.
        // ("sst5" — an int-label suite; the option-key suites cap their
        // label universe at the key consts since issue 077.)
        let s = stratified_split(&envelope(grouped_rows(7, 2)), "sst5", 10);
        assert_eq!(s.n_picks, 10);
        let mut per_label: std::collections::HashMap<char, usize> =
            std::collections::HashMap::new();
        for t in front_texts(&s) {
            *per_label.entry(t.chars().nth(1).unwrap()).or_insert(0) += 1;
        }
        assert_eq!(per_label.len(), 7);
        assert!(per_label.values().all(|&n| (1..=2).contains(&n)));
    }

    #[test]
    fn unlabeled_rows_never_pick_and_stay_in_rest() {
        let mut rows = grouped_rows(2, 2);
        rows.push(serde_json::json!({ "text": "no_label" }));
        let s = stratified_split(&envelope(rows), "ag_news", 2);
        assert_eq!(s.n_picks, 2);
        assert!(!front_texts(&s).iter().any(|t| t == "no_label"));
        assert!(rest_texts(&s).contains(&"no_label".to_string()));
    }

    #[test]
    fn massive_label_rule_buckets_by_label_text() {
        // label_text rows (massive/label_text rule), first-appearance order.
        let rows = vec![
            serde_json::json!({ "label_text": "audio", "text": "a0" }),
            serde_json::json!({ "label_text": "play", "text": "p0" }),
            serde_json::json!({ "label_text": "audio", "text": "a1" }),
            serde_json::json!({ "label_text": "play", "text": "p1" }),
        ];
        let s = stratified_split(&envelope(rows), "massive_intent_en", 2);
        assert_eq!(front_texts(&s), ["a0", "p0"]);
    }

    #[test]
    fn mmlupro_jdi_sample_is_hash_ordered_not_first_n() {
        // The Bench-143/144 question, settled by execution: 30 rows with
        // question_ids 0..30, budget 5 — the picks must be the 5 SMALLEST
        // fnv1a64(decimal qid) hashes, NOT the first-N prefix, and a
        // TAIL append must be able to change the selection.
        let rows: Vec<serde_json::Value> = (0..30)
            .map(|i| serde_json::json!({ "question_id": i, "text": format!("t{i}") }))
            .collect();
        let s = stratified_split(&envelope(rows), "mmlupro_jdi", 5);
        assert_eq!(s.n_picks, 5);
        let picked: Vec<String> = front_texts(&s);
        // Recompute the expected set by the same hash law.
        let mut keyed: Vec<(u64, usize)> = (0..30)
            .map(|i| {
                let id = i.to_string();
                let mut h: u64 = 0xcbf2_9ce4_8422_2325;
                for &b in id.as_bytes() {
                    h ^= u64::from(b);
                    h = h.wrapping_mul(0x0010_0000_01b3);
                }
                (h, i)
            })
            .collect();
        keyed.sort();
        let want: Vec<String> = keyed.iter().take(5).map(|(_, i)| format!("t{i}")).collect();
        assert_eq!(picked, want, "the mmlupro sample must be hash-ordered");
        // And it is definitively not the first-N prefix (t0..t4) unless the
        // hash order says so — assert against the trivial law explicitly.
        let first_n: Vec<String> = (0..5).map(|i| format!("t{i}")).collect();
        assert_ne!(
            picked, first_n,
            "hash order must not coincide with file order here"
        );
    }

    /// Issue 088 adjudication (2026-10-14): the house fold's multiplier is
    /// `0x0010_0000_01b3` (2³⁶+435) — a digit transposition of the true
    /// FNV-1a 64 prime `0x0100_0000_01b3` — and the offset is standard.
    /// These vectors pin the HOUSE law so (a) external replication can
    /// reproduce it, and (b) any future "fix the prime" move fails loud
    /// here before it silently re-orders every JDI sample and invalidates
    /// every recorded JDI number and digest (the measured-wire break the
    /// adjudication declined to take).
    #[test]
    fn house_fold_known_vectors_not_canonical_fnv1a64() {
        let fold = |s: &str| {
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for &b in s.as_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0010_0000_01b3);
            }
            h
        };
        assert_eq!(fold(""), 0xcbf2_9ce4_8422_2325, "empty folds to the offset");
        // The Issue-088 discovery vector: canonical fnv1a64("9400") is
        // 0x15109e34d09bdec0; the house fold differs in the high 32 bits.
        assert_eq!(fold("9400"), 0x10b1_3a74_d09b_dec0);
        // And the divergence is in the multiplier, not the offset: the
        // canonical fold of the same byte differs.
        let canonical = |s: &str| {
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for &b in s.as_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
            h
        };
        assert_eq!(canonical("9400"), 0x1510_9e34_d09b_dec0);
        assert_ne!(fold("9400"), canonical("9400"));
    }
}

#[cfg(test)]
mod thai_builder_tests {
    //! Plan 003 T3.2 — the Thai probe-suite builders. Plan 010: also hosts
    //! the S1MB generic-builder tests (same module shape: builder + row
    //! envelope fixtures).

    use super::{QKind, build_s1mb_choice, build_s1mb_noul, build_s1mb_score};
    use super::{WISESIGHT_KEYS, build_thai_sib200, build_thai_wisesight, train_docs};

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        let wrapped: Vec<serde_json::Value> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| serde_json::json!({ "row_idx": i, "row": r }))
            .collect();
        serde_json::json!({ "rows": wrapped })
    }

    /// The FETCHED ClassLabel order (dataset_manifest.md — verified at
    /// fetch 2026-09-28): the builder's fixed criteria MUST match it.
    #[test]
    fn wisesight_fixed_options_match_the_classlabel_order() {
        let env = envelope(vec![serde_json::json!({ "texts": "สวัสดี", "category": 0 })]);
        let s = build_thai_wisesight(&env, 0);
        let crit = s.cases[0].questions[0].criteria.as_object().unwrap();
        let keys: Vec<&String> = crit.keys().collect();
        assert_eq!(keys, ["pos", "neu", "neg", "q"]);
    }

    #[test]
    fn wisesight_gold_is_the_int_category() {
        let env = envelope(vec![
            serde_json::json!({ "texts": "ดีมาก", "category": 0 }),
            serde_json::json!({ "texts": "เฉยๆ", "category": 1 }),
            serde_json::json!({ "texts": "แย่", "category": 2 }),
            serde_json::json!({ "texts": "ช่วยหน่อย", "category": 3 }),
        ]);
        let s = build_thai_wisesight(&env, 0);
        assert_eq!(s.cases.len(), 4);
        assert_eq!(s.cases[0].state["text"], "ดีมาก");
        assert_eq!(s.cases[3].gold[0].idx, 3);
        // state carries the `texts` column under the standard `text` key.
        assert_eq!(s.cases[0].id, "thai_wisesight:0");
    }

    #[test]
    fn wisesight_train_docs_read_the_texts_column() {
        let env = envelope(vec![
            serde_json::json!({ "texts": "a", "category": 1 }),
            serde_json::json!({ "texts": "b", "category": 2 }),
        ]);
        let docs = train_docs(&env, "thai_wisesight");
        assert_eq!(docs.len(), 2);
        // Issue 077: the int category maps through the OPTION-KEY consts —
        // the engine's domain names (content binding), not "1"/"2".
        assert_eq!(
            (docs[0].label.as_str(), docs[0].text.as_str()),
            (WISESIGHT_KEYS[1], "a")
        );
        assert_eq!(
            (docs[1].label.as_str(), docs[1].text.as_str()),
            (WISESIGHT_KEYS[2], "b")
        );
    }

    #[test]
    fn sib200_option_universe_is_sorted_unique_categories() {
        // Rows out of sorted order on purpose: the universe must come out
        // sorted regardless of first-appearance order.
        let env = envelope(vec![
            serde_json::json!({ "index_id": 1, "category": "travel", "text": "t1" }),
            serde_json::json!({ "index_id": 2, "category": "health", "text": "t2" }),
            serde_json::json!({ "index_id": 3, "category": "travel", "text": "t3" }),
        ]);
        let s = build_thai_sib200(&env, 0);
        let crit = s.cases[0].questions[0].criteria.as_object().unwrap();
        let keys: Vec<&String> = crit.keys().collect();
        assert_eq!(keys, ["health", "travel"]);
        // gold = the key position of the row's own category.
        assert_eq!(s.cases[0].gold[0].idx, 1); // travel
        assert_eq!(s.cases[1].gold[0].idx, 0); // health
        assert_eq!(s.cases[0].id, "thai_sib200:0");
    }

    #[test]
    fn sib200_train_docs_carry_the_category_string() {
        let env = envelope(vec![
            serde_json::json!({ "index_id": 1, "category": "politics", "text": "x" }),
            serde_json::json!({ "index_id": 2, "category": "sports", "text": "y" }),
        ]);
        let docs = train_docs(&env, "thai_sib200");
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].label, "politics");
        assert_eq!(docs[1].label, "sports");
    }

    // ── S1MB (Plan 010): the generic converter-row builder ─────────────

    fn s1mb_row(subset: &str, q: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "subset": subset,
            "group": "domain",
            "case_id": format!("{subset}:hash"),
            "state": "{\"prompt\": \"x\"}",
            "question": q,
        })
    }

    #[test]
    fn s1mb_choice_builder_maps_gold_by_presented_position() {
        let env = envelope(vec![s1mb_row(
            "arc",
            serde_json::json!({
                "qid": "answer", "kind": "choice", "instructions": "pick",
                "criteria": {"B": "desc-b", "C": null, "A": null, "D": null},
                "keys": ["B", "C", "A", "D"],
                "gold_idx": 2, "gold_score": null,
                "soft": [0.0, 0.0, 1.0, 0.0],
            }),
        )]);
        let s = build_s1mb_choice(&env, 0);
        assert_eq!(s.name, "s1mb_choice");
        assert_eq!(s.cases.len(), 1);
        let c = &s.cases[0];
        assert_eq!(c.id, "arc:hash:answer");
        // The state rides verbatim (Value::String) — serialize_state passthrough.
        assert_eq!(c.state.as_str(), Some("{\"prompt\": \"x\"}"));
        let q = &c.questions[0];
        assert_eq!(q.kind, QKind::Choice);
        let keys: Vec<_> = q.criteria.as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, vec!["B", "C", "A", "D"]);
        assert_eq!(c.gold[0].idx, 2);
        assert_eq!(c.gold[0].soft, vec![0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn s1mb_score_builder_carries_numeric_levels_and_gold_score() {
        let env = envelope(vec![s1mb_row(
            "s1mb-generalization-diverse-score",
            serde_json::json!({
                "qid": "rate", "kind": "score", "instructions": "rate",
                "criteria": [0.0, 0.3333333333333333, 0.6666666666666666, 1.0],
                "keys": ["0", "1", "2", "3"],
                "gold_idx": 2, "gold_score": 0.6666666666666666,
                "soft": [0.0, 0.0, 1.0, 0.0],
            }),
        )]);
        let s = build_s1mb_score(&env, 0);
        let q = &s.cases[0].questions[0];
        assert_eq!(q.kind, QKind::Score);
        assert_eq!(q.criteria.as_array().unwrap().len(), 4);
        assert_eq!(s.cases[0].gold[0].gold_score, Some(0.6666666666666666));
        assert_eq!(s.cases[0].gold[0].idx, 2);
    }

    #[test]
    fn s1mb_noul_builder_maps_false_true_and_the_label_arm_keys_the_corpus() {
        let env = envelope(vec![
            s1mb_row(
                "babi_nli",
                serde_json::json!({
                    "qid": "support", "kind": "noul", "instructions": "Does it support?",
                    "criteria": null, "keys": ["false", "true"],
                    "gold_idx": 1, "gold_score": null, "soft": [0.0, 1.0],
                }),
            ),
            s1mb_row(
                "babi_nli",
                serde_json::json!({
                    "qid": "support", "kind": "noul", "instructions": "Does it support?",
                    "criteria": null, "keys": ["false", "true"],
                    "gold_idx": 0, "gold_score": null, "soft": [1.0, 0.0],
                }),
            ),
        ]);
        let s = build_s1mb_noul(&env, 0);
        assert_eq!(s.cases[0].questions[0].kind, QKind::Noul);
        assert!(s.cases[0].questions[0].criteria.is_null());
        assert_eq!(s.cases[0].gold[0].idx, 1);
        assert_eq!(s.cases[1].gold[0].idx, 0);
        // The corpus label = the gold key string ("true"/"false") — the
        // noul-exempt branch rides train-doc labels, so the arm must speak
        // the same spelling the gold positions do.
        let docs = train_docs(&env, "s1mb_noul");
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].label, "true");
        assert_eq!(docs[1].label, "false");
        // The choice corpus label = the gold KEY string (the option-key
        // union branch — default — must meet it).
        let env2 = envelope(vec![s1mb_row(
            "arc",
            serde_json::json!({
                "qid": "answer", "kind": "choice", "instructions": "pick",
                "criteria": {"B": "d", "C": null, "A": null, "D": null},
                "keys": ["B", "C", "A", "D"],
                "gold_idx": 0, "gold_score": null, "soft": [1.0, 0.0, 0.0, 0.0],
            }),
        )]);
        let docs2 = train_docs(&env2, "s1mb_choice");
        assert_eq!(docs2[0].label, "B");
        assert_eq!(docs2[0].text, "{\"prompt\": \"x\"}");
    }
}

// Issue 114 T3: the When2Call builder — the official MCQ form rendering.
#[cfg(test)]
mod when2call_builder_tests {
    use super::{WHEN2CALL_KEYS, build_when2call};

    /// A row in the fetched shape: `answers` keyed by the fixed category
    /// order, `tools` a list of schema strings, gold naming the category.
    fn row(correct: &str, tools: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "uuid": "u-1",
            "question": "book a table for six",
            "correct_answer": correct,
            "answers": {
                "direct": "Your table is booked.",
                "tool_call": "{\"name\": \"book_table\", \"arguments\": {\"size\": 6}}",
                "request_for_info": "How many people should I book for?",
                "cannot_answer": "I cannot help with that."
            },
            "tools": tools,
        })
    }

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({ "rows": rows.into_iter().map(|r| { serde_json::json!({ "row": r }) }).collect::<Vec<_>>() })
    }

    #[test]
    fn when2call_builder_renders_the_official_mcq_form() {
        let env = envelope(vec![row(
            "tool_call",
            serde_json::json!(["{\"name\": \"book_table\"}"]),
        )]);
        let s = build_when2call(&env, 0);
        assert_eq!(s.name, "when2call");
        assert_eq!(s.cases.len(), 1);
        let c = &s.cases[0];
        // Option keys = the fixed category universe in answers-key order.
        let keys: Vec<_> = c.questions[0]
            .criteria
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, WHEN2CALL_KEYS);
        // Criteria VALUES = the row's own response candidates (the official
        // MCQ choices), carried per option.
        let crit = c.questions[0].criteria.as_object().unwrap();
        assert_eq!(crit["direct"], "Your table is booked.");
        assert_eq!(
            crit["tool_call"],
            "{\"name\": \"book_table\", \"arguments\": {\"size\": 6}}"
        );
        // Gold = the category's position in the answers-key order (the
        // official eval's answer_names.index law).
        assert_eq!(c.gold[0].idx, 1);
        // State = query + the tool-schema strings AS STORED.
        assert_eq!(c.state["query"], "book a table for six");
        assert_eq!(
            c.state["tools"],
            serde_json::json!(["{\"name\": \"book_table\"}"])
        );
    }

    #[test]
    fn when2call_builder_keeps_the_empty_tools_tier_and_the_cannot_answer_gold() {
        // The 258-row no-tools tier (BFCL Live Simple cannot_answer rows):
        // the empty tools list rides verbatim — never filled.
        let env = envelope(vec![row("cannot_answer", serde_json::json!([]))]);
        let s = build_when2call(&env, 0);
        let c = &s.cases[0];
        assert_eq!(c.state["tools"], serde_json::json!([]));
        assert_eq!(c.gold[0].idx, 3); // cannot_answer's answers-key position
    }

    #[test]
    #[should_panic(expected = "answers-object key order drifted")]
    fn when2call_builder_refuses_a_drifted_answers_key_order() {
        // The gold index is the POSITION in the answers-key order — a
        // reordered object would silently rebind every row, so the order is
        // pinned per row (the clinc150 ClassLabel law).
        let env = envelope(vec![serde_json::json!({
            "uuid": "u-1",
            "question": "q",
            "correct_answer": "tool_call",
            "answers": {
                "tool_call": "a",
                "direct": "b",
                "request_for_info": "c",
                "cannot_answer": "d"
            },
            "tools": ["t"]
        })]);
        let _ = build_when2call(&env, 0);
    }
}

#[cfg(test)]
mod bfcl_builder_tests {
    use super::{BFCL_ABSTAIN_KEY, build_bfcl};

    fn pick_row(id: &str, gold: &str, names: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "question": [[{"role": "user", "content": "the query"}]],
            "function": names.iter().map(|n| serde_json::json!({
                "name": n, "description": format!("desc {n}")
            })).collect::<Vec<_>>(),
            "ground_truth": [{ gold: {} }],
        })
    }

    fn irrelevance_row(id: &str, names: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "question": [[{"role": "user", "content": "the query"}]],
            "function": names.iter().map(|n| serde_json::json!({
                "name": n, "description": format!("desc {n}")
            })).collect::<Vec<_>>(),
        })
    }

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({ "rows": rows.into_iter().map(|r| { serde_json::json!({ "row": r }) }).collect::<Vec<_>>() })
    }

    fn keys_of(s: &super::Suite, case: usize) -> Vec<String> {
        s.cases[case].questions[0]
            .criteria
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }

    #[test]
    fn bfcl_pick_row_golds_the_joined_function_name() {
        let env = envelope(vec![pick_row(
            "simple_0",
            "fn_b",
            &["fn_a", "fn_b", "fn_c"],
        )]);
        let s = build_bfcl(&env, 0);
        assert_eq!(s.name, "bfcl");
        assert_eq!(s.cases.len(), 1);
        assert_eq!(s.cases[0].gold[0].idx, 1); // fn_b's position
        // The state is the row's own task input AS STORED.
        assert_eq!(
            s.cases[0].state["tools"],
            serde_json::json!([
                {"name": "fn_a", "description": "desc fn_a"},
                {"name": "fn_b", "description": "desc fn_b"},
                {"name": "fn_c", "description": "desc fn_c"}
            ])
        );
        // The pick tier carries NO abstain option (the official pick eval
        // always has a gold function — a never-gold distractor would be an
        // invented transform).
        assert!(!keys_of(&s, 0).iter().any(|k| k == BFCL_ABSTAIN_KEY));
    }

    #[test]
    fn bfcl_irrelevance_row_appends_the_abstain_option_last_and_golds_it() {
        let env = envelope(vec![irrelevance_row("irrelevance_0", &["fn_x", "fn_y"])]);
        let s = build_bfcl(&env, 0);
        let keys = keys_of(&s, 0);
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.last().unwrap(), BFCL_ABSTAIN_KEY);
        assert_eq!(s.cases[0].gold[0].idx, 2); // the abstain option, last
    }

    #[test]
    fn bfcl_gold_resolution_matches_the_unique_leaf_name() {
        // The pinned simple_363 shape: the gold spells the leaf name while
        // the candidate carries the module-qualified form.
        let env = envelope(vec![pick_row(
            "simple_363",
            "find_closest",
            &["restaurant_search.find_closest", "other_thing"],
        )]);
        let s = build_bfcl(&env, 0);
        assert_eq!(s.cases[0].gold[0].idx, 0);
    }

    #[test]
    #[should_panic(expected = "matches no candidate by full or unique leaf name")]
    fn bfcl_refuses_a_gold_matching_no_candidate() {
        let env = envelope(vec![pick_row("simple_9", "ghost_fn", &["fn_a", "fn_b"])]);
        let _ = build_bfcl(&env, 0);
    }

    #[test]
    #[should_panic(expected = "matches no candidate by full or unique leaf name")]
    fn bfcl_refuses_an_ambiguous_leaf_match() {
        // Two candidates sharing the gold's leaf name — the leaf rule must
        // stay unambiguous.
        let env = envelope(vec![pick_row(
            "simple_9",
            "get",
            &["a.get", "b.get", "c.other"],
        )]);
        let _ = build_bfcl(&env, 0);
    }

    #[test]
    #[should_panic(expected = "carries no joined ground_truth")]
    fn bfcl_refuses_a_pick_row_without_the_joined_gold() {
        let env = envelope(vec![serde_json::json!({
            "id": "simple_0",
            "question": [[{"role": "user", "content": "q"}]],
            "function": [{"name": "fn_a", "description": "d"}],
        })]);
        let _ = build_bfcl(&env, 0);
    }

    #[test]
    fn bfcl_drops_the_empty_candidate_irrelevance_rows() {
        let env = envelope(vec![
            irrelevance_row("live_irrelevance_120-9-0", &[]),
            irrelevance_row("live_irrelevance_124-9-4", &["fn_z"]),
        ]);
        let s = build_bfcl(&env, 0);
        assert_eq!(s.cases.len(), 1); // the empty-candidate row dropped
    }
}

#[cfg(test)]
mod api_bank_builder_tests {
    use super::build_api_bank;

    fn bank_row(gold_api: &str, candidates: &[&str], expected: &str) -> serde_json::Value {
        let mut block = String::from("ctx\nAPI descriptions:\n");
        for c in candidates {
            block.push_str(&format!(
                "{{\"name\": \"{c}\", \"description\": \"desc {c}\"}}\n"
            ));
        }
        serde_json::json!({
            "id": 0,
            "instruction": block,
            "expected_output": expected.replace("GOLD", gold_api),
            "input": "User: hi\nAI: hello",
        })
    }

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({ "rows": rows.into_iter().map(|r| { serde_json::json!({ "row": r }) }).collect::<Vec<_>>() })
    }

    #[test]
    fn api_bank_golds_the_expected_output_api_name() {
        let env = envelope(vec![bank_row(
            "ApiTwo",
            &["ApiOne", "ApiTwo"],
            "API-Request: [GOLD(key='v')]",
        )]);
        let s = build_api_bank(&env, 0);
        assert_eq!(s.name, "api_bank");
        assert_eq!(s.cases.len(), 1);
        assert_eq!(s.cases[0].gold[0].idx, 1); // ApiTwo's position
        let keys: Vec<_> = s.cases[0].questions[0]
            .criteria
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, vec!["ApiOne".to_string(), "ApiTwo".to_string()]);
        // The state carries the instruction VERBATIM (dialogue + candidate
        // specs AS STORED — the no-silent-transforms law).
        assert!(
            s.cases[0].state["instruction"]
                .as_str()
                .unwrap()
                .contains("API descriptions:")
        );
    }

    #[test]
    fn api_bank_drops_a_row_without_the_api_request_prefix() {
        let env = envelope(vec![bank_row("ApiOne", &["ApiOne"], "no call")]);
        let s = build_api_bank(&env, 0);
        assert!(s.cases.is_empty()); // census pins 444 kept (399 + 45)
    }

    #[test]
    fn api_bank_drops_the_downstream_api_level2_rows() {
        // The level-2 ToolSearcher-gated shape: the presented candidate is
        // the only choice; a gold naming the DOWNSTREAM API is not
        // answerable as a choice and drops (the R2b scoreable-partial law,
        // row level).
        let env = envelope(vec![
            bank_row(
                "DownstreamApi",
                &["ToolSearcher"],
                "API-Request: [GOLD(k='v')]",
            ),
            bank_row(
                "ToolSearcher",
                &["ToolSearcher"],
                "API-Request: [GOLD(k='v')]",
            ),
        ]);
        let s = build_api_bank(&env, 0);
        assert_eq!(s.cases.len(), 1); // only the ToolSearcher-gold row kept
        assert_eq!(s.cases[0].gold[0].idx, 0);
    }
}

// Issue 114 T8: the FinEntity builder — the entity-given sentiment span
// cases, the conflicting-exact-span quarantine, and the gold mapping.
#[cfg(test)]
mod finentity_builder_tests {
    use super::{FINENTITY_KEYS, build_finentity};

    fn doc(annotations: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "content": "Johnson & Johnson <JNJ.N> shares gained after beating expectations.",
            "annotations": annotations,
        })
    }

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({ "rows": rows.into_iter().map(|r| { serde_json::json!({ "row": r }) }).collect::<Vec<_>>() })
    }

    #[test]
    fn finentity_renders_one_case_per_span_with_the_state_faithful() {
        let doc = doc(serde_json::json!([
            {"value": "Johnson & Johnson", "start": 0, "end": 17, "label": "Positive", "tag": "Positive"},
            {"value": "JNJ.N", "start": 18, "end": 23, "label": "Neutral", "tag": "Neutral"}
        ]));
        let env = envelope(vec![doc]);
        let s = build_finentity(&env, 1); // nonzero cap: the census asserts rest
        assert_eq!(s.name, "finentity");
        assert_eq!(s.cases.len(), 2);
        let keys: Vec<_> = s.cases[0].questions[0]
            .criteria
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, FINENTITY_KEYS); // sorted unique upstream labels
        // Gold = the span's label position in FINENTITY_KEYS.
        assert_eq!(s.cases[0].gold[0].idx, 2); // Positive
        assert_eq!(s.cases[1].gold[0].idx, 1); // Neutral
        // State = {text, entity, start, end} — the span PROVIDED (the
        // board's own explainer), the document verbatim.
        assert_eq!(s.cases[0].state["entity"], "Johnson & Johnson");
        assert_eq!(s.cases[0].state["start"], 0);
        assert_eq!(s.cases[0].state["end"], 17);
        assert!(
            s.cases[0].state["text"]
                .as_str()
                .unwrap()
                .starts_with("Johnson & Johnson")
        );
    }

    #[test]
    fn finentity_quarantines_both_rows_of_the_conflicting_exact_span() {
        // Doc 299's shape: the SAME (start, end) span labelled Neutral and
        // Positive — an unresolved annotation conflict; BOTH rows drop
        // (never a guessed pick), same-document neighbours stay.
        let doc = doc(serde_json::json!([
            {"value": "S&P 500", "start": 20, "end": 27, "label": "Neutral", "tag": "Neutral"},
            {"value": "S&P 500", "start": 20, "end": 27, "label": "Positive", "tag": "Positive"},
            {"value": "SPLRCM", "start": 80, "end": 87, "label": "Negative", "tag": "Negative"}
        ]));
        let env = envelope(vec![doc]);
        let s = build_finentity(&env, 1);
        assert_eq!(s.cases.len(), 1); // only the unambiguous span kept
        assert_eq!(s.cases[0].gold[0].idx, 0); // Negative
        assert_eq!(s.cases[0].state["entity"], "SPLRCM");
    }

    #[test]
    fn finentity_keeps_same_label_exact_duplicates() {
        // Two identical same-label rows for one span (the BlackRock
        // census shape) — both stay (the board's fields count pins it).
        let doc = doc(serde_json::json!([
            {"value": "BlackRock Inc", "start": 0, "end": 13, "label": "Neutral", "tag": "Neutral"},
            {"value": "BlackRock Inc", "start": 0, "end": 13, "label": "Neutral", "tag": "Neutral"}
        ]));
        let env = envelope(vec![doc]);
        let s = build_finentity(&env, 1);
        assert_eq!(s.cases.len(), 2);
        assert_eq!(s.cases[0].gold[0].idx, 1);
        assert_eq!(s.cases[1].gold[0].idx, 1);
    }

    #[test]
    #[should_panic(expected = "outside FINENTITY_KEYS")]
    fn finentity_refuses_a_drifted_label_universe() {
        let doc = doc(serde_json::json!([
            {"value": "X", "start": 0, "end": 1, "label": "Bullish", "tag": "Bullish"}
        ]));
        let env = envelope(vec![doc]);
        let _ = build_finentity(&env, 1);
    }
}

// Issue 114 T9: the ContractNLI builder — the (contract, hypothesis)
// pair cases, the choice-key mapping, and the split-column eval
// discriminator.
#[cfg(test)]
mod contractnli_builder_tests {
    use super::{CONTRACTNLI_KEYS, build_contractnli};

    fn pair(choice: &str, split: &str) -> serde_json::Value {
        serde_json::json!({
            "contract": "All Confidential Information shall be returned upon termination.",
            "hypothesis": "Confidential information must be returned at the end.",
            "choice": choice,
            "split": split,
        })
    }

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({ "rows": rows.into_iter().map(|r| { serde_json::json!({ "row": r }) }).collect::<Vec<_>>() })
    }

    #[test]
    fn contractnli_renders_the_pair_case_with_keys_and_gold() {
        let env = envelope(vec![
            pair("Entailment", "test"),
            pair("NotMentioned", "test"),
            pair("Contradiction", "test"),
        ]);
        let s = build_contractnli(&env, 3); // nonzero cap: the census assert rests
        assert_eq!(s.name, "contractnli");
        assert_eq!(s.cases.len(), 3);
        let keys: Vec<_> = s.cases[0].questions[0]
            .criteria
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, CONTRACTNLI_KEYS); // sorted unique upstream choices
        assert_eq!(s.cases[0].gold[0].idx, 1); // Entailment
        assert_eq!(s.cases[1].gold[0].idx, 2); // NotMentioned
        assert_eq!(s.cases[2].gold[0].idx, 0); // Contradiction
        // State = contract + hypothesis verbatim (the recorded transform's
        // pair shape).
        assert_eq!(
            s.cases[0].state["hypothesis"],
            "Confidential information must be returned at the end."
        );
    }

    #[test]
    fn contractnli_train_rows_build_the_cal_front_without_the_census() {
        // The train envelope reads through the SAME builder (the corpus-
        // ON posture): split=train rows never trip the eval census.
        let env = envelope(vec![
            pair("Entailment", "train"),
            pair("Contradiction", "train"),
        ]);
        let s = build_contractnli(&env, 0); // full load, but split=train
        assert_eq!(s.cases.len(), 2);
    }

    #[test]
    #[should_panic(expected = "outside CONTRACTNLI_KEYS")]
    fn contractnli_refuses_a_drifted_choice_universe() {
        let env = envelope(vec![pair("Maybe", "test")]);
        let _ = build_contractnli(&env, 1);
    }
}

// Issue 114 T10: the iSarcasmEval builder — the track-A binary pair,
// the sarcastic-class-positive key order, and the split-dispatched census.
#[cfg(test)]
mod isarcasmeval_builder_tests {
    use super::{ISARCASM_KEYS, build_isarcasmeval};

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({ "rows": rows.into_iter().map(|r| { serde_json::json!({ "row": r }) }).collect::<Vec<_>>() })
    }

    fn a_row(text: &str, sarcastic: &str, split: &str) -> serde_json::Value {
        serde_json::json!({ "text": text, "sarcastic": sarcastic, "split": split })
    }

    #[test]
    fn isarcasmeval_renders_one_case_per_tweet_with_sarcastic_positive() {
        let env = envelope(vec![
            a_row("Pinball!", "0", "test"),
            a_row("Oh great.", "1", "test"),
        ]);
        let s = build_isarcasmeval(&env, 1); // nonzero cap: the census asserts rest
        assert_eq!(s.name, "isarcasmeval");
        assert_eq!(s.cases.len(), 1);
        let keys: Vec<_> = s.cases[0].questions[0]
            .criteria
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, ISARCASM_KEYS); // sorted upstream spellings; index 1 = sarcastic
        assert_eq!(keys, ["non-sarcastic", "sarcastic"]);
        // Gold = the binary label mapped onto the key order — index 1 is
        // the sarcastic class, exactly where hard_metrics::positive_f1
        // (binary, index-1 side) and the board metric read it.
        assert_eq!(s.cases[0].gold[0].idx, 0); // "0" → non-sarcastic
        let s2 = build_isarcasmeval(&envelope(vec![a_row("Oh great.", "1", "test")]), 1);
        assert_eq!(s2.cases[0].gold[0].idx, 1); // "1" → sarcastic
        // State = the tweet verbatim (the author label carries the whole
        // signal — the response-alone law transfers).
        assert_eq!(s.cases[0].state["tweet"], "Pinball!");
    }

    #[test]
    fn isarcasmeval_train_rows_build_the_cal_front_without_the_census() {
        let env = envelope(vec![a_row("a", "0", "train"), a_row("b", "1", "train")]);
        let s = build_isarcasmeval(&env, 0); // full load, but split=train
        assert_eq!(s.cases.len(), 2);
    }

    #[test]
    #[should_panic(expected = "outside ISARCASM_KEYS")]
    fn isarcasmeval_refuses_a_drifted_label_universe() {
        let env = envelope(vec![a_row("x", "2", "test")]);
        let _ = build_isarcasmeval(&env, 1);
    }
}

// ── Issue 114 T11: the home_appliance builder tests ────────────────────────
#[cfg(test)]
mod home_appliance_builder_tests {
    use super::build_home_appliance;
    use crate::harness::runner::question_option_count;
    use serde_json::json;

    /// One board-shaped row: two questions (the 9-option resolution
    /// universe + a 2-option target-member), opaque k_ keys, the texts in
    /// the criteria values, expected naming the gold KEY.
    fn board_row(id: &str, gold_key: &str, member_gold: &str) -> serde_json::Value {
        json!({
            "id": id,
            "group_id": id,
            "split": "test",
            "state": {"request": "Start the evening group, please.", "rooms": ["kitchen", "den"]},
            "questions": {
                "q_res": {"type": "choice", "instructions": "How should the assistant handle the request?",
                    "criteria": {"k_a": "execute now", "k_b": "execute available parts now", "k_c": "schedule for later",
                                 "k_d": "save an automation rule", "k_e": "ask a clarifying question", "k_f": "make no change because already satisfied",
                                 "k_g": "reject as unsupported", "k_h": "block under household policy", "k_i": "answer a state query"}},
                "q_member": {"type": "choice", "instructions": "Is kitchen_light in the justified device target set for this request?",
                    "criteria": {"k_y": "yes, this device is in the justified target set",
                                 "k_n": "no, this device is not in the justified target set"}}
            },
            "expected": {"q_res": gold_key, "q_member": member_gold}
        })
    }

    fn envelope(rows: Vec<serde_json::Value>) -> serde_json::Value {
        json!({"rows": rows.into_iter().map(|r| json!({"row": r})).collect::<Vec<_>>()})
    }

    #[test]
    fn home_appliance_maps_the_board_row_to_a_multi_question_case() {
        let env = envelope(vec![board_row(
            "home-appliance:test:indigo:00",
            "k_e",
            "k_y",
        )]);
        let s = build_home_appliance(&env, 1); // nonzero cap: the census asserts rest
        assert_eq!(s.name, "home_appliance");
        assert_eq!(s.cases.len(), 1);
        let c = &s.cases[0];
        assert_eq!(c.id, "home_appliance:home-appliance:test:indigo:00");
        assert_eq!(c.questions.len(), 2);
        // The k walk: resolution 9, member 2 (the case-exact chance input).
        let ks: Vec<usize> = c.questions.iter().map(question_option_count).collect();
        assert_eq!(ks, vec![9, 2]);
        // The gold is the criteria KEY's position (k_e = the 5th inserted).
        assert_eq!(c.gold[0].idx, 4);
        assert_eq!(c.gold[1].idx, 0); // k_y = first inserted
        // Soft vectors are zero-sum (soft_metrics reads None — the convention).
        assert!(c.gold[0].soft.iter().all(|&v| v == 0.0));
        // State rides verbatim (the engine reads the request from it).
        assert_eq!(c.state["request"], "Start the evening group, please.");
        // The criteria objects carry the opaque keys + the texts (the
        // engine renders "{key}: {value}" — insertion order preserved).
        let crit = c.questions[0].criteria.as_object().unwrap();
        assert_eq!(crit.len(), 9);
        assert_eq!(crit.values().next().unwrap(), "execute now");
    }

    #[test]
    #[should_panic(expected = "is not a criteria key")]
    fn home_appliance_refuses_a_gold_key_outside_the_criteria() {
        let env = envelope(vec![board_row("r1", "k_MISSING", "k_y")]);
        let _ = build_home_appliance(&env, 1);
    }

    #[test]
    #[should_panic(expected = "choice/noul only")]
    fn home_appliance_refuses_a_score_question() {
        let mut row = board_row("r1", "k_a", "k_y");
        row["questions"]["q_res"]["type"] = json!("score");
        let env = envelope(vec![row]);
        let _ = build_home_appliance(&env, 1);
    }

    #[test]
    #[should_panic(expected = "drifted from the pinned kept set")]
    fn home_appliance_full_load_pins_the_population() {
        let env = envelope(vec![board_row("r1", "k_a", "k_y")]);
        let _ = build_home_appliance(&env, 0); // the census fires at max_rows 0
    }
}
