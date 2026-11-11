#!/usr/bin/env python3
"""Paraphrase the EVAL shards of a suite set (rethink Issue 057, the ESC
contamination shadow read; Issue 063 widens it to the JDI QA suites).

- Paraphraser: gemma-3-4b-it Q4_K_M via llama-server (127.0.0.1:8097), greedy
  (temperature 0). Independent lineage from the mammoth-1 teacher (a Qwen3.5
  finetune). Residual: gemma-3 may itself have seen the public benchmarks;
  disclosed, unquantifiable. The trigram-overlap gate controls the
  surface-persistence channel.
- Only the paraphrase-surface field inside each eval shard changes; row order,
  labels, option/answer fields, train shards and splits.json are byte-
  preserved, so the harness's selection picks the SAME cases with paraphrased
  surfaces, and the seat's cal slice (train-derived) + corpus stay at the
  production posture.
- Per-suite surfaces (FIELD_MAP): classification suites paraphrase row["text"]
  (the Bench 089 posture). QA suites paraphrase the stem and byte-preserve the
  answer-bearing fields:
    winogrande_jdi - `sentence` (its exactly-one `_` blank must survive: the
      case IS the blank; option1/option2/answer untouched)
    hellaswag_jdi  - `ctx_a` paraphrased, `ctx_b` byte-verbatim (the continuity
      subject the endings agree with), `ctx` rebuilt as `ctx_a + " " + ctx_b`
      (join rule verified 8000/8000 validation rows); endings + activity_label
      untouched; ctx_a/ctx_b themselves stay original (provenance)
- Per-item gates: non-empty; not identical (case-insensitive) to the source;
  preserve-gate (each preserve token's count unchanged); char-length ratio in
  [0.4, 2.5]; word-trigram overlap <= 0.65. Up to three attempts (greedy base
  prompt, greedy stronger prompt, temp 0.7); a case that still fails keeps the
  ORIGINAL text and is flagged unchanged.
"""
import concurrent.futures as cf
import json
import os
import re
import shutil
import sys
import time
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
# Env overrides (rethink Issue 063): the wino/hswag shadow read runs the same
# instrument on a different box against the CANONICAL datasets seat (the JDI
# suites are absent from datasets_t20k). Defaults are the Bench 089 posture.
# parents[1] is the repo root from the tracked scripts/ home — the original
# .raw/para-model copy sat one level deeper, which is where parents[2] came
# from; the tracked move never re-ran the script until this issue.
SRC = Path(os.environ["PARA_SRC"]) if os.environ.get("PARA_SRC") else REPO / ".raw" / "datasets_t20k"
DST = Path(os.environ["PARA_DST"]) if os.environ.get("PARA_DST") else REPO / ".raw" / "datasets_t20k_para"
MANIFEST_DIR = Path(os.environ["PARA_MANIFEST_DIR"]) if os.environ.get("PARA_MANIFEST_DIR") else REPO / ".raw" / "para-model"
SERVER = os.environ.get("PARA_SERVER", "http://127.0.0.1:8097")
SUITES = [s.strip() for s in os.environ.get("PARA_SUITES", "massive_intent_en,banking77").split(",") if s.strip()]
# The eval-shard prefix: the JDI suites (winogrande_jdi, hellaswag_jdi) seat
# on the CANONICAL datasets' VALIDATION shards (runner.rs SuiteSpec
# eval_split: "validation" — HF test is unlabeled for winogrande), so the
# shadow read paraphrases validation-*.json there. Default "test" keeps the
# Bench 089 posture byte-identical.
EVAL_PREFIX = os.environ.get("PARA_EVAL_PREFIX", "test")
# Per-suite paraphrase surfaces; suites absent from the map use ["text"] with
# no preserve gate (the 089 posture).
FIELD_MAP = {
    "winogrande_jdi": {"sentence": {"preserve": ["_"]}},
    "hellaswag_jdi": {"ctx_join": {"a": "ctx_a", "b": "ctx_b", "into": "ctx"}},
}
SYSTEM = (
    "You paraphrase single utterances. Rewrite with different wording and "
    "sentence structure while preserving the exact same meaning and intent. "
    "Reply with ONLY the rewritten utterance - no quotes, no preamble, no "
    "explanation."
)
MAX_ATTEMPTS = 3
MAX_TOKENS = 120


