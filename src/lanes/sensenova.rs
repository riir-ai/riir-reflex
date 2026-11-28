//! The SenseNova-Loop-9B local-teacher lane (rethink Issue 068's cell,
//! feeding riir-reflex Issue 134): SenseTime's qwen3.5-family looped
//! transformer served by the `OpenSenseNova/llama.cpp` FORK @ `24ea7bb7`
//! over llama-server's OpenAI-compatible wire — the FIRST chat-completions
//! lane in this repo (every sibling speaks the SystemOne dialect; this one
//! owns the OpenAI translation, which is why it exists: the mammoth lane's
//! model-field-only reuse was REFUTED at the 068 premise check —
//! `lanes/mammoth.rs` speaks `build_body_for_model`'s SystemOne grammar,
//! not `messages`).
//!
//! **License:** weights Apache-2.0 (teacher-eligible), runtime
//! Apache-2.0 (the fork, same as mainline llama.cpp), GGUF build official
//! (release-day addition 2026-11-18; Q4_K_M sha256
//! `5528b06c41a92d0a2413e827ab08531be56b10f881606c6659361b5da507eeed` —
//! the lossy-surface law: the quant posture rides the
//! [`DEVICE_ENV`] stamp beside every number). MEASUREMENT-ONLY (the
//! mammoth lane's T10.2 law, inherited): no capture artifact, no RIDT arm,
//! no raw-body persistence beyond the determinism rerun's in-memory
//! compare.
//!
//! **Serving:** the FORK is the only runtime — mainline llama.cpp does NOT
//! carry the `sensenova_loop` arch (the Dory precedent repeating: fork-only
//! support, unmerged; the fork is mainline `9c003a97a` + exactly TWO
//! commits, `40de778` Loop + `24ea7bb` Loop MTP — the smallest possible
//! support delta, reviewable as a future mainline PR; upstreaming is an
//! owner call, never a lane act). Boot line (the 068 probe's recorded
//! posture): `llama-server -m <gguf> --port 18080 -ngl 99` (Metal `-ngl 99`
//! full offload; peak RSS ≈ 6.9 GiB; load-to-listening ≈ 12 s).
//!
//! **Wire (observed at the 068 probe — the parser's authority, never
//! guessed):**
//!
//! * request: OpenAI `/v1/chat/completions` — `model` (set it EXPLICITLY:
//!   the server echoes the LOAD PATH as the id when launched `-m <path>`;
//!   under `-hf org/repo:Q4_K_M` it is the HF id + tag), `messages`
//!   (single user turn), `temperature: 0.0` (the judge-seat law), `stream:
//!   false`.
//! * **`content` carries ONLY the final answer — the chain-of-thought
//!   lands in `message.reasoning_content`.** A consumer must read BOTH or
//!   it sees an empty content when `max_tokens` is consumed by reasoning
//!   (the probe's own first smoke hit it: max_tokens=16 → content:""
//!   finish:"length"). This lane therefore sets a GENEROUS `max_tokens`
//!   (the reasoning must finish before the answer spells) and reads
//!   `content` as the answer channel, recording `reasoning_content`'s
//!   presence in the parse errors when the answer is missing.
//! * `finish_reason`: `stop` = answered; `length` = the budget was
//!   consumed (content may be empty or truncated — a LOUD parse error
//!   naming the knob, never a silent empty-answer fold).
//!
//! **Answer extraction:** the model answers in PROSE over the presented
//! options; the parser is a strict labeled extractor over `content`:
//! choice/score/noul answers must carry the option key or level index
//! verbatim (the pplx parser's loud-error law — never a positional or
//! fuzzy guess). The extraction grammar is pinned in
//! [`extract_answer_key`]; anything it cannot name is a loud refusal
//! naming the content, so a wire/model drift surfaces on the first row,
//! not as silent miskeys.
//!
//! Env contract (never a committed credential — a loopback llama-server
//! needs none, so the gate is the explicit forwarder/no-auth acknowledgment):
//!
//! * `SENSENOVA_SERVE_URL` — default `http://127.0.0.1:18080` (the probe's
//!   recorded scratch port);
//! * `SENSENOVA_MODEL` — the request `model` field; REQUIRED (the server
//!   echoes the load path — a default here would pin THIS box's path into
//!   every other box's records);
//! * `SENSENOVA_AUTH_OK` — the explicit `=1` acknowledgment that the
//!   endpoint is the operator's own loopback/fenced serve (the mammoth
//!   FORWARDER_AUTH shape; a bare internet endpoint is refused — this lane
//!   is a SELF-HOSTED posture, never a hosted one);
//! * `SENSENOVA_DEVICE` — the quant-posture disclosure stamp, REQUIRED
//!   (the lossy-surface law: e.g. `metal+q4_k_m-ngl99`; which quant
//!   produced the scores gates the reads);
//! * `SENSENOVA_TIMEOUT_MS` — per-request timeout, default 120000 (a
//!   reasoning model spends tokens before the answer; the loaded-box
//!   0.94 tok/s probe posture needs the headroom);
//! * `SENSENOVA_MAX_TOKENS` — the generation budget, default 2048 (the
//!   reasoning must finish before the answer spells);
//! * `SENSENOVA_SMOKE_MAX_CASES` — the spend ceiling (cases per suite),
//!   default 50 (a slow self-host is metered like a hosted endpoint — the
//!   clef A6 law); a larger run refuses unless `SENSENOVA_ALLOW_UNCAPPED=1`.
//!
//! **Reasoning-class checkpoints (the i068 cell's recorded constraint):**
//! a REASONING-trained checkpoint served WITHOUT a reasoning dial needs
//! `SENSENOVA_TIMEOUT_MS` and `SENSENOVA_MAX_TOKENS` sized to its reasoning
//! length, not to the answer's — SenseNova-Loop-9B at Metal-Q4 reasoned
//! 2000–4000+ tokens per trivial question (~7.3 t/s ⇒ 300 s+ before the
//! first content token; both the 2048 budget and the 120 s/300 s timeouts
//! starved before any answer). The OpenSenseNova fork's dials are the
//! escape: `--reasoning-budget 0` (or `--reasoning off`) at serve time
//! collapses the reasoning phase — the i068 cell's practical posture (1
//! s/case at the dial vs 300+ s trained). TRADE-OFF of record: the no-CoT
//! posture answers off-contract often enough that the verbatim-key
//! extraction grammar aborts suites under the harness's refuses-loud law
//! (banking77 died at case 3, massive at 187 — the reasoning phase was
//! organizing the answers; strip it and the prose goes sloppy). Full
//! full record: rethink Issue 068 (parked, UNDECIDABLE-by-absence).
//!
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::harness::suites::{QKind, SuiteCase, SuiteQuestion};
use crate::lanes::http_mini;

