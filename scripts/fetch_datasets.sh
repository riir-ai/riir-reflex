#!/bin/sh
# fetch_datasets.sh — riir-reflex Plan 603 T1.5 benchmark dataset fetch layer.
#
# Pulls every benchmark suite's rows from the HF datasets-server /rows API into
# .raw/datasets/<suite>/<split>-<NNN>.json — one file per 100-row page, offset
# paging until a page returns fewer than 100 rows or the suite's row cap is
# hit. Companion probes (/splits, /size) land beside the pages as splits.json
# / size.json, and the hub licence probe (riir-rethink Issue 023 T1) lands as
# license.json — the drift tripwire beside the manifest's VERIFIED licence
# table. .raw/ is gitignored; re-running regenerates the files (the
# committed manifest .docs/02_protocols/dataset_manifest.md records their blake3 digests,
# byte sizes and row counts).
#
# Idempotent: an existing page file with the row count its position expects is
# skipped; FORCE=1 refetches everything. Politeness: 0.3s sleep between calls
# (SLEEP=<sec> overrides — empirically HF datasets-server 429s somewhere
# around ~130 rapid unauthenticated requests, so long runs want SLEEP=1.5 or
# more; re-run after a 429 wall once the limiter cools down, the skip logic
# resumes exactly where it stopped), curl --max-time 60 --retry 2, plus one
# 5s-backoff retry per page for non-200 / JSON-error responses. A failed page
# is recorded loudly and the remaining suites still run (never abort
# everything).
#
# Issue 096 — the resume law. A SKIPPED page that holds fewer rows than its
# position wants is NOT necessarily the terminal page: the end-of-run dedupe
# (Issue 058) rewrites mid-sequence pages in place, so a 100-row page can sit
# on disk as a 99-row file. The resume predicate therefore consults the
# page's own num_rows_total (the /rows API stamps it on EVERY page and the
# dedupe's `.rows |=` rewrite preserves it): offset+n < total means the
# dataset holds more rows — keep walking. A short page with no readable
# num_rows_total falls back to the legacy stop, LOUDLY. Proven by
# `scripts/fetch_datasets.sh --self-test` (hermetic loopback server; layer 8
# of ci_feature_guard.sh).
#
# banking77 — PolyAI/banking77 (the Colab variant) is script-based and 404s on
# /rows; the mteb/banking77 mirror (the bench_apps variant) is fetched instead.
# dataset, which the datasets-server no longer serves — /rows 404s and /splits
# 500s. The suite stays in the attempt list so every run re-proves the gap.
#
# Usage:  scripts/fetch_datasets.sh
#         FORCE=1 scripts/fetch_datasets.sh
#         BASE=http://127.0.0.1:PORT ... (test override — the datasets-server
#         root; self-test rides it, never the real hub)
#         scripts/fetch_datasets.sh --self-test
#
# Exit: 0 = every attempted page fetched clean; 1 = at least one failure
# (see the FAIL lines and the manifest's Gaps section).

set -eu

PAGE=100
SLEEP="${SLEEP:-0.3}"
FORCE="${FORCE:-0}"
BASE="${BASE:-https://datasets-server.huggingface.co}"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
# OUT / TRAIN_CAP overrides (issue 038 T2): a larger train pull goes to a
# SEPARATE dir so the canonical .raw/datasets (the manifest-digested set
# every published table reads) is never rewritten under a sibling run.
OUT="${OUT:-$ROOT/.raw/datasets}"
TRAIN_CAP="${TRAIN_CAP:-20000}"
# Issue 058 (verdict condition 1, 2026-10-01): the default OUT is the
# canonical .raw/datasets, which is now the FULL-pull pool — a 4000-row
# default here was the root cause of the silent board regression (a fresh
# fetch rebuilt 4k-row pools under the same dir name and every number
# moved). The default now matches the t20k re-baseline scale; the
# slice-integrity gate's per-suite pool floors (MIN_POOL_ROWS) refuse a
# shrunken pull at run time regardless.
# SUITES (issue 041): comma-separated subset filter, e.g.
#   SUITES=ag_news OUT=$ROOT/.raw/datasets_agnews_full TRAIN_CAP=120000 scripts/fetch_datasets.sh
# Default "all" = the birth behavior (every suite's blocks run). The
# probes at the top of main() honor the filter too — a filtered run
# spends its politeness budget only on the named suites.
SUITES="${SUITES:-all}"
# The DIRECT-RESOLVE lane's host root (issue 114 T4/T5): the gorilla-llm BFCL
# mirror and the API-Bank mirror are raw JSON/JSON-LINES files the
# datasets-server does not serve (/rows 404s / cast-errors — the issue's probe
# record), so their fetch rides the hub's /resolve/main/<file> endpoint. The
# self-test overrides this beside BASE (hermetic loopback, never the hub).
RAW_BASE="${RAW_BASE:-https://huggingface.co}"
MAX_PARTS=$(( TRAIN_CAP / 100 > 200 ? TRAIN_CAP / 100 + 1 : 200 ))   # hard stop: never below the 200-page birth guard

FILES=0
ROWS=0
FAILED=0

log() { printf '%s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }

# True when SUITES is "all" or names <suite> in its comma list.
want_suite() {
    [ "$SUITES" = "all" ] && return 0
    case ",$SUITES," in *",$1,"*) return 0 ;; esac
    return 1
}

# Python interpreter for the jq-less fallback paths (Issue 018: the windows
# lane). `python3` on Windows is the Microsoft Store alias STUB — it exists
# on PATH but only prints "Python was not found". Resolve a WORKING
# interpreter once: prefer python3, fall back to `py -3` (the standard
# Windows launcher). Sets PY3 to the full invocation string, or empty when
# neither works (the jq paths below never consult PY3 when jq is present).
PY3=""
if ! have jq; then
    if have python3 && python3 -c pass >/dev/null 2>&1; then
        PY3="python3"
    elif have py && py -3 -c pass >/dev/null 2>&1; then
        PY3="py -3"
    fi
fi

# curl GET -> file; -f so non-2xx exits non-zero; --retry handles transient
# transport errors / 429 / 5xx at the curl level.
http_get() {
    curl -fsSL --max-time 60 --retry 2 -o "$2" "$1"
}

# Row count of a /rows response file.
rows_count() {
    if have jq; then
        jq '.rows | length' "$1"
    elif [ -n "$PY3" ]; then
        $PY3 - "$1" <<'EOF'
import json, sys
with open(sys.argv[1], encoding="utf-8") as f:
    print(len(json.load(f).get("rows", [])))
EOF
    else
        echo 0
    fi
}

# True if the file is a well-formed /rows response (an object with "rows").
rows_valid() {
    if have jq; then
        jq -e 'type == "object" and has("rows")' "$1" >/dev/null 2>&1
    elif [ -n "$PY3" ]; then
        $PY3 - "$1" <<'EOF'
import json, sys
try:
    with open(sys.argv[1], encoding="utf-8") as f:
        d = json.load(f)
    sys.exit(0 if isinstance(d, dict) and "rows" in d else 1)
except Exception:
    sys.exit(1)
EOF
    else
        return 1
    fi
}

# The dataset total a /rows response page carries (Issue 096): the API
# stamps num_rows_total on EVERY page, and the dedupe's `.rows |=` rewrite
# preserves every top-level field — so any stored page can answer "does the
# dataset hold more rows beyond my offset?" without re-reading size.json
# (whose config/split keying is lossy for the as_split-renamed suites).
# Prints the integer, or nothing when absent/unreadable (the caller falls
# back to the legacy stop, loudly).
rows_total_of() {
    if have jq; then
        jq -r '.num_rows_total // empty' "$1" 2>/dev/null || true
    elif [ -n "$PY3" ]; then
        $PY3 - "$1" <<'EOF'
import json, sys
try:
    with open(sys.argv[1], encoding="utf-8") as f:
        v = json.load(f).get("num_rows_total", "")
    if isinstance(v, int):
        print(v)
except Exception:
    pass
EOF
    fi
}

# True if <suite>/splits.json shows the config/split pair.
splits_have() {
    if have jq; then
        jq -e --arg c "$2" --arg s "$3" \
           '.splits[]? | select(.config == $c and .split == $s)' \
           "$OUT/$1/splits.json" >/dev/null 2>&1
    elif [ -n "$PY3" ]; then
        $PY3 - "$OUT/$1/splits.json" "$2" "$3" <<'EOF'
import json, sys
try:
    with open(sys.argv[1], encoding="utf-8") as f:
        d = json.load(f)
    hit = any(s.get("config") == sys.argv[2] and s.get("split") == sys.argv[3]
              for s in d.get("splits", []))
    sys.exit(0 if hit else 1)
except Exception:
    sys.exit(1)
EOF
    else
        return 1
    fi
}

# fetch_suite <suite> <urlencoded-dataset> <urlencoded-config>
#             <urlencoded-split> <cap: number | "all"> [<as-split>]
#
# <as-split> (Plan 012): the FILENAME prefix, when the stored split must
# differ from the fetched one — mmlupro_jdi has no train split, so its
# 70-row validation pool is stored as the suite's `train` (the corpus/cal
# pool the harness reads), and anli_jdi's train_r1 rounds are stored as
# `train`. Defaults to <split> (every pre-existing caller is unchanged).
fetch_suite() {
    suite=$1; ds=$2; cfg=$3; split=$4; cap=$5
    as_split="${6:-$split}"
    dir="$OUT/$suite"
    mkdir -p "$dir"
    part=0
    total=0
    files=0
    suite_failed=0
    while [ "$part" -lt "$MAX_PARTS" ]; do
        if [ "$cap" != "all" ] && [ "$total" -ge "$cap" ]; then
            break
        fi
        pagenum=$(printf '%03d' "$part")
        file="$dir/$as_split-$pagenum.json"
        want=$PAGE
        if [ "$cap" != "all" ]; then
            want=$((cap - total))
            if [ "$want" -gt "$PAGE" ]; then
                want=$PAGE
            fi
        fi
        if [ "$FORCE" != "1" ] && [ -s "$file" ] && rows_valid "$file"; then
            n=$(rows_count "$file" 2>/dev/null || echo 0)
            if [ "$n" -gt 0 ]; then
                log "  skip   $suite/$as_split-$pagenum ($n rows)"
                total=$((total + n))
                files=$((files + 1))
                if [ "$n" -lt "$want" ]; then
                    # Issue 096: a short SKIPPED page is the terminal page
                    # only if the dataset really ends here. The end-of-run
                    # dedupe rewrites mid-sequence pages in place (a 100-row
                    # page becomes a 99-row FILE), so decide with the page's
                    # own num_rows_total, not the row count.
                    ds_total=$(rows_total_of "$file" 2>/dev/null || echo "")
                    offset=$((part * PAGE))
                    case "$ds_total" in
                        ''|*[!0-9]*)
                            log "  note   $suite/$as_split-$pagenum short page with no readable num_rows_total — treated as terminal (legacy stop, Issue 096 fallback)"
                            break
                            ;;
                    esac
                    if [ "$((offset + n))" -lt "$ds_total" ]; then
                        log "  resume $suite/$as_split-$pagenum short page is NOT terminal ($n rows at offset $offset, dataset total $ds_total) — continuing"
                        part=$((part + 1))
                        continue
                    fi
                    break   # offset+n == num_rows_total: the true terminal page
                fi
                if [ "$cap" != "all" ] && [ "$total" -ge "$cap" ]; then
                    break
                fi
                part=$((part + 1))
                continue
            fi
        fi
        offset=$((part * PAGE))
        url="$BASE/rows?dataset=$ds&config=$cfg&split=$split&offset=$offset&length=$PAGE"
        tmp="$file.tmp"
        ok=0
        if http_get "$url" "$tmp"; then
            ok=1
        fi
        if [ "$ok" -eq 0 ]; then
            sleep 5
            if http_get "$url" "$tmp"; then
                ok=1
            fi
        fi
        if [ "$ok" -eq 0 ]; then
            rm -f "$tmp"
            log "  FAIL   $suite/$as_split-$pagenum (http error, offset=$offset)"
            FAILED=$((FAILED + 1))
            suite_failed=1
            break
        fi
        if ! rows_valid "$tmp"; then
            rm -f "$tmp"
            log "  FAIL   $suite/$as_split-$pagenum (JSON error body, offset=$offset)"
            FAILED=$((FAILED + 1))
            suite_failed=1
            break
        fi
        n=$(rows_count "$tmp" 2>/dev/null || echo 0)
        if [ "$n" -eq 0 ]; then
            rm -f "$tmp"
            log "  done   $suite/$as_split (no more rows at offset=$offset)"
            break
        fi
        mv "$tmp" "$file"
        FILES=$((FILES + 1))
        ROWS=$((ROWS + n))
        total=$((total + n))
        files=$((files + 1))
        log "  fetch  $suite/$as_split-$pagenum rows=$n offset=$offset"
        if [ "$n" -lt "$PAGE" ]; then
            break
        fi
        if [ "$cap" != "all" ] && [ "$total" -ge "$cap" ]; then
            break
        fi
        part=$((part + 1))
        sleep "$SLEEP"
    done
    if [ "$part" -ge "$MAX_PARTS" ]; then
        log "  FAIL   $suite/$as_split hit MAX_PARTS=$MAX_PARTS guard"
        FAILED=$((FAILED + 1))
        suite_failed=1
    fi
    rows_total=$(jq -r '.num_rows_total // "?"' "$dir/$as_split-000.json" 2>/dev/null || echo "?")
    if [ "$suite_failed" -eq 0 ]; then
        log "  ==     $suite/$as_split: $total rows in $files file(s) (dataset total: $rows_total)"
    else
        log "  ==     $suite/$as_split: $total rows in $files file(s) BEFORE FAILURE (dataset total: $rows_total)"
    fi
    sleep "$SLEEP"
}