def chat(text, stronger, temp):
    user = "Paraphrase this utterance: " + text
    if stronger:
        user = ("Paraphrase this utterance using completely different words "
                "where possible: " + text)
    body = json.dumps({
        "model": "gemma-3-4b-it",
        "temperature": temp,
        "max_tokens": MAX_TOKENS,
        "messages": [
            {"role": "system", "content": SYSTEM},
            {"role": "user", "content": user},
        ],
    }).encode()
    req = urllib.request.Request(
        SERVER + "/v1/chat/completions",
        data=body,
        headers={"Content-Type": "application/json"},
    )
    last = None
    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                out = json.loads(r.read().decode("utf-8"))["choices"][0]["message"]["content"]
            return out.strip().strip('"').strip()
        except Exception as e:
            last = e
            time.sleep(2 * (attempt + 1))
    raise RuntimeError("chat failed after retries: %s" % last)


def tokens(s):
    return re.findall(r"[a-z0-9']+", s.lower())


def trigram_overlap(a, b):
    ta, tb = tokens(a), tokens(b)
    if len(ta) < 3 or len(tb) < 3:
        return 1.0 if ta == tb else 0.0
    ga = set(tuple(ta[i:i + 3]) for i in range(len(ta) - 2))
    gb = set(tuple(tb[i:i + 3]) for i in range(len(tb) - 2))
    if not ga or not gb:
        return 0.0
    return len(ga & gb) / min(len(ga), len(gb))


def _gates(orig, para, preserve, flags):
    """Shared post-conditions; returns True when the candidate is accepted."""
    if para.lower() == orig.lower():
        return False
    for tok in preserve:
        if para.count(tok) != orig.count(tok):
            flags.append("preserve_miss:%s" % tok)
            return False
    if not 0.4 <= len(para) / max(len(orig), 1) <= 2.5:
        flags.append("len_ratio")
        return False
    ov = trigram_overlap(orig, para)
    if ov > 0.65:
        flags.append("high_overlap:%.2f" % ov)
        return "last"
    return True


def paraphrase_one(orig, preserve=()):
    flags, attempts = [], 0
    for attempt in range(MAX_ATTEMPTS):
        attempts += 1
        stronger = attempt == 1
        temp = 0.0 if attempt < 2 else 0.7
        para = chat(orig, stronger, temp)
        if not para:
            continue
        verdict = _gates(orig, para, preserve, flags)
        if verdict is True:
            return para, flags, attempts
        if verdict == "last" and attempt == MAX_ATTEMPTS - 1:
            return para, flags, attempts
    return orig, flags + ["unchanged"], attempts


def paraphrase_ctx_join(orig_a, orig_b):
    """Paraphrase the stem context, ctx_b byte-verbatim (Issue 063)."""
    orig_ctx = orig_a + " " + orig_b
    flags, attempts = [], 0
    for attempt in range(MAX_ATTEMPTS):
        attempts += 1
        stronger = attempt == 1
        temp = 0.0 if attempt < 2 else 0.7
        para_a = chat(orig_a, stronger, temp)
        if not para_a:
            continue
        new_ctx = para_a + " " + orig_b
        verdict = _gates(orig_ctx, new_ctx, (), flags)
        if verdict is True:
            return new_ctx, flags, attempts
        if verdict == "last" and attempt == MAX_ATTEMPTS - 1:
            return new_ctx, flags, attempts
    return orig_ctx, flags + ["unchanged"], attempts


def surface_jobs(suite, wrapper):
    """(kind, payload, orig) jobs for one row; kind: plain | ctx_join."""
    spec = FIELD_MAP.get(suite)
    if not spec:
        return [("plain", "text", wrapper["row"]["text"])]
    jobs = []
    for field, cfg in spec.items():
        if field == "ctx_join":
            jobs.append(("ctx_join", cfg, wrapper["row"][cfg["a"]] + " " + wrapper["row"][cfg["b"]]))
        else:
            jobs.append(("plain", field, wrapper["row"][field]))
    return jobs


