//! The mid-run half of the box-state law (sprint!136).
//!
//! `box_state` stamps the box ONCE at the start AND once at the end of a run
//! (Issue 021 T7); `scripts/bench_preflight.sh` certifies the box at LAUNCH
//! only. The sprint!020 bars round-3 cell measured the gap between those two
//! posts: the gate passed at launch (load 3.01), a sibling session started
//! mid-window, and the cell finished with load 6.86 — above the 6.0 ceiling —
//! plus sixteen poison windows, while the record's own PROVENANCE line kept
//! asserting the launch state. A latency figure whose box state was only ever
//! sampled at the edges is not provenance; it is an alibi.
//!
//! This module is the sampler that runs THROUGH the window: a background
//! thread re-reading `box_state::capture().load_1m` on a cadence and the
//! aggregate the run prints into its own log body beside the tables:
//!
//! ```text
//! - mid-run load watch (sprint!136): samples=17 unread=0 window=512.3s cadence=30s min=2.10 load_mid_run_max=6.86 mean=3.44 max@381s — ⛔ load_mid_run_max=6.86 exceeds 6 — the window did NOT hold; out-of-spec beside the launch gate (sprint!020 round-3 precedent)
//! ```
//!
//! `load_mid_run_max=` is the grep token: a log-grep for a poison-free cell
//! now has to survive this line, not just the launch PROVENANCE (the
//! sprint!020 detection lesson — the poison verdict rode the end MAIL, never
//! the cell's own log body; that blind class is closed here).
//!
//! Env laws (the env-switch law):
//! - `REFLEX_LOAD_WATCH=0` — the exact-literal DEMOTE. The watcher never
//!   spawns and the run's output is byte-identical to the incumbent. Any
//!   other value (unset included) is the armed, default-ON posture: a record
//!   that forgot to think about mid-run load still carries the mid-run half.
//! - `REFLEX_LOAD_WATCH_CADENCE_SECS=<f64>` — the cadence override (default
//!   30s, the design's suggestion). A junk value is never a silent pass: the
//!   default applies with one loud stderr line naming the junk.
//! - `REFLEX_LOAD_WATCH_STRICT=1|true|on` — PROMOTE-ONLY strict mode: a
//!   ceiling breach turns the run's exit into 2 (the out-of-spec label the
//!   sprint!020 verdict lane applied post-hoc becomes automatic). Default is
//!   flag-only — the line prints, the exit code is untouched.
//!
//! Samples land ON the cadence grid (t = cadence, 2·cadence, …), never at
//! t = 0 — a sample at spawn would only restate the launch gate's reading.
//! A window shorter than one cadence therefore honestly reports zero
//! samples and the line says so; no number is invented for it.
//!
//! Cost disclosure: every sample is one `box_state::capture()` — on Windows
//! that is one PowerShell spawn (~1s of one core; the CIM queries there
//! price at ~a second each), on macOS a handful of sysctl/pmset reads. At
//! the 30s default the sampler's own duty cycle is a few percent of one
//! core and its blip moves the load figure it reads by well under the
//! ceiling's margin — but a microsecond-tail-sensitive cell should take the
//! demote literal and keep the watch off, knowingly.
//!
//! Scope law: the watcher spans the WHOLE harness process (spawned in the
//! bin's `main`, joined after the worker), so every exclusive early-exit
//! probe mode gets the line through ONE seam — never per-runner wiring
//! across the ten `box_start`/`box_end` meta sites. A `die()` refusal kills
//! the process from the worker thread and the line is skipped — those paths
//! are refusals, never quotable cells, and their logs carry the refusal
//! reason.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::box_state::{MAX_LOAD, capture};

/// The design's suggested cadence. `REFLEX_LOAD_WATCH_CADENCE_SECS` overrides.
pub const DEFAULT_CADENCE_SECS: f64 = 30.0;

/// One mid-run load reading: seconds since the watcher spawned, and the SAME
/// normalized load quantity `box_state` stamps (`load_1m`; on Windows the
/// loadavg-equivalent — see that module's docs before quoting).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LoadSample {
    pub at_secs: f64,
    pub load_1m: Option<f64>,
}

