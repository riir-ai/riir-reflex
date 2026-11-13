#!/bin/sh
# bench_preflight.sh — REFUSE to publish a latency number the box invalidates.
#
# riir-reflex Issue 021. AGENTS.md already says "a latency number without its
# BOX STATE is not a measurement" and names free RAM, commit-vs-limit and
# concurrent jobs. On a LAPTOP that list is incomplete in a first-order way:
# Apple Silicon drops sustained GPU clocks on battery, and this box was
# measured unplugged (09:56:10, 100% -> 48%) through an entire A/B session
# whose AC-taken predecessors read 26.2 ms where the battery ones read
# 33-35 ms on the SAME binary. Power state is not a footnote; it is an arm.
#
# Deliberately a SHELL script with NO exit trap and no temp files: there is
# nothing for an aborted run to launder (AGENTS.md trap-sentinel class), and
# every refusal exits non-zero with the reason on its own line.
#
# Usage:
#   scripts/bench_preflight.sh                # refuse unless the box is fit
#   MAX_LOAD=8 scripts/bench_preflight.sh     # raise the load ceiling
#   SETTLE_MIN=10 scripts/bench_preflight.sh  # longer post-plug-in settle
#   CANARY_RUNS=9 scripts/bench_preflight.sh  # more canary runs (best-of-N)
#   CANARY_REF_US= scripts/bench_preflight.sh # unpin: print the canary, never judge
#
# Exit: 0 = fit to bench (the provenance line is on stdout, QUOTE IT in the
# record); 1 = refused, reason named; 2 = the instrument itself could not read
# the box (never a green zero).
#
# Issue 118: the POWER axis is cross-OS — pmset on macOS, a Win32_Battery
# PowerShell probe on Windows (Git Bash), /sys/class/power_supply on Linux.
# The macOS-only axes (powermode enum, pmset settle history) degrade to a
# disclosed warn off macOS; an unreadable power state still exits 2 and a
# battery source still refuses, on every arm.

set -eu

MAX_LOAD="${MAX_LOAD:-6.0}"
SETTLE_MIN="${SETTLE_MIN:-5}"
CANARY_BIN="${CANARY_BIN:-./target/release/examples/sgemm_shape_timing}"
# Pinned AC canary reference, microseconds, shape 317x1024x1024, as the
# BEST-OF-N statistic below (Issue 021 T2, 2026-09-24 11:50 +0700): AC,
# powermode 2 (High Power), 47 min after plug-in, load 4.7, swap ~2.9 GB.
# 30 single runs spread 140.3-165.6 us (18% — wider than any tolerance a
# gate can use), while the six best-of-5 minima spread 140.3-144.3 us
# (2.9%): contention adds time to SOME runs, a throttled clock raises the
# FLOOR of all of them, so the minimum separates the two. Pinned at 141
# (the minima's median). Taken at load 4.7 and not < 2 because a quiet
# window never came — a later quiet best-of-N LOWER than this should
# replace it (ratchet down only; a higher one is the box, not the ref).
# The reference is a powermode-2 number and is judged ONLY in powermode 2:
# Automatic (0) is a different arm and would be refused or blessed for
# the wrong reason. Set CANARY_REF_US= (empty) to print without judging.
CANARY_REF_US="${CANARY_REF_US-141}"
CANARY_REF_POWERMODE=2
CANARY_TOL_PCT="${CANARY_TOL_PCT:-15}"
CANARY_RUNS="${CANARY_RUNS:-5}"

fail=0
note() { printf '%s\n' "$*"; }
refuse() { note "REFUSE — $*"; fail=1; }