/// The env var naming the serve base URL.
pub const SERVE_URL_ENV: &str = "SENSENOVA_SERVE_URL";
/// The default posture: the 068 probe's recorded scratch port.
pub const DEFAULT_SERVE_URL: &str = "http://127.0.0.1:18080";
/// The request `model` field — REQUIRED (the server echoes the load path;
/// a default would pin one box's path into every record).
pub const MODEL_ENV: &str = "SENSENOVA_MODEL";
/// The explicit self-hosted-endpoint acknowledgment (`=1`).
pub const AUTH_OK_ENV: &str = "SENSENOVA_AUTH_OK";
/// The quant-posture disclosure stamp — REQUIRED.
pub const DEVICE_ENV: &str = "SENSENOVA_DEVICE";
/// Per-request timeout, milliseconds.
pub const TIMEOUT_ENV: &str = "SENSENOVA_TIMEOUT_MS";
pub const DEFAULT_TIMEOUT_MS: u64 = 120_000;
/// The generation budget (reasoning spends before the answer spells).
pub const MAX_TOKENS_ENV: &str = "SENSENOVA_MAX_TOKENS";
pub const DEFAULT_MAX_TOKENS: u32 = 2048;
/// The spend ceiling (cases per suite), the clef A6 law.
pub const MAX_CASES_ENV: &str = "SENSENOVA_SMOKE_MAX_CASES";
pub const DEFAULT_MAX_CASES: usize = 50;
/// The explicit uncapped override (`=1`).
pub const UNCAPPED_ENV: &str = "SENSENOVA_ALLOW_UNCAPPED";

/// The license token every artifact from this lane must carry (the
/// attribution law; weights + runtime both Apache-2.0 — teacher-eligible,
/// the mammoth contrast).
pub const LICENSE_LINE: &str = "Apache-2.0 (sensenova/SenseNova-Loop-9B-GGUF \
     weights; OpenSenseNova/llama.cpp @ 24ea7bb7 runtime) — served by the \
     OpenSenseNova fork (mainline lacks the arch; the Dory fork-only \
     precedent); quant posture rides SENSENOVA_DEVICE";

/// The prompt the lane sends per question (the OpenAI wire's `messages`
/// payload): the state as the context, the instructions + the presented
/// options verbatim, and the closed-form answer contract. The option keys
/// are the criteria's OWN keys in presentation order — the same order
/// [`extract_answer_key`] matches against (one presentation law, request
/// and parse).
fn question_prompt(q: &SuiteQuestion, state: &Value) -> Result<String, String> {
    let state_text = match state {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    let options_block = match q.kind {
        QKind::Choice => {
            let obj = q
                .criteria
                .as_object()
                .ok_or_else(|| format!("qid {}: choice criteria must be an object", q.qid))?;
            let mut lines = Vec::with_capacity(obj.len());
            for (k, v) in obj {
                match v.as_str() {
                    Some(desc) if !desc.is_empty() => lines.push(format!("- {k}: {desc}")),
                    _ => lines.push(format!("- {k}")),
                }
            }
            lines.join("\n")
        }
        QKind::Score => {
            let arr = q
                .criteria
                .as_array()
                .ok_or_else(|| format!("qid {}: score criteria must be an array", q.qid))?;
            let mut lines = Vec::with_capacity(arr.len());
            for (i, v) in arr.iter().enumerate() {
                match v.as_str() {
                    Some(desc) if !desc.is_empty() => lines.push(format!("- {i}: {desc}")),
                    _ => lines.push(format!("- {i}")),
                }
            }
            lines.join("\n")
        }
        QKind::Noul => {
            // noul = p(true); criteria optional (the pplx wire's law —
            // the per-side prose keys when present).
            match q.criteria.as_object() {
                Some(obj) if !obj.is_empty() => {
                    let yes = obj
                        .get("true")
                        .and_then(Value::as_str)
                        .unwrap_or("Yes / true");
                    let no = obj
                        .get("false")
                        .and_then(Value::as_str)
                        .unwrap_or("No / false");
                    format!("- true: {yes}\n- false: {no}")
                }
                _ => "- true\n- false".to_string(),
            }
        }
    };
    let kind_word = match q.kind {
        QKind::Choice => "choice",
        QKind::Score => "score level",
        QKind::Noul => "verdict",
    };
    Ok(format!(
        "{state}\n\nQuestion: {instructions}\n\nOptions:\n{options}\n\nAnswer with \
         ONLY the {kind_word} key from the options list, verbatim, and nothing \
         else. Do not explain.",
        state = state_text,
        instructions = q.instructions,
        options = options_block,
        kind_word = kind_word,
    ))
}

/// The request body for one question: single user turn, deterministic
/// temperature, generous budget (reasoning spends before the answer
/// spells — the probe's finish:"length" lesson).
fn request_body(model: &str, prompt: &str, max_tokens: u32) -> Value {
    json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.0_f64,
        "max_tokens": max_tokens,
        "stream": false,
    })
}