def main():
    t0 = time.time()
    for suite in SUITES:
        src_dir, dst_dir = SRC / suite, DST / suite
        dst_dir.mkdir(parents=True, exist_ok=True)
        eval_glob = "%s-*.json" % EVAL_PREFIX
        test_files = sorted(src_dir.glob(eval_glob))
        pages = []
        for f in test_files:
            pages.append((f, json.loads(f.read_text(encoding="utf-8"))))
        n_rows = sum(len(p["rows"]) for _, p in pages)
        print("[%s] %d eval file(s), %d row(s)" % (suite, len(test_files), n_rows), flush=True)

        flat = []
        for fi, (f, page) in enumerate(pages):
            for ri, wrapper in enumerate(page["rows"]):
                for ji, (kind, payload, orig) in enumerate(surface_jobs(suite, wrapper)):
                    flat.append((fi, ri, ji, kind, payload, orig))

        done = 0
        results = {}

        def run_job(item):
            _, _, _, kind, payload, orig = item
            if kind == "ctx_join":
                a, b = wrapper_orig_a_b(item)
                return paraphrase_ctx_join(a, b)
            preserve = FIELD_MAP.get(suite, {}).get(payload, {}).get("preserve", ()) if suite in FIELD_MAP else ()
            return paraphrase_one(orig, preserve)

        # capture the row's original stem halves for the join job
        page_by_fi = {fi: p for fi, p in pages}

        def wrapper_orig_a_b(item):
            fi, ri, ji, _, cfg, _ = item
            row = page_by_fi[fi]["rows"][ri]["row"]
            return row[cfg["a"]], row[cfg["b"]]

        with cf.ThreadPoolExecutor(max_workers=8) as ex:
            futs = {ex.submit(run_job, item): item for item in flat}
            for fut in cf.as_completed(futs):
                item = futs[fut]
                results[(item[0], item[1], item[2])] = fut.result()
                done += 1
                if done % 250 == 0:
                    print("[%s] %d/%d (%.0fs)" % (suite, done, len(flat), time.time() - t0), flush=True)

        mpath = MANIFEST_DIR / ("para_manifest_%s.jsonl" % suite)
        unchanged = high_overlap = flagged_len = 0
        with mpath.open("w", encoding="utf-8") as mf:
            for fi, (f, page) in enumerate(pages):
                for ri, wrapper in enumerate(page["rows"]):
                    jobs = surface_jobs(suite, wrapper)
                    for ji, (kind, payload, orig) in enumerate(jobs):
                        para, flags, attempts = results[(fi, ri, ji)]
                        field = payload if kind == "plain" else FIELD_MAP[suite]["ctx_join"]["into"]
                        if "unchanged" in flags:
                            unchanged += 1
                        if any(x.startswith("high_overlap") for x in flags):
                            high_overlap += 1
                        if "len_ratio" in flags:
                            flagged_len += 1
                        if para != orig:
                            if kind == "ctx_join":
                                wrapper["row"][FIELD_MAP[suite]["ctx_join"]["into"]] = para
                            else:
                                wrapper["row"][payload] = para
                        mf.write(json.dumps({
                            "file": f.name,
                            "row_idx": wrapper["row_idx"],
                            "field": field,
                            "orig": orig,
                            "para": para,
                            "flags": flags,
                            "attempts": attempts,
                        }, ensure_ascii=False) + "\n")
                (dst_dir / f.name).write_text(
                    json.dumps(page, ensure_ascii=False), encoding="utf-8"
                )
            mf.write(json.dumps({
                "summary": suite,
                "rows": n_rows,
                "unchanged_kept_original": unchanged,
                "high_overlap_accepted": high_overlap,
                "len_ratio_flagged": flagged_len,
            }) + "\n")
        print(
            "[%s] written: unchanged=%d high_overlap=%d len_ratio=%d -> %s"
            % (suite, unchanged, high_overlap, flagged_len, dst_dir),
            flush=True,
        )

    for suite in SUITES:
        src_dir, dst_dir = SRC / suite, DST / suite
        for f in sorted(src_dir.iterdir()):
            if f.name.startswith(EVAL_PREFIX + "-"):
                continue
            target = dst_dir / f.name
            if not target.exists():
                shutil.copy2(f, target)
        print("[%s] non-eval files copied verbatim" % suite, flush=True)
    print("ALL DONE in %.0fs" % (time.time() - t0), flush=True)


if __name__ == "__main__":
    sys.exit(main())
