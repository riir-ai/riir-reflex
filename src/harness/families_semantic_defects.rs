//! Issue 061 — the `semantic_defects` family: a code-defect question
//! family over an ORIGINAL Rust fixture set, seeded from the perch
//! taxonomy (`lakeday-org/perch @ 07d38cac`, MIT — the distill verdict is
//! MARGINAL corpus / LANE INTEL HIGH in riir-refine `.distill/001`;
//! perch ships NO labelled eval set, verified at the pin, so every gold
//! here is ours).
//!
//! The question: which defect class does this routine exhibit — perch's
//! production scan shape (class × reachability), narrowed to the class
//! axis as a single Choice. Six labels: `clean` (the faithful control) +
//! five perch classes in Rust-native spelling (`off_by_one`,
//! `inverted_condition`, `unwrapped_none` ← unhandled_null,
//! `swapped_lookup` ← wrong_lookup, `swallowed_error` ← error_ignored).
//!
//! Authoring law (the wide gates own the enforcement):
//! - dual grounding at authoring time — each fixture makes its gold
//!   class derivable AND the five rivals refutable (the scan.yaml
//!   true/false-criteria bar, per issue 061);
//! - eval AND cal carry no label token (the text never names its own
//!   answer — a `None` literal, the word "unwrap" in prose, or the class
//!   names would leak);
//! - the code SHOWS the defect; the prose never pronounces the verdict;
//! - 6 × 17 eval balance, corpus 12 docs per class (issue 126 grew the
//!   corpus 3 → 12 per class to close the engine-vs-bm25 gap; additions
//!   pre-validated against the zero-3-gram wall + unigram ceilings;
//!   corpus MAY name the classes — that is what teaches the
//!   corpus-conditioned drafter).

use crate::harness::families::FamilyDef;
use crate::harness::suites::QKind;

static DEFECTS_INSTR: &str = "This routine was audited for a single defect \
class. clean = the code does exactly what its contract states; off_by_one = \
a bound or index steps a single position past (or falls a single position \
short of) the intended range; inverted_condition = a guard or comparison \
tests the opposite of the documented intent; unwrapped_none = a value that \
may be absent is force-decoded without handling the absent case; \
swapped_lookup = the wrong key, field, or index is read where a different \
target was intended; swallowed_error = a failing result is discarded \
silently instead of propagated or surfaced.";