# ---- 1. power source (Issue 118: cross-OS) --------------------------------
os_name=$(uname -s 2>/dev/null || echo unknown)
power_state=""
ps_out=""
case "$os_name" in
    Darwin)
        ps_out=$(pmset -g ps 2>/dev/null || true)
        if [ -z "$ps_out" ]; then
            note "⛔ pmset unreadable — the power axis is UNKNOWN, not fine"
            exit 2
        fi
        case "$ps_out" in
            *"'AC Power'"*) power_state="AC"; note "ok   power        AC" ;;
            *"'Battery Power'"*)
                pct=$(printf '%s' "$ps_out" | grep -o '[0-9]\{1,3\}%' | head -1 || true)
                power_state="BATT ${pct:-?}%"
                refuse "on BATTERY (${pct:-?}) — Apple Silicon sheds sustained GPU clock off AC; plug in and re-run"
                ;;
            *) note "⛔ power source unrecognised in: $ps_out"; exit 2 ;;
        esac
        ;;
    Linux)
        # AC adapters expose A*/online (1 = present); batteries expose
        # BAT*/status. No readable supply at all is UNKNOWN, never a green.
        ac_online=""
        for ac_f in /sys/class/power_supply/A*; do
            [ -e "$ac_f/online" ] || continue
            if [ "$(cat "$ac_f/online" 2>/dev/null)" = "1" ]; then ac_online=1; break; fi
        done
        if [ "$ac_online" = "1" ]; then
            power_state="AC"
            note "ok   power        AC (sysfs)"
        else
            bat_status=""
            bat_cap=""
            for bat_f in /sys/class/power_supply/BAT*; do
                [ -e "$bat_f/status" ] || continue
                bat_status=$(cat "$bat_f/status" 2>/dev/null || true)
                bat_cap=$(cat "$bat_f/capacity" 2>/dev/null || true)
                break
            done
            case "$bat_status" in
                Charging|Full|"Not charging")
                    power_state="AC"
                    note "ok   power        AC (battery ${bat_cap:-?}% ${bat_status:-plugged})"
                    ;;
                Discharging)
                    power_state="BATT ${bat_cap:-?}%"
                    refuse "on BATTERY (${bat_cap:-?}%) — sustained clocks shed off AC; plug in and re-run"
                    ;;
                *)
                    note "⛔ power state unrecognised (sysfs status '${bat_status:-missing}')"
                    exit 2
                    ;;
            esac
        fi
        ;;
    MINGW*|MSYS*|CYGWIN*)
        # Win32_Battery.BatteryStatus: on-AC = 2 (AC), 3 (full), 6-9 + 11
        # (charging variants); discharging = 1, 4, 5; anything else UNKNOWN.
        # No battery at all = a desktop: AC assumed, DISCLOSED, never silent.
        ps_probe=$(powershell.exe -NoProfile -NonInteractive -Command '
$b = Get-CimInstance -ClassName Win32_Battery -ErrorAction SilentlyContinue | Select-Object -First 1
if ($null -eq $b) { "NONE" }
elseif ($b.BatteryStatus -in 2,3,6,7,8,9,11) { "AC" }
elseif ($b.BatteryStatus -in 1,4,5) { "BATT " + [int]$b.EstimatedChargeRemaining }
else { "UNKNOWN " + $b.BatteryStatus }
' 2>/dev/null || true)
        case "$ps_probe" in
            NONE)
                power_state="AC-desktop"
                note "ok   power        AC assumed (no battery present — desktop, disclosed)"
                ;;
            AC)
                power_state="AC"
                note "ok   power        AC (Win32_Battery on-AC)"
                ;;
            BATT\ *)
                power_state="$ps_probe"
                refuse "on BATTERY (${ps_probe#BATT }%) — plug in and re-run"
                ;;
            *)
                note "⛔ power axis unreadable on Windows (probe: '${ps_probe:-empty}')"
                exit 2
                ;;
        esac
        ;;
    *)
        note "⛔ unsupported OS for the power axis: $os_name"
        exit 2
        ;;
esac

# ---- 2. power mode (macOS-only axis; disclosed n/a elsewhere — Issue 118) --
# `pmset powermode` on a 14"/16" Apple Silicon MacBook Pro is a THREE-state
# enum, not a Low-Power-Mode boolean: 0 = Automatic, 1 = Low Power, 2 = High
# Power. The first version of this gate read "not 0" as Low Power and refused
# the one mode that is BEST for a sustained number (measured: this box's AC
# profile is powermode 2, its battery profile 0). Only 1 refuses; 0 and 2 are
# both fit but are DIFFERENT arms, so the mode is named in the provenance
# line and a canary reference is only comparable within one mode.
pmode=""
pmode_name="?"
if [ "$os_name" = "Darwin" ]; then
    pmode=$(pmset -g 2>/dev/null | awk '/powermode/{print $2}' || true)
    case "$pmode" in
        "") note "warn powermode    unreadable (disclosed, not assumed)"; pmode_name="?" ;;
        0)  pmode_name="auto"; note "ok   powermode    0 (Automatic)" ;;
        1)  pmode_name="low";  refuse "Low Power Mode is ON (powermode=1) — it caps clocks by design" ;;
        2)  pmode_name="high"; note "ok   powermode    2 (High Power)" ;;
        *)  pmode_name="unknown"; note "⛔ powermode=$pmode is not a value this gate knows (0/1/2)"; exit 2 ;;
    esac
