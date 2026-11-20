# Issue 125 — typed_decisions gold schema is heterogeneous per workflow: the standing sweep panics at hard_metrics

**Status:** Open — filed 2026-11-19 (zed-b7afc49e@4090, the 121 standing-sweep lane; sweep log `.benchmarks/200_standing_sweep_bm25_floor_run1.log`)
**Claimed:** zed-m3-i125floor@m3, 1795146954, expires 2026-11-22T04:00:00+00:00

## The panic

`hard_metrics: gold index 4 out of range for 4 probs` (`src/harness/metrics.rs:90`) — the sweep's FIRST suite (typed_decisions), before any floor row.

## The corpus fact (measured, test-000..003.json, 400 rows)

4 workflows x 100 rows, each with its OWN structured-JSON gold schema: `{"action": {...}}` (agent_trace_observability + customer_service), `{"discrepancy_severity": {...}}` (invoice_processing), `{"credential_compromise": {...}}` (security_incidents). The loader's class space (4 domains) and the per-workflow gold keys cannot share one index mapping — a gold from workflow B parses against workflow A's class count and lands out of range.

## Class

Loader/corpus contract defect (per-workflow label spaces need either a per-workflow class mapping in the loader or a suite split). NOT the engine-vs-floor class — this blocks the suite outright. The 121 standing sweep proceeds on the remaining 6 suites with this exclusion disclosed LOUD.