/// Extract the answer KEY from `content` — a strict labeled extractor over
/// the presented options. The content is expected to be the verbatim key
/// (possibly wrapped in quotes/whitespace or one short sentence carrying
/// it); the FIRST criteria key found as a whole-word/whole-phrase match in
/// content order wins; ambiguity (two or more keys matching) is a loud
/// refusal — a prose answer naming two options is not an answer. Score
/// levels match on the level INDEX as a standalone number (so a level
/// description quoting another level's number cannot miskey — the digit
/// must stand alone). Noul matches `true`/`false`/`yes`/`no` on word
/// boundaries; polarity negations inside one short answer ("not true")
/// resolve to the negated side, anything longer is ambiguous.
pub(crate) fn extract_answer_key(q: &SuiteQuestion, content: &str) -> Result<String, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "qid {}: empty content — the generation budget was likely consumed \
             by reasoning (finish \"length\"; raise SENSENOVA_MAX_TOKENS)",
            q.qid
        ));
    }
    let hay = trimmed.to_ascii_lowercase();
    match q.kind {
        QKind::Noul => {
            // Negation-aware word scan: the FIRST polarity word wins its
            // own clause; a "not"/"n't" immediately before it flips it.
            let words: Vec<&str> = hay
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|w| !w.is_empty())
                .collect();
            let mut flipped = false;
            for w in &words {
                let polarity = match *w {
                    "true" | "yes" => Some(true),
                    "false" | "no" => Some(false),
                    _ => None,
                };
                if let Some(p) = polarity {
                    let v = if flipped { !p } else { p };
                    return Ok(if v { "true" } else { "false" }.to_string());
                }
                flipped = matches!(*w, "not" | "nt" | "never" | "neither");
            }
            Err(format!(
                "qid {}: noul content names no verdict side: {trimmed:?} — the \
                 extraction grammar is verbatim-key, never a guess",
                q.qid
            ))
        }
        QKind::Score => {
            let levels: Vec<String> = (0..q.criteria.as_array().map_or(0, Vec::len))
                .map(|i| i.to_string())
                .collect();
            // Standalone-number scan in content order: the first level
            // index that appears as its own token wins; a bare number that
            // is NOT a level index is ignored only if some level index
            // matches later — otherwise loud (a numeric answer outside the
            // level set is wire/model drift).
            let mut seen_out_of_range = false;
            for tok in hay.split(|c: char| !c.is_ascii_digit()) {
                if tok.is_empty() {
                    continue;
                }
                if levels.iter().any(|l| l == tok) {
                    return Ok(tok.to_string());
                }
                seen_out_of_range = true;
            }
            if seen_out_of_range {
                return Err(format!(
                    "qid {}: score content names a number outside the level set: \
                     {trimmed:?}",
                    q.qid
                ));
            }
            // Fall through to the text-key scan (a level description may
            // carry the label text rather than the number).
            text_key_scan(&levels, q, trimmed, &hay)
        }
        QKind::Choice => {
            let keys: Vec<String> = q
                .criteria
                .as_object()
                .map(|m| m.keys().cloned().collect())
                .ok_or_else(|| format!("qid {}: choice criteria must be an object", q.qid))?;
            text_key_scan(&keys, q, trimmed, &hay)
        }
    }
}