pub static DEFECTS: FamilyDef = FamilyDef {
    name: "semantic_defects",
    qid: "defect_class",
    kind: QKind::Choice,
    labels: &[
        "clean",
        "off_by_one",
        "inverted_condition",
        "unwrapped_none",
        "swapped_lookup",
        "swallowed_error",
    ],
    instructions: DEFECTS_INSTR,
    score_levels: &[],
    note: "authored Rust scan fixtures (issue 061; taxonomy from lakeday-org/perch \
           @ 07d38cac, which ships no labelled set — gold is ours); gold = the audited \
           defect class; dual-grounding authoring bar (true-criteria + false-criteria \
           derivable per fixture); eval/cal label-token-free; 6x17 balance",
    corpus: &[
        // ── clean (0) ──
        (
            0,
            "A faithful routine needs no patch: its bounds match the declared range, \
             absence is handled where absence can happen, the intended field is read, \
             and failures travel to the caller through the return type.",
        ),
        (
            0,
            "The walk covers exactly the documented window — the final element is \
             visited and nothing beyond it is touched, including at the last tile, \
             the final page, and the wrap point.",
        ),
        (
            0,
            "Where a table read can miss, the fallback runs at the read site: a \
             default is supplied, and a failing sub-call returns its Result upward \
             instead of being reduced to a bare flag.",
        ),
        // ── clean (0) — issue 126 additions (per-class corpus growth) ──
        (
            0,
            "The ring's write cursor steps `(write + 1) % slots` and lands back \
             on its first position after serving every reserved entry, so the \
             storage bound is never crossed.",
        ),
        (
            0,
            "Bounds are proven before indexing: the check refuses a \
             length-equal position, and the read beneath it sees only \
             in-range slots.",
        ),
        (
            0,
            "Reads that can miss pick their fallback where the read happens — \
             `unwrap_or_else(Default::default)` spelled once — so no caller \
             inherits an absent value.",
        ),
        (
            0,
            "The throttle's condition agrees with its comment: hot paths shed \
             load while the flag is up, and the quiet path holds its fast lane.",
        ),
        (
            0,
            "The rollup binds each metric to the header column that names it, \
             and the export's field order matches the schema, so no figure \
             lands under a neighbor's label.",
        ),
        (
            0,
            "Each fallible helper hands its Result to the caller through `?`, \
             the error naming the step that failed, so a half-finished batch \
             is visible as exactly that.",
        ),
        (
            0,
            "The lookup feeds `match`, and the miss arm builds the documented \
             default; the suite drives both arms through populated and empty \
             inputs.",
        ),
        (
            0,
            "The pass consumes precisely the `count` the header declares — the \
             last element is read once and the loop leaves with the cursor \
             resting on the end boundary.",
        ),
        (
            0,
            "Grouping folds whole blocks of four and lets the trailing partial \
             block fold its own true width, so the final sum matches the byte \
             count.",
        ),
        // ── off_by_one (1) ──
        (
            1,
            "off_by_one: the loop or slice bound reaches a single element beyond \
             the buffer's end, or stops a single element before the last item the \
             contract promises to cover — inclusive `..=` on a count, `<=` on a \
             length, a stride tile whose end index outruns the allocation.",
        ),
        (
            1,
            "The classic shape is a count used as an inclusive endpoint, so index \
             `count` is touched although valid slots run `0 .. count` exclusive; \
             the cousin starts the walk at 1 and silently skips the first slot.",
        ),
        (
            1,
            "Window arithmetic that adds a width to a start position without \
             re-checking the sum against the length reads past the end on the \
             final window; wrap math with `% (capacity + 1)` claims a slot the \
             array never reserved.",
        ),
        // ── off_by_one (1) — issue 126 additions ──
        (
            1,
            "The paginator's window closes with `..=`, so neighboring pages \
             share one boundary record and the closing page's upper edge reads \
             the vector at its length.",
        ),
        (
            1,
            "Chunked copies step a fixed width without clamping the end; an \
             array whose length leaves a remainder makes the final slice reach \
             past the storage.",
        ),
        (
            1,
            "A walk bounded by `0 ..= n` treats the count as an endpoint and \
             touches one position past the final valid slot — a sentinel \
             element no allocation ever held.",
        ),
        (
            1,
            "Ring math adds one to the capacity before the modulo, so the \
             write position can equal the size and overwrite the cell adjacent \
             to the array.",
        ),
        (
            1,
            "The history cut starts at `len() - 1` beneath a comment promising \
             the final pair, and the result carries one fewer record than the \
             contract describes.",
        ),
        (
            1,
            "Retry bookkeeping loops through `max_attempts` inclusive, so one \
             extra attempt runs after the policy's final permitted call.",
        ),
        (
            1,
            "Window arithmetic sums a start and a width and slices on the sum, \
             never comparing the end position against the data's length.",
        ),
        (
            1,
            "The scan begins its counter at one and so never touches the first \
             row — the earliest record is skipped without a warning or a log.",
        ),
        (
            1,
            "Anchor positions add the span to the base and cut the segment on \
             the total, reading one byte past the section's end on the final \
             call.",
        ),
        // ── inverted_condition (2) ──
        (
            2,
            "inverted_condition: the guard tests the opposite sense of its \
             documented purpose — the branch meant for the rare case runs for the \
             common case, and the normal path is skipped. Every token is spelled \
             correctly; only the sense is wrong.",
        ),
        (
            2,
            "A negation in the wrong place flips eligibility: members are refused \
             while guests pass, fresh keys never recompute while cached ones do, \
             the retry triggers exactly when it should not.",
        ),
        (
            2,
            "Equality intended, inequality written (or the boolean literal flipped): \
             the comparison passes precisely when it should fail, so the alarm \
             fires on agreement and sleeps through divergence.",
        ),
        // ── inverted_condition (2) — issue 126 additions ──
        (
            2,
            "The gate guards entry with `if !cleared { admit(); }`, so the \
             badge check lets through exactly the holders it was written to \
             refuse.",
        ),
        (
            2,
            "Merge is specified for equal fingerprints, but the branch fires \
             on `!=` and the store deduplicates the mismatches while identical \
             records pile up as separate rows.",
        ),
        (
            2,
            "The backstop re-arms when the queue is EMPTY — the sense of the \
             emptiness test is reversed, so the watchdog wakes on the one \
             situation where nothing needs saving.",
        ),
        (
            2,
            "A flipped boolean literal inverts the policy: the maintenance \
             window opens on production during business hours and stays shut \
             for the nightly slot it was built for.",
        ),
        (
            2,
            "Membership reads `if applicant.external { approve(); }`, though \
             the doc says approval is for internal accounts; the sign-up flow \
             waves outsiders through.",
        ),
        (
            2,
            "The freshness check compares staleness with `<` where the policy \
             draws the line at `>`, so caches refresh precisely when they are \
             already current and serve stale data otherwise.",
        ),
        (
            2,
            "Escalation triggers on success: the pager fires when the job \
             reports healthy and stays silent through the failing runs it \
             exists to catch.",
        ),
        (
            2,
            "The retry clause sits in `if attempts == limit`, spending its \
             budget on the one attempt that was supposed to give up.",
        ),
        (
            2,
            "Expiry tests `now < deadline` under a policy that reads \
             revoke-on-or-after, so lapsed credentials keep their tokens alive \
             a step past the wall.",
        ),
        // ── unwrapped_none (3) ──
        (
            3,
            "unwrapped_none: a value that can legitimately be absent gets \
             force-decoded at the read site — a missing entry panics instead of \
             taking the documented fallback.",
        ),
        (
            3,
            "The absence path exists in the type but the code steps around it: \
             `.unwrap()` or a reassuring `.expect(..)` replaces the match the \
             signature asks for, and the panic lands on data the docs call \
             optional — an omitted config key, a truncated file, a raced pop.",
        ),
        (
            3,
            "Optional input decoded with unwrap at boot or at the API boundary: \
             any deployment or request that omits the optional piece dies before \
             the service is reachable, though the type system offered the \
             fallback arm.",
        ),
        // ── unwrapped_none (3) — issue 126 additions ──
        (
            3,
            "The optional telemetry endpoint is read with `.unwrap()` during \
             boot, so a deployment that omits the key dies before the listener \
             binds.",
        ),
        (
            3,
            "`config.timeout.expect(\"set\")` decorates an optional knob; the \
             panic message reassures while the absent case the type allows is \
             never handled.",
        ),
        (
            3,
            "The drain task force-pops the queue inside its tick, and the \
             raced removal between the length check and the read turns an \
             empty moment into a crash.",
        ),
        (
            3,
            "A header table parsed from a truncated file force-decodes each \
             entry, and the missing trailing row panics the loader instead of \
             reporting the short read.",
        ),
        (
            3,
            "The cache's `map[key].clone()` assumes presence for a key the \
             writer may not have published at that point; the reader thread \
             panics on the first cold lookup.",
        ),
        (
            3,
            "Environment parsing unwraps the locale variable the docs call \
             optional, so containers without it refuse to start though a \
             default existed in the type.",
        ),
        (
            3,
            "The shutdown path unwraps the handle it may have already closed, \
             and the second stop request panics the process it was meant to \
             wind down.",
        ),
        (
            3,
            "Iterator stepping reads `.next().unwrap()` past the promised \
             element count; the optional the signature returns exists \
             precisely so the walk can stop.",
        ),
        (
            3,
            "The probe decodes an optional measurement with `expect` and the \
             annotated-absent case — a sensor that reports nothing — kills \
             the sampling loop.",
        ),
        // ── swapped_lookup (4) ──
        (
            4,
            "swapped_lookup: the read targets the wrong key, field, or axis — an \
             identifier used where a name was intended, the adjacent getter \
             called, rows and columns transposed — so the value returned belongs \
             to a different row.",
        ),
        (
            4,
            "Copy-paste leaves the neighboring variable in the index expression; \
             the two tables have compatible value types, so the compiler stays \
             silent and only the data notices.",
        ),
        (
            4,
            "Twin keys from the same table are the usual culprit: `verbose` bound \
             into `quiet`, `p95` recorded as `p99`, the stable slot read where \
             the rollout candidate was wanted, `en_US` served for `en_GB`.",
        ),
        // ── swapped_lookup (4) — issue 126 additions ──
        (
            4,
            "Grid indices arrive as `[column][line]` in the renderer's cell \
             pass; square demo boards hide the flip until rectangular \
             production data skews each cell.",
        ),
        (
            4,
            "The resolver consults the alias table where the canonical \
             registry indexes by UUID; repeated aliases across regions fold \
             distinct accounts into one balance.",
        ),
        (
            4,
            "Copy-paste leaves the sibling variable inside the brackets — \
             `settings[read]` bound where `settings[write]` was intended — \
             and the two tables share a value type, so the compiler is \
             silent.",
        ),
        (
            4,
            "The telemetry path records `quantile(0.95)` into the field the \
             schema names `p99`, and the dashboard's tail latency reads one \
             bucket optimistic.",
        ),
        (
            4,
            "The locale picker serves `en_US` strings for the `en_GB` \
             requests; the keys sit one token apart in the same table.",
        ),
        (
            4,
            "The rollout reads the stable slot where the candidate slot was \
             wanted, so every canary measurement describes the incumbent's \
             behavior.",
        ),
        (
            4,
            "Adjacent getters differ by one word — `created_at()` bound where \
             `updated_at()` was intended — and the change history sorts by \
             the wrong moment.",
        ),
        (
            4,
            "The export writes rows under the column order of a previous \
             schema version; each value lands one column left of its label.",
        ),
        (
            4,
            "The permissions check reads the role map with the group key, and \
             because both tables key on strings the lookup succeeds with a \
             different group's rights.",
        ),
        // ── swallowed_error (5) ──
        (
            5,
            "swallowed_error: the failure half of a Result is discarded at the \
             call site — `let _ =`, `.ok()`, a wildcard match arm mapping every \
             failure to a default — so the routine reports success regardless.",
        ),
        (
            5,
            "The diagnostic payload vanishes with the outcome: no log line, no \
             counter, no poisoned-batch signal. Outages masquerade as ordinary \
             empty results, and the SLO summary reads a clean run that never \
             happened.",
        ),
        (
            5,
            "Durable-write contracts are the common victim: a dropped append, an \
             ignored flush, a torn config that leaves yesterday's policy running \
             — each visible only after the data is gone.",
        ),
        // ── swallowed_error (5) — issue 126 additions ──
        (
            5,
            "`let _ = unlink(staging);` precedes the next pass, and leftover \
             temporaries pile up in the work directory while the descriptor \
             stays busy.",
        ),
        (
            5,
            "The scheduling tick calls `publisher.flush().ok();`, so a write \
             that fails leaves that period's counters unrecorded while the \
             dashboards stay green.",
        ),
        (
            5,
            "A wildcard match arm maps every write failure to the default \
             value, so the importer reports a clean run having skipped an \
             entire batch.",
        ),
        (
            5,
            "The checkpoint routine discards the WAL append with `let _ =`, \
             and the printed checkpoint line claims durability the log never \
             received.",
        ),
        (
            5,
            "`send(...).await.ok();` sits in the shutdown drain; queued work \
             evaporates under load and the process still exits zero.",
        ),
        (
            5,
            "The config reloader ignores the parse result and keeps \
             yesterday's policy running through the failed deploy, with no \
             counter incremented and no log line.",
        ),
        (
            5,
            "The sync loop drops the replicate result on the floor; outages \
             read as ordinary empty diffs until the data is gone.",
        ),
        (
            5,
            "An ignored shrink precedes the rewrite; when the shortening fails \
             the file keeps its previous length and the previous contents are \
             read back as if they were new records.",
        ),
        (
            5,
            "The exporter's error branch returns `Ok(defaults)` for the \
             failed section; the summary reports success and the missing \
             section is invisible.",
        ),
    ],
    cal: &[
        crate::harness::families::FamilyText {
            text: "Iterates `0 .. window_count` collecting exactly the windows the \
                   header declares; the collector's length matches the count field.",
            gold: 0,
        },
        crate::harness::families::FamilyText {
            text: "Cache misses resolve via `unwrap_or(Loadout::default())`, spelled \
                   at the single read site; callers always see a value.",
            gold: 0,
        },
        crate::harness::families::FamilyText {
            text: "The report selects `leads[lead_uuid]`, matching the column the CSV \
                   header names; each figure lands in its own column.",
            gold: 0,
        },
        crate::harness::families::FamilyText {
            text: "Reads `&tape[pos ..= pos + frame]` while stepping forward, touching \
                   the frame after the window at every step but the last.",
            gold: 1,
        },
        crate::harness::families::FamilyText {
            text: "`if !throttled { throttle(); }` sits beneath a comment promising \
                   the reverse policy for hot paths.",
            gold: 2,
        },
        crate::harness::families::FamilyText {
            text: "`ports.get(\"https\").unwrap()` inside a library whose docs \
                   promise a plain-HTTP fallback when TLS is unavailable.",
            gold: 3,
        },
        crate::harness::families::FamilyText {
            text: "`ledgers[acct_alias]` fetched where the balance table keys \
                   `acct_uuid`; aliases repeat across regions and merge balances.",
            gold: 4,
        },
        crate::harness::families::FamilyText {
            text: "`let _ = mirror.flush();` precedes the checkpoint line; a dropped \
                   flush still prints the checkpoint as written.",
            gold: 5,
        },
        crate::harness::families::FamilyText {
            text: "The stepper walks `0 ..= steps.len()`, advancing through the \
                   sentinel position past the final plan step.",
            gold: 1,
        },
        crate::harness::families::FamilyText {
            text: "`metrics[\"rps\"]` is written into the struct field named \
                   `latency_ms` in the exported rollup.",
            gold: 4,
        },
        crate::harness::families::FamilyText {
            text: "`queue.front().unwrap()` sits in the tick handler, which the \
                   design doc says drains to empty between ticks.",
            gold: 3,
        },
        crate::harness::families::FamilyText {
            text: "The purger tests `entry.age() < horizon` where policy says \
                   entries older than the horizon are the purge set.",
            gold: 2,
        },
        crate::harness::families::FamilyText {
            text: "The guard reads `if user.banned { welcome(user); }` beneath a \
                   docstring about enforcement at the door.",
            gold: 2,
        },
        crate::harness::families::FamilyText {
            text: "Wrap arithmetic `% (slots + 1)` lets the producer claim a \
                   position the ring array never allocated.",
            gold: 1,
        },
        crate::harness::families::FamilyText {
            text: "`retry(attempt + 1)` loops `0 ..= max_attempts`, issuing a \
                   further attempt past the documented policy count.",
            gold: 1,
        },
        crate::harness::families::FamilyText {
            text: "`tx.send(batch).await.ok();` sits in the shutdown path; queued \
                   work evaporates under load with the exit code still zero.",
            gold: 5,
        },
        crate::harness::families::FamilyText {
            text: "`tiles[y][x]` is read in the pathfinder's neighbor probe where \
                   every other site reads `tiles[x][y]`.",
            gold: 4,
        },
        crate::harness::families::FamilyText {
            text: "Anchor math adds `span` to the cursor with no length re-check; \
                   the final call reads past the segment end.",
            gold: 1,
        },
    ],
    eval: &crate::harness::families_semantic_defects_eval::DEFECTS_WIDE_EVAL,
};
