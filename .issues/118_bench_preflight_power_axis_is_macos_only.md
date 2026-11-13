# Issue 118 — bench_preflight.sh power axis is macOS-only (`pmset`); Windows boxes refuse the preflight

**Status:** Open — unclaimed. Filed 2026-11-14 from the 4090 (plan-498 T0.1/T0.3 lane, zed-4430bf70@4090 + the on-box lane zed-5ba62a6b@4090).
**Claimed:** zed-14c3ed6a@m3 (m3), 1794604504, expires 2026-11-15T21:15:04Z

`scripts/bench_preflight.sh` refuses on Windows: its power axis reads `pmset` (macOS-only), so the preflight exits before the provenance line on every Windows box — plan 498's T0.1 rows carried a Windows-side substitute disclosure instead (AC system via `powercfg -list`, uncontended GPU, disclosed box state in the 0055 house format), with the gap recorded as a limitation in the lane's pre-registration.

- [ ] **T1 — port the power axis:** branch on OS — keep `pmset` on macOS; on Windows read the AC/battery state via `powercfg` (`powercfg /getactivespowerscheme` or `/batteryrequest`), on Linux read `/sys/class/power_supply/AC*/online`. Same fail-loud posture: an unreadable power state refuses the provenance line rather than printing a green.
- [ ] **T2 — regression row:** one Windows-box preflight run recorded (any lane) proving the PROVENANCE line prints on a non-macOS rig; the substitute-disclosure pattern retires once the port lands.