/// The shared whole-key scan for choice/score-text keys: a key matches
/// when it appears in content bounded by non-key characters (start/end or
/// a char outside the key's own alphabet on each side) — so key "art" does
/// not match inside "party", and key "no" does not match every "not".
/// Exactly one distinct matching key = the answer; zero = loud; two or
/// more = loud (a prose answer naming two options is not an answer).
fn text_key_scan(
    keys: &[String],
    q: &SuiteQuestion,
    trimmed: &str,
    hay: &str,
) -> Result<String, String> {
    let mut hits: Vec<&String> = Vec::new();
    for k in keys {
        let lk = k.to_ascii_lowercase();
        if lk.is_empty() {
            continue;
        }
        let mut from = 0usize;
        let mut matched = false;
        while let Some(pos) = hay[from..].find(&lk) {
            let abs = from + pos;
            let before_ok = abs == 0
                || !hay[..abs].ends_with(|c: char| lk.starts_with(c) || c.is_ascii_alphanumeric());
            let after = abs + lk.len();
            let after_ok = after == hay.len()
                || !hay[after..]
                    .starts_with(|c: char| lk.ends_with(c) || c.is_ascii_alphanumeric());
            if before_ok && after_ok {
                matched = true;
                break;
            }
            from = abs + 1;
        }
        if matched {
            hits.push(k);
        }
    }
    hits.dedup();
    match hits.as_slice() {
        [one] => Ok((*one).clone()),
        [] => Err(format!(
            "qid {}: content names no presented option key: {trimmed:?} — the \
             extraction grammar is verbatim-key, never a guess",
            q.qid
        )),
        many => Err(format!(
            "qid {}: content is ambiguous across {} option keys ({}) — a prose \
             answer naming two options is not an answer: {trimmed:?}",
            q.qid,
            many.len(),
            many.iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// The lane: a thin plaintext-HTTP/1.1 client to the operator's
/// llama-server (the OpenSenseNova fork). std-only (the shared
/// `crate::lanes::http_mini` micro-client; no new deps — the G-ISO-4
/// import law: `crate::harness::suites` + `http_mini` + std + serde_json).
#[derive(Debug, Clone)]
pub struct SensenovaLane {
    host: String,
    port: u16,
    model: String,
    timeout: Duration,
    max_tokens: u32,
}

impl SensenovaLane {
    /// Build from the env contract. REFUSES (loud, naming the missing
    /// envs) unless `SENSENOVA_MODEL` + `SENSENOVA_DEVICE` are set AND the
    /// `SENSENOVA_AUTH_OK=1` acknowledgment is present — the no-creds gate
    /// (the mammoth FORWARDER_AUTH shape): this lane is a SELF-HOSTED
    /// posture; an endpoint nobody attested is refused, never half-run.
    pub fn from_env() -> Result<Self, String> {
        let missing: Vec<&str> = [
            (
                MODEL_ENV,
                std::env::var(MODEL_ENV).ok().filter(|m| !m.is_empty()),
            ),
            (
                DEVICE_ENV,
                std::env::var(DEVICE_ENV).ok().filter(|m| !m.is_empty()),
            ),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.is_none().then_some(k))
        .collect();
        let auth_ok = std::env::var(AUTH_OK_ENV).is_ok_and(|v| v == "1");
        if !missing.is_empty() || !auth_ok {
            return Err(format!(
                "refusing: {AUTH_OK_ENV}=1 (the self-hosted-endpoint \
                 acknowledgment) and {MODEL_ENV} + {DEVICE_ENV} (the quant \
                 posture) are REQUIRED — missing: {missing:?}; this lane is a \
                 SELF-HOSTED posture (the OpenSenseNova fork on loopback), \
                 never a hosted one"
            ));
        }
        let url = std::env::var(SERVE_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVE_URL.into());
        let rest = url
            .strip_prefix("http://")
            .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(&url));
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().unwrap_or(80)),
            None => (rest.to_string(), 80),
        };
        Ok(Self {
            host,
            port,
            model: std::env::var(MODEL_ENV).unwrap_or_default(),
            timeout: Duration::from_millis(
                std::env::var(TIMEOUT_ENV)
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_TIMEOUT_MS),
            ),
            max_tokens: std::env::var(MAX_TOKENS_ENV)
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(DEFAULT_MAX_TOKENS),
        })
    }

    /// The request `model` field (the env-set id — provenance, never a
    /// hardcoded name).
    pub fn model(&self) -> &str {
        &self.model
    }

    /// The posture disclosure stamp ([`DEVICE_ENV`], required at construct).
    pub fn device(&self) -> String {
        std::env::var(DEVICE_ENV).unwrap_or_else(|_| "undisclosed".to_string())
    }

    /// Build directly (the test seam — the env contract is pinned by the
    /// constants + the creds-gate test; the wire tests never touch process
    /// env).
    #[cfg(test)]
    pub(crate) fn for_test(addr: std::net::SocketAddr, model: &str, max_tokens: u32) -> Self {
        Self {
            host: addr.ip().to_string(),
            port: addr.port(),
            model: model.to_string(),
            timeout: Duration::from_secs(10),
            max_tokens,
        }
    }

    /// The handshake (the clef law): ONE minimal noul probe. A 200 whose
    /// `choices[0]` parses and whose answer key resolves proves the wire
    /// AND the model; the echoed `model` field is the provenance (it may
    /// differ from the request id when launched `-hf` — record what the
    /// server says). Returns `(model_echo, device)`.
    pub fn info(&self) -> Result<(String, String), String> {
        let probe = SuiteCase {
            id: "handshake".into(),
            state: Value::String(
                "handshake: the lane's minimal noul probe (the clef A2 law; \
                 discarded, never a measured case)"
                    .into(),
            ),
            questions: vec![crate::harness::suites::SuiteQuestion {
                qid: "probe".into(),
                kind: QKind::Noul,
                instructions: "Is this the handshake probe? Answer true or false.".into(),
                criteria: Value::Null,
            }],
            gold: vec![],
        };
        let _ = self.decide(&probe)?;
        Ok((self.model.clone(), self.device()))
    }

    /// One decision round: build the prompt (the presented keys verbatim),
    /// POST `/v1/chat/completions`, extract the answer key from `content`
    /// (the strict labeled extractor — `reasoning_content` is NEVER the
    /// answer channel), map it into the presented-order probability vector
    /// (one-hot; the teacher probabilities this cell records are the
    /// ARGMAX'S, the runner's keyed tail consumes them positionally).
    /// Returns `(answer triples, client_ms, completion_tokens)`.
    pub fn decide(
        &self,
        case: &SuiteCase,
    ) -> Result<(Vec<crate::lanes::pplx::AnswerTriple>, f64, u64), String> {
        let mut out = Vec::with_capacity(case.questions.len());
        let t0 = Instant::now();
        for q in &case.questions {
            let prompt = question_prompt(q, &case.state)?;
            let body = request_body(&self.model, &prompt, self.max_tokens).to_string();
            let (status, _headers, raw) = self
                .post(&body)
                .map_err(|e| format!("sensenova round trip ({}): {e}", case.id))?;
            if status != 200 {
                return Err(format!(
                    "sensenova round trip ({}) -> HTTP {status}: {}",
                    case.id,
                    String::from_utf8_lossy(&raw)
                ));
            }
            let parsed: Value = serde_json::from_slice(&raw)
                .map_err(|e| format!("sensenova response ({}): {e}", case.id))?;
            let key = parse_choice_message(q, &parsed)?;
            out.push(one_hot_triple(q, &key)?);
        }
        let client_ms = t0.elapsed().as_secs_f64() * 1000.0;
        Ok((out, client_ms, 0))
    }

    /// The response body for one case, VERBATIM — the determinism check's
    /// in-memory compare input. Nothing is persisted (the MEASUREMENT-ONLY
    /// law).
    pub fn decide_raw(&self, case: &SuiteCase) -> Result<String, String> {
        let Some(q) = case.questions.first() else {
            return Err(format!("case {}: no questions", case.id));
        };
        let prompt = question_prompt(q, &case.state)?;
        let body = request_body(&self.model, &prompt, self.max_tokens).to_string();
        let (status, _headers, raw) = self.post(&body)?;
        if status != 200 {
            return Err(format!(
                "sensenova round trip ({}) -> HTTP {status}: {}",
                case.id,
                String::from_utf8_lossy(&raw)
            ));
        }
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    /// The determinism-compare projection of one raw body: the decision
    /// content (`model` + `choices`' message + `usage`) with the serve-meta
    /// blocks REMOVED (`id`, `created`, `system_fingerprint`, and any
    /// timing/latency fields a fork stamps — llama-server varies them per
    /// call BY DESIGN; the mammoth lane's qev lesson, the same law one wire
    /// over). The projection stays in memory — nothing persisted.
    pub fn determinism_view(raw: &str) -> Result<String, String> {
        let parsed: Value =
            serde_json::from_str(raw).map_err(|e| format!("determinism view parse: {e}"))?;
        let mut obj = match parsed {
            Value::Object(m) => m,
            _ => return Err("determinism view: body is not a JSON object".into()),
        };
        for k in ["id", "created", "system_fingerprint"] {
            obj.shift_remove(k);
        }
        Ok(Value::Object(obj).to_string())
    }

    /// The spend ceiling check (the clef A6 law): `n` cases per suite must
    /// fit `SENSENOVA_SMOKE_MAX_CASES` unless `SENSENOVA_ALLOW_UNCAPPED=1`.
    pub fn check_capped(n: usize) -> Result<(), String> {
        let cap = std::env::var(MAX_CASES_ENV)
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_CASES);
        let uncapped = std::env::var(UNCAPPED_ENV).is_ok_and(|v| v == "1");
        if n > cap && !uncapped {
            return Err(format!(
                "refusing: {n} cases > {MAX_CASES_ENV}={cap} — a slow self-host is \
                 metered like a hosted endpoint (the clef A6 law); the uncapped \
                 posture is the explicit {UNCAPPED_ENV}=1 acknowledgment"
            ));
        }
        Ok(())
    }

    fn post(&self, body: &str) -> Result<http_mini::HttpReply, String> {
        // One bounded retry on the timeout class (the i068 completion's
        // diagnosis: a handshake read can stall past SENSENOVA_TIMEOUT_MS
        // when the server serializes a prior request — typed's 2000 fine
        // decisions vs two dead handshakes). A retry opens a FRESH
        // connection; llama-server cancels the abandoned task on
        // disconnect (the read2 log's "cancel task" lines), so the retry
        // is clean — never a double generation. Any second timeout is a
        // LOUD named error carrying the knob.
        let mut last_timeout: Option<u64> = None;
        for attempt in 0..2 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_secs(2));
            }
            match http_mini::request(
                &self.host,
                self.port,
                "POST",
                "/v1/chat/completions",
                Some(body.as_bytes()),
                self.timeout,
                &[],
            ) {
                Ok(reply) => return Ok(reply),
                Err(http_mini::LaneHttpError::ReadTimeout { timeout_secs }) => {
                    last_timeout = Some(timeout_secs);
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        Err(format!(
            "sensenova post: read timed out after {}s on BOTH attempts — the \
             server stalled past the socket timeout twice; raise \
             SENSENOVA_TIMEOUT_MS or check the serve",
            last_timeout.unwrap_or_default()
        ))
    }
}

