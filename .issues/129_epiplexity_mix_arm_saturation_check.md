# Issue 129: Epiplexity mix arm + suite saturation check on the synth-corpus lane

**Status:** OPEN — filed from katgpt-rs Research 685 (arXiv:2608.11746, EpiGen/§H.3). Eval-only; no GPU; lands on the existing `--corpus-ab` paired-LB95 gates. Claimed by: nobody yet.

## Finding

The synth lane's `--corpus-ab` compares gold vs synth two-arm. The paper's mix law (arXiv:2608.11746 Fig. 5 + App. H.3): **25–50% synthetic/real mixes beat either arm alone**, and grounding the reward/eval buffer in real reference data (a buffer that is never trained on, never grown) is what keeps a synthetic pipeline from the two collapse modes it catalogues — level-only rewards → repetition collapse (4-gram rep 0.47), buffer-less rewards → adversarial-noise collapse (frozen-scorer ppl 169.7). A two-arm comparison therefore systematically UNDERSTATES synth value here, and the lane has no saturation check on its gold corpora.

## Tasks

- [ ] T1 — three-arm `--corpus-ab` mode: gold / synth / mix (mix fraction 0.25 and 0.50 sub-arms), paired LB95, one frozen test read, echo gates unchanged (the lane dies on echo — same law the paper found independently by reward ablation; corroboration, not duplication).
- [ ] T2 — suite saturation pre-check (the paper's Pile finding as a harness gate): per suite, does the dominant corpus class alone match full-suite accuracy? If yes, print SATURATED — the suite is uninformative for any selection/allocation method, and the report says so before anyone spends teacher budget on it.
- [ ] T3 — prequential-corpus probe (report-only): corpus-is-the-model means corpus append IS the update — measure Δprobe-accuracy on a FROZEN, never-grown probe set before/after a corpus append. This is the exact prequential delta with zero training and the most faithful import of EpiGen's reward in the workspace. If it shows signal, file the follow-up for gain-weighted corpus-budget allocation (the consumer of katgpt-rs Plan 641) — do NOT wire it into serving paths (stationary-surface ban: allocation surfaces only, never the decide path).
- [ ] T4 — collapse canaries for any future learned-generator lane: the two paper collapse modes become planted refusal arms (repetition ceiling + frozen-scorer admission band); similarity×novelty conjunction is load-bearing — level-only admission is a documented negative import.

## Gates

Existing lane discipline carries: paired LB95, echo gates, learnability gate, veto. New: no exit-code change from T2/T3 (report-only); T1 rides the existing paired-verdict machinery.

## References

katgpt-rs Research 685 (arXiv:2608.11746) · riir-train Plan 506 (training-side sibling) · katgpt-rs Plan 641 (the allocation primitive this issue's T3 would consume).