/// Watcher config; only the cadence is tunable today.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadWatchConfig {
    pub cadence: Duration,
}

impl Default for LoadWatchConfig {
    fn default() -> Self {
        Self {
            cadence: Duration::from_secs_f64(DEFAULT_CADENCE_SECS),
        }
    }
}

/// `REFLEX_LOAD_WATCH`: only the exact literal "0" demotes (the env-switch
/// law — the demote literal restores the incumbent byte-identically; any
/// other spelling, unset included, stays the armed default-ON posture).
fn env_demoted(v: Option<&str>) -> bool {
    v == Some("0")
}

/// `REFLEX_LOAD_WATCH_STRICT`: PROMOTE-ONLY — arms on the exact `1`/`true`/
/// `on` spellings and nothing else.
fn env_strict(v: Option<&str>) -> bool {
    matches!(v, Some("1") | Some("true") | Some("on"))
}

/// `REFLEX_LOAD_WATCH_CADENCE_SECS`: a finite positive value is the cadence;
/// unset is the default; junk/non-positive falls back to the default WITH a
/// loud note (junk is never a silent pass).
fn cadence_from(v: Option<&str>) -> (Duration, Option<String>) {
    match v {
        None => (Duration::from_secs_f64(DEFAULT_CADENCE_SECS), None),
        Some(raw) => match raw.trim().parse::<f64>() {
            Ok(s) if s.is_finite() && s > 0.0 => (Duration::from_secs_f64(s), None),
            _ => (
                Duration::from_secs_f64(DEFAULT_CADENCE_SECS),
                Some(format!(
                    "load watch: REFLEX_LOAD_WATCH_CADENCE_SECS={raw:?} is not a \
                     positive number — using the default {DEFAULT_CADENCE_SECS}s"
                )),
            ),
        },
    }
}

/// A spawned mid-run sampler. `finish` consumes it: stops the thread, joins,
/// and returns the aggregate the run prints.
pub struct LoadWatch {
    stop: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<LoadSample>>>,
    thread: Option<std::thread::JoinHandle<()>>,
    started: Instant,
    cadence: Duration,
}

impl LoadWatch {
    /// The production watcher over the real box probe, or `None` under the
    /// demote literal. Junk cadence fallback prints one loud stderr line.
    pub fn from_env() -> Option<LoadWatch> {
        if env_demoted(std::env::var("REFLEX_LOAD_WATCH").ok().as_deref()) {
            return None;
        }
        let (cadence, note) = cadence_from(
            std::env::var("REFLEX_LOAD_WATCH_CADENCE_SECS")
                .ok()
                .as_deref(),
        );
        if let Some(n) = note {
            eprintln!("{n}");
        }
        Some(Self::spawn_with(
            || capture().load_1m,
            LoadWatchConfig { cadence },
        ))
    }

    /// Test/alternative-probe constructor: the sampler reads through
    /// `probe`, never the box.
    pub fn spawn_with(
        probe: impl Fn() -> Option<f64> + Send + 'static,
        cfg: LoadWatchConfig,
    ) -> LoadWatch {
        let stop = Arc::new(AtomicBool::new(false));
        let samples = Arc::new(Mutex::new(Vec::new()));
        let thread_stop = Arc::clone(&stop);
        let thread_samples = Arc::clone(&samples);
        let started = Instant::now();
        let cadence = cfg.cadence;
        let thread = std::thread::Builder::new()
            .name("load-watch".to_string())
            .spawn(move || {
                while !thread_stop.load(Ordering::SeqCst) {
                    // Sleep one cadence in short stop-checkable slices BEFORE
                    // sampling: samples land on the cadence grid, so a window
                    // shorter than one cadence honestly reports zero samples.
                    let slice = Duration::from_millis(10).min(cadence);
                    let next = Instant::now() + cadence;
                    while Instant::now() < next && !thread_stop.load(Ordering::SeqCst) {
                        let remain = next.saturating_duration_since(Instant::now());
                        std::thread::sleep(slice.min(remain));
                    }
                    if thread_stop.load(Ordering::SeqCst) {
                        break;
                    }
                    let at = started.elapsed().as_secs_f64();
                    let load = probe();
                    if let Ok(mut q) = thread_samples.lock() {
                        q.push(LoadSample {
                            at_secs: at,
                            load_1m: load,
                        });
                    }
                }
            })
            .expect("spawn the load-watch thread");
        LoadWatch {
            stop,
            samples,
            thread: Some(thread),
            started,
            cadence,
        }
    }