/// Parse one chat-completions reply into the answer KEY: `choices[0]` must
/// exist; `finish_reason: "length"` is a LOUD error naming the knob (the
/// reasoning consumed the budget — never a silent fold); `content` is the
/// only answer channel (`reasoning_content` is recorded in the error text
/// when content is unusable, so the operator sees where the tokens went).
fn parse_choice_message(q: &SuiteQuestion, parsed: &Value) -> Result<String, String> {
    let choice = parsed
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .ok_or_else(|| format!("qid {}: reply carries no choices[0]", q.qid))?;
    let finish = choice.get("finish_reason").and_then(Value::as_str);
    let message = choice
        .get("message")
        .ok_or_else(|| format!("qid {}: choices[0] carries no message", q.qid))?;
    let content = message.get("content").and_then(Value::as_str).unwrap_or("");
    let reasoning_present = message
        .get("reasoning_content")
        .and_then(Value::as_str)
        .is_some_and(|r| !r.trim().is_empty());
    match extract_answer_key(q, content) {
        Ok(k) => Ok(k),
        Err(e) => {
            let note = match (finish, reasoning_present) {
                (Some("length"), _) => {
                    "finish \"length\" — raise SENSENOVA_MAX_TOKENS (the reasoning \
                     consumed the budget)"
                }
                (_, true) => {
                    "reasoning_content present but content names no key — the model \
                     answered in its reasoning channel; the answer channel is \
                     `content`, never `reasoning_content` (the probe's wire law)"
                }
                _ => "the model answered off-contract",
            };
            Err(format!("{} ({note})", e))
        }
    }
}