else
    note "warn powermode    n/a on $os_name (macOS-only axis, disclosed)"
    pmode_name="n/a"
fi

# ---- 3. settle window since the last power transition (macOS-only axis) ----
# A box plugged in one minute ago is still carrying the heat it built on
# battery; AC is necessary, not sufficient. ENFORCED, not just disclosed:
# the first version printed the transition and left the arithmetic to a
# reader, who is exactly the person in a hurry to bench.
if [ "$os_name" = "Darwin" ]; then
    last=$(pmset -g log 2>/dev/null | grep -E "Using (AC|Batt)" | tail -1 || true)
    if [ -n "$last" ]; then
        last_ts=$(printf '%s' "$last" | cut -c1-19)
        last_kind=$(printf '%s' "$last" | grep -oE "Using (AC|Batt)" | head -1)
        note "info last power transition: $last_ts ($last_kind)"
        last_epoch=$(date -j -f "%Y-%m-%d %H:%M:%S" "$last_ts" +%s 2>/dev/null || true)
        if [ -n "$last_epoch" ] && [ "$last_kind" = "Using AC" ]; then
            on_ac_min=$(( ( $(date +%s) - last_epoch ) / 60 ))
            if [ "$on_ac_min" -lt "$SETTLE_MIN" ]; then
                refuse "on AC for only ${on_ac_min} min (< SETTLE_MIN=${SETTLE_MIN}) — still carrying battery-era heat"
            else
                note "ok   settle       on AC ${on_ac_min} min (>= ${SETTLE_MIN})"
            fi
        elif [ -z "$last_epoch" ]; then
            note "warn settle window UNVERIFIED — could not parse '$last_ts'"
        fi
    else
        note "warn no power-transition history — settle window UNVERIFIED"
    fi
else
    note "warn settle window UNVERIFIED — no power-transition history axis on $os_name"
fi

# ---- 4. load (Issue 118: cross-OS read) -----------------------------------
# macOS keeps the uptime load average. Linux reads /proc/loadavg (uptime
# fallback). MSYS2's /proc/loadavg is a STUB (measured 0.00 with cargo
# running — Issue 118 T2 cell) and Git Bash ships no uptime, so Windows
# estimates busy cores from Win32_Processor: median of 3 one-second
# LoadPercentage samples × logical processors — the same busy-core units as
# a unix load average, DISCLOSED as an estimate (an instant CPU read, not a
# 1-minute average).
load_src=""
la=""
case "$os_name" in
    Darwin)
        la=$(uptime | sed 's/.*averages*: *//' | awk '{print $1}' | tr -d ',' || true)
        ;;
    Linux)
        la=$(cut -d ' ' -f1 /proc/loadavg 2>/dev/null || true)
        if [ -z "$la" ]; then
            la=$(uptime | sed 's/.*averages*: *//' | awk '{print $1}' | tr -d ',' || true)
        fi
        ;;
    MINGW*|MSYS*|CYGWIN*)
        load_src="win-est"
        la=$(powershell.exe -NoProfile -NonInteractive -Command '
$s = @()
1..3 | ForEach-Object {
    $p = Get-CimInstance -ClassName Win32_Processor
    $cpu = ($p | Measure-Object -Property LoadPercentage -Average).Average
    $lp = ($p | Measure-Object -Property NumberOfLogicalProcessors -Maximum).Maximum
    if ($null -ne $cpu -and $null -ne $lp) { $s += [math]::Round(($cpu / 100) * $lp, 2) }
    Start-Sleep -Seconds 1
}
($s | Sort-Object)[1]
' 2>/dev/null | tr -d '\r' | tr ',' '.' || true)
        ;;
esac
if [ -z "$la" ]; then
    note "⛔ load average unreadable"; exit 2
fi
if awk "BEGIN{exit !($la > $MAX_LOAD)}"; then
    if [ -n "$load_src" ]; then
        refuse "load $la ($load_src) exceeds MAX_LOAD=$MAX_LOAD — a sibling session is on the box"
    else
        refuse "load average $la exceeds MAX_LOAD=$MAX_LOAD — a sibling session is on the box"
    fi
else
    if [ -n "$load_src" ]; then
        note "ok   load         $la ($load_src) (ceiling $MAX_LOAD)"
    else
        note "ok   load         $la (ceiling $MAX_LOAD)"
    fi
fi

# ---- 5. memory pressure ---------------------------------------------------
swap=$(sysctl -n vm.swapusage 2>/dev/null | awk '{print $6}' | tr -d 'M' || true)
if [ -n "$swap" ]; then
    if awk "BEGIN{exit !(${swap:-0} > 1024)}"; then
        note "warn swap used   ${swap}M — paging can masquerade as a slow kernel"
    else
        note "ok   swap used   ${swap}M"
    fi
else
    note "warn swap usage unreadable"
fi

# ---- 6. GPU clock canary (macOS-only axis — Issue 118) --------------------
# The ONLY throttle detector available without sudo on the macOS box: run a
# fixed Metal kernel and read its absolute time. powermetrics needs root;
# there is no sudo-free thermal-pressure sysctl on Apple Silicon here
# (measured). LAYA_DEVICE=metal is meaningless off macOS, so the canary is a
# macOS axis: off macOS it SKIPS with a disclosure, never runs, and never
# pastes a non-numeric reading into the provenance.
# BEST-OF-N: one run's p50 still carries whatever a sibling put on the GPU
# during its ~0.4 s (measured single-run spread 18%); the minimum over N
# runs is what a throttled clock moves and contention mostly does not.
canary=""
canary_n=0
if [ "$os_name" = "Darwin" ] && [ -x "$CANARY_BIN" ]; then
    i=0
    while [ "$i" -lt "$CANARY_RUNS" ]; do
        i=$((i + 1))
        one=$(LAYA_DEVICE=metal "$CANARY_BIN" 2>/dev/null \
            | awk '/317x1024x1024/{gsub("gpu_p50=","");gsub("us","");print $3}' || true)
        case "$one" in ''|*[!0-9.]*) continue ;; esac
        canary_n=$((canary_n + 1))
        if [ -z "$canary" ] || awk "BEGIN{exit !($one < $canary)}"; then
            canary="$one"
        fi
    done
    # A partial read is disclosed, never silently promoted to best-of-N.
    if [ -n "$canary" ] && [ "$canary_n" -lt "$CANARY_RUNS" ]; then
        note "warn canary       only ${canary_n}/${CANARY_RUNS} runs produced a reading"
    fi