    /// Stop the sampler, join it, aggregate the window.
    pub fn finish(mut self) -> MidRunLoad {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        let taken = self
            .samples
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default();
        MidRunLoad::aggregate(taken, self.started.elapsed().as_secs_f64(), self.cadence)
    }
}

/// The mid-run aggregate. Every field is derived from the samples; nothing
/// is invented for an empty or unreadable window (the honest-absence law).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MidRunLoad {
    /// Samples taken (attempted — includes probes that came back unreadable).
    pub samples: usize,
    /// Probes that returned no load reading.
    pub unread: usize,
    pub min: Option<f64>,
    /// The grep token's value: the window's highest observed load.
    pub max: Option<f64>,
    pub mean: Option<f64>,
    /// When the max landed (seconds since spawn); first-occurrence tie rule
    /// (the determinism law's strict first-index form).
    pub max_at_secs: Option<f64>,
    /// Spawn→finish seconds — the window the samples describe.
    pub window_secs: f64,
    pub cadence_secs: f64,
}

impl MidRunLoad {
    /// Aggregate samples + window. Pure; the tests drive this directly.
    pub fn aggregate(samples: Vec<LoadSample>, window_secs: f64, cadence: Duration) -> MidRunLoad {
        let samples_n = samples.len();
        let unread = samples.iter().filter(|s| s.load_1m.is_none()).count();
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;
        let mut max_at: Option<f64> = None;
        let mut sum = 0.0f64;
        let mut read = 0usize;
        for s in &samples {
            let Some(l) = s.load_1m else { continue };
            read += 1;
            sum += l;
            min = Some(match min {
                Some(m) if m <= l => m,
                _ => l,
            });
            // Strict `>` keeps the FIRST occurrence of the max (tie rule).
            if max.is_none_or(|m| l > m) {
                max = Some(l);
                max_at = Some(s.at_secs);
            }
        }
        MidRunLoad {
            samples: samples_n,
            unread,
            min,
            max,
            mean: (read > 0).then(|| sum / read as f64),
            max_at_secs: max_at,
            window_secs,
            cadence_secs: cadence.as_secs_f64(),
        }
    }

    /// The breach: the max STRICTLY above the shared ceiling (`>` — the same
    /// comparison `box_state::judge` applies at the edges, so a launch-pass /
    /// mid-run-breach pair can never disagree about the boundary value).
    pub fn ceiling_breach(&self) -> Option<f64> {
        self.max.filter(|m| *m > MAX_LOAD)
    }

    /// Whether `REFLEX_LOAD_WATCH_STRICT` is armed (promote-only spellings).
    pub fn strict_armed() -> bool {
        env_strict(std::env::var("REFLEX_LOAD_WATCH_STRICT").ok().as_deref())
    }