/// The one-hot probability vector over the question's presented keys, the
/// extracted key at its presentation position (the runner's keyed tail
/// consumes them positionally — the openthai_keyed_probs law: the keys are
/// the criteria's own order).
fn one_hot_triple(
    q: &SuiteQuestion,
    key: &str,
) -> Result<crate::lanes::pplx::AnswerTriple, String> {
    let keys: Vec<String> = match q.kind {
        QKind::Noul => vec!["false".to_string(), "true".to_string()],
        QKind::Score => (0..q.criteria.as_array().map_or(0, Vec::len))
            .map(|i| i.to_string())
            .collect(),
        QKind::Choice => q
            .criteria
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .ok_or_else(|| format!("qid {}: choice criteria must be an object", q.qid))?,
    };
    let idx = keys.iter().position(|k| k == key).ok_or_else(|| {
        format!(
            "qid {}: extracted key {key:?} is not a presented option",
            q.qid
        )
    })?;
    let mut probs = vec![0.0_f64; keys.len()];
    probs[idx] = 1.0;
    Ok((probs, idx, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{BufRead, BufReader, Read, Write};

    fn choice_q() -> SuiteQuestion {
        SuiteQuestion {
            qid: "q0".into(),
            kind: QKind::Choice,
            instructions: "Pick the sentiment.".into(),
            criteria: json!({"negative": "neg", "neutral": "mid", "positive": "pos"}),
        }
    }

    fn score_q() -> SuiteQuestion {
        SuiteQuestion {
            qid: "q0".into(),
            kind: QKind::Score,
            instructions: "Rate it.".into(),
            criteria: json!([
                "very negative",
                "negative",
                "neutral",
                "positive",
                "very positive"
            ]),
        }
    }

    fn noul_q() -> SuiteQuestion {
        SuiteQuestion {
            qid: "q0".into(),
            kind: QKind::Noul,
            instructions: "Is it true?".into(),
            criteria: Value::Null,
        }
    }

    #[test]
    fn extract_reads_the_verbatim_key_one_hot_in_presentation_order() {
        let q = choice_q();
        let key = extract_answer_key(&q, "positive").unwrap();
        let (probs, pick, conf) = one_hot_triple(&q, &key).unwrap();
        // The criteria's INSERTION order is the presentation order
        // (serde_json preserve_order): negative, neutral, positive.
        assert_eq!(probs, vec![0.0, 0.0, 1.0]);
        assert_eq!(pick, 2);
        assert_eq!(conf, 1.0);
    }

    #[test]
    fn extract_is_word_bounded_not_substring() {
        let q = choice_q();
        // "art" is not a key here, but the bound matters for keys that
        // are substrings of prose: "no" inside "not" must not match a
        // hypothetical "no" key.
        let q2 = SuiteQuestion {
            criteria: json!({"no": "", "notes": ""}),
            ..choice_q()
        };
        // A key inside a longer word does NOT match (word-boundary law):
        // "no" never matches inside "not actually".
        assert!(extract_answer_key(&q2, "not actually").is_err());
        // A non-key string matches nothing, loud.
        assert!(extract_answer_key(&q, "the party is over").is_err());
        // A real key inside a longer word ("positively") does NOT match.
        assert!(extract_answer_key(&q, "positively great").is_err());
    }

    #[test]
    fn extract_refuses_ambiguous_and_missing_loud() {
        let q = choice_q();
        let both = extract_answer_key(&q, "positive or negative, hard to say");
        assert!(both.is_err());
        let none = extract_answer_key(&q, "it is fine I guess");
        assert!(none.is_err());
    }

    #[test]
    fn extract_score_reads_standalone_level_numbers() {
        let q = score_q();
        assert_eq!(extract_answer_key(&q, "3").unwrap(), "3");
        assert_eq!(extract_answer_key(&q, "level 4").unwrap(), "4");
        // A level description naming its own number is fine.
        assert_eq!(extract_answer_key(&q, "I choose 1").unwrap(), "1");
        // A number outside the level set is loud, never folded.
        assert!(extract_answer_key(&q, "9").is_err());
        assert!(extract_answer_key(&q, "no number at all").is_err());
    }

    #[test]
    fn extract_noul_is_negation_aware() {
        let q = noul_q();
        assert_eq!(extract_answer_key(&q, "true").unwrap(), "true");
        assert_eq!(extract_answer_key(&q, "yes").unwrap(), "true");
        assert_eq!(extract_answer_key(&q, "false").unwrap(), "false");
        assert_eq!(extract_answer_key(&q, "no").unwrap(), "false");
        assert_eq!(extract_answer_key(&q, "not true").unwrap(), "false");
        assert_eq!(extract_answer_key(&q, "that is not false").unwrap(), "true");
        assert_eq!(extract_answer_key(&q, "True.").unwrap(), "true");
        assert!(extract_answer_key(&q, "maybe").is_err());
    }

    #[test]
    fn extract_empty_content_names_the_budget_knob() {
        let q = choice_q();
        let err = extract_answer_key(&q, "").unwrap_err();
        assert!(err.contains("SENSENOVA_MAX_TOKENS"));
    }

    #[test]
    fn prompt_carries_the_state_options_and_the_verbatim_contract() {
        let q = choice_q();
        let p = question_prompt(&q, &Value::String("The movie was great.".into())).unwrap();
        assert!(p.contains("The movie was great."));
        assert!(p.contains("- negative: neg"));
        assert!(p.contains("- positive: pos"));
        assert!(p.contains("nothing else"));
        // Score renders numbered levels; noul renders the true/false pair.
        let sp = question_prompt(&score_q(), &Value::Null).unwrap();
        assert!(sp.contains("- 0: very negative"));
        assert!(sp.contains("- 4: very positive"));
        let np = question_prompt(&noul_q(), &Value::Null).unwrap();
        assert!(np.contains("- true"));
        assert!(np.contains("- false"));
    }

    #[test]
    fn parse_refuses_length_finish_loud_and_names_the_knob() {
        let q = choice_q();
        let body = json!({
            "choices": [{"message": {"content": "", "reasoning_content": "long thought"},
                         "finish_reason": "length"}],
        });
        let err = parse_choice_message(&q, &body).unwrap_err();
        assert!(err.contains("finish \"length\""));
        assert!(err.contains("SENSENOVA_MAX_TOKENS"));
    }

    #[test]
    fn parse_treats_content_as_the_only_answer_channel() {
        let q = choice_q();
        // The reasoning channel carries the answer; content does not — the
        // lane refuses (the probe's wire law: content is the answer
        // channel, reasoning_content never).
        let body = json!({
            "choices": [{"message": {"content": "I think it is positive",
                                     "reasoning_content": "clearly positive"},
                         "finish_reason": "stop"}],
        });
        // "positive" appears in content → the answer resolves from CONTENT.
        assert_eq!(parse_choice_message(&q, &body).unwrap(), "positive");
        let empty_content = json!({
            "choices": [{"message": {"content": "",
                                     "reasoning_content": "clearly positive"},
                         "finish_reason": "stop"}],
        });
        let err = parse_choice_message(&q, &empty_content).unwrap_err();
        assert!(err.contains("reasoning channel"));
    }

    #[test]
    fn determinism_view_strips_the_serve_meta_blocks() {
        let raw = json!({
            "id": "cmpl-x", "created": 1730000000u64, "model": "m",
            "system_fingerprint": "fp1",
            "choices": [{"message": {"content": "ok"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 5, "completion_tokens": 1},
        })
        .to_string();
        let view = SensenovaLane::determinism_view(&raw).unwrap();
        assert!(!view.contains("cmpl-x"));
        assert!(!view.contains("1730000000"));
        assert!(!view.contains("fp1"));
        assert!(view.contains("\"content\":\"ok\""));
        // Idempotent: the projection is already clean.
        assert_eq!(SensenovaLane::determinism_view(&view).unwrap(), view);
    }

    #[test]
    fn spend_ceiling_refuses_over_cap_unless_uncapped() {
        // The pure form: the cap math is the env contract's, tested via
        // the constants (the env itself is process state — the wire tests
        // never touch it).
        assert_eq!(DEFAULT_MAX_CASES, 50);
        assert_eq!(DEFAULT_MAX_TOKENS, 2048);
    }

    #[test]
    fn one_hot_maps_noul_over_the_fixed_pair() {
        let q = noul_q();
        let (probs, pick, _conf) = one_hot_triple(&q, "true").unwrap();
        // The gold idx space: 0 = false, 1 = true (the suites' own law).
        assert_eq!(probs, vec![0.0, 1.0]);
        assert_eq!(pick, 1);
    }

    /// A one-shot stub server (the startlux/d3 stub, verbatim — one shape,
    /// many lanes): canned reply per request, request capture on a channel.
    fn serve(canned: String) -> std::net::SocketAddr {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            loop {
                let (stream, _) = match listener.accept() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let mut writer = match stream.try_clone() {
                    Ok(w) => w,
                    Err(_) => continue,
                };
                let mut reader = BufReader::new(stream);
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }
                let mut content_length = 0usize;
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            let trimmed = line.trim_end();
                            if trimmed.is_empty() {
                                break;
                            }
                            if let Some((k, v)) = trimmed.split_once(':')
                                && k.eq_ignore_ascii_case("content-length")
                            {
                                content_length = v.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                }
                let mut body = vec![0u8; content_length];
                let _ = reader.read_exact(&mut body);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    canned.len(),
                    canned
                );
                let _ = writer.write_all(resp.as_bytes());
                let _ = writer.flush();
            }
        });
        addr
    }

    fn chat_reply(content: &str, finish: &str) -> String {
        json!({
            "id": "cmpl-t", "created": 1u64, "model": "the-load-path.gguf",
            "choices": [{"message": {"role": "assistant", "content": content},
                         "finish_reason": finish}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 1},
        })
        .to_string()
    }

    #[test]
    fn decide_round_trips_the_wire_and_maps_one_hot() {
        let addr = serve(chat_reply("positive", "stop"));
        let lane = SensenovaLane::for_test(addr, "the-load-path.gguf", 64);
        let case = SuiteCase {
            id: "c0".into(),
            state: Value::String("The movie was great.".into()),
            questions: vec![choice_q()],
            gold: vec![],
        };
        let (answers, _ms, _tokens) = lane.decide(&case).unwrap();
        assert_eq!(answers.len(), 1);
        assert_eq!(answers[0].0, vec![0.0, 0.0, 1.0]);
        assert_eq!(answers[0].1, 2);
    }

    #[test]
    fn decide_refuses_the_length_finish_loud() {
        let addr = serve(chat_reply("", "length"));
        let lane = SensenovaLane::for_test(addr, "m", 8);
        let case = SuiteCase {
            id: "c0".into(),
            state: Value::Null,
            questions: vec![choice_q()],
            gold: vec![],
        };
        let err = lane.decide(&case).unwrap_err();
        assert!(err.contains("SENSENOVA_MAX_TOKENS"));
    }

    #[test]
    fn handshake_walks_the_real_decide_path() {
        // The handshake IS a decide round (the clef law's OpenAI form): a
        // live server answering the noul probe proves the wire.
        let addr = serve(chat_reply("true", "stop"));
        let lane = SensenovaLane::for_test(addr, "the-load-path.gguf", 64);
        let (model_echo, _device) = lane.info().unwrap();
        assert_eq!(model_echo, "the-load-path.gguf");
    }

    /// The i068 completion's defect class, fixed: a first request that
    /// STALLS past the socket timeout (the read2/read3 handshake EAGAINs)
    /// is retried ONCE on a fresh connection — the second request
    /// answers and the lane completes. The stall stub holds the first
    /// connection without replying (the client's 1s timeout fires), the
    /// second gets the canned reply.
    #[test]
    fn post_retries_the_timeout_class_once_and_completes() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let canned = chat_reply("positive", "stop");
        std::thread::spawn(move || {
            let mut first = true;
            loop {
                let (stream, _) = match listener.accept() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let mut writer = match stream.try_clone() {
                    Ok(w) => w,
                    Err(_) => continue,
                };
                let mut reader = BufReader::new(stream);
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }
                let mut content_length = 0usize;
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            let trimmed = line.trim_end();
                            if trimmed.is_empty() {
                                break;
                            }
                            if let Some((k, v)) = trimmed.split_once(':')
                                && k.eq_ignore_ascii_case("content-length")
                            {
                                content_length = v.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                }
                let mut body = vec![0u8; content_length];
                let _ = reader.read_exact(&mut body);
                if first {
                    // The stall: hold the connection well past the
                    // client's timeout, never reply — ON A DETACHED
                    // THREAD, so the accept loop stays free for the
                    // retry's connection (a blocking hold here would
                    // strand the retry in the listen backlog and defeat
                    // the test's own purpose).
                    first = false;
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_secs(5));
                        drop(writer);
                    });
                    continue;
                }
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    canned.len(),
                    canned
                );
                let _ = writer.write_all(resp.as_bytes());
                let _ = writer.flush();
            }
        });
        let lane = SensenovaLane {
            host: addr.ip().to_string(),
            port: addr.port(),
            model: "m".to_string(),
            timeout: Duration::from_millis(1000),
            max_tokens: 64,
        };
        let case = SuiteCase {
            id: "c0".into(),
            state: Value::String("The movie was great.".into()),
            questions: vec![choice_q()],
            gold: vec![],
        };
        let (answers, _ms, _tokens) = lane.decide(&case).expect("the retry must complete");
        assert_eq!(answers[0].0, vec![0.0, 0.0, 1.0]);
    }

    /// Both attempts stalling is a LOUD named error carrying the knob
    /// (never a cryptic socket error — the i068 class named at the lane
    /// surface).
    #[test]
    fn double_stall_names_sensenova_timeout_ms_loud() {
        // A stub that stalls EVERY connection (accept on its own thread
        // — the accept loop must stay free, the honest silent-server
        // shape).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            loop {
                let Ok((stream, _)) = listener.accept() else {
                    continue;
                };
                std::thread::spawn(move || {
                    // Hold without replying; dropped at the end.
                    std::thread::sleep(Duration::from_secs(30));
                    drop(stream);
                });
            }
        });
        let lane = SensenovaLane {
            host: addr.ip().to_string(),
            port: addr.port(),
            model: "m".to_string(),
            timeout: Duration::from_millis(300),
            max_tokens: 64,
        };
        let case = SuiteCase {
            id: "c0".into(),
            state: Value::Null,
            questions: vec![choice_q()],
            gold: vec![],
        };
        let err = lane.decide(&case).unwrap_err();
        assert!(err.contains("SENSENOVA_TIMEOUT_MS"), "{err}");
        assert!(err.contains("BOTH attempts"), "{err}");
    }
}
