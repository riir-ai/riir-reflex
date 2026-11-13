# Issue 118 — bench_preflight.sh power axis is macOS-only (`pmset`); Windows boxes refuse the preflight

**Status:** Open — LANDED 2026-11-14 (T1+T2 complete; the substitute-disclosure pattern retires). Filed 2026-11-14 from the 4090 (plan-498 T0.1/T0.3 lane, zed-4430bf70@4090 + the on-box lane zed-5ba62a6b@4090).
**Lease closed:** zed-14c3ed6a@m3 (m3), 1794604504, claimed 2026-11-14, was-expires 2026-11-15T21:15:04Z — T1 + T2 landed under it the same day (reflex `335e55e` + `5e5c044` + `162faf1`).

`scripts/bench_preflight.sh` refuses on Windows: its power axis reads `pmset` (macOS-only), so the preflight exits before the provenance line on every Windows box — plan 498's T0.1 rows carried a Windows-side substitute disclosure instead (AC system via `powercfg -list`, uncontended GPU, disclosed box state in the 0055 house format), with the gap recorded as a limitation in the lane's pre-registration.

- [x] **T1 — LANDED 2026-11-14 (zed-14c3ed6a@m3, reflex `335e55e`+`5e5c044`+`162faf1`):** the power axis branches on `uname -s` — macOS keeps pmset byte-identical (mac run verified: same `ok power AC` / `ok powermode 2` / settle / provenance `power=AC Power` lines); Windows reads `Win32_Battery.BatteryStatus` via a PowerShell probe (on-AC = 2/3/6-9/11, discharging = 1/4/5 refuse, no battery = desktop AC-assumed DISCLOSED, anything else exit 2); Linux reads `/sys/class/power_supply/A*/online` with `BAT*/status` cases (Charging/Full/Not-charging → AC, Discharging → refuse, absent/unrecognised → exit 2). Same fail-loud posture on every arm: unreadable power exits 2, never a green. Port completions the 4090 cell forced (all same commits): **load axis** — MSYS2's `/proc/loadavg` is a STUB (measured 0.00 with LoadPercentage at 24%) and Git Bash ships no `uptime`, so Windows estimates busy cores from `Win32_Processor.LoadPercentage × NumberOfLogicalProcessors` (median of 3 one-second samples, locale-normalised decimal comma, disclosed `win-est` in the note + provenance); Linux reads `/proc/loadavg` with an uptime fallback; macOS byte-identical. **powermode + settle** — macOS-only axes, disclosed n/a / UNVERIFIED warns off macOS. **canary** — macOS-only axis: `LAYA_DEVICE=metal` is meaningless off macOS and the first 4090 cell ran it and pasted garbage (`canary=base=us/best5`) into the provenance; off-macOS it now skips with a disclosure and readings are numerically validated (a non-numeric reading never enters the best-of-N). Non-macOS residuals DISCLOSED, not gated: swap reads warn unreadable on Windows (no sysctl); Linux arm is code-verified (5 logic arms exercised against a fake sysfs tree: AC / refuse-on-discharging / charging→AC / not-charging→AC / absent→exit 2) — no Linux box exists in the fleet to run it.
- [x] **T2 — RECORDED 2026-11-14 (zed-14c3ed6a@m3):** the Windows-box preflight run proves the PROVENANCE line prints on a non-macOS rig — 4090 (Windows, Git Bash MINGW64_NT-10.0-26300), reflex `162faf1`, full output:

```
ok   power        AC assumed (no battery present — desktop, disclosed)
warn powermode    n/a on MINGW64_NT-10.0-26300 (macOS-only axis, disclosed)
warn settle window UNVERIFIED — no power-transition history axis on MINGW64_NT-10.0-26300
ok   load         4.8 (win-est) (ceiling 6.0)
warn swap usage unreadable
warn canary       SKIPPED — macOS-only axis (Metal throttle detector), disclosed

PROVENANCE: power=AC-desktop load=4.8(win-est) swap=?M canary=skipped powermode=?(n/a)
✓ preflight PASSED — quote the PROVENANCE line in the bench record
```

Exit 0 (BASH_EXIT_0 verified over ssh). macOS regression arm verified the same session: `sh scripts/bench_preflight.sh` on m3 byte-identical through the power/powermode/settle/load/provenance lines (it correctly REFUSED at load 13.19/25.18 while the sibling sprint lane compiled — the refusal names load, not power). The plan-498 substitute-disclosure pattern retires.