fi
if [ "$os_name" != "Darwin" ]; then
    note "warn canary       SKIPPED — macOS-only axis (Metal throttle detector), disclosed"
elif [ -z "$canary" ]; then
    note "warn canary       SKIPPED — build it first:"
    note "                  cargo build --release --features laya-riir-metal \\"
    note "                      --example sgemm_shape_timing"
elif [ -z "$CANARY_REF_US" ]; then
    note "info canary       ${canary} us best-of-${canary_n} (317x1024x1024) — reference UNPINNED, not judged"
elif [ "${pmode:-}" != "$CANARY_REF_POWERMODE" ]; then
    note "info canary       ${canary} us best-of-${canary_n} — NOT judged: the reference is a"
    note "                  powermode ${CANARY_REF_POWERMODE} number and this box is in powermode ${pmode:-?}"
else
    if awk "BEGIN{exit !($canary > $CANARY_REF_US * (1 + $CANARY_TOL_PCT/100.0))}"; then
        refuse "canary ${canary} us best-of-${canary_n} vs AC reference ${CANARY_REF_US} us (+${CANARY_TOL_PCT}% tol) — the GPU is throttled or contended"
    else
        note "ok   canary       ${canary} us best-of-${canary_n} vs ref ${CANARY_REF_US} us"
    fi
fi

# ---- provenance -----------------------------------------------------------
if [ "$os_name" = "Darwin" ]; then
    prov="power=$(printf '%s' "$ps_out" | grep -o "'[A-Za-z ]*Power'" | head -1 | tr -d "'")"
else
    prov="power=$power_state"
fi
if [ -n "$canary" ]; then canary_prov="${canary}us/best${canary_n}"; else canary_prov="skipped"; fi
prov="$prov load=$la${load_src:+($load_src)} swap=${swap:-?}M canary=${canary_prov} powermode=${pmode:-?}(${pmode_name})"
note ""
note "PROVENANCE: $prov"

if [ "$fail" -ne 0 ]; then
    note "✗ preflight REFUSED — do not publish a latency number from this box now"
    exit 1
fi
note "✓ preflight PASSED — quote the PROVENANCE line in the bench record"