# probe_splits <suite> <urlencoded-dataset> — save the /splits listing.
probe_splits() {
    suite=$1; ds=$2
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/splits.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ]; then
        log "  skip   $suite/splits.json (cached)"
        return 0
    fi
    url="$BASE/splits?dataset=$ds"
    ok=0
    if http_get "$url" "$file.tmp"; then
        ok=1
    fi
    if [ "$ok" -eq 0 ]; then
        sleep 5
        if http_get "$url" "$file.tmp"; then
            ok=1
        fi
    fi
    if [ "$ok" -eq 1 ]; then
        mv "$file.tmp" "$file"
        log "  probe  $suite/splits.json"
        if have jq; then
            jq -r '.splits[] | "         \(.config)/\(.split)"' "$file"
        fi
    else
        rm -f "$file.tmp"
        log "  FAIL   $suite/splits.json probe"
        FAILED=$((FAILED + 1))
    fi
    sleep "$SLEEP"
}

# probe_size <suite> <urlencoded-dataset> — save the /size listing.
probe_size() {
    suite=$1; ds=$2
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/size.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ]; then
        log "  skip   $suite/size.json (cached)"
        return 0
    fi
    url="$BASE/size?dataset=$ds"
    ok=0
    if http_get "$url" "$file.tmp"; then
        ok=1
    fi
    if [ "$ok" -eq 0 ]; then
        sleep 5
        if http_get "$url" "$file.tmp"; then
            ok=1
        fi
    fi
    if [ "$ok" -eq 1 ]; then
        mv "$file.tmp" "$file"
        log "  probe  $suite/size.json"
        if have jq; then
            jq -r '.size.splits[]? | "         \(.config)/\(.split) num_rows=\(.num_rows)"' "$file"
        fi
    else
        rm -f "$file.tmp"
        log "  FAIL   $suite/size.json probe"
        FAILED=$((FAILED + 1))
    fi
    sleep "$SLEEP"
}

# probe_license <suite> <urlencoded-dataset> — save the HUB metadata's
# licence signal (riir-rethink Issue 023 T1: the licence ledger's fetch-side
# tripwire). The hub API carries the licence as a TAG (`license:xxx`) for
# most datasets and as the top-level `license` field for some; the probe
# records BOTH plus the cardData license when present. The VERIFIED table
# in .docs/02_protocols/dataset_manifest.md §Licences stays AUTHORITATIVE
# (a card saying only `license:other` — e.g. dair-ai/emotion's
# research-only terms — is resolved by the human read, never guessed);
# this file exists so a hub-side licence CHANGE surfaces at refetch time
# (diff the probe against the manifest row).
HUB="https://huggingface.co/api/datasets"
probe_license() {
    suite=$1; ds=$2
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/license.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ]; then
        log "  skip   $suite/license.json (cached)"
        return 0
    fi
    # The hub API wants the literal slash (unlike the datasets-server
    # probes, which want %2F) — decode a two-segment dataset id here
    # (issue 114 T8: the first two-segment name through this probe).
    ds_literal=$(printf '%s' "$ds" | sed 's|%2F|/|g')
    url="$HUB/$ds_literal"
    ok=0
    if http_get "$url" "$file.tmp"; then
        ok=1
    fi
    if [ "$ok" -eq 0 ]; then
        sleep 5
        if http_get "$url" "$file.tmp"; then
            ok=1
        fi
    fi
    if [ "$ok" -eq 1 ]; then
        mv "$file.tmp" "$file"
        log "  probe  $suite/license.json"
        if have jq; then
            lic=$(jq -r '([.license] + [.tags[]? | select(startswith("license:"))]) | map(select(. != null and . != "")) | join(", ")' "$file" 2>/dev/null)
            log "         licence signal: ${lic:-none — hub tags absent, the manifest licence table is authoritative}"
        fi
    else
        rm -f "$file.tmp"
        log "  FAIL   $suite/license.json probe"
        FAILED=$((FAILED + 1))
    fi
    sleep "$SLEEP"
}

# dedupe_train <suite> — the Issue-058 slice-integrity law at the fetch
# plane: a train pull is de-duplicated against itself and against the test
# split at EXACT-row identity, dropping only TRAIN-side copies (the test
# population is the eval surface and is never rewritten). The harness's
# slice_guard refuses a suite whose pool∩test or pool∩cal rows are not
# distinct, so a fetch that skips this law produces a pull the run gate
# refuses — the disclosure here is the cheap half. jq-path only: without
# jq the pass SKIPS LOUD (the 4090 syncs data by copying from this box —
# the 077 procedure — so the jq box is the producer of clean pulls).
dedupe_train() {
    suite=$1
    dir="$OUT/$suite"
    if ! have jq; then
        log "  dedupe $suite/train SKIPPED (no jq on this box — sync data from a jq host)"
        return 0
    fi
    [ -d "$dir" ] || return 0
    ls "$dir"/test-*.json >/dev/null 2>&1 || return 0
    keys=$(mktemp)          # newline-separated JSON strings (the seen set)
    kjson=$(mktemp)         # the same set as one JSON array (jq --slurpfile)
    for f in "$dir"/test-*.json; do
        jq -c '.rows[] | (.row | to_entries | sort_by(.key) | from_entries | tojson)' "$f" >> "$keys"
    done
    jq -s . "$keys" > "$kjson"
    removed=0
    for f in "$dir"/train-*.json; do
        [ -s "$f" ] || continue
        n=$(jq --slurpfile seen "$kjson" '
            [.rows[]
             | (.row | to_entries | sort_by(.key) | from_entries | tojson) as $k
             | select($seen[0] | any(. == $k))] | length' "$f") || n=0
        if [ "${n:-0}" -gt 0 ]; then
            jq --slurpfile seen "$kjson" '
                .rows |= [.[] | select(
                    (.row | to_entries | sort_by(.key) | from_entries | tojson) as $k
                    | ($seen[0] | any(. == $k)) | not)]' "$f" > "$f.tmp" \
                && mv "$f.tmp" "$f"
            removed=$((removed + n))
            log "  dedupe $suite/$(basename "$f"): dropped $n exact-duplicate row(s)"
        fi
        # Extend the seen-set with this page's survivors so later pages
        # drop internal duplicates too.
        jq -c '.rows[] | (.row | to_entries | sort_by(.key) | from_entries | tojson)' "$f" >> "$keys"
        jq -s . "$keys" > "$kjson"
    done
    rm -f "$keys" "$kjson"
    if [ "$removed" -gt 0 ]; then
        log "  dedupe $suite/train: $removed exact-duplicate row(s) dropped (self + test overlap)"
    fi
}

# ── The DIRECT-RESOLVE lane (issue 114 T4/T5) ──────────────────────────
# The gorilla-llm BFCL mirror and the API-Bank mirror are raw JSON / JSON-
# LINES files the datasets-server does not serve (/rows 404s "no supported
# data files" on the BFCL repo; the API-Bank files cast-error with
# inconsistent columns — the issue's probe record). Their fetch rides the
# hub's /resolve/main/<file> endpoint and the converter wraps the rows into
# the datasets-server ENVELOPE shape the builders read
# ({"num_rows_total": N, "rows": [{"row_idx": i, "row": {...}}]}). The
# transform is recorded here + in the manifest row; the digest pins the
# OUTPUT envelope, never the upstream bytes. jq-path only: without jq the
# conversion SKIPS LOUD (the dedupe_train posture — this box is the
# producer of clean pulls).

# _raw_convert <raw-file> <shape: jsonl|array> <out-file> <label>
#   Wraps an upstream JSONL or JSON-array file into the envelope shape.
#   Every element must be a JSON object; a non-object element fails loud.
_raw_convert() {
    raw=$1; shape=$2; out=$3; label=$4
    if have jq; then
        if [ "$shape" = "jsonl" ]; then
            jq -s -e 'length > 0 and all(.[]; type == "object")' "$raw" >/dev/null 2>&1 || {
                rm -f "$raw"
                log "  FAIL   $label (jsonl validation: empty or a non-object line)"
                FAILED=$((FAILED + 1))
                return 1
            }
            jq -s '{num_rows_total: length, rows: (to_entries | map({row_idx: .key, row: .value}))}' "$raw" > "$out"
        else
            jq -e 'type == "array" and length > 0 and all(.[]; type == "object")' "$raw" >/dev/null 2>&1 || {
                rm -f "$raw"
                log "  FAIL   $label (array validation: not a non-empty array of objects)"
                FAILED=$((FAILED + 1))
                return 1
            }
            jq '{num_rows_total: length, rows: (to_entries | map({row_idx: .key, row: .value}))}' "$raw" > "$out"
        fi
        return 0
    fi
    if [ -n "$PY3" ]; then
        $PY3 - "$raw" "$shape" "$out" <<'EOF'
import json, sys
raw, shape, out = sys.argv[1], sys.argv[2], sys.argv[3]
with open(raw, encoding="utf-8") as f:
    if shape == "jsonl":
        rows = [json.loads(line) for line in f if line.strip()]
    else:
        rows = json.load(f)
if not isinstance(rows, list) or not rows or not all(isinstance(r, dict) for r in rows):
    sys.exit(2)
env = {"num_rows_total": len(rows),
       "rows": [{"row_idx": i, "row": r} for i, r in enumerate(rows)]}
with open(out, "w", encoding="utf-8") as f:
    json.dump(env, f)
EOF
        rc=$?
        if [ "$rc" -ne 0 ]; then
            rm -f "$raw"
            log "  FAIL   $label (raw validation failed, rc=$rc)"
            FAILED=$((FAILED + 1))
            return 1
        fi
        return 0
    fi
    rm -f "$raw"
    log "  FAIL   $label (no jq and no working python3 — the raw lane cannot convert)"
    FAILED=$((FAILED + 1))
    return 1
}

# fetch_raw_json <suite> <repo> <repo-path> <as-split> <shape: jsonl|array>
#   One /resolve/main/<path> GET, validated + converted to the envelope
#   shape. Idempotent: an existing valid envelope page is skipped (FORCE
#   refetches).
fetch_raw_json() {
    suite=$1; repo=$2; path=$3; split=$4; shape=$5
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/$split-000.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ] && rows_valid "$file"; then
        n=$(rows_count "$file" 2>/dev/null || echo 0)
        if [ "$n" -gt 0 ]; then
            log "  skip   $suite/$split-000 ($n rows)"
            return 0
        fi
    fi
    url="$RAW_BASE/datasets/$repo/resolve/main/$path"
    raw="$file.raw"
    ok=0
    if http_get "$url" "$raw"; then
        ok=1
    fi
    if [ "$ok" -eq 0 ]; then
        sleep 5
        if http_get "$url" "$raw"; then
            ok=1
        fi
    fi
    if [ "$ok" -eq 0 ]; then
        rm -f "$raw"
        log "  FAIL   $suite/$split-000 (http error, resolve $path)"
        FAILED=$((FAILED + 1))
        return 0
    fi
    if _raw_convert "$raw" "$shape" "$file.tmp" "$suite/$split-000"; then
        rm -f "$raw"
        n=$(rows_count "$file.tmp" 2>/dev/null || echo 0)
        mv "$file.tmp" "$file"
        FILES=$((FILES + 1))
        ROWS=$((ROWS + n))
        log "  fetch  $suite/$split-000 rows=$n (resolve $path)"
    fi
    sleep "$SLEEP"
}