    /// The run-log line. Greppable tokens: `load_mid_run_max=`, `exceeds`,
    /// `held` / `did NOT hold`. Empty and unreadable windows render honest
    /// absences, never invented numbers.
    pub fn render_line(&self) -> String {
        let n = |v: Option<f64>| v.map_or("?".to_string(), |v| format!("{v:.2}"));
        let head = format!(
            "- mid-run load watch (sprint!136): samples={} unread={} window={:.1}s cadence={:.0}s min={} load_mid_run_max={} mean={} max@{}",
            self.samples,
            self.unread,
            self.window_secs,
            self.cadence_secs,
            n(self.min),
            n(self.max),
            n(self.mean),
            self.max_at_secs
                .map_or("?".to_string(), |t| format!("{t:.0}s")),
        );
        if self.samples == 0 {
            return format!("{head} — window shorter than one cadence; no mid-run sample taken");
        }
        if self.unread == self.samples {
            return format!(
                "{head} — every probe unreadable; mid-run state UNJUDGED (never a green)"
            );
        }
        match self.ceiling_breach() {
            Some(m) => format!(
                "{head} — ⛔ load_mid_run_max={m:.2} exceeds {MAX_LOAD} — the window did NOT hold; out-of-spec beside the launch gate (sprint!020 round-3 precedent)"
            ),
            None => format!("{head} — window HELD under {MAX_LOAD}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    fn s(at: f64, load: Option<f64>) -> LoadSample {
        LoadSample {
            at_secs: at,
            load_1m: load,
        }
    }

    #[test]
    fn aggregates_min_max_mean_with_holes_and_first_max_tie() {
        let m = MidRunLoad::aggregate(
            vec![
                s(1.0, Some(2.0)),
                s(2.0, None),
                s(3.0, Some(4.0)),
                s(4.0, Some(4.0)),
            ],
            5.0,
            Duration::from_secs(30),
        );
        assert_eq!(m.samples, 4);
        assert_eq!(m.unread, 1);
        assert_eq!(m.min, Some(2.0));
        assert_eq!(m.max, Some(4.0));
        assert_eq!(m.mean, Some(10.0 / 3.0));
        // Tie on the max: the FIRST occurrence wins (strict > keeps it).
        assert_eq!(m.max_at_secs, Some(3.0));
    }

    #[test]
    fn empty_and_all_unread_windows_render_honest_absences() {
        let empty = MidRunLoad::aggregate(Vec::new(), 0.4, Duration::from_secs(30));
        let line = empty.render_line();
        assert!(line.contains("samples=0"), "{line}");
        assert!(line.contains("no mid-run sample taken"), "{line}");

        let unjudged = MidRunLoad::aggregate(
            vec![s(1.0, None), s(2.0, None)],
            2.5,
            Duration::from_secs(30),
        );
        let line = unjudged.render_line();
        assert!(line.contains("unread=2"), "{line}");
        assert!(line.contains("UNJUDGED"), "{line}");
        assert_eq!(unjudged.ceiling_breach(), None);
    }

    #[test]
    fn breach_line_carries_the_grep_tokens_and_held_line_names_the_ceiling() {
        let breach = MidRunLoad::aggregate(
            vec![s(1.0, Some(2.12)), s(61.0, Some(6.86))],
            90.0,
            Duration::from_secs(30),
        );
        let line = breach.render_line();
        assert!(line.contains("load_mid_run_max=6.86"), "{line}");
        assert!(line.contains("exceeds 6"), "{line}");
        assert!(line.contains("did NOT hold"), "{line}");
        assert!(line.contains("sprint!020"), "{line}");
        assert_eq!(breach.ceiling_breach(), Some(6.86));

        let held = MidRunLoad::aggregate(
            vec![s(1.0, Some(2.12)), s(61.0, Some(5.99))],
            90.0,
            Duration::from_secs(30),
        );
        assert!(
            held.render_line().contains("window HELD"),
            "{}",
            held.render_line()
        );
        assert_eq!(held.ceiling_breach(), None);
    }

    #[test]
    fn boundary_max_equals_the_ceiling_is_not_a_breach() {
        // `judge` refuses on load > MAX_LOAD; the mid-run half uses the same
        // strict comparison so the two posts can never disagree at 6.0.
        let m = MidRunLoad::aggregate(vec![s(1.0, Some(MAX_LOAD))], 2.0, Duration::from_secs(30));
        assert_eq!(m.ceiling_breach(), None);
        assert!(m.render_line().contains("window HELD"));
    }

    #[test]
    fn demote_literal_is_exact_zero_only() {
        // The law: only the exact literal "0" demotes; every other spelling
        // (unset included) stays the armed default-ON posture.
        assert!(env_demoted(Some("0")));
        for v in ["1", "true", "on", "off", "false", "00", " 0"] {
            assert!(!env_demoted(Some(v)), "spelling {v:?} must NOT demote");
        }
        assert!(!env_demoted(None));
    }

    #[test]
    fn strict_arms_only_on_the_promote_only_spellings() {
        for v in ["1", "true", "on"] {
            assert!(env_strict(Some(v)), "spelling {v:?} must arm strict");
        }
        for v in ["", "0", "yes", "TRUE", "On", "enable", "1 "] {
            assert!(!env_strict(Some(v)), "spelling {v:?} must NOT arm strict");
        }
        assert!(!env_strict(None));
    }

    #[test]
    fn cadence_env_parses_overrides_and_loudly_falls_back_on_junk() {
        let (c, note) = cadence_from(None);
        assert_eq!(c, Duration::from_secs(30));
        assert!(note.is_none());

        let (c, note) = cadence_from(Some("45"));
        assert_eq!(c, Duration::from_secs(45));
        assert!(note.is_none());

        let (c, note) = cadence_from(Some(" 12.5 "));
        assert_eq!(c, Duration::from_secs_f64(12.5));
        assert!(note.is_none());

        for raw in ["junk", "0", "-3", "inf", "nan", ""] {
            let (c, note) = cadence_from(Some(raw));
            assert_eq!(c, Duration::from_secs(30), "raw {raw:?} must fall back");
            let note = note.expect("junk cadence must carry a loud note");
            assert!(note.contains("REFLEX_LOAD_WATCH_CADENCE_SECS"), "{note}");
            assert!(
                note.contains(DEFAULT_CADENCE_SECS.to_string().as_str()),
                "{note}"
            );
        }
    }

    #[test]
    fn watcher_thread_samples_stops_and_joins() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let w = LoadWatch::spawn_with(
            move || {
                c.fetch_add(1, Ordering::SeqCst);
                Some(1.5)
            },
            LoadWatchConfig {
                cadence: Duration::from_millis(20),
            },
        );
        std::thread::sleep(Duration::from_millis(120));
        let m = w.finish();
        assert!(
            m.samples >= 2,
            "expected several samples, got {}",
            m.samples
        );
        assert_eq!(m.max, Some(1.5));
        assert!(m.window_secs >= 0.1);
        assert_eq!(counter.load(Ordering::SeqCst), m.samples);
        // The stop landed: no further probes fire after finish().
        let after = counter.load(Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(counter.load(Ordering::SeqCst), after);
    }

    #[test]
    fn watcher_window_shorter_than_cadence_takes_no_sample() {
        let w = LoadWatch::spawn_with(
            || Some(9.9),
            LoadWatchConfig {
                cadence: Duration::from_secs(3600),
            },
        );
        let m = w.finish();
        assert_eq!(m.samples, 0);
        assert_eq!(m.max, None);
        assert!(m.render_line().contains("no mid-run sample taken"));
    }

    /// Live smoke on the real box (run explicitly: `cargo test --lib
    /// load_watch -- --ignored`): three real `box_state::capture()` probes
    /// at 1s cadence. Exercises the platform arm (the Windows PowerShell
    /// probe on the 4090 lane box) end to end — a mechanism smoke, never a
    /// latency claim.
    #[test]
    #[ignore]
    fn live_box_smoke_three_real_probes() {
        let w = LoadWatch::spawn_with(
            || capture().load_1m,
            LoadWatchConfig {
                cadence: Duration::from_secs(1),
            },
        );
        // The Windows probe itself costs ~2s per sample (the PowerShell
        // spawn), so a sample's wall cost is cadence + probe — give the
        // window room for several ticks.
        std::thread::sleep(Duration::from_millis(7500));
        let m = w.finish();
        println!("{}", m.render_line());
        assert!(m.samples >= 2, "live smoke took {} samples", m.samples);
        assert!(m.unread < m.samples, "every live probe unreadable");
    }
}