# fetch_bfcl_join <suite> <repo> <eval-path> <gold-path> <as-split> [norm-from:to]
#   The BFCL pick-tier fetch: the eval JSONL + its official possible_answer
#   file, joined by EXACT id at fetch time (the recorded enrichment — the
#   gold lives in a separate upstream file, and the builder is pure/no-I/O,
#   so the join happens here; the manifest row records the transform and the
#   digest pins the joined OUTPUT). The join is all-or-nothing: every eval
#   row must find exactly one gold entry and every gold entry must name an
#   eval row — a mismatch fails the whole fetch loud (never a partial
#   suite). The OPTIONAL 6th arg is the ONE pinned upstream-typo
#   normalization (literal "from:to" id rewrite applied before the join):
#   BFCL_v3_live_multiple carries exactly one such pair — the eval file's
#   "live_multiple_1052-79-0" vs the gold file's "live_multiple_1052-279-0",
#   where the gold id-chain (…277, …278, …279) proves 279 is the intended
#   spelling and the eval file dropped a digit (the question/gold content
#   match is unambiguous). Any id mismatch OUTSIDE the pinned pair still
#   fails the fetch loud. jq-only (the join needs --slurpfile; without jq
#   the pass SKIPS LOUD).
fetch_bfcl_join() {
    suite=$1; repo=$2; evalpath=$3; goldpath=$4; split=$5
    normfrom=""
    normto=""
    if [ "$#" -ge 6 ] && [ -n "$6" ]; then
        normfrom=${6%%:*}
        normto=${6#*:}
    fi
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/$split-000.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ] && rows_valid "$file"; then
        n=$(rows_count "$file" 2>/dev/null || echo 0)
        if [ "$n" -gt 0 ]; then
            log "  skip   $suite/$split-000 ($n rows, gold joined)"
            return 0
        fi
    fi
    if ! have jq; then
        log "  FAIL   $suite/$split-000 (the gold join needs jq — sync data from a jq host)"
        FAILED=$((FAILED + 1))
        return 0
    fi
    evalraw="$dir/$split.eval.raw"
    goldraw="$dir/$split.gold.raw"
    eurl="$RAW_BASE/datasets/$repo/resolve/main/$evalpath"
    gurl="$RAW_BASE/datasets/$repo/resolve/main/$goldpath"
    fetch_one() { # <url> <out> <label>
        ok=0
        if http_get "$1" "$2"; then
            ok=1
        fi
        if [ "$ok" -eq 0 ]; then
            sleep 5
            if http_get "$1" "$2"; then
                ok=1
            fi
        fi
        [ "$ok" -eq 1 ] || { rm -f "$2"; log "  FAIL   $3 (http error, resolve)"; return 1; }
        return 0
    }
    ok=1
    fetch_one "$eurl" "$evalraw" "$suite/$split eval" || ok=0
    fetch_one "$gurl" "$goldraw" "$suite/$split gold" || ok=0
    if [ "$ok" -eq 0 ]; then
        rm -f "$evalraw" "$goldraw"
        FAILED=$((FAILED + 1))
        return 0
    fi
    # Gold slurp: the possible_answer files are JSONL too (one {id,
    # ground_truth} object per line).
    garr="$dir/$split.gold.json"
    jq -s -e 'length > 0 and all(.[]; type == "object" and has("id") and has("ground_truth"))' "$goldraw" > /dev/null 2>&1 || {
        rm -f "$evalraw" "$goldraw"
        log "  FAIL   $suite/$split-000 (gold validation: not non-empty JSONL {id, ground_truth} objects)"
        FAILED=$((FAILED + 1))
        return 0
    }
    jq -s . "$goldraw" > "$garr"
    tmp="$file.tmp"
    if [ -n "$normfrom" ]; then
        log "  norm   $suite/$split-000: $normfrom -> $normto (the pinned upstream-typo normalization)"
        jq -c --arg from "$normfrom" --arg to "$normto" \
            'if .id == $from then .id = $to else . end' "$evalraw" > "$evalraw.norm"
        mv "$evalraw.norm" "$evalraw"
    fi
    jq -s --slurpfile g "$garr" '
        ($g[0] | map({key: .id, value: .ground_truth}) | from_entries) as $gmap
        | {joined: [.[] | select($gmap[.id] != null) | . + {ground_truth: $gmap[.id]}],
           missing: [.[] | select($gmap[.id] == null) | .id],
           extra: (($g[0] | map(.id)) - [.[] | .id])}' "$evalraw" > "$tmp" 2>/dev/null \
        || {
        rm -f "$evalraw" "$goldraw" "$garr" "$tmp"
        log "  FAIL   $suite/$split-000 (join jq failed)"
        FAILED=$((FAILED + 1))
        return 0
    }
    m=$(jq -r '.missing | length' "$tmp")
    x=$(jq -r '.extra | length' "$tmp")
    if [ "$m" -gt 0 ] || [ "$x" -gt 0 ]; then
        rm -f "$evalraw" "$goldraw" "$garr" "$tmp"
        log "  FAIL   $suite/$split-000 (id-set mismatch: $m eval row(s) without gold, $x gold row(s) without an eval row)"
        FAILED=$((FAILED + 1))
        return 0
    fi
    jq '{num_rows_total: (.joined | length), rows: (.joined | to_entries | map({row_idx: .key, row: .value}))}' "$tmp" > "$tmp.env"
    rm -f "$evalraw" "$goldraw" "$garr" "$tmp"
    n=$(rows_count "$tmp.env" 2>/dev/null || echo 0)
    mv "$tmp.env" "$file"
    FILES=$((FILES + 1))
    ROWS=$((ROWS + n))
    log "  fetch  $suite/$split-000 rows=$n (resolve $evalpath + gold join)"
    sleep "$SLEEP"
}

# _pairs_flatten <in-json> <out-json> <split>
#   The ContractNLI recorded transform (issue 114 T9): the authors' nested
#   per-document JSON (documents[].annotation_sets[].annotations keyed by
#   hypothesis id + a top-level labels map carrying the hypothesis texts)
#   flattens to ONE pair row per (document, hypothesis) annotation:
#   {contract, hypothesis, choice, split}. Emits a JSON ARRAY (the
#   _raw_convert input shape). python3-only (the flatten is a real
#   program, not a jq one-liner — the script's PY3 resolution gates it).
_pairs_flatten() {
    in=$1; out=$2; split=$3
    # PY3 is only pre-resolved on jq-less boxes; the flatten is a real
    # program (python3-only), so resolve it here when empty.
    if [ -z "$PY3" ]; then
        if have python3 && python3 -c pass >/dev/null 2>&1; then
            PY3="python3"
        elif have py && py -3 -c pass >/dev/null 2>&1; then
            PY3="py -3"
        fi
    fi
    if [ -z "$PY3" ]; then
        log "  FAIL   pairs flatten (no working python3 — the ContractNLI lane cannot convert)"
        FAILED=$((FAILED + 1))
        return 1
    fi
    $PY3 - "$in" "$out" "$split" <<'PYEOF'
import json, sys
src, dst, split = sys.argv[1], sys.argv[2], sys.argv[3]
with open(src, encoding="utf-8") as f:
    d = json.load(f)
labels = d["labels"]
rows = []
for doc in d["documents"]:
    for s in doc.get("annotation_sets", []):
        for hyp_id, a in s.get("annotations", {}).items():
            rows.append({
                "contract": doc["text"],
                "hypothesis": labels[hyp_id]["hypothesis"],
                "choice": a["choice"],
                "split": split,
            })
with open(dst, "w", encoding="utf-8") as f:
    json.dump(rows, f, ensure_ascii=False)
print(f"pairs={len(rows)}")
PYEOF
}

# fetch_contractnli_pairs <suite> <owner/repo> <sha> <zip-path>
#   The ContractNLI fetch (issue 114 T9): ONE zip from the authors'
#   canonical GitHub repo at the board's pinned sha (raw.githubusercontent
#   — the hub /resolve lane does not serve GitHub), extracted to both
#   split envelopes via the recorded flatten transform. The zip downloads
#   ONCE (65 MB) and is reused for both splits; each envelope is
#   idempotent (skip on a valid cached page, FORCE refetches).
fetch_contractnli_pairs() {
    suite=$1; owner_repo=$2; sha=$3; zippath=$4
    dir="$OUT/$suite"
    mkdir -p "$dir"
    zipf="$dir/src.zip"
    need_zip=0
    for split in test train; do
        file="$dir/$split-000.json"
        if [ "$FORCE" = "1" ] || ! [ -s "$file" ] || ! rows_valid "$file"; then
            need_zip=1
        fi
    done
    if [ "$need_zip" = "0" ]; then
        for split in test train; do
            n=$(rows_count "$dir/$split-000.json" 2>/dev/null || echo 0)
            log "  skip   $suite/$split-000 ($n rows)"
        done
        return 0
    fi
    if ! [ -s "$zipf" ] || [ "$FORCE" = "1" ]; then
        url="https://raw.githubusercontent.com/$owner_repo/$sha/$zippath"
        ok=0
        if http_get "$url" "$zipf.tmp"; then ok=1; fi
        if [ "$ok" -eq 0 ]; then
            sleep 5
            if http_get "$url" "$zipf.tmp"; then ok=1; fi
        fi
        if [ "$ok" -eq 0 ]; then
            rm -f "$zipf.tmp"
            log "  FAIL   $suite/src.zip (http error, $url)"
            FAILED=$((FAILED + 1))
            return 0
        fi
        mv "$zipf.tmp" "$zipf"
        log "  fetch  $suite/src.zip ($(wc -c < "$zipf" | tr -d ' ') bytes)"
    fi
    for split in test train; do
        file="$dir/$split-000.json"
        if [ "$FORCE" != "1" ] && [ -s "$file" ] && rows_valid "$file"; then
            n=$(rows_count "$file" 2>/dev/null || echo 0)
            log "  skip   $suite/$split-000 ($n rows)"
            continue
        fi
        inner="contract-nli/$split.json"
        raw="$dir/$split.raw.json"
        if ! unzip -p "$zipf" "$inner" > "$raw" 2>/dev/null || ! [ -s "$raw" ]; then
            rm -f "$raw"
            log "  FAIL   $suite/$split-000 (unzip $inner)"
            FAILED=$((FAILED + 1))
            continue
        fi
        pairs="$dir/$split.pairs.json"
        if ! _pairs_flatten "$raw" "$pairs" "$split"; then
            rm -f "$raw" "$pairs"
            continue
        fi
        rm -f "$raw"
        if _raw_convert "$pairs" "array" "$file.tmp" "$suite/$split-000"; then
            rm -f "$pairs"
            n=$(rows_count "$file.tmp" 2>/dev/null || echo 0)
            mv "$file.tmp" "$file"
            FILES=$((FILES + 1))
            ROWS=$((ROWS + n))
            log "  fetch  $suite/$split-000 rows=$n (zip $inner, pairs flatten)"
        else
            rm -f "$pairs"
        fi
        sleep "$SLEEP"
    done
    # The zip stays on disk (the manifest pins its blake3; a re-extract
    # needs no second download).
}

# _csv_a_rows <in-csv> <out-json> <split>
#   The iSarcasmEval recorded transform (issue 114 T10): the task-A CSV
#   (test: `text,sarcastic`; train: unnamed-index,tweet,sarcastic,rephrase,
#   +6 category columns) projects to row dicts {text, sarcastic, split} —
#   the A-track binary pair only (the sarcastic/rephrase columns; the
#   category columns are track B's annotations, out of this suite's
#   scope). Emits the {"rows": [...]} envelope directly (python3-only —
#   a real CSV program: tweets carry embedded commas/newlines, so naive
#   splitting is wrong).
_csv_a_rows() {
    in=$1; out=$2; split=$3
    if [ -z "$PY3" ]; then
        if have python3 && python3 -c pass >/dev/null 2>&1; then
            PY3="python3"
        elif have py && py -3 -c pass >/dev/null 2>&1; then
            PY3="py -3"
        fi
    fi
    if [ -z "$PY3" ]; then
        log "  FAIL   csv a-rows (no working python3 — the isarcasmeval lane cannot convert)"
        FAILED=$((FAILED + 1))
        return 1
    fi
    $PY3 - "$in" "$out" "$split" <<'PYEOF'
import csv, json, sys
src, dst, split = sys.argv[1], sys.argv[2], sys.argv[3]
with open(src, encoding="utf-8", newline="") as f:
    reader = csv.DictReader(f)
    rows = []
    for r in reader:
        text = (r.get("text") or r.get("tweet") or "").strip()
        label = (r.get("sarcastic") or "").strip()
        if not text or label not in ("0", "1"):
            continue
        rows.append({"row": {"text": text, "sarcastic": label, "split": split}})
with open(dst, "w", encoding="utf-8") as f:
    json.dump({"rows": rows}, f, ensure_ascii=False)
print(f"rows={len(rows)}")
PYEOF
}

# _csv_b_rows <in-csv> <out-json> <split>
#   The iSarcasmEval track-B recorded transform (issue 114 T13): the
#   task-B CSV — text + the six 0/1 technique columns (sarcasm, irony,
#   satire, understatement, overstatement, rhetorical_question) —
#   projects to row dicts carrying the text, all six columns AS STRINGS
#   (the _csv_a_rows convention row_str reads), and the split tag. ALL
#   1,400 rows go through faithfully: the all-zero technique rows are
#   the track-A negative population and the BUILDER's carrier filter
#   drops them counted at the pinned census — the fetch stays
#   mechanical-from-fetched, never pre-filtered.
_csv_b_rows() {
    in=$1; out=$2; split=$3
    if [ -z "$PY3" ]; then
        if have python3 && python3 -c pass >/dev/null 2>&1; then
            PY3="python3"
        elif have py && py -3 -c pass >/dev/null 2>&1; then
            PY3="py -3"
        fi
    fi
    if [ -z "$PY3" ]; then
        log "  FAIL   csv b-rows (no working python3 — the isarcasm_b lane cannot convert)"
        FAILED=$((FAILED + 1))
        return 1
    fi
    $PY3 - "$in" "$out" "$split" <<'PYEOF'
import csv, json, sys
src, dst, split = sys.argv[1], sys.argv[2], sys.argv[3]
TECHNIQUES = ["sarcasm", "irony", "satire", "understatement",
              "overstatement", "rhetorical_question"]
with open(src, encoding="utf-8", newline="") as f:
    reader = csv.DictReader(f)
    rows = []
    for r in reader:
        text = (r.get("text") or "").strip()
        if not text:
            continue
        row = {"text": text}
        for c in TECHNIQUES:
            v = (r.get(c) or "").strip()
            if v not in ("0", "1"):
                print(f"bad technique column {c}={v!r}", file=sys.stderr)
                sys.exit(1)
            row[c] = v
        row["split"] = split
        rows.append({"row": row})
with open(dst, "w", encoding="utf-8") as f:
    json.dump({"rows": rows}, f, ensure_ascii=False)
print(f"rows={len(rows)}")
PYEOF
}

# fetch_isarcasmeval <suite> <owner_repo> <sha>
#   The iSarcasmEval fetch (issue 114 T10): the task-A English CSVs at the
#   board's pinned sha via the GitHub raw lane (the hub /resolve does not
#   serve GitHub; the board's suite.md pins `iabufarha/iSarcasmEval
#   @dfc708b, test/task_{A,B,C}_{En,Ar}_test.csv`). This lane takes the
#   TRACK-A ENGLISH files only (test/task_A_En_test.csv = the board row's
#   index metric population; train/train.En.csv = the corpus-ON pool) —
#   the B/C/Arabic tracks are the recorded re-opens (issue 114 T10; the
#   triage's English-first posture decision, never a silent drop). Each
#   envelope is idempotent (skip on a valid cached page, FORCE refetches).
#   Issue 114 T13: the lane ALSO takes the track-B test file
#   (test/task_B_En_test.csv → the `test_b` split, the multilabel
#   technique columns; the same pinned sha the T10 MIT read covers) when
#   the caller names isarcasm_b — the B/C/Arabic re-open note narrows:
#   track B is LANED, C/Arabic stay the recorded re-opens.
fetch_isarcasmeval() {
    suite=$1; owner_repo=$2; sha=$3
    dir="$OUT/$suite"
    mkdir -p "$dir"
    for spec in "test:test/task_A_En_test.csv" "train:train/train.En.csv" \
                "test_b:test/task_B_En_test.csv:_csv_b_rows"; do
        split=${spec%%:*}; rest=${spec#*:}
        path=${rest%%:*}
        transform=${rest#*:}
        [ "$transform" = "$path" ] && transform=_csv_a_rows
        file="$dir/$split-000.json"
        if [ "$FORCE" != "1" ] && [ -s "$file" ] && rows_valid "$file"; then
            n=$(rows_count "$file" 2>/dev/null || echo 0)
            if [ "$n" -gt 0 ]; then
                log "  skip   $suite/$split-000 ($n rows)"
                continue
            fi
        fi
        url="https://raw.githubusercontent.com/$owner_repo/$sha/$path"
        raw="$file.raw"
        ok=0
        if http_get "$url" "$raw"; then ok=1; fi
        if [ "$ok" -eq 0 ]; then
            sleep 5
            if http_get "$url" "$raw"; then ok=1; fi
        fi
        if [ "$ok" -eq 0 ]; then
            rm -f "$raw"
            log "  FAIL   $suite/$split-000 (http error, $url)"
            FAILED=$((FAILED + 1))
            continue
        fi
        if "$transform" "$raw" "$file.tmp" "$split"; then
            rm -f "$raw"
            n=$(rows_count "$file.tmp" 2>/dev/null || echo 0)
            mv "$file.tmp" "$file"
            FILES=$((FILES + 1))
            ROWS=$((ROWS + n))
            log "  fetch  $suite/$split-000 rows=$n (raw.githubusercontent $path)"
        else
            rm -f "$raw" "$file.tmp"
        fi
        sleep "$SLEEP"
    done
}

# fetch_home_appliance <suite> <clone>
#   Issue 114 T11 (the JDI ladder wave-2 machinery's first consumer; board
#   row 9, the home-appliance simulator — MIT, board-generated). The
#   board's hub rows file is PRIVATE (the HF lane refuses anonymous
#   reads), so this lane regenerates the rows with the BOARD'S OWN
#   generator (`decision_index/suite/build/home_appliance.py`, stdlib-only
#   and deterministic at seed 2026091807) and filters to the release-v2.1
#   kept set — the 88 run_ids the subset file pins, verified against the
#   file's own `run_ids_sha256` (sha256 of the newline-joined list;
#   verified matching at the landing). Regeneration proofs at the landing:
#   240 rows (160 test / 80 dev), the duplicate-state census 24 = the
#   subset's `duplicate_rows_dropped`, kept∩test = 88/88, kept∩dev-state
#   collisions 0, 1,590 questions over the kept rows. <clone> is the
#   board's decision-index checkout (default
#   .raw/mammoth-local/decision-index; its commit sha is the generator's
#   provenance pin). Envelope = one row per kept case, the board's
#   `id`/`state`/`questions`/`expected` carried verbatim (plus
#   `split: "test"` + `group_id: <id>` — each request is its own case).
#   Idempotent like every lane (skip on a valid cached page, FORCE
#   refetches).
fetch_home_appliance() {
    suite=$1; clone=$2
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/test-000.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ] && rows_valid "$file"; then
        n=$(rows_count "$file" 2>/dev/null || echo 0)
        if [ "$n" -gt 0 ]; then
            log "  skip   $suite/test-000 ($n rows)"
            return 0
        fi
    fi
    [ -d "$clone" ] || {
        log "  FAIL   $suite (the board's decision-index clone is missing at $clone — clone it or set HOME_APPLIANCE_CLONE)"
        FAILED=$((FAILED + 1))
        return 1
    }
    if [ -z "$PY3" ]; then
        if have python3 && python3 -c pass >/dev/null 2>&1; then
            PY3="python3"
        elif have py && py -3 -c pass >/dev/null 2>&1; then
            PY3="py -3"
        fi
    fi
    if [ -z "$PY3" ]; then
        log "  FAIL   $suite (no working python3 — the generator lane cannot run)"
        FAILED=$((FAILED + 1))
        return 1
    fi
    if $PY3 - "$clone" "$file.tmp" <<'PYEOF'
import hashlib, importlib.util, json, sys
from pathlib import Path

clone, dst = Path(sys.argv[1]), sys.argv[2]
sub_path = clone / "decision_index/data/release-v2.1/home-appliances-subset.json"
gen_path = clone / "decision_index/suite/build/home_appliance.py"
sub = json.loads(sub_path.read_text(encoding="utf-8"))
ids = sub["run_ids"]
# The kept set is cryptographically pinned: sha256 of the newline-joined
# run_id list (verified against the subset file's own pin — a drift here
# means the clone's subset file changed under us; refuse loud).
pin = hashlib.sha256("\n".join(ids).encode()).hexdigest()
if pin != sub["run_ids_sha256"]:
    sys.exit(f"subset run_ids_sha256 drift: computed {pin} != pinned {sub['run_ids_sha256']}")
spec_ = importlib.util.spec_from_file_location("home_appliance_gen", gen_path)
mod = importlib.util.module_from_spec(spec_)
spec_.loader.exec_module(mod)
rows = [mod.make_row(h, i) for h in range(len(mod.HOUSEHOLDS)) for i in range(10)]
test = [r for r in rows if r["split"] == "test"]
dev = [r for r in rows if r["split"] != "test"]
# Regeneration proofs — the census the builder's population pin rests on.
if len(rows) != 240 or len(test) != 160 or len(dev) != 80:
    sys.exit(f"generator census drift: {len(rows)} rows / {len(test)} test / {len(dev)} dev")

def canon(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False)
dup = len(test) - len({canon({k: v for k, v in r["state"].items()}) for r in test})
if dup != sub["duplicate_rows_dropped"]:
    sys.exit(f"duplicate-state census drift: {dup} != pinned {sub['duplicate_rows_dropped']}")
dev_states = {canon({k: v for k, v in r["state"].items()}) for r in dev}
by_id = {r["id"]: r for r in test}
kept_ids = [rid[len("9:Home-Appliance:"):] for rid in ids if rid.startswith("9:Home-Appliance:")]
missing = [i for i in kept_ids if i not in by_id]
if missing:
    sys.exit(f"{len(missing)} kept run_ids regenerated no test row (first: {missing[0]})")
collisions = [i for i in kept_ids if canon(by_id[i]["state"]) in dev_states]
if collisions:
    sys.exit(f"{len(collisions)} kept rows collide with a dev state (first: {collisions[0]})")
qcount = sum(len(by_id[i]["questions"]) for i in kept_ids)
if qcount != 1590:
    sys.exit(f"kept question census drift: {qcount} != pinned 1590")
out = []
for rid in kept_ids:
    r = by_id[rid]
    out.append({"row": {
        "id": r["id"],
        "group_id": r["id"],  # each request is its own case (the board's group grammar)
        "split": "test",
        "state": r["state"],
        "questions": r["questions"],
        "expected": r["expected"],
    }})
Path(dst).write_text(json.dumps({"rows": out}, ensure_ascii=False), encoding="utf-8")
print(f"rows={len(out)} questions={qcount} pin={pin[:12]}")
PYEOF
    then
        n=$(rows_count "$file.tmp" 2>/dev/null || echo 0)
        mv "$file.tmp" "$file"
        FILES=$((FILES + 1))
        ROWS=$((ROWS + n))
        log "  fetch  $suite/test-000 rows=$n (generator @ clone $(git -C "$clone" rev-parse --short HEAD 2>/dev/null || echo '?') + the pinned kept set)"
    else
        rm -f "$file.tmp"
        log "  FAIL   $suite/test-000 (the generator/verify lane refused — see the error above)"
        FAILED=$((FAILED + 1))
    fi
}

main() {
    if have b3sum; then
        digest_tool="b3sum (blake3)"
    else
        digest_tool="shasum -a 256 (sha256 fallback)"
    fi
    log "fetch_datasets.sh: out=$OUT page=$PAGE force=$FORCE"
    log "digest tool seen on this box (used by the manifest): $digest_tool"
    mkdir -p "$OUT"

    log ""
    log "fetch_datasets.sh: out=$OUT page=$PAGE force=$FORCE suites=$SUITES"
    log "digest tool seen on this box (used by the manifest): $digest_tool"
    mkdir -p "$OUT"

    if want_suite typed_decisions; then
        log "[probe] typed_decisions /splits"
        probe_splits typed_decisions "LocalLLaMA%2Ftyped-decisions"
    fi
    if want_suite massive_intent_en; then
        log "[probe] massive_intent_en /splits"
        probe_splits massive_intent_en "mteb%2Famazon_massive_intent"
    fi
    if want_suite prompt_injections; then
        log "[probe] prompt_injections /size"
        probe_size prompt_injections "deepset%2Fprompt-injections"
    fi
    if want_suite thai_wisesight; then
        log "[probe] thai_wisesight /splits"
        probe_splits thai_wisesight "pythainlp%2Fwisesight_sentiment"
    fi
    if want_suite thai_sib200; then
        log "[probe] thai_sib200 /splits"
        probe_splits thai_sib200 "Davlan%2Fsib200"
    fi
    if want_suite wanli_en; then
        log "[probe] wanli_en /splits"
        probe_splits wanli_en "alisawuffles%2FWANLI"
    fi
    if want_suite winogrande_jdi; then
        log "[probe] winogrande_jdi /splits"
        probe_splits winogrande_jdi "allenai%2Fwinogrande"
    fi
    if want_suite hellaswag_jdi; then
        log "[probe] hellaswag_jdi /splits"
        probe_splits hellaswag_jdi "Rowan%2Fhellaswag"
    fi
    if want_suite anli_jdi; then
        log "[probe] anli_jdi /splits"
        probe_splits anli_jdi "facebook%2Fanli"
    fi
    if want_suite mmlupro_jdi; then
        log "[probe] mmlupro_jdi /splits"
        probe_splits mmlupro_jdi "TIGER-Lab%2FMMLU-Pro"
    fi
    # Issue 114 T2 (the JDI ladder wave 1): CLINC150 — the canonical
    # clinc/clinc_oos is parquet-backed now (the old script-dataset 404 is
    # gone; the bare `clinc_oos` name 404s with "renamed"). Config `plus`
    # (the balanced 100/intent train variant — the corpus pool wants every
    # intent represented; test is identical across configs).
    if want_suite clinc150; then
        log "[probe] clinc150 /splits"
        probe_splits clinc150 "clinc%2Fclinc_oos"
        log "[probe] clinc150 /size"
        probe_size clinc150 "clinc%2Fclinc_oos"
    fi

    # Licence probes (riir-rethink Issue 023 T1): one hub call per selected
    # suite — the drift tripwire beside the manifest's VERIFIED table.
    # NOTE: the hub API takes the dataset id with a LITERAL slash (unlike
    # the datasets-server endpoints, which take %2F).
    if want_suite typed_decisions; then
        probe_license typed_decisions "LocalLLaMA/typed-decisions"
    fi
    if want_suite ag_news; then
        probe_license ag_news "fancyzhx/ag_news"
    fi
    if want_suite emotion; then
        probe_license emotion "dair-ai/emotion"
    fi
    if want_suite sst5; then
        probe_license sst5 "SetFit/sst5"
    fi
    if want_suite banking77; then
        probe_license banking77 "mteb/banking77"
    fi
    if want_suite prompt_injections; then
        probe_license prompt_injections "deepset/prompt-injections"
    fi
    if want_suite massive_intent_en; then
        probe_license massive_intent_en "mteb/amazon_massive_intent"
    fi
    if want_suite xnli_en; then
        probe_license xnli_en "facebook/xnli"
    fi
    if want_suite thai_wisesight; then
        probe_license thai_wisesight "pythainlp/wisesight_sentiment"
    fi
    if want_suite thai_sib200; then
        probe_license thai_sib200 "Davlan/sib200"
    fi
    if want_suite wanli_en; then
        probe_license wanli_en "alisawuffles/WANLI"
    fi
    # Plan 012 (the JDI panel tranche 1): the four board rows' licence
    # probes — the hub-side tripwire beside the manifest's VERIFIED rows.
    if want_suite winogrande_jdi; then
        probe_license winogrande_jdi "allenai/winogrande"
    fi
    if want_suite hellaswag_jdi; then
        probe_license hellaswag_jdi "Rowan/hellaswag"
    fi
    if want_suite anli_jdi; then
        probe_license anli_jdi "facebook/anli"
    fi
    if want_suite mmlupro_jdi; then
        probe_license mmlupro_jdi "TIGER-Lab/MMLU-Pro"
    fi
    # Issue 114 T2: the hub-side licence tripwire for the ladder's CLINC150
    # row (cc-by-3.0; the manifest row stays authoritative — the card's own
    # prose Licensing section is an unfilled stub, resolved by the structured
    # metadata + the canonical clinc org ownership, never guessed).
    if want_suite clinc150; then
        probe_license clinc150 "clinc/clinc_oos"
    fi
    # Issue 114 T3: the hub-side licence tripwire for the ladder's When2Call
    # row (cc-by-4.0, the card's Licensing section named verbatim — NVIDIA,
    # commercial-use declared; the manifest row stays authoritative).
    if want_suite when2call; then
        probe_license when2call "nvidia/When2Call"
    fi

    # 1. typed_decisions — test: ALL rows; train: ALL 1200 rows (Issue 052:
    #    the birth cap 800 stopped inside the security_incidents train block
    #    (offsets 900–1199) + 100 invoice rows; the extended pool measures
    #    A0' 0.5725 vs the capped A0 0.4655 at the published posture, Bench
    #    078. The cal slice is the STRATIFIED round-robin front over the whole
    #    split (Issue 039 T2) — it re-derives on the wider pool; the test
    #    split is untouched and the published row is re-measured).
    if want_suite typed_decisions; then
        log "[suite] typed_decisions LocalLLaMA/typed-decisions config=all split=test cap=all"
        fetch_suite typed_decisions "LocalLLaMA%2Ftyped-decisions" all test all
        if splits_have typed_decisions all train; then
            log "[suite] typed_decisions config=all split=train cap=1200 (train present per probe)"
            fetch_suite typed_decisions "LocalLLaMA%2Ftyped-decisions" all train 1200
            dedupe_train typed_decisions
        else
            log "[suite] typed_decisions config=all split=train SKIPPED (no train split for config all, or probe missing)"
        fi
    fi

    # 2. ag_news
    if want_suite ag_news; then
        log "[suite] ag_news fancyzhx/ag_news config=default split=test cap=400"
        fetch_suite ag_news "fancyzhx%2Fag_news" default test 400
        log "[suite] ag_news config=default split=train cap=$TRAIN_CAP"
        fetch_suite ag_news "fancyzhx%2Fag_news" default train "$TRAIN_CAP"
        dedupe_train ag_news
    fi

    # 3. emotion
    if want_suite emotion; then
        log "[suite] emotion dair-ai/emotion config=split split=test cap=400"
        fetch_suite emotion "dair-ai%2Femotion" split test 400
        log "[suite] emotion config=split split=train cap=4000"
        fetch_suite emotion "dair-ai%2Femotion" split train "$TRAIN_CAP"
        dedupe_train emotion
    fi

    # 4. sst5
    if want_suite sst5; then
        log "[suite] sst5 SetFit/sst5 config=default split=test cap=600"
        fetch_suite sst5 "SetFit%2Fsst5" default test 600
        log "[suite] sst5 config=default split=train cap=4000"
        fetch_suite sst5 "SetFit%2Fsst5" default train "$TRAIN_CAP"
        dedupe_train sst5
    fi

    # 5. banking77 — currently 404s on /rows (script-based dataset; see
    #    header). Attempted so every run records the gap honestly.
# banking77 gap resolution: PolyAI/banking77 is a script-based dataset that
# datasets-server cannot serve (/rows 404). The reference itself has a second
# variant on the parquet-backed mirror mteb/banking77 (bench_apps protocol:
# labels = sorted unique label_text, gold = key position) — fetch THAT.
    if want_suite banking77; then
        log "[suite] banking77 mteb/banking77 config=default split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite banking77 "mteb%2Fbanking77" default test all
        log "[suite] banking77 mteb/banking77 config=default split=train cap=4000"
        fetch_suite banking77 "mteb%2Fbanking77" default train "$TRAIN_CAP"
        dedupe_train banking77
    fi

    # 6. prompt_injections — test: ALL (~116); train: first 1000 (actual
    #    size 546 per /size probe, so the cap exhausts the split).
    if want_suite prompt_injections; then
        log "[suite] prompt_injections deepset/prompt-injections config=default split=test cap=all"
        fetch_suite prompt_injections "deepset%2Fprompt-injections" default test all
        log "[suite] prompt_injections config=default split=train cap=1000 (actual 546 per probe)"
        fetch_suite prompt_injections "deepset%2Fprompt-injections" default train 1000
        dedupe_train prompt_injections
    fi

    # 7. massive_intent_en — config verified via /splits: the configs are
    #    bare locale codes; the English one is "en" (not "en-US").
    if want_suite massive_intent_en; then
        log "[suite] massive_intent_en mteb/amazon_massive_intent config=en split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite massive_intent_en "mteb%2Famazon_massive_intent" en test all
        log "[suite] massive_intent_en config=en split=train cap=4000"
        fetch_suite massive_intent_en "mteb%2Famazon_massive_intent" en train "$TRAIN_CAP"
        dedupe_train massive_intent_en
    fi

    # 8. xnli_en
    if want_suite xnli_en; then
        log "[suite] xnli_en facebook/xnli config=en split=test cap=300"
        fetch_suite xnli_en "facebook%2Fxnli" en test 300
        log "[suite] xnli_en config=en split=train cap=$TRAIN_CAP"
        fetch_suite xnli_en "facebook%2Fxnli" en train "$TRAIN_CAP"
        dedupe_train xnli_en
        # Issue 047 R1: the M1 reopen lane's confirmation surface — the
        # full validation split (~2490 rows), fetched once into the SAME
        # suite dir and read ONCE by `--nli-m1 --suites xnli_en_val`.
        log "[suite] xnli_en config=en split=validation cap=all (issue 047 confirmation surface)"
        fetch_suite xnli_en "facebook%2Fxnli" en validation all
    fi

    # 9. thai_wisesight — Plan 003 T3.1: the OpenThai board's WEAKEST
    #    published set (51.6 / ECE 0.353 — chosen for honesty, not
    #    cherry-picking). 4-class sentiment choice; the ClassLabel order
    #    pos/neu/neg/q is verified at fetch and pinned in
    #    dataset_manifest.md (the ag_news VERIFY-AT-PORT law).
    if want_suite thai_wisesight; then
        log "[suite] thai_wisesight pythainlp/wisesight_sentiment config=wisesight_sentiment split=test cap=400"
        fetch_suite thai_wisesight "pythainlp%2Fwisesight_sentiment" wisesight_sentiment test 400
        log "[suite] thai_wisesight config=wisesight_sentiment split=train cap=$TRAIN_CAP"
        fetch_suite thai_wisesight "pythainlp%2Fwisesight_sentiment" wisesight_sentiment train "$TRAIN_CAP"
        dedupe_train thai_wisesight
    fi

    # 10. thai_sib200 — Plan 003 T3.1: the SIB-200 Thai topical probe
    #     (7-way topic), test + train fetched WHOLE (204 / 701 rows).
    if want_suite thai_sib200; then
        log "[suite] thai_sib200 Davlan/sib200 config=tha_Thai split=train cap=all (701 rows whole-set)"
        fetch_suite thai_sib200 "Davlan%2Fsib200" tha_Thai train all
        dedupe_train thai_sib200
    fi

    # 11. wanli_en — the ESC re-source NLI lane (riir-rethink Issue 024 /
    #     Research 002; reflex issue 080): WANLI re-enters the xnli_en niche
    #     at the same 3-choice wire (premise/hypothesis/gold, gold a STRING
    #     in entailment/neutral/contradiction — no ClassLabel order to
    #     verify at the port, the builder maps the string through XNLI_KEYS).
    #     Test fetched WHOLE (5,000 rows; the eval cap lives in the harness,
    #     the banking77 posture), train capped at $TRAIN_CAP (the slice
    #     floor 16_000 sits on this pull).
    if want_suite wanli_en; then
        log "[suite] wanli_en alisawuffles/WANLI config=default split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite wanli_en "alisawuffles%2FWANLI" default test all
        log "[suite] wanli_en config=default split=train cap=$TRAIN_CAP"
        fetch_suite wanli_en "alisawuffles%2FWANLI" default train "$TRAIN_CAP"
        dedupe_train wanli_en
    fi

    # 12. winogrande_jdi — Plan 012 A1 (the JDI panel tranche 1): the
    #     pronoun-resolution board row (validation 1267 = the board's cases;
    #     HF `test` is unlabeled). Train fetched for the corpus pool.
    if want_suite winogrande_jdi; then
        log "[suite] winogrande_jdi allenai/winogrande config=winogrande_debiased split=validation cap=all (the board's population; eval cap lives in the harness)"
        fetch_suite winogrande_jdi "allenai%2Fwinogrande" winogrande_debiased validation all
        log "[suite] winogrande_jdi config=winogrande_debiased split=train cap=4000 (the corpus pool)"
        fetch_suite winogrande_jdi "allenai%2Fwinogrande" winogrande_debiased train 4000
    fi

    # 13. hellaswag_jdi — Plan 012 A2: the 4-way continuation board row
    #     (validation 10042). Eval universe fetched at 8000 (the eval cap
    #     lives in the harness — the banking77 posture); train at the same
    #     bound (the corpus pool needs no more).
    if want_suite hellaswag_jdi; then
        log "[suite] hellaswag_jdi Rowan/hellaswag config=default split=validation cap=8000 (eval universe; eval cap lives in the harness)"
        fetch_suite hellaswag_jdi "Rowan%2Fhellaswag" default validation 8000
        log "[suite] hellaswag_jdi config=default split=train cap=8000 (the corpus pool)"
        fetch_suite hellaswag_jdi "Rowan%2Fhellaswag" default train 8000
    fi

    # 14. anli_jdi — Plan 012 A3: the NLI board row = test_r1+test_r2+test_r3
    #     concatenated in that order (3200 = the board's cases); the harness
    #     reads the comma-list eval split (load_rows). Train rounds: train_r1
    #     ONLY (r2/r3 trains are huge; one round's pool is enough for the
    #     corpus and keeps the fetch bounded) — stored as the suite's `train`.
    if want_suite anli_jdi; then
        log "[suite] anli_jdi facebook/anli config=plain_text split=test_r1 cap=all (1000 rows)"
        fetch_suite anli_jdi "facebook%2Fanli" plain_text test_r1 all
        log "[suite] anli_jdi config=plain_text split=test_r2 cap=all (1000 rows)"
        fetch_suite anli_jdi "facebook%2Fanli" plain_text test_r2 all
        log "[suite] anli_jdi config=plain_text split=test_r3 cap=all (1200 rows)"
        fetch_suite anli_jdi "facebook%2Fanli" plain_text test_r3 all
        log "[suite] anli_jdi config=plain_text split=train_r1 cap=4000 (round-1 pool only; stored as the suite's train)"
        fetch_suite anli_jdi "facebook%2Fanli" plain_text train_r1 4000 train
    fi

    # 15. mmlupro_jdi — Plan 012 A4: the knowledge board row (test 12032;
    #     universe rows, the eval cap lives in the harness). NO train split
    #     exists — the 70-row validation split (the few-shot prompt pool) is
    #     fetched AS the suite's `train` (the corpus/cal pool; the starved
    #     corpus is disclosed in the registry + the manifest).
    if want_suite mmlupro_jdi; then
        log "[suite] mmlupro_jdi TIGER-Lab/MMLU-Pro config=default split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite mmlupro_jdi "TIGER-Lab%2FMMLU-Pro" default test all
        log "[suite] mmlupro_jdi config=default split=validation cap=all (70 rows — no train split exists; stored AS the suite's train = the corpus/cal pool)"
        fetch_suite mmlupro_jdi "TIGER-Lab%2FMMLU-Pro" default validation all train
    fi

    # 16. clinc150 — Issue 114 T2 (the JDI ladder wave 1): clinc/clinc_oos
    #     config=plus. test fetched WHOLE (5,500 rows = the option universe,
    #     the banking77 posture — eval cap 500 lives in the harness, the
    #     slice stratified over the 151 labels); train fetched WHOLE (15,250
    #     rows — the plus split is intent-ORDERED, so a capped prefix would
    #     orphan ~2/3 of the intents and the corpus guard would starve them;
    #     corpus_cap_per_label 40 spans 151 labels with a remainder). oos
    #     rides as the regular 151st option (the board row is 150+OOS).
    if want_suite clinc150; then
        log "[suite] clinc150 clinc/clinc_oos config=plus split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite clinc150 "clinc%2Fclinc_oos" plus test all
        log "[suite] clinc150 config=plus split=train cap=all (intent-ordered split — fetched whole so the corpus pool spans all 151 intents)"
        fetch_suite clinc150 "clinc%2Fclinc_oos" plus train all
        dedupe_train clinc150
    fi

    # 17. when2call — Issue 114 T3 (the JDI ladder wave 1): nvidia/When2Call
    #     config=test, split=mcq (3,652 rows — the board's population, fetched
    #     WHOLE; stored AS the suite's `test` — the mmlupro as-split precedent,
    #     reversed: the fetched mcq split IS the eval surface). CORPUS-OPTIONAL
    #     suite: NO train split is fetched, ever — the llm_judge split is a
    #     documented subset of the mcq eval data (uuid overlap measured 11/100;
    #     the slice-integrity law refuses it as a pool), train_pref/train_sft
    #     carry no class label (the marker covers tool_call only — deriving the
    #     other three classes is the no-silent-transforms refusal), and the mcq
    #     rows' answers objects are TEST rows (grammar-not-answers). The corpus
    #     re-open paths (incl. the xlam-60k synthesis route) live in the issue;
    #     this stanza fetches the eval surface only.
    if want_suite when2call; then
        log "[probe] when2call /splits"
        probe_splits when2call "nvidia%2FWhen2Call"
        log "[probe] when2call /size"
        probe_size when2call "nvidia%2FWhen2Call"
    fi
    if want_suite when2call; then
        log "[suite] when2call nvidia/When2Call config=test split=mcq cap=all (the board population; stored as the suite's test split)"
        fetch_suite when2call "nvidia%2FWhen2Call" test mcq all test
    fi

    # 18. bfcl — Issue 114 T4 (the JDI ladder wave 1): the gorilla-llm
    #     Berkeley-Function-Calling-Leaderboard mirror, wave-1 scope = the
    #     pick tier (simple/multiple + their live twins — the board row's
    #     "pick the function to call") + the irrelevance tier (the
    #     "irrelevance-detect" abstain signal, the When2Call-shaped second
    #     option-key universe). NOT datasets-server-backed (raw JSONL — the
    #     /rows call 404s; the direct-resolve lane above). The pick files
    #     are ENRICHED at fetch time with their official possible_answer
    #     files (the gold join by exact id — the recorded transform; the
    #     digest pins the joined OUTPUT). The irrelevance files carry no
    #     possible_answer files (the official gold is the binary label) —
    #     the builder assigns the fixed abstain option as gold. Categories
    #     outside wave-1 scope (parallel*/java/js/sql/rest/exe*/multi_turn*/
    #     chatable/relevance) are NEVER fetched — the R2b wire verdict in
    #     the issue (pick + irrelevance-detect only). CORPUS-OPTIONAL: no
    #     train split exists in the mirror, and the candidate functions ARE
    #     the test rows' content (grammar-not-answers) — none is lawful to
    #     invent; the corpus re-open is the rethink-side synthesis route.
    if want_suite bfcl; then
        log "[probe] bfcl hub metadata"
        probe_license bfcl "gorilla-llm/Berkeley-Function-Calling-Leaderboard"
    fi
    if want_suite bfcl; then
        BFR="gorilla-llm/Berkeley-Function-Calling-Leaderboard"
        log "[suite] bfcl pick tier (simple/multiple/live_simple/live_multiple; official possible_answer gold joined by id)"
        fetch_bfcl_join bfcl "$BFR" "BFCL_v3_simple.json" "possible_answer/BFCL_v3_simple.json" test_simple
        fetch_bfcl_join bfcl "$BFR" "BFCL_v3_multiple.json" "possible_answer/BFCL_v3_multiple.json" test_multiple
        fetch_bfcl_join bfcl "$BFR" "BFCL_v3_live_simple.json" "possible_answer/BFCL_v3_live_simple.json" test_live_simple
        fetch_bfcl_join bfcl "$BFR" "BFCL_v3_live_multiple.json" "possible_answer/BFCL_v3_live_multiple.json" test_live_multiple \
            "live_multiple_1052-79-0:live_multiple_1052-279-0"
        log "[suite] bfcl irrelevance tier (the abstain signal; no gold file — the builder assigns the fixed abstain option)"
        fetch_raw_json bfcl "$BFR" "BFCL_v3_irrelevance.json" test_irrelevance jsonl
        fetch_raw_json bfcl "$BFR" "BFCL_v3_live_irrelevance.json" test_live_irrelevance jsonl
    fi

    # 19. api_bank — Issue 114 T5 (the JDI ladder wave 1): the
    #     liminghao1630/API-Bank mirror, wave-1 scope = the which-API tier
    #     (level-1-api + level-2-api — the board row's "API-call selection
    #     in a dialogue"). NOT datasets-server-servable (the raw files
    #     cast-error with inconsistent columns — the issue's probe), so the
    #     direct-resolve lane fetches the two -api.json arrays WHOLE. The
    #     multi-turn level-3 files and the response-generation files are
    #     OUT of wave-1 scope (the R2b verdict). CORPUS-OPTIONAL (the T3
    #     decision tree): the candidate API descriptions ARE the test
    #     rows' content (grammar-not-answers); the authors' own -api-train
    #     files overlap the same catalog and are never fetched. The gold
    #     is IN the row (expected_output "API-Request: [Name(...)") — the
    #     builder parses it (the recorded transform, pinned per row).
    if want_suite api_bank; then
        log "[probe] api_bank hub metadata"
        probe_license api_bank "liminghao1630/API-Bank"
    fi
    if want_suite api_bank; then
        log "[suite] api_bank which-API tier (level-1-api + level-2-api, arrays fetched whole)"
        fetch_raw_json api_bank "liminghao1630/API-Bank" "test-data/level-1-api.json" test_lv1 array
        fetch_raw_json api_bank "liminghao1630/API-Bank" "test-data/level-2-api.json" test_lv2 array
    fi

    # 20. ragtruth — Issue 114 T7 (the JDI ladder wave 3, the language
    #     rung's first suite; rethink 060 R3a): the response-level
    #     hallucination-detection row (board id 59, added edition 0.3 —
    #     chance 0.5177 = "always hallucinated · F1"; the arithmetic pins
    #     the shape: 2*943/(943+2700) = 0.51771 over ALL 2,700 test
    #     responses, 943 with ≥1 annotated span — no quality filtering).
    #     EVAL = the UPSTREAM ParticleMedia/RAGTruth response.jsonl+
    #     source_info.jsonl joined by source_id, MIT verified at the
    #     source (rethink 060 R1c residue 2026-11-12); the page files are
    #     the recorded transform (gold = 1 iff the response carries ≥1
    #     annotated span; the bfcl gold-join precedent — the digest pins
    #     the JOINED OUTPUT). The three task-type splits (QA/Summary/
    #     Data2txt, 900 rows each — 900/900/900 agrees with the
    #     flowaicom/RAGTruth_test mirror, whose `score` column is the
    #     INVERTED polarity (1 = grounded; verified 0-exception on all
    #     2,700 rows against the upstream spans) and stays the
    #     cross-verification witness, never the fetch source. The fetch
    #     lane pins the upstream repo state beside the digests (repo HEAD
    #     at fetch time).
    #     CORPUS-ON: the upstream TRAIN split is the lawful pool
    #     (grammar-not-answers) — 15,090 response rows from the same
    #     jsonl (the 060 R1c scan proved wandb-train == upstream-train at
    #     the response level; the 6,721 hallucinated count agrees with
    #     the wandb card), projected to the same recorded transform
    #     (corpus text = the response alone; the label through
    #     hallucination_labels_processed counts, any > 0 = hallucinated).
    #     The jev_*/prediction columns of any mirror row are NEVER read.
    if want_suite ragtruth; then
        log "[probe] ragtruth — the datasets-server shape probes stay (the mirrors are witnesses, not sources)"
        probe_splits ragtruth "flowaicom%2FRAGTruth_test"
        probe_size ragtruth "flowaicom%2FRAGTruth_test"
        probe_license ragtruth "flowaicom/RAGTruth_test"
    fi
    if want_suite ragtruth; then
        log "[suite] ragtruth eval+pool — the upstream-direct recorded transform (see the lane comment)"
        log "        source: ParticleMedia/RAGTruth @ the pinned fetch state (rethink .raw_r1c/ragtruth_upstream)"
        log "        the page files are generated by the 114 T7 lane; this stanza re-asserts the probes"
    fi

    # 21. finentity — Issue 114 T8 (the JDI ladder wave-3 residue; the
    #     language rung's second suite; rethink 060 R3a): the B_mechanical
    #     entity-given sentiment row (board id 39 — macro-F1, chance 0.3201
    #     "Random choices · F1", Monte Carlo; the analytic value over the
    #     same population is 0.32016). SOURCE = the authors' OWN dataloader
    #     yixuantt/FinEntity (the GitHub readme at the board's pinned sha
    #     3b6cedc links it; its FinEntity.json is BYTE-IDENTICAL to the
    #     GitHub data/FinEntity.json at that sha — blake3 e98766ac… pinned
    #     in the manifest §23). LICENCE: the authors' readme at that sha
    #     states "FinEntity is licensed under ODC-BY" and the authors' own
    #     HF card tags license:odc-by — two authoritative surfaces agreeing
    #     (the mirror-tag-alone lesson never bites here: the card IS the
    #     authors'). The existing /resolve lane fetches it whole (the
    #     banking77 parquet stanza does not apply — raw JSON array, one
    #     document per row: content + annotations[span/label/…]).
    #     SHAPE: one case per annotated SPAN (state = {text, entity,
    #     start, end}; options = Negative/Neutral/Positive; gold = the
    #     span's label). The board's own disclosed quarantine applies at
    #     build: ONE conflicting exact span (doc 299's S&P 500, Neutral +
    #     Positive) drops BOTH rows → 2,131 − 2 = 2,129 cases = the
    #     board's fields count (the builder's census asserts pin it).
    #     CORPUS-OPTIONAL: evaluation-only unsplit source (the board's own
    #     disclosure) — the spans+labels ARE the test content
    #     (grammar-not-answers); modelless_lane: false.
    if want_suite finentity; then
        log "[probe] finentity hub metadata"
        probe_splits finentity "yixuantt%2FFinEntity"
        probe_size finentity "yixuantt%2FFinEntity"
        probe_license finentity "yixuantt%2FFinEntity"
    fi
    if want_suite finentity; then
        log "[suite] finentity — the authors' own dataloader, fetched whole (byte-identical to the board's pinned GitHub sha)"
        fetch_raw_json finentity "yixuantt/FinEntity" "FinEntity.json" test array
    fi

    # 22. contractnli — Issue 114 T9 (the JDI ladder wave-3; the language
    #     rung's third suite; rethink 060 R3a): the A_direct entailment row
    #     (board id 11 — macro-F1, chance 0.3085 "Random choices · F1",
    #     Monte Carlo; the analytic value over the same distribution is
    #     0.30793, Δ 6e-4 MC noise). LICENCE RESOLVED 2026-11-12: the
    #     authors' canonical repo stanfordnlp/contract-nli (gh-pages; the
    #     board's suite.md pins @eced652) carries the full CC-BY-4.0
    #     LICENSE (18,656 bytes), the bundled TERMS grants rights "in
    #     accordance with the terms and conditions of the Creative Commons
    #     Attribution 4.0 International Public License", and the dataset
    #     site's License section names it — no NC/ShareAlike anywhere
    #     upstream. The kiddothe2b HF mirror's cc-by-nc-sa-4.0 tag is a
    #     THIRD-PARTY curator contradiction of the upstream — that mirror
    #     is never a source (the flowaicom-witness pattern); the
    #     mteb/legalbench cc-by-4.0 curation tag was right. SOURCE = the
    #     authors' zip at the board's pinned sha (resources/contract-
    #     nli.zip), the GitHub raw lane (the hub /resolve does not serve
    #     GitHub), ONE 65 MB download reused for both splits. THE
    #     RECORDED TRANSFORM (the digest pins the OUTPUT envelopes): the
    #     nested per-document JSON flattens to pair rows {contract,
    #     hypothesis, choice, split} — test 123×17 = 2,091 rows = the
    #     board's fields count; train 423×17 = 7,191 rows. The board
    #     metric is pair-level macro-F1 (its chance arithmetic pins the
    #     frame; the catalog's requests=123 documents bookkeeping repeats
    #     the finentity note — disclosed, never inherited).
    #     CORPUS-ON: the upstream train split is the task's own released
    #     training data (the SemEval-2024 paper documents systems
    #     fine-tuning on it; the contrast interventions were withheld
    #     from training only and are NOT in the board's track — the
    #     original 2,091 test pairs). Corpus text = contract + hypothesis
    #     (the contract is what flips the label; the ragtruth
    #     response-alone rule does not transfer).
    if want_suite contractnli; then
        log "[suite] contractnli — the authors' zip at the board's pinned sha, both splits flattened (the recorded transform)"
        fetch_contractnli_pairs contractnli "stanfordnlp/contract-nli" "eced652" "resources/contract-nli.zip"
    fi

    # Issue 114 T10 (the JDI ladder wave 3): iSarcasmEval — the authors'
    # repo at the board's pinned sha (the tree sha read live at the licence
    # probe), track-A English files only (the board row's index metric is
    # track-A-English sarcastic-class F1; the B/C/Arabic tracks are the
    # recorded re-opens).
    if want_suite isarcasmeval; then
        log "[suite] isarcasmeval — track-A English CSVs at the board's pinned sha (the recorded transform)"
        fetch_isarcasmeval isarcasmeval "iabufarha/iSarcasmEval" "dfc708b53bde1bb571abfb5692f63231c2232195"
    fi

    # Issue 114 T13 (the wave-2 residue): the track-B multilabel envelope —
    # the SAME pinned sha (the T10 MIT read covers the whole repo tree;
    # test/task_B_En_test.csv rides it), the SAME shared isarcasmeval
    # dataset dir. The fetch lane takes the test_b split alongside the
    # track-A files (idempotent skips on the cached pages).
    if want_suite isarcasm_b; then
        log "[suite] isarcasm_b — the track-B technique CSV at the same pinned sha (the T10 MIT read covers; the T13 multilabel envelope)"
        fetch_isarcasmeval isarcasmeval "iabufarha/iSarcasmEval" "dfc708b53bde1bb571abfb5692f63231c2232195"
    fi

    # 25. home_appliance — Issue 114 T11 (the JDI ladder wave-2 machinery's
    #     first consumer; board row 9). The generator + pinned-kept-set
    #     lane (the hub rows file is private; see the function's header).
    if want_suite home_appliance; then
        log "[suite] home_appliance — the board's own generator + the release-v2.1 pinned kept set (the board's hub rows are private)"
        fetch_home_appliance home_appliance "${HOME_APPLIANCE_CLONE:-$ROOT/.raw/mammoth-local/decision-index}"
    fi

    log ""
    log "summary: files=$FILES rows=$ROWS failed_requests=$FAILED"
    if [ "$FAILED" -gt 0 ]; then
        log "FAILURES PRESENT — see FAIL lines above and .docs/02_protocols/dataset_manifest.md Gaps"
        exit 1
    fi
}

# ── Issue 096 self-test ─────────────────────────────────────────────────
# Hermetic: a loopback python3 server plays the datasets-server (zero hub
# traffic — the licence probe is pre-planted cached); the fixture plants a
# DEDUPED mid-sequence page (99 rows, num_rows_total intact — the `.rows |=`
# rewrite preserves it) and asserts the resume walks PAST it. Three
# scenarios:
#   1. the defect shape — resume over a deduped 99-row page-000 completes
#      the pull (pages 002+003 fetched; summary 349 rows in 4 files); the
#      old predicate stopped at 000 and reported a clean 199-row "complete"
#      pull (rethink Bench-071 provisioning hit exactly this on sst5);
#   2. idempotence + the true terminal — the re-run fetches nothing, fires
#      the resume line ONLY for the still-short page-000, and the genuinely
#      terminal 50-row page-003 stops without one (no page-004);
#   3. degraded metadata — a short page with NO num_rows_total falls back
#      to the legacy stop, loudly.
ST_TMP=""
ST_SERVER_PID=""
ST_FAIL_COUNT=0
st_fail() { log "  selftest FAIL: $*"; ST_FAIL_COUNT=$((ST_FAIL_COUNT + 1)); }
st_fail_log() { # <log-file> <reason> — dump the child log so a red run is diagnosable
    st_fail "$2"
    [ -f "$1" ] && sed 's/^/    | /' "$1"
    return 0
}
st_expect_file() { [ -f "$1" ] || st_fail "missing file: $1"; }
st_expect_grep() { grep -q "$1" "$2" 2>/dev/null || st_fail "$3"; }
st_write_page() { # <path> <total|-1 none> <start> <count> [drop_idx]
    python3 - "$@" <<'EOF'
import json, sys
path, total, start, count = (sys.argv[1], int(sys.argv[2]),
                             int(sys.argv[3]), int(sys.argv[4]))
drop = int(sys.argv[5]) if len(sys.argv) > 5 else -1
rows = [{"row_idx": i,
         "row": {"sentence": "train-%d" % i, "label": i % 5},
         "truncated_cells": []}
        for i in range(start, start + count) if i != drop]
body = {"features": [], "rows": rows}
if total >= 0:
    body["num_rows_total"] = total
with open(path, "w", encoding="utf-8") as f:
    json.dump(body, f)
EOF
}
st_run_fetch() { # <out-dir> <log-file> — the child fetch, hermetic posture
    BASE="$st_base" OUT="$1" SUITES=sst5 SLEEP=0 TRAIN_CAP=400 FORCE=0 \
        sh "$SCRIPT_DIR/fetch_datasets.sh" >"$2" 2>&1
}
self_test() {
    have python3 || { log "self-test: REFUSED — python3 is required (the loopback fixture server)"; return 1; }
    have curl || { log "self-test: REFUSED — curl is required"; return 1; }
    ST_TMP=$(mktemp -d "${TMPDIR:-/tmp}/fetch_datasets_selftest.XXXXXX")
    cat > "$ST_TMP/server.py" <<'PYEOF'
import json
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse, parse_qs

TOTALS = {"test": 9, "train": 350}

# Issue 114 T4/T5 — the direct-resolve lane's fixture files (served under
# /datasets/<repo>/resolve/main/<path>): a tiny BFCL-shaped pick eval + its
# gold, an irrelevance eval, and an API-Bank-shaped array.
RESOLVE = {
    "acme/pick/resolve/main/eval.jsonl": '\n'.join([
        '{"id": "simple_0", "question": [[{"role": "user", "content": "q0"}]], "function": [{"name": "fn_a", "description": "d a"}]}',
        '{"id": "simple_1", "question": [[{"role": "user", "content": "q1"}]], "function": [{"name": "fn_b", "description": "d b"}, {"name": "fn_c", "description": "d c"}]}',
    ]) + '\n',
    "acme/pick/resolve/main/possible_answer/eval.jsonl": '\n'.join([
        '{"id": "simple_0", "ground_truth": [{"fn_a": {}}]}',
        '{"id": "simple_1", "ground_truth": [{"fn_c": {}}]}',
    ]) + '\n',
    "acme/pick/resolve/main/irr.jsonl": '{"id": "irrelevance_0", "question": [[{"role": "user", "content": "q2"}]], "function": [{"name": "fn_x", "description": "d x"}]}\n',
    "acme/pick/resolve/main/badgold.jsonl": '{"id": "simple_0", "ground_truth": [{"fn_a": {}}]}\n{"id": "ghost_9", "ground_truth": [{"fn_z": {}}]}\n',
    "acme/pick/resolve/main/tyeval.jsonl": '{"id": "t_79", "question": [[{"role": "user", "content": "qt"}]], "function": [{"name": "fn_t", "description": "d t"}]}\n',
    "acme/pick/resolve/main/tygold.jsonl": '{"id": "t_279", "ground_truth": [{"fn_t": {}}]}\n',
    "acme/bank/resolve/main/lv1.json": json.dumps([
        {"id": 0, "instruction": 'ctx\nAPI descriptions:\n{"name": "ApiOne", "description": "one"}\n{"name": "ApiTwo", "description": "two"}',
         "expected_output": "API-Request: [ApiTwo(key='v')]"},
        {"id": 1, "instruction": 'ctx2\nAPI descriptions:\n{"name": "ApiThree", "description": "three"}',
         "expected_output": "no call"},
    ]),
}

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_GET(self):
        u = urlparse(self.path)
        if u.path.startswith("/datasets/"):
            key = u.path[len("/datasets/"):]
            body = RESOLVE.get(key)
            if body is None:
                self.send_error(404)
                return
            body = body.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if u.path != "/rows":
            self.send_error(404)
            return
        q = {k: v[0] for k, v in parse_qs(u.query).items()}
        split = q.get("split", "train")
        offset = int(q.get("offset", "0"))
        length = int(q.get("length", "100"))
        total = TOTALS.get(split, 0)
        rows = [{"row_idx": i,
                 "row": {"sentence": "%s-%d" % (split, i), "label": i % 5},
                 "truncated_cells": []}
                for i in range(offset, min(offset + length, total))]
        body = json.dumps(
            {"num_rows_total": total, "features": [], "rows": rows}).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

server = HTTPServer(("127.0.0.1", 0), Handler)
print(server.server_address[1], flush=True)
server.serve_forever()
PYEOF
    python3 "$ST_TMP/server.py" > "$ST_TMP/port" 2> "$ST_TMP/server.log" &
    ST_SERVER_PID=$!
    tries=0
    while [ ! -s "$ST_TMP/port" ] && [ "$tries" -lt 100 ]; do
        tries=$((tries + 1))
        sleep 0.1
    done
    if [ ! -s "$ST_TMP/port" ]; then
        log "self-test: FAIL — fixture server never reported a port (log: $ST_TMP/server.log)"
        return 1
    fi
    st_base="http://127.0.0.1:$(cat "$ST_TMP/port")"

    # ── Scenario 1: the defect shape ──
    out1="$ST_TMP/out1"
    mkdir -p "$out1/sst5"
    printf '{}' > "$out1/sst5/license.json"   # the hub licence probe skips cached
    st_write_page "$out1/sst5/train-000.json" 350 0 100 7
    st_write_page "$out1/sst5/train-001.json" 350 100 100
    log1="$ST_TMP/run1.log"
    st_rc=0
    st_run_fetch "$out1" "$log1" || st_rc=$?
    [ "$st_rc" -eq 0 ] || st_fail "scenario 1: child fetch exited $st_rc"
    st_expect_file "$out1/sst5/test-000.json"
    st_expect_grep '  resume sst5/train-000' "$log1" \
        "scenario 1: no resume line for the deduped page (the walk stopped at it)"
    st_expect_file "$out1/sst5/train-002.json"
    st_expect_file "$out1/sst5/train-003.json"
    st_expect_grep 'sst5/train: 349 rows in 4 file(s)' "$log1" \
        "scenario 1: summary is not the completed pull"

    # ── Scenario 2: the completed tree re-run (idempotence + true terminal) ──
    log2="$ST_TMP/run2.log"
    st_rc=0
    st_run_fetch "$out1" "$log2" || st_rc=$?
    [ "$st_rc" -eq 0 ] || st_fail "scenario 2: child fetch exited $st_rc"
    st_n=$(grep -c '^  fetch' "$log2" 2>/dev/null || true)
    [ "$st_n" -eq 0 ] || st_fail "scenario 2: $st_n page(s) re-fetched on a complete tree (expected 0)"
    st_n=$(grep -c '^  resume' "$log2" 2>/dev/null || true)
    [ "$st_n" -eq 1 ] || st_fail "scenario 2: expected exactly 1 resume line (the still-short 000), got $st_n"
    if [ -f "$out1/sst5/train-004.json" ]; then
        st_fail "scenario 2: train-004 fetched past the true terminal page"
    fi
    st_expect_grep 'sst5/train: 349 rows in 4 file(s)' "$log2" \
        "scenario 2: summary drifted on the re-run"

    # ── Scenario 3: degraded metadata — legacy stop, loud ──
    out3="$ST_TMP/out3"
    mkdir -p "$out3/sst5"
    printf '{}' > "$out3/sst5/license.json"
    st_write_page "$out3/sst5/train-000.json" -1 0 100 7
    log3="$ST_TMP/run3.log"
    st_rc=0
    st_run_fetch "$out3" "$log3" || st_rc=$?
    [ "$st_rc" -eq 0 ] || st_fail "scenario 3: child fetch exited $st_rc"
    st_expect_grep 'no readable num_rows_total' "$log3" \
        "scenario 3: the legacy-stop fallback was not loud"
    st_n=$(grep -c '^  fetch  sst5/train' "$log3" 2>/dev/null || true)
    [ "$st_n" -eq 0 ] || st_fail "scenario 3: $st_n train page(s) fetched past a metadata-less short page (expected the legacy stop)"
    st_expect_grep 'sst5/train: 99 rows in 1 file(s)' "$log3" \
        "scenario 3: summary is not the legacy-stop state"

    # ── Scenario 4: the direct-resolve lane (issue 114 T4/T5) ──
    # fetch_raw_json both shapes + the skip law; fetch_bfcl_join's all-or-
    # nothing gold join + its LOUD missing-gold failure.
    out4="$ST_TMP/out4"
    mkdir -p "$out4"
    RAW_BASE="$st_base" OUT="$out4" SLEEP=0 FORCE=0 \
        fetch_raw_json rawt acme/pick irr.jsonl test jsonl \
        > "$ST_TMP/run4a.log" 2>&1 || st_fail "scenario 4a: fetch_raw_json exited $?"
    st_expect_file "$out4/rawt/test-000.json"
    st_expect_grep 'rows=1 (resolve irr.jsonl)' "$ST_TMP/run4a.log" \
        "scenario 4a: the jsonl envelope was not written"
    RAW_BASE="$st_base" OUT="$out4" SLEEP=0 FORCE=0 \
        fetch_raw_json rawt acme/pick irr.jsonl test jsonl \
        > "$ST_TMP/run4b.log" 2>&1 || st_fail "scenario 4b: re-run exited $?"
    st_expect_grep 'skip   rawt/test-000 (1 rows)' "$ST_TMP/run4b.log" \
        "scenario 4b: the idempotence skip law did not hold"
    RAW_BASE="$st_base" OUT="$out4" SLEEP=0 FORCE=0 \
        fetch_raw_json rawt acme/bank lv1.json arr array \
        > "$ST_TMP/run4c.log" 2>&1 || st_fail "scenario 4c: array shape exited $?"
    st_expect_grep 'rows=2 (resolve lv1.json)' "$ST_TMP/run4c.log" \
        "scenario 4c: the array envelope was not written"
    RAW_BASE="$st_base" OUT="$out4" SLEEP=0 FORCE=0 FORCE=0 \
        fetch_bfcl_join rawt acme/pick eval.jsonl possible_answer/eval.jsonl pick \
        > "$ST_TMP/run4d.log" 2>&1 || st_fail "scenario 4d: join exited $?"
    RAW_BASE="$st_base" OUT="$out4" SLEEP=0 FORCE=0 \
        fetch_bfcl_join rawt acme/pick eval.jsonl possible_answer/eval.jsonl pick \
        > "$ST_TMP/run4d.log" 2>&1 || true
    [ -f "$out4/rawt/pick-000.json" ] || st_fail_log "$ST_TMP/run4d.log" \
        "scenario 4d: the gold join did not produce the joined envelope"
    if [ -f "$out4/rawt/pick-000.json" ]; then
        joined=$(jq -r '.rows[1].row.ground_truth[0] | keys[0]' "$out4/rawt/pick-000.json" 2>/dev/null || echo UNREAD)
        [ "$joined" = "fn_c" ] || st_fail_log "$ST_TMP/run4d.log" \
            "scenario 4d: the joined gold drifted (row 1 gold: $joined)"
    fi
    # The loud half: an id-set mismatch fails the whole fetch, never a
    # partial suite (simple_1 has no gold; ghost_9 has no eval row).
    RAW_BASE="$st_base" OUT="$out4" SLEEP=0 FORCE=0 \
        fetch_bfcl_join rawt acme/pick eval.jsonl badgold.jsonl badpick \
        > "$ST_TMP/run4e.log" 2>&1 || true
    st_expect_grep 'id-set mismatch: 1 eval row(s) without gold' "$ST_TMP/run4e.log" \
        "scenario 4e: the missing-gold failure was not loud" || sed 's/^/    | /' "$ST_TMP/run4e.log"
    st_expect_grep '1 gold row(s) without an eval row' "$ST_TMP/run4e.log" \
        "scenario 4e: the extra-gold failure was not loud" || true
    [ -f "$out4/rawt/badpick-000.json" ] && st_fail "scenario 4e: the mismatched fetch still wrote an envelope"
    # The pinned-typo path: the one literal id rewrite joins cleanly; the
    # norm line names the pair (never a silent rewrite).
    RAW_BASE="$st_base" OUT="$out4" SLEEP=0 FORCE=0 \
        fetch_bfcl_join rawt acme/pick tyeval.jsonl tygold.jsonl typick "t_79:t_279" \
        > "$ST_TMP/run4f.log" 2>&1 || true
    st_expect_grep 'norm   rawt/typick-000: t_79 -> t_279' "$ST_TMP/run4f.log" \
        "scenario 4f: the pinned normalization was not logged loud" || sed 's/^/    | /' "$ST_TMP/run4f.log"
    st_expect_grep 'rows=1 (resolve tyeval.jsonl + gold join)' "$ST_TMP/run4f.log" \
        "scenario 4f: the normalized fetch did not join" || true

    kill "$ST_SERVER_PID" 2>/dev/null || true
    wait "$ST_SERVER_PID" 2>/dev/null || true
    rm -rf "$ST_TMP"
    if [ "$ST_FAIL_COUNT" -eq 0 ]; then
        log "self-test: PASS — resume walks past deduped pages; terminal stays terminal; degraded metadata falls back loud; the direct-resolve lane converts both shapes, skips idempotently, joins gold all-or-nothing"
        return 0
    fi
    log "self-test: FAIL ($ST_FAIL_COUNT assertion(s) — see the selftest FAIL lines above)"
    return 1
}

case "${1:-}" in
    --self-test) self_test ;;
    *) main "$@" ;;
esac
