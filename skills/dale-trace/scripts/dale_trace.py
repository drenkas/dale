#!/usr/bin/env python3
"""Dale Trace: draw one chat's agent tree as a self-contained HTML page.

Reads the history of one chat and the agents it created from T3 Code, Claude
Code, or Codex, read-only, and writes an HTML page plus an optional JSON model.
Python 3.9+ standard library only. Run it as `python3 dale_trace.py`; do not
rely on a shebang or the executable bit.

Never exported: prompts, results, messages, file paths, working directories.
"""
from __future__ import annotations

import argparse
import calendar
import glob
import html
import json
import os
import re
import shutil
import sqlite3
import subprocess
import sys
import time
from collections import deque
from pathlib import Path
from urllib.request import pathname2url

sys.setrecursionlimit(20000)

SCHEMA = "dale-trace/1"
SKILL_DIR = Path(__file__).resolve().parent.parent
TEMPLATE_PATH = SKILL_DIR / "assets" / "trace-template.html"
PRICE_URL = (
    "https://raw.githubusercontent.com/BerriAI/litellm/main/"
    "model_prices_and_context_window.json"
)
PRICE_NOTE = "API list-price equivalent (estimate), not what you paid"
USAGE_FIELDS = ("input", "cache_read", "cache_write_5m", "cache_write_1h", "output")
STALE_PRICE_DAYS = 30
ONE_HOUR_MS = 3600000
# Warnings that mean the priced figure is a lower bound, so the cost is shown as partial.
COST_FLOOR_CODES = ("cache_ttl_unsplit", "tiered_price_ignored", "fast_mode_premium_ignored")
MAX_TITLE = 120
PRIVATE_KINDS = ("api-request",)

WARN_TEXT = {
    "cache_ttl_unsplit": "T3 recorded no prompt-cache lifetime for some turns; their cache writes are priced at the 5-minute rate, so the cost is a lower bound.",
    "transcript_missing": "The Claude transcript for this agent was not found; its usage is unknown.",
    "workflow_record_missing": "The workflow record was not found; the usage of its agents is unknown.",
    "unmapped_workflow_agents": "Transcripts in the workflow run folder that no record entry references were added to the workflow node.",
    "orphan_by_lineage": "This agent was not reached through a parent link and was attached by its lineage record.",
    "node_running": "This agent is still running; its open turn is excluded from the active time.",
    "usage_partial": "Some turns of this agent have no complete token record.",
    "price_missing": "No price was found for this model; its cost is unknown.",
    "rate_missing": "A price field this model needs is missing; its cost is unknown.",
    "tiered_price_ignored": "The model has token-tier prices; base rates were applied, so the estimate may be low.",
    "fast_mode_premium_ignored": "Fast mode was used; its price premium is not included.",
    "prices_stale": "The price file is more than 30 days old.",
    "no_prices": "No price source was available; all costs are unknown.",
    "zstd_unavailable": "A compressed Codex rollout could not be read without zstd; only the total token count is known.",
    "rollout_missing": "The Codex rollout file was not found; only the total token count is known.",
    "codex_tokens_mismatch": "The rollout total differs from the thread's recorded token count.",
    "codex_total_reset": "A Codex running total went down; the new value was taken as a fresh base.",
    "claude_cost_state_mismatch": "The tree totals differ from Claude Code's own token record by more than 0.5%.",
    "parent_not_found": "The tool call that started this agent was not found; it is attached to the root.",
    "title_from_prompt": "The chat has no name; --prompt-titles put the start of its first prompt in the title.",
    "db_immutable_snapshot": "The database was opened as an immutable snapshot; the newest writes may be missing.",
    "skipped_lines": "Some history lines could not be parsed and were skipped.",
    "other_usage_unattributed": "T3 recorded usage outside the turn totals; it is shown as unattributed and not priced.",
    "unknown_model": "No model name was recorded for some of this agent's usage.",
}


class TraceError(Exception):
    def __init__(self, code, message):
        Exception.__init__(self, message)
        self.code = code
        self.message = message


# --------------------------------------------------------------------------
# small helpers
# --------------------------------------------------------------------------

_ISO = re.compile(
    r"^(\d{4})-(\d\d)-(\d\d)[T ](\d\d):(\d\d):(\d\d)(?:\.(\d+))?\s*(Z|[+-]\d\d(?::?\d\d)?)?$"
)


def parse_ts(value):
    """ISO string, epoch seconds or epoch milliseconds -> integer ms, or None."""
    if value is None or isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        n = float(value)
        return int(n if n > 1e11 else n * 1000)
    m = _ISO.match(str(value).strip())
    if not m:
        return None
    y, mo, d, h, mi, s, frac, tz = m.groups()
    ms = int((frac or "0")[:3].ljust(3, "0"))
    base = calendar.timegm((int(y), int(mo), int(d), int(h), int(mi), int(s), 0, 0, 0)) * 1000 + ms
    if tz and tz != "Z":
        sign = 1 if tz[0] == "+" else -1
        digits = tz[1:].replace(":", "")
        off = int(digits[:2]) * 60 + (int(digits[2:4]) if len(digits) >= 4 else 0)
        base -= sign * off * 60000
    return base


def iso(ms):
    if ms is None:
        return None
    t = time.gmtime(ms // 1000)
    return "%04d-%02d-%02dT%02d:%02d:%02d.%03dZ" % (
        t.tm_year, t.tm_mon, t.tm_mday, t.tm_hour, t.tm_min, t.tm_sec, ms % 1000,
    )


def as_int(v):
    if isinstance(v, bool):
        return 0
    if isinstance(v, (int, float)):
        return int(v)
    return 0


def tilde(path):
    p = str(path)
    home = str(Path.home())
    if p == home:
        return "~"
    if p.startswith(home + os.sep):
        return "~" + p[len(home):]
    return p


def clip(text, limit):
    text = " ".join(str(text or "").split())
    if len(text) <= limit:
        return text
    return text[: limit - 1].rstrip() + "\u2026"


def norm_status(value):
    s = str(value or "").lower()
    if s in ("completed", "complete", "succeeded", "success", "done", "finished"):
        return "completed"
    if s in ("running", "in_progress", "starting", "active", "working", "pending", "queued", "waiting", "preparing"):
        return "running"
    if s in ("failed", "error", "errored"):
        return "failed"
    if s in ("cancelled", "canceled", "interrupted", "stopped", "aborted"):
        return "cancelled"
    return "unknown"


def provider_of(model, hint=None):
    m = (model or "").lower()
    if m.startswith("claude") or "anthropic" in m:
        return "anthropic"
    if m.startswith("gpt") or re.match(r"^o\d", m) or "openai" in m or "codex" in m:
        return "openai"
    if m.startswith("gemini") or "google" in m:
        return "google"
    h = (hint or "").lower()
    if "claude" in h or "anthropic" in h:
        return "anthropic"
    if "codex" in h or "openai" in h:
        return "openai"
    if "gemini" in h or "google" in h:
        return "google"
    return "unknown"


def zero_usage():
    return {"input": 0, "cache_read": 0, "cache_write_5m": 0, "cache_write_1h": 0,
            "output": 0, "reasoning": 0, "total": 0}


def finish_usage(u):
    u["total"] = sum(u[f] for f in USAGE_FIELDS)
    return u


def add_usage(dst, src):
    for k in USAGE_FIELDS + ("reasoning",):
        dst[k] += src.get(k) or 0
    return finish_usage(dst)


def merge_by_model(dst, src):
    for model, u in src.items():
        add_usage(dst.setdefault(model, zero_usage()), u)


def safe_id(value):
    return re.sub(r"[^A-Za-z0-9]", "", str(value))[:8] or "chat"


# --------------------------------------------------------------------------
# tree + warnings
# --------------------------------------------------------------------------

class Tree:
    def __init__(self):
        self.nodes = {}
        self.order = []
        self.warnings = []
        self._seen = set()
        self.global_warnings = []

    def add(self, key, parent, kind, origin, title):
        node = {
            "id": None, "source_id": key, "parent_id": parent, "depth": 0, "children": [],
            "kind": kind, "origin": origin, "provider": "unknown",
            "title": clip(title, MAX_TITLE) or "untitled", "role": None,
            "models": [], "effort": None, "status": "unknown",
            "started_at": None, "ended_at": None, "span_ms": None, "active_ms": None,
            "usage": {"by_model": {}, "total": None},
            "usage_status": "unknown", "usage_source": "none",
            "cost": {"usd": None, "by_model": {}, "status": "unknown"},
            "subtree": {}, "unattributed": None, "attempts": None, "notes": [],
            "_key": key, "_start": None, "_end": None, "_fast": False,
        }
        self.nodes[key] = node
        self.order.append(key)
        return node

    def warn(self, code, node=None, detail=None):
        key = (code, node["_key"] if node else None)
        if key in self._seen:
            return
        self._seen.add(key)
        message = WARN_TEXT.get(code, code)
        if detail:
            message = "%s (%s)" % (message, detail)
        self.warnings.append({"code": code, "node_id": node["_key"] if node else None, "message": message})
        if node is not None and code not in node["notes"]:
            node["notes"].append(code)

    def set_times(self, node, start, end):
        node["_start"], node["_end"] = start, end
        node["started_at"], node["ended_at"] = iso(start), iso(end)
        node["span_ms"] = end - start if start is not None and end is not None and end >= start else None


def set_usage(node, by_model, status, source):
    node["usage"]["by_model"] = by_model
    total = zero_usage()
    for u in by_model.values():
        add_usage(total, u)
    node["usage"]["total"] = total if by_model else None
    node["usage_status"] = status
    node["usage_source"] = source
    if by_model:
        node["models"] = sorted(by_model.keys())


# --------------------------------------------------------------------------
# pricing
# --------------------------------------------------------------------------

RATE_FIELDS = {
    "input": "input_cost_per_token",
    "cache_read": "cache_read_input_token_cost",
    "cache_write_5m": "cache_creation_input_token_cost",
    "cache_write_1h": "cache_creation_input_token_cost_above_1hr",
    "output": "output_cost_per_token",
}
_TIER = re.compile(
    r"^(input_cost_per_token|output_cost_per_token|cache_read_input_token_cost|"
    r"cache_creation_input_token_cost)_above_(\d+)k_tokens$"
)


class Prices:
    def __init__(self, doc=None, source="none", label=None, fetched_ms=None):
        self.doc = doc if isinstance(doc, dict) else {}
        self.source = source if self.doc else "none"
        self.label = label
        self.fetched_ms = fetched_ms

    def lookup(self, model):
        if not self.doc or not model:
            return None
        cands = [model]
        stripped = re.sub(r"\[[^\]]*\]$", "", model)
        cands.append(stripped)
        cands.append(re.sub(r"-\d{8}$", "", stripped))
        seen = set()
        for cand in cands:
            if cand in seen:
                continue
            seen.add(cand)
            for prefix in ("", "anthropic/", "openai/", "gemini/", "vertex_ai/"):
                entry = self.doc.get(prefix + cand)
                if isinstance(entry, dict):
                    return entry
        return None

    def cost(self, model, usage, max_request_input=None):
        """-> (usd or None, [warning codes with detail]).

        max_request_input is the largest input of one request, when the source
        records it: a price tier above it cannot have applied.
        """
        entry = self.lookup(model)
        if entry is None:
            return None, [("price_missing", model)]
        notes = []
        tiers = [int(m.group(2)) * 1000 for m in (_TIER.match(k) for k in entry) if m]
        if tiers and (max_request_input is None or max_request_input > min(tiers)):
            notes.append(("tiered_price_ignored", model))
        usd = 0.0
        for field, rate_key in RATE_FIELDS.items():
            count = usage.get(field) or 0
            if count <= 0:
                continue
            rate = entry.get(rate_key)
            if not isinstance(rate, (int, float)) or isinstance(rate, bool):
                return None, notes + [("rate_missing", "%s: %s" % (model, rate_key))]
            usd += count * float(rate)
        return round(usd, 10), notes


def candidate_t3_homes(explicit):
    """An explicit --t3-home is exclusive; otherwise $T3CODE_HOME, ~/.t3, ~/.sigma-t3 in that order."""
    homes = []
    for h in ((explicit,) if explicit else (os.environ.get("T3CODE_HOME"), "~/.t3", "~/.sigma-t3")):
        if not h:
            continue
        p = Path(os.path.expanduser(h))
        if p not in homes:
            homes.append(p)
    return homes


def load_prices(args, tree):
    if args.prices:
        try:
            raw = json.loads(Path(os.path.expanduser(args.prices)).read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            raise TraceError(4, "cannot read --prices file: %s" % exc)
        fetched = None
        doc = raw
        if isinstance(raw, dict) and isinstance(raw.get("document"), dict):
            doc, fetched = raw["document"], raw.get("fetchedAtMs")
        label = Path(args.prices).name
        return Prices(doc, "file", label, parse_ts(fetched) if fetched else None)
    best = None
    for home in candidate_t3_homes(args.t3_home):
        path = home / "userdata" / "usage-model-rates.json"
        try:
            raw = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if not isinstance(raw, dict) or not isinstance(raw.get("document"), dict):
            continue
        fetched = as_int(raw.get("fetchedAtMs"))
        if best is None or fetched > best[1]:
            best = (raw["document"], fetched, "usage-model-rates.json")
    if best:
        return Prices(best[0], "t3-rates", best[2], best[1] or None)
    if args.fetch_prices:
        from urllib.request import Request, urlopen

        try:
            req = Request(PRICE_URL, headers={"User-Agent": "dale-trace"})
            with urlopen(req, timeout=20) as resp:
                doc = json.loads(resp.read().decode("utf-8"))
        except Exception as exc:  # network is opt-in; report and continue unpriced
            tree.global_warnings.append(("no_prices", "price fetch failed: %s" % type(exc).__name__))
            return Prices()
        return Prices(doc, "litellm-fetch", "model_prices_and_context_window.json", int(time.time() * 1000))
    return Prices()


# --------------------------------------------------------------------------
# read-only database access
# --------------------------------------------------------------------------

def open_ro(path, tree):
    base = "file:" + pathname2url(str(path)) + "?mode=ro"
    last = None
    for suffix, mode in (("", "ro"), ("&immutable=1", "immutable")):
        try:
            conn = sqlite3.connect(base + suffix, uri=True, timeout=2, isolation_level=None)
            conn.execute("PRAGMA query_only=ON")
            conn.execute("SELECT 1 FROM sqlite_master LIMIT 1").fetchall()
            conn.execute("BEGIN")
            if mode == "immutable":
                tree.global_warnings.append(("db_immutable_snapshot", tilde(path)))
            return conn, mode
        except sqlite3.Error as exc:
            last = exc
    raise TraceError(4, "cannot open %s read-only: %s" % (tilde(path), last))


def close_ro(conn):
    try:
        conn.execute("ROLLBACK")
    except sqlite3.Error:
        pass
    conn.close()


def require_columns(conn, table, columns):
    have = {r[1] for r in conn.execute("PRAGMA table_info(%s)" % table)}
    if not have:
        raise TraceError(4, "required table %s is missing" % table)
    for c in columns:
        if c not in have:
            raise TraceError(4, "required column %s.%s is missing" % (table, c))


def jx(path):
    return "CASE WHEN json_valid(payload_json) THEN json_extract(payload_json,'%s') END" % path


def loads(text):
    if text is None:
        return None
    if isinstance(text, (dict, list, int, float)):
        return text
    try:
        return json.loads(text)
    except ValueError:
        return text


# --------------------------------------------------------------------------
# Claude transcripts (shared by Claude Code and T3 delegated Claude agents)
# --------------------------------------------------------------------------

SKIP_HEAD = ('"api-request', '"queue-operation', '"attachment"', '"last-prompt"', '"tools-used"')


def parse_claude_transcript(path, main=False):
    """Stream one transcript. Returns None if unreadable."""
    msgs = {}
    out = {"by_model": {}, "first": None, "last": None, "turn_ms": None, "skipped": 0,
           "fast": False, "tool_uses": set(), "agent_results": {}, "custom_title": None,
           "ai_title": None, "cost_state": None, "messages": 0}
    try:
        fh = open(str(path), "r", encoding="utf-8", errors="replace")
    except OSError:
        return None
    with fh:
        for line in fh:
            head = line[:40]
            if any(s in head for s in SKIP_HEAD):
                continue
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except ValueError:
                out["skipped"] += 1
                continue
            if not isinstance(r, dict):
                continue
            t = r.get("type")
            if main and r.get("isSidechain") is True and t in ("assistant", "user", "system"):
                continue
            if t in ("assistant", "user", "system"):
                ts = parse_ts(r.get("timestamp"))
                if ts is not None:
                    out["first"] = ts if out["first"] is None else min(out["first"], ts)
                    out["last"] = ts if out["last"] is None else max(out["last"], ts)
            if t == "assistant":
                msg = r.get("message")
                if not isinstance(msg, dict):
                    continue
                content = msg.get("content")
                if isinstance(content, list):
                    for b in content:
                        if isinstance(b, dict) and b.get("type") == "tool_use" and b.get("name") in ("Agent", "Task"):
                            if isinstance(b.get("id"), str):
                                out["tool_uses"].add(b["id"])
                usage = msg.get("usage")
                model = msg.get("model")
                if not isinstance(usage, dict) or model == "<synthetic>":
                    continue
                if usage.get("speed") == "fast":
                    out["fast"] = True
                cc = usage.get("cache_creation") if isinstance(usage.get("cache_creation"), dict) else {}
                cw_total = as_int(usage.get("cache_creation_input_tokens"))
                cw_1h = as_int(cc.get("ephemeral_1h_input_tokens"))
                details = usage.get("output_tokens_details") if isinstance(usage.get("output_tokens_details"), dict) else {}
                fields = {
                    "input": as_int(usage.get("input_tokens")),
                    "cache_read": as_int(usage.get("cache_read_input_tokens")),
                    "cache_write_1h": cw_1h,
                    "cache_write_5m": max(0, cw_total - cw_1h),
                    "output": as_int(usage.get("output_tokens")),
                    "reasoning": as_int(details.get("thinking_tokens")),
                }
                mid = msg.get("id") if isinstance(msg.get("id"), str) else r.get("uuid")
                prev = msgs.get(mid)
                if prev is None:
                    msgs[mid] = {"model": model or "unknown", "u": fields}
                else:
                    prev["model"] = model or prev["model"]
                    for k, v in fields.items():
                        prev["u"][k] = max(prev["u"][k], v)
            elif t == "user":
                tr = r.get("toolUseResult")
                if isinstance(tr, dict) and isinstance(tr.get("agentId"), str):
                    out["agent_results"][tr["agentId"]] = {
                        "duration_ms": tr.get("totalDurationMs") if isinstance(tr.get("totalDurationMs"), (int, float)) else None,
                        "model": tr.get("resolvedModel") if isinstance(tr.get("resolvedModel"), str) else None,
                    }
            elif t == "system":
                if r.get("subtype") == "turn_duration" and isinstance(r.get("durationMs"), (int, float)):
                    out["turn_ms"] = (out["turn_ms"] or 0) + int(r["durationMs"])
            elif t == "custom-title" and isinstance(r.get("customTitle"), str):
                out["custom_title"] = r["customTitle"]
            elif t == "ai-title" and isinstance(r.get("aiTitle"), str):
                out["ai_title"] = r["aiTitle"]
            elif t == "cost-state" and isinstance(r.get("modelUsage"), dict):
                out["cost_state"] = r["modelUsage"]
    for m in msgs.values():
        u = m["u"]
        target = out["by_model"].setdefault(m["model"], zero_usage())
        for k, v in u.items():
            target[k] += v
        finish_usage(target)
    out["messages"] = len(msgs)
    return out


def claude_agent_type(transcript_path):
    """The agent type Claude Code recorded beside a subagent transcript, or None."""
    meta_path = str(transcript_path)[:-len(".jsonl")] + ".meta.json"
    try:
        with open(meta_path, "r", encoding="utf-8") as fh:
            meta = json.load(fh)
    except (OSError, ValueError):
        return None
    kind = meta.get("agentType") if isinstance(meta, dict) else None
    return clip(kind, 40) if isinstance(kind, str) and kind else None


def find_claude_session_dir(claude_home, session):
    pattern = str(Path(claude_home) / "projects" / "*" / glob.escape(session))
    for p in sorted(glob.glob(pattern)):
        if os.path.isdir(p):
            return Path(p)
    return None


def find_claude_session_file(claude_home, session):
    pattern = str(Path(claude_home) / "projects" / "*" / (glob.escape(session) + ".jsonl"))
    hits = sorted(glob.glob(pattern))
    return Path(hits[0]) if hits else None


def load_workflow_records(session_dir):
    """taskId -> trimmed record. Only the keys needed are kept."""
    recs = {}
    if session_dir is None:
        return recs
    for p in sorted(glob.glob(str(session_dir / "workflows" / "*.json"))):
        try:
            with open(p, "r", encoding="utf-8") as fh:
                raw = json.load(fh)
        except (OSError, ValueError):
            continue
        if not isinstance(raw, dict):
            continue
        agents = {}
        progress = raw.get("workflowProgress")
        for e in progress if isinstance(progress, list) else []:
            if isinstance(e, dict) and e.get("type") == "workflow_agent" and isinstance(e.get("index"), int):
                agents.setdefault(e["index"], []).append({
                    k: e.get(k) for k in ("agentId", "label", "model", "phaseTitle", "attempt",
                                          "startedAt", "lastProgressAt", "durationMs", "state")
                })
        rec = {
            "runId": raw.get("runId") if isinstance(raw.get("runId"), str) else Path(p).stem,
            "taskId": raw.get("taskId"),
            "name": raw.get("workflowName") if isinstance(raw.get("workflowName"), str) else "workflow",
            "status": raw.get("status"),
            "durationMs": raw.get("durationMs"),
            "startTime": raw.get("startTime"),
            "agents": agents,
        }
        del raw
        recs[str(rec["taskId"]) if rec["taskId"] else rec["runId"]] = rec
    return recs


def workflow_agent_usage(session_dir, rec, index, tree, node):
    """Sum all attempts of one workflow agent. Returns (by_model, found, expected, fast)."""
    by_model, found, fast = {}, 0, False
    entries = sorted(rec["agents"].get(index, []), key=lambda e: as_int(e.get("attempt")))
    for e in entries:
        aid = e.get("agentId")
        if not isinstance(aid, str):
            continue
        p = session_dir / "subagents" / "workflows" / rec["runId"] / ("agent-" + aid + ".jsonl")
        tr = parse_claude_transcript(p)
        if tr is None:
            continue
        found += 1
        fast = fast or tr["fast"]
        merge_by_model(by_model, tr["by_model"])
        if tr["skipped"]:
            tree.warn("skipped_lines", node)
    return by_model, found, len(entries), fast


def apply_workflow_agent(tree, node, session_dir, rec, index):
    entries = sorted(rec["agents"].get(index, []), key=lambda e: as_int(e.get("attempt")))
    node["kind"] = "workflow_agent"
    if entries:
        last = entries[-1]
        if isinstance(last.get("label"), str):
            node["title"] = clip(last["label"], MAX_TITLE)
        if isinstance(last.get("phaseTitle"), str):
            node["role"] = clip(last["phaseTitle"], 40)
        if isinstance(last.get("model"), str):
            node["models"] = sorted(set(node["models"]) | {last["model"]})
        node["attempts"] = len(entries)
        durations = [e["durationMs"] for e in entries if isinstance(e.get("durationMs"), (int, float))]
        if durations:
            node["active_ms"] = int(sum(durations))
        starts = [parse_ts(e.get("startedAt")) for e in entries if parse_ts(e.get("startedAt")) is not None]
        ends = [parse_ts(e.get("lastProgressAt")) for e in entries if parse_ts(e.get("lastProgressAt")) is not None]
        if node["_start"] is None and starts and ends:
            tree.set_times(node, min(starts), max(ends))
        states = {norm_status(e.get("state")) for e in entries}
        if node["status"] == "unknown":
            node["status"] = "running" if "running" in states else (entries and norm_status(entries[-1].get("state"))) or "unknown"
    by_model, found, expected, fast = workflow_agent_usage(session_dir, rec, index, tree, node)
    if found == 0:
        tree.warn("transcript_missing", node)
        node["usage_status"], node["usage_source"] = "unknown", "none"
    else:
        status = "complete" if found == expected and expected > 0 else "partial"
        set_usage(node, by_model, status, "claude-transcript")
        if status == "partial":
            tree.warn("transcript_missing", node, "%d of %d attempts" % (found, expected))
    if fast:
        node["_fast"] = True


def workflow_unmapped(tree, wf_node, session_dir, rec):
    folder = session_dir / "subagents" / "workflows" / rec["runId"]
    referenced = set()
    for entries in rec["agents"].values():
        for e in entries:
            if isinstance(e.get("agentId"), str):
                referenced.add(e["agentId"])
    by_model, count = {}, 0
    for p in sorted(glob.glob(str(folder / "agent-*.jsonl"))):
        aid = Path(p).name[len("agent-"):-len(".jsonl")]
        if aid in referenced:
            continue
        tr = parse_claude_transcript(p)
        if tr is None:
            continue
        count += 1
        merge_by_model(by_model, tr["by_model"])
    if count:
        set_usage(wf_node, by_model, "complete", "claude-transcript")
        tree.warn("unmapped_workflow_agents", wf_node, "%d transcripts" % count)


# --------------------------------------------------------------------------
# T3 Code
# --------------------------------------------------------------------------

T3_TABLES = {
    "orchestration_v2_projection_threads": ["thread_id", "title", "created_at", "updated_at", "deleted_at",
                                           "archived_at", "provider_instance_id", "payload_json"],
    "orchestration_v2_projection_subagents": ["subagent_id", "thread_id", "child_thread_id", "run_id", "origin",
                                              "driver", "provider_instance_id", "status", "started_at",
                                              "completed_at", "payload_json"],
    "orchestration_v2_projection_provider_turns": ["thread_id", "started_at", "completed_at", "status", "payload_json"],
    "orchestration_v2_projection_provider_threads": ["thread_id", "driver", "updated_at", "payload_json"],
    "orchestration_v2_projection_runs": ["thread_id", "status", "requested_at", "completed_at", "payload_json"],
}
T3_ROOT_INDEX = "orchestration_v2_projection_threads_root_thread_idx"

Q_T3_THREAD = (
    "SELECT thread_id,title,created_at,updated_at,deleted_at,provider_instance_id,"
    + jx("$.lineage") + "," + jx("$.modelSelection")
    + " FROM orchestration_v2_projection_threads WHERE thread_id=?"
)
Q_T3_CHILDREN = (
    "SELECT subagent_id,child_thread_id,run_id,origin,driver,provider_instance_id,status,started_at,completed_at,"
    + jx("$.title") + "," + jx("$.model") + "," + jx("$.modelSelection") + "," + jx("$.nativeTaskRef")
    + " FROM orchestration_v2_projection_subagents WHERE thread_id=?"
)
Q_T3_PARENT_OF = "SELECT thread_id FROM orchestration_v2_projection_subagents WHERE child_thread_id=?"
Q_T3_TURNS = (
    "SELECT started_at,completed_at,status," + jx("$.model") + "," + jx("$.turnTokenUsage")
    + "," + jx("$.tokenUsage.promptCacheTtlMs") + " FROM orchestration_v2_projection_provider_turns WHERE thread_id=?"
)
Q_T3_PROVIDER_THREADS = (
    "SELECT driver," + jx("$.nativeThreadRef.nativeId")
    + " FROM orchestration_v2_projection_provider_threads WHERE thread_id=?"
)
Q_T3_RUNS = (
    "SELECT status,requested_at,completed_at," + jx("$.startedAt")
    + " FROM orchestration_v2_projection_runs WHERE thread_id=?"
)
Q_T3_ROOT_MEMBERS = (
    "SELECT thread_id FROM orchestration_v2_projection_threads WHERE "
    "CASE WHEN json_valid(payload_json) THEN json_extract(payload_json,'$.lineage.rootThreadId') END = ?"
)
Q_T3_LIST = (
    "SELECT thread_id,title,updated_at FROM orchestration_v2_projection_threads "
    "WHERE deleted_at IS NULL AND archived_at IS NULL "
    "AND CASE WHEN json_valid(payload_json) THEN json_extract(payload_json,'$.lineage.parentThreadId') END IS NULL "
    "ORDER BY updated_at DESC LIMIT ?"
)
T3_QUERIES = {
    "thread": Q_T3_THREAD, "children": Q_T3_CHILDREN, "parent_of": Q_T3_PARENT_OF, "turns": Q_T3_TURNS,
    "provider_threads": Q_T3_PROVIDER_THREADS, "runs": Q_T3_RUNS, "root_members": Q_T3_ROOT_MEMBERS,
}

NATIVE_AGENT = re.compile(r"^a[0-9a-f]{6,}$")
WORKFLOW_TASK = re.compile(r"^w[a-z0-9]{5,}$")
AGENT_SUFFIX = re.compile(r":agent:(\d+)$")
UUIDISH = re.compile(r"^[0-9a-fA-F-]{32,40}$")


def t3_db_path(home):
    return Path(home) / "userdata" / "statev2.sqlite"


def t3_check_schema(conn):
    for table, cols in T3_TABLES.items():
        require_columns(conn, table, cols)


def t3_thread_row(conn, tid):
    r = conn.execute(Q_T3_THREAD, (tid,)).fetchone()
    if r is None:
        return None
    return {"id": r[0], "title": r[1], "created_at": r[2], "updated_at": r[3], "deleted_at": r[4],
            "instance": r[5], "lineage": loads(r[6]) or {}, "selection": loads(r[7]) or {}}


def t3_has(conn, tid):
    if t3_thread_row(conn, tid) is not None:
        return True
    return conn.execute(Q_T3_PARENT_OF, (tid,)).fetchone() is not None


def t3_find_root(conn, tid):
    """Resolve a thread to the root of its tree. Returns (root_id, resolved_from)."""
    start = tid
    for _ in range(64):
        row = t3_thread_row(conn, tid)
        if row is None:
            parent = conn.execute(Q_T3_PARENT_OF, (tid,)).fetchone()
            if not parent:
                break
            tid = parent[0]
            continue
        lin = row["lineage"] if isinstance(row["lineage"], dict) else {}
        root = lin.get("rootThreadId")
        if isinstance(root, str) and root != tid and t3_thread_row(conn, root) is not None:
            tid = root
            break
        parent = lin.get("parentThreadId")
        if not parent:
            break
        tid = parent
    return tid, (start if tid != start else None)


def t3_selection(sel):
    model, effort, fast = None, None, False
    if isinstance(sel, dict):
        model = sel.get("model") if isinstance(sel.get("model"), str) else None
        for o in sel.get("options") if isinstance(sel.get("options"), list) else []:
            if isinstance(o, dict):
                if o.get("id") in ("effort", "reasoningEffort") and isinstance(o.get("value"), str):
                    effort = o["value"]
                if o.get("id") == "fastMode" and o.get("value") is True:
                    fast = True
    return model, effort, fast


def t3_turn_usage(turns, node, tree):
    """Fold the turn rows of one thread into the node. Returns True if any turn row exists."""
    if not turns:
        return False
    by_model, complete, partial = {}, 0, 0
    active, running, starts, ends = 0, False, [], []
    other_in, other_out, seen_other = 0, 0, False
    fallback_model = node["models"][0] if node["models"] else "unknown"
    for started, completed, status, model, usage_raw, cache_ttl in turns:
        s, e = parse_ts(started), parse_ts(completed)
        if s is not None:
            starts.append(s)
        if e is not None:
            ends.append(e)
        if s is not None and e is not None and e >= s:
            active += e - s
        elif s is not None and e is None:
            running = True
        usage = loads(usage_raw)
        if not isinstance(usage, dict) or not isinstance(usage.get("inputTokens"), (int, float)):
            partial += 1
            continue
        m = model if isinstance(model, str) and model else fallback_model
        cr, cw = as_int(usage.get("cachedInputTokens")), as_int(usage.get("cacheCreationTokens"))
        u = zero_usage()
        u["cache_read"] = cr
        # T3 records the prompt-cache lifetime of the turn, not a split per write.
        ttl_known = isinstance(cache_ttl, (int, float)) and not isinstance(cache_ttl, bool) and cache_ttl > 0
        if ttl_known and cache_ttl >= ONE_HOUR_MS:
            u["cache_write_1h"] = cw
        else:
            u["cache_write_5m"] = cw
        u["input"] = max(0, as_int(usage.get("inputTokens")) - cr - cw)
        u["output"] = as_int(usage.get("outputTokens"))
        u["reasoning"] = as_int(usage.get("reasoningTokens"))
        finish_usage(u)
        add_usage(by_model.setdefault(m, zero_usage()), u)
        if cw and not ttl_known:
            tree.warn("cache_ttl_unsplit", node)
        if usage.get("usageStatus") == "complete":
            complete += 1
        else:
            partial += 1
        other = usage.get("otherUsage")
        if isinstance(other, dict):
            oi, oo = as_int(other.get("inputTokens")), as_int(other.get("outputTokens"))
            if oi or oo:
                seen_other = True
                other_in += oi
                other_out += oo
    status = "unknown" if not by_model else ("complete" if partial == 0 else "partial")
    set_usage(node, by_model, status, "t3-turns" if by_model else "none")
    if partial and by_model:
        tree.warn("usage_partial", node)
    if seen_other:
        node["unattributed"] = {"input": other_in, "output": other_out}
        tree.warn("other_usage_unattributed", node)
    node["active_ms"] = active
    if running:
        node["status"] = "running"
        tree.warn("node_running", node)
    elif node["status"] == "unknown":
        node["status"] = "completed"
    if starts and node["_start"] is None:
        tree.set_times(node, min(starts), None if running or not ends else max(ends))
    return True


def t3_claude_sessions(conn, tid):
    sessions = []
    for driver, native in conn.execute(Q_T3_PROVIDER_THREADS, (tid,)):
        if isinstance(native, str) and UUIDISH.match(native) and "claude" in str(driver or "").lower():
            if native not in sessions:
                sessions.append(native)
    return sessions


def build_t3(conn, root_id, claude_home, tree):
    """BFS over the tree of one T3 chat. Returns the root node."""
    t3_check_schema(conn)
    has_root_idx = conn.execute("SELECT 1 FROM sqlite_master WHERE name=?", (T3_ROOT_INDEX,)).fetchone() is not None
    queue = deque([(root_id, None, None, [], None)])
    seen = set()
    root_node = None
    session_cache = {}
    wf_cache = {}
    while True:
        while queue:
            key, parent_key, sub, inherited, orphan = queue.popleft()
            if key in seen:
                continue
            seen.add(key)
            row = t3_thread_row(conn, key)
            is_root = parent_key is None
            title = (row or {}).get("title") or (sub or {}).get("title") or "untitled"
            origin = "root" if is_root else {"app_owned": "app_owned", "provider_native": "provider_native"}.get(
                (sub or {}).get("origin"), "app_owned" if sub is None else "provider_native")
            node = tree.add(key, parent_key, "chat" if is_root else "thread", origin, title)
            if is_root:
                root_node = node
            if row and row.get("deleted_at"):
                node["notes"].append("deleted")
            lin = (row or {}).get("lineage") if isinstance((row or {}).get("lineage"), dict) else {}
            role = lin.get("role")
            node["role"] = clip(role, 40) if isinstance(role, str) else None
            model, effort, fast = t3_selection((row or {}).get("selection") or (sub or {}).get("selection"))
            if model is None and sub:
                model = sub.get("model") if isinstance(sub.get("model"), str) else None
            node["_fast"] = fast
            node["effort"] = effort
            node["provider"] = provider_of(model, (row or {}).get("instance") or (sub or {}).get("instance")
                                           or (sub or {}).get("driver"))
            if model:
                node["models"] = [model]
            if sub:
                node["status"] = norm_status(sub.get("status"))
                s, e = parse_ts(sub.get("started")), parse_ts(sub.get("completed"))
                tree.set_times(node, s, e)
            if orphan:
                tree.warn("orphan_by_lineage", node)
            # children
            child_rows = []
            for r in conn.execute(Q_T3_CHILDREN, (key,)):
                child_rows.append({
                    "subagent_id": r[0], "child": r[1], "run_id": r[2], "origin": r[3], "driver": r[4],
                    "instance": r[5], "status": r[6], "started": r[7], "completed": r[8],
                    "title": r[9], "model": r[10], "selection": loads(r[11]) or {}, "native": loads(r[12]) or {},
                })
            child_rows.sort(key=lambda c: (parse_ts(c["started"]) or 0, str(c["subagent_id"])))
            # sessions for Claude transcripts
            sessions = list(inherited)
            if any(c["driver"] == "claudeAgent" or "claude" in str(c["instance"] or "").lower() for c in child_rows):
                own = session_cache.get(key)
                if own is None:
                    own = session_cache[key] = t3_claude_sessions(conn, key)
                if own:
                    sessions = own
            elif row:
                own = t3_claude_sessions(conn, key)
                if own:
                    sessions = own
            node["_sessions"] = sessions
            # workflow node classification (needs its children)
            native_id = ((sub or {}).get("native") or {}).get("nativeId") if sub else None
            native_id = native_id if isinstance(native_id, str) else ""
            kids_agent = [c for c in child_rows if AGENT_SUFFIX.search(str((c["native"] or {}).get("nativeId", "")))]
            if sub and kids_agent and (WORKFLOW_TASK.match(native_id) or str((sub or {}).get("title") or "").startswith("Workflow")):
                node["kind"] = "workflow"
                node["origin"] = "workflow"
                node["_workflow_task"] = native_id
            wf_agent = None
            if sub and parent_key is not None:
                m = AGENT_SUFFIX.search(native_id)
                if m and tree.nodes.get(parent_key, {}).get("kind") == "workflow":
                    wf_agent = int(m.group(1))
            # turns
            turns = conn.execute(Q_T3_TURNS, (key,)).fetchall()
            had_turns = t3_turn_usage(turns, node, tree)
            # no turn rows: transcripts / workflow records
            if not had_turns:
                handled = False
                if wf_agent is not None:
                    parent = tree.nodes[parent_key]
                    rec = parent.get("_wf_rec")
                    sdir = parent.get("_wf_dir")
                    if rec is not None and sdir is not None:
                        apply_workflow_agent(tree, node, sdir, rec, wf_agent)
                        handled = True
                    else:
                        node["kind"] = "workflow_agent"
                        tree.warn("workflow_record_missing", node)
                        handled = True
                elif node["kind"] == "workflow":
                    rec, sdir = None, None
                    for sid in sessions:
                        sd = find_claude_session_dir(claude_home, sid)
                        if sd is None:
                            continue
                        recs = wf_cache.get(str(sd))
                        if recs is None:
                            recs = wf_cache[str(sd)] = load_workflow_records(sd)
                        if native_id in recs:
                            rec, sdir = recs[native_id], sd
                            break
                    node["title"] = clip(node["title"] if node["title"].startswith("Workflow") else
                                         "Workflow \u00b7 " + node["title"], MAX_TITLE)
                    node["usage_status"] = "n/a"
                    if rec is not None:
                        node["_wf_rec"], node["_wf_dir"] = rec, sdir
                        node["title"] = clip("Workflow \u00b7 " + rec["name"], MAX_TITLE)
                        if isinstance(rec.get("durationMs"), (int, float)):
                            node["active_ms"] = int(rec["durationMs"])
                        workflow_unmapped(tree, node, sdir, rec)
                    else:
                        tree.warn("workflow_record_missing", node)
                    handled = True
                elif NATIVE_AGENT.match(native_id) and sessions:
                    tpath = None
                    for sid in sessions:
                        sd = find_claude_session_dir(claude_home, sid)
                        if sd is not None:
                            cand = sd / "subagents" / ("agent-" + native_id + ".jsonl")
                            if cand.exists():
                                tpath = cand
                                break
                    tr = parse_claude_transcript(tpath) if tpath else None
                    if tr is None:
                        tree.warn("transcript_missing", node)
                    else:
                        if node["role"] is None:
                            node["role"] = claude_agent_type(tpath)
                        set_usage(node, tr["by_model"], "complete" if tr["by_model"] else "unknown",
                                  "claude-transcript" if tr["by_model"] else "none")
                        if tr["fast"]:
                            node["_fast"] = True
                        if tr["skipped"]:
                            tree.warn("skipped_lines", node)
                        if node["_start"] is None and tr["first"] is not None:
                            tree.set_times(node, tr["first"], tr["last"])
                        handled = True
                if not handled and sub is not None:
                    tree.warn("transcript_missing", node, "no turn records or transcript")
                if node["active_ms"] is None and node["span_ms"] is not None:
                    node["active_ms"] = node["span_ms"]
            # root without turn rows: fall back to runs
            if is_root and node["_start"] is None:
                runs = conn.execute(Q_T3_RUNS, (key,)).fetchall()
                starts = [parse_ts(r[3]) or parse_ts(r[1]) for r in runs if (parse_ts(r[3]) or parse_ts(r[1])) is not None]
                ends = [parse_ts(r[2]) for r in runs if parse_ts(r[2]) is not None]
                if starts:
                    tree.set_times(node, min(starts), max(ends) if ends else None)
            if is_root and node["status"] == "unknown":
                node["status"] = "completed"
            # edges + queue
            for c in child_rows:
                ckey = c["child"] or c["subagent_id"]
                if ckey in seen:
                    continue
                queue.append((ckey, key, c, sessions, None))
        if not has_root_idx:
            break
        members = [r[0] for r in conn.execute(Q_T3_ROOT_MEMBERS, (root_id,))]
        missing = sorted(m for m in members if m not in seen)
        if not missing:
            break
        for m in missing:
            row = t3_thread_row(conn, m)
            lin = (row or {}).get("lineage") if isinstance((row or {}).get("lineage"), dict) else {}
            parent = lin.get("parentThreadId")
            if parent not in tree.nodes:
                parent = root_id
            queue.append((m, parent, None, tree.nodes[parent].get("_sessions", []), True))
    return root_node


def t3_edge_kind(node):
    if node["kind"] in ("workflow", "workflow_agent"):
        return "workflow"
    return "native" if node["origin"] == "provider_native" else "delegate"


def t3_list(conn, limit, home=None):
    out = []
    for tid, title, updated in conn.execute(Q_T3_LIST, (limit,)):
        out.append({"harness": "t3", "id": tid, "title": clip(title, 80), "updated_at": updated, "home": home})
    return out


# --------------------------------------------------------------------------
# Claude Code
# --------------------------------------------------------------------------

def build_claude(claude_home, session, tree):
    main_path = find_claude_session_file(claude_home, session)
    if main_path is None:
        raise TraceError(3, "Claude session not found")
    session_dir = main_path.with_suffix("")
    main = parse_claude_transcript(main_path, main=True)
    if main is None:
        raise TraceError(4, "cannot read the Claude session file")
    title = main["custom_title"] or main["ai_title"] or "untitled"
    root = tree.add(session, None, "chat", "root", title)
    root["status"] = "completed"
    root["provider"] = "anthropic"
    set_usage(root, main["by_model"], "complete" if main["by_model"] else "unknown",
              "claude-transcript" if main["by_model"] else "none")
    root["_fast"] = main["fast"]
    root["active_ms"] = main["turn_ms"]
    tree.set_times(root, main["first"], main["last"])
    if main["skipped"]:
        tree.warn("skipped_lines", root, "%d lines" % main["skipped"])
    # subagents
    parsed, metas = {}, {}
    for mp in sorted(glob.glob(str(session_dir / "subagents" / "agent-*.meta.json"))):
        aid = Path(mp).name[len("agent-"):-len(".meta.json")]
        try:
            with open(mp, "r", encoding="utf-8") as fh:
                meta = json.load(fh)
        except (OSError, ValueError):
            continue
        if not isinstance(meta, dict):
            continue
        metas[aid] = meta
        parsed[aid] = parse_claude_transcript(session_dir / "subagents" / ("agent-" + aid + ".jsonl"))
    owners = {tid: session for tid in main["tool_uses"]}
    for aid, tr in parsed.items():
        for tid in (tr["tool_uses"] if tr else ()):
            owners.setdefault(tid, aid)
    results = dict(main["agent_results"])
    for tr in parsed.values():
        if tr:
            for k, v in tr["agent_results"].items():
                results.setdefault(k, v)
    pending = []
    for aid, meta in sorted(metas.items()):
        tr = parsed.get(aid)
        owner = owners.get(meta.get("toolUseId"))
        node = tree.add(aid, owner or session, "subagent", "agent_tool",
                        meta.get("description") if isinstance(meta.get("description"), str) else aid)
        if owner is None:
            tree.warn("parent_not_found", node)
        if isinstance(meta.get("agentType"), str):
            node["role"] = clip(meta["agentType"], 40)
        if isinstance(meta.get("effort"), str):
            node["effort"] = meta["effort"]
        res = results.get(aid, {})
        meta_model = next((meta[k] for k in ("model", "pickedEffortModel") if isinstance(meta.get(k), str)), None) or res.get("model")
        if tr is None:
            tree.warn("transcript_missing", node)
            if meta_model:
                node["models"] = [meta_model]
        else:
            set_usage(node, tr["by_model"], "complete" if tr["by_model"] else "unknown",
                      "claude-transcript" if tr["by_model"] else "none")
            if not tr["by_model"] and meta_model:
                node["models"] = [meta_model]
            node["_fast"] = tr["fast"]
            tree.set_times(node, tr["first"], tr["last"])
            if tr["skipped"]:
                tree.warn("skipped_lines", node)
        dur = res.get("duration_ms")
        node["active_ms"] = int(dur) if isinstance(dur, (int, float)) else node["span_ms"]
        node["status"] = "completed" if meta.get("finished") is True else ("running" if meta.get("finished") is False else "unknown")
        if node["status"] == "running":
            tree.warn("node_running", node)
        node["provider"] = provider_of((node["models"] or [None])[0])
    # workflows
    for task, rec in sorted(load_workflow_records(session_dir).items(), key=lambda kv: str(kv[1]["startTime"])):
        wf = tree.add("wf:" + task, session, "workflow", "workflow", "Workflow \u00b7 " + rec["name"])
        wf["usage_status"] = "n/a"
        wf["status"] = norm_status(rec.get("status"))
        if isinstance(rec.get("durationMs"), (int, float)):
            wf["active_ms"] = int(rec["durationMs"])
        st = parse_ts(rec.get("startTime"))
        if st is not None and isinstance(rec.get("durationMs"), (int, float)):
            tree.set_times(wf, st, st + int(rec["durationMs"]))
        workflow_unmapped(tree, wf, session_dir, rec)
        for index in sorted(rec["agents"]):
            child = tree.add("%s:agent:%d" % (rec["runId"], index), wf["_key"], "workflow_agent", "workflow", "agent %d" % index)
            apply_workflow_agent(tree, child, session_dir, rec, index)
            child["provider"] = provider_of((child["models"] or [None])[0])
    # cross-check with Claude Code's own record
    cs = main["cost_state"]
    if cs:
        want = {"input": 0, "output": 0, "cache_read": 0, "cache_write": 0}
        for mu in cs.values():
            if isinstance(mu, dict):
                want["input"] += as_int(mu.get("inputTokens"))
                want["output"] += as_int(mu.get("outputTokens"))
                want["cache_read"] += as_int(mu.get("cacheReadInputTokens"))
                want["cache_write"] += as_int(mu.get("cacheCreationInputTokens"))
        have = {"input": 0, "output": 0, "cache_read": 0, "cache_write": 0}
        for n in tree.nodes.values():
            t = n["usage"]["total"]
            if t:
                have["input"] += t["input"]
                have["output"] += t["output"]
                have["cache_read"] += t["cache_read"]
                have["cache_write"] += t["cache_write_5m"] + t["cache_write_1h"]
        bad = [k for k in want if abs(want[k] - have[k]) > max(0.005 * want[k], 0)]
        if bad:
            tree.warn("claude_cost_state_mismatch", root, ", ".join(bad))
        root["_cross_check"] = "mismatch" if bad else "match"
    return root


def claude_list(claude_home, limit):
    items = []
    for p in glob.glob(str(Path(claude_home) / "projects" / "*" / "*.jsonl")):
        try:
            items.append((os.path.getmtime(p), p))
        except OSError:
            continue
    items.sort(reverse=True)
    out = []
    for mtime, p in items[:limit]:
        title = "untitled"
        try:
            with open(p, "rb") as fh:
                fh.seek(0, 2)
                size = fh.tell()
                fh.seek(max(0, size - 65536))
                tail = fh.read().decode("utf-8", "replace")
            for line in reversed(tail.splitlines()):
                if line.startswith('{"type":"custom-title"') or line.startswith('{"type":"ai-title"'):
                    try:
                        r = json.loads(line)
                    except ValueError:
                        continue
                    title = r.get("customTitle") or r.get("aiTitle") or title
                    break
        except OSError:
            pass
        out.append({"harness": "claude", "id": Path(p).stem, "title": clip(title, 80), "updated_at": iso(int(mtime * 1000))})
    return out


# --------------------------------------------------------------------------
# Codex
# --------------------------------------------------------------------------

CODEX_COLUMNS = ["id", "rollout_path", "name", "title", "model", "reasoning_effort", "agent_nickname",
                 "agent_role", "agent_path", "thread_source", "tokens_used", "created_at_ms", "updated_at_ms"]
Q_CODEX_THREAD = "SELECT " + ",".join(CODEX_COLUMNS) + " FROM threads WHERE id=?"
Q_CODEX_CHILDREN = "SELECT child_thread_id,status FROM thread_spawn_edges WHERE parent_thread_id=?"
Q_CODEX_PARENT = "SELECT parent_thread_id FROM thread_spawn_edges WHERE child_thread_id=?"
Q_CODEX_LIST = (
    "SELECT id,name,title,updated_at_ms FROM threads WHERE thread_source != 'subagent' "
    "ORDER BY recency_at_ms DESC LIMIT ?"
)
CODEX_QUERIES = {"thread": Q_CODEX_THREAD, "children": Q_CODEX_CHILDREN, "parent": Q_CODEX_PARENT}


def codex_db_path(codex_home):
    best = None
    for p in glob.glob(str(Path(codex_home) / "state_*.sqlite")):
        m = re.search(r"state_(\d+)\.sqlite$", p)
        if m and (best is None or int(m.group(1)) > best[0]):
            best = (int(m.group(1)), p)
    return Path(best[1]) if best else None


def codex_check_schema(conn):
    require_columns(conn, "threads", CODEX_COLUMNS)
    require_columns(conn, "thread_spawn_edges", ["parent_thread_id", "child_thread_id", "status"])


def codex_thread_row(conn, tid):
    r = conn.execute(Q_CODEX_THREAD, (tid,)).fetchone()
    return dict(zip(CODEX_COLUMNS, r)) if r else None


def codex_find_root(conn, tid):
    start = tid
    for _ in range(64):
        p = conn.execute(Q_CODEX_PARENT, (tid,)).fetchone()
        if not p:
            break
        tid = p[0]
    return tid, (start if tid != start else None)


def locate_rollout(codex_home, rollout_path):
    cands = []
    if rollout_path:
        cands += [rollout_path, rollout_path + ".zst"]
    for c in cands:
        if os.path.isfile(c):
            return Path(c)
    base = os.path.basename(rollout_path or "")
    base = re.sub(r"\.zst$", "", base)
    if not base:
        return None
    m = re.search(r"rollout-(\d{4})-(\d\d)-(\d\d)T", base)
    roots = []
    if m:
        roots.append(Path(codex_home) / "sessions" / m.group(1) / m.group(2) / m.group(3))
    roots.append(Path(codex_home) / "archived_sessions")
    for root in roots:
        for name in (base, base + ".zst"):
            p = root / name
            if p.is_file():
                return p
    return None


def open_lines(path, tree):
    """Yield text lines of a rollout. Raises TraceError(0) with code 'zstd' if it cannot be decompressed."""
    path = str(path)
    if not path.endswith(".zst"):
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            for line in fh:
                yield line
        return
    try:
        import compression.zstd as zst  # Python 3.14+

        with zst.open(path, "rt", encoding="utf-8", errors="replace") as fh:
            for line in fh:
                yield line
        return
    except ImportError:
        pass
    exe = shutil.which("zstd")
    if not exe:
        raise TraceError(0, "zstd")
    proc = subprocess.Popen([exe, "-dc", "--", path], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    try:
        for raw in proc.stdout:
            yield raw.decode("utf-8", "replace")
    finally:
        proc.stdout.close()
        proc.wait()


_TS_HEAD = re.compile(r'"timestamp"\s*:\s*"([^"]+)"')


def parse_codex_rollout(path, default_model, tree, node):
    by_model, prev, cur_model, cur_effort = {}, None, default_model, None
    first = last = None
    final_total = None
    active, open_turns, completed = 0, {}, 0
    models = []
    max_request = {}
    for line in open_lines(path, tree):
        head = line[:120]
        m = _TS_HEAD.search(head)
        if m:
            ts = parse_ts(m.group(1))
            if ts is not None:
                first = ts if first is None else min(first, ts)
                last = ts if last is None else max(last, ts)
        if ('"token_count"' not in line and '"task_started"' not in line and '"task_complete"' not in line
                and '"turn_context"' not in line):
            continue
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if not isinstance(r, dict) or not isinstance(r.get("payload"), dict):
            continue
        t, pl = r.get("type"), r["payload"]
        if t == "turn_context":
            if isinstance(pl.get("model"), str):
                cur_model = pl["model"]
                if cur_model not in models:
                    models.append(cur_model)
            if isinstance(pl.get("effort"), str):
                cur_effort = pl["effort"]
        elif t == "event_msg":
            pt = pl.get("type")
            if pt == "task_started":
                open_turns[pl.get("turn_id")] = pl.get("started_at")
            elif pt == "task_complete":
                completed += 1
                open_turns.pop(pl.get("turn_id"), None)
                d = pl.get("duration_ms")
                if isinstance(d, (int, float)):
                    active += int(d)
                elif isinstance(pl.get("started_at"), (int, float)) and isinstance(pl.get("completed_at"), (int, float)):
                    active += int((pl["completed_at"] - pl["started_at"]) * 1000)
            elif pt == "token_count":
                info = pl.get("info")
                tu = info.get("total_token_usage") if isinstance(info, dict) else None
                if not isinstance(tu, dict):
                    continue
                cur = {k: as_int(tu.get(k)) for k in ("input_tokens", "cached_input_tokens", "cache_write_input_tokens",
                                                      "output_tokens", "reasoning_output_tokens", "total_tokens")}
                if prev is None or cur["total_tokens"] < prev["total_tokens"]:
                    if prev is not None:
                        tree.warn("codex_total_reset", node)
                    delta = dict(cur)
                else:
                    delta = {k: max(0, cur[k] - prev[k]) for k in cur}
                prev = cur
                final_total = cur["total_tokens"]
                if not any(delta.values()):
                    continue
                u = zero_usage()
                u["cache_read"] = delta["cached_input_tokens"]
                u["cache_write_5m"] = delta["cache_write_input_tokens"]
                u["input"] = max(0, delta["input_tokens"] - delta["cached_input_tokens"] - delta["cache_write_input_tokens"])
                u["output"] = delta["output_tokens"]
                u["reasoning"] = delta["reasoning_output_tokens"]
                finish_usage(u)
                key = cur_model or "unknown"
                add_usage(by_model.setdefault(key, zero_usage()), u)
                # Largest single request: the per-request figure when recorded, else this delta.
                last_use = info.get("last_token_usage")
                request_input = as_int(last_use.get("input_tokens")) if isinstance(last_use, dict) else 0
                max_request[key] = max(max_request.get(key, 0), request_input or delta["input_tokens"])
    return {"by_model": by_model, "first": first, "last": last, "final_total": final_total,
            "active": active if completed else None, "running": bool(open_turns), "effort": cur_effort,
            "models": models, "max_request_input": max_request}


def build_codex(conn, codex_home, root_id, tree, prompt_titles=False):
    codex_check_schema(conn)
    queue = deque([(root_id, None, None)])
    seen = set()
    root = None
    while queue:
        tid, parent, edge_status = queue.popleft()
        if tid in seen:
            continue
        seen.add(tid)
        row = codex_thread_row(conn, tid)
        if row is None:
            if parent is None:
                raise TraceError(3, "Codex thread not found")
            continue
        is_root = parent is None
        role = row["agent_role"] or (os.path.basename(row["agent_path"].rstrip("/")) if row["agent_path"] else None)
        # threads.title of an unnamed thread is its first prompt: never a default title.
        from_prompt = False
        if is_root:
            title = row["name"]
            if not title and prompt_titles and row["title"]:
                title, from_prompt = clip(row["title"], 60), True
            title = title or "untitled chat"
        else:
            title = row["agent_nickname"] or role or row["name"] or "subagent %d" % len(seen)
        node = tree.add(tid, parent, "chat" if is_root else "subagent", "root" if is_root else "spawn", title)
        if from_prompt:
            tree.warn("title_from_prompt", node)
        if is_root:
            root = node
        node["role"] = clip(role, 40) if role else None
        node["provider"] = "openai"
        node["effort"] = row["reasoning_effort"]
        if row["model"]:
            node["models"] = [row["model"]]
        node["status"] = "unknown"
        created, updated = parse_ts(row["created_at_ms"]), parse_ts(row["updated_at_ms"])
        tree.set_times(node, created, updated)
        rollout = locate_rollout(codex_home, row["rollout_path"])
        fallback = None
        if rollout is None:
            fallback = "rollout_missing"
        else:
            try:
                res = parse_codex_rollout(rollout, row["model"], tree, node)
            except TraceError as exc:
                if exc.code != 0:
                    raise
                fallback = "zstd_unavailable"
            except OSError:
                fallback = "rollout_missing"
        if fallback:
            tree.warn(fallback, node)
            tokens = as_int(row["tokens_used"])
            total = {"input": None, "cache_read": None, "cache_write_5m": None, "cache_write_1h": None,
                     "output": None, "reasoning": None, "total": tokens}
            node["usage"] = {"by_model": {}, "total": total}
            node["usage_status"], node["usage_source"] = "partial", "codex-tokens-used"
        else:
            set_usage(node, res["by_model"], "complete" if res["by_model"] else "unknown",
                      "codex-rollout" if res["by_model"] else "none")
            if res["models"]:
                node["models"] = sorted(set(node["models"]) | set(res["models"]))
            if res["effort"] and not node["effort"]:
                node["effort"] = res["effort"]
            if res["first"] is not None:
                tree.set_times(node, res["first"], res["last"])
            node["active_ms"] = res["active"]
            node["_max_request_input"] = res["max_request_input"]
            node["status"] = "running" if res["running"] else "completed"
            if res["running"]:
                tree.warn("node_running", node)
            expected = as_int(row["tokens_used"])
            if res["final_total"] is not None and expected and res["final_total"] != expected:
                tree.warn("codex_tokens_mismatch", node, "rollout %d, thread %d" % (res["final_total"], expected))
        for child, status in conn.execute(Q_CODEX_CHILDREN, (tid,)).fetchall():
            queue.append((child, tid, status))
    return root


def codex_list(conn, limit, prompt_titles=False):
    out = []
    for tid, name, title, updated in conn.execute(Q_CODEX_LIST, (limit,)):
        # an unnamed thread's title is its first prompt: listed only on request
        shown = name or (clip(title, 60) if prompt_titles and title else "untitled chat")
        out.append({"harness": "codex", "id": tid, "title": clip(shown, 60),
                    "updated_at": iso(parse_ts(updated))})
    return out


# --------------------------------------------------------------------------
# finalize: ids, depth, cost, aggregates
# --------------------------------------------------------------------------

def edge_kind_for(node, harness):
    if node["parent_id"] is None:
        return None
    if harness == "t3":
        return t3_edge_kind(node)
    if node["kind"] in ("workflow", "workflow_agent"):
        return "workflow"
    return "spawn" if harness == "codex" else "agent_tool"


def agg_status(items):
    """items: list of (value_known: bool, complete: bool) for non-n/a nodes."""
    if not items:
        return "n/a"
    known = sum(1 for k, _ in items if k)
    if known == 0:
        return "unknown"
    return "complete" if all(k and c for k, c in items) else "partial"


def finalize(tree, root, harness, prices, now_ms):
    nodes = tree.nodes
    # attach children, orphans to root
    for key in tree.order:
        n = nodes[key]
        if n is root:
            n["parent_id"] = None
            continue
        if n["parent_id"] not in nodes or n["parent_id"] == key:
            n["parent_id"] = root["_key"]
        nodes[n["parent_id"]]["children"].append(key)

    def sort_key(k):
        n = nodes[k]
        return (n["_start"] if n["_start"] is not None else 1 << 62, str(k))

    # cut cycles: only nodes reachable from root are kept
    order, stack = [], [root["_key"]]
    reach = set()
    while stack:
        k = stack.pop()
        if k in reach:
            continue
        reach.add(k)
        order.append(k)
        kids = sorted(nodes[k]["children"], key=sort_key)
        nodes[k]["children"] = kids
        stack.extend(reversed(kids))
    # preorder ids
    pre = []

    def walk(k, depth):
        nodes[k]["depth"] = depth
        pre.append(k)
        for c in nodes[k]["children"]:
            walk(c, depth + 1)

    walk(root["_key"], 0)
    ids = {k: "n%d" % i for i, k in enumerate(pre)}
    now_stale = prices.fetched_ms is not None and (now_ms - prices.fetched_ms) > STALE_PRICE_DAYS * 86400000

    # cost per node
    price_warns = []
    for k in pre:
        n = nodes[k]
        by_model = n["usage"]["by_model"]
        if n["usage_status"] == "n/a" and not by_model:
            n["cost"] = {"usd": None, "by_model": {}, "status": "n/a"}
            continue
        if not by_model:
            n["cost"] = {"usd": None, "by_model": {}, "status": "unknown"}
            continue
        per, total, known, miss = {}, 0.0, 0, 0
        for model, u in sorted(by_model.items()):
            usd, notes = prices.cost(model, u, (n.get("_max_request_input") or {}).get(model))
            for code, detail in notes:
                tree.warn(code, n, detail)
            per[model] = usd
            if usd is None:
                miss += 1
            else:
                known += 1
                total += usd
        if n["_fast"]:
            tree.warn("fast_mode_premium_ignored", n)
        floor = any(code in n["notes"] for code in COST_FLOOR_CODES)
        status = "unknown" if known == 0 else (
            "complete" if miss == 0 and n["usage_status"] == "complete" and not floor else "partial")
        n["cost"] = {"usd": round(total, 10) if known else None, "by_model": per, "status": status}
    # subtree aggregates (post-order)
    for k in reversed(pre):
        n = nodes[k]
        fields = dict((f, 0) for f in USAGE_FIELDS + ("reasoning",))
        sub_total, sub_known, split_known = 0, False, False
        active, active_known = 0, False
        cost, cost_known = 0.0, False
        cost_items, usage_items = [], []
        active_missing = n["active_ms"] is None
        agents = 0 if n["kind"] == "workflow" else 1
        # own part
        t = n["usage"]["total"]
        if t is not None:
            sub_known = True
            sub_total += t["total"]
            if t.get("input") is not None:
                split_known = True
                for f in fields:
                    fields[f] += t[f] or 0
        if n["active_ms"] is not None:
            active += n["active_ms"]
            active_known = True
        if n["cost"]["usd"] is not None:
            cost += n["cost"]["usd"]
            cost_known = True
        if n["cost"]["status"] != "n/a":
            cost_items.append((n["cost"]["usd"] is not None, n["cost"]["status"] == "complete"))
        if n["usage_status"] != "n/a":
            usage_items.append((n["usage_status"] != "unknown", n["usage_status"] == "complete"))
        for c in n["children"]:
            s = nodes[c]["subtree"]
            agents += s["agents"]
            if s["usage"] is not None:
                sub_known = True
                sub_total += s["usage"]["total"]
                if s["usage"].get("input") is not None:
                    split_known = True
                    for f in fields:
                        fields[f] += s["usage"][f] or 0
            if s["active_ms"] is not None:
                active += s["active_ms"]
                active_known = True
            if s["active_status"] != "complete":
                active_missing = True
            if s["cost_usd"] is not None:
                cost += s["cost_usd"]
                cost_known = True
            cost_items.extend(s["_cost_items"])
            usage_items.extend(s["_usage_items"])
        n["subtree"] = {
            "agents": agents,
            "active_ms": active if active_known else None,
            "active_status": "unknown" if not active_known else ("partial" if active_missing else "complete"),
            "usage": (dict(fields, total=sub_total) if split_known else
                      dict(dict((f, None) for f in fields), total=sub_total)) if sub_known else None,
            "usage_status": agg_status(usage_items),
            "cost_usd": round(cost, 10) if cost_known else None,
            "cost_status": agg_status(cost_items),
            "_cost_items": cost_items, "_usage_items": usage_items,
        }
    # totals
    rs = root["subtree"]
    models = set()
    by_model_tot = {}
    for k in pre:
        n = nodes[k]
        models.update(n["usage"]["by_model"].keys())
        if not n["usage"]["by_model"]:
            models.update(n["models"])
            if n["usage_status"] != "n/a":
                # ran on this model, but its usage is unknown: the model's row is partial
                for model in n["models"]:
                    e = by_model_tot.setdefault(model, {"model": model, "agents": 0, "usage": None, "_cost": 0.0,
                                                        "_known": 0, "_miss": 0, "_partial": False, "_usage_partial": False})
                    e["agents"] += 1
                    e["_partial"] = e["_usage_partial"] = True
        for model, u in n["usage"]["by_model"].items():
            e = by_model_tot.setdefault(model, {"model": model, "agents": 0, "usage": None, "_cost": 0.0,
                                                "_known": 0, "_miss": 0, "_partial": False, "_usage_partial": False})
            e["agents"] += 1
            if e["usage"] is None:
                e["usage"] = zero_usage()
            add_usage(e["usage"], u)
            if n["usage_status"] != "complete":
                e["_usage_partial"] = True
            if n["cost"]["status"] != "complete":
                e["_partial"] = True
            c = n["cost"]["by_model"].get(model)
            if c is None:
                e["_miss"] += 1
            else:
                e["_known"] += 1
                e["_cost"] += c
    starts = [nodes[k]["_start"] for k in pre if nodes[k]["_start"] is not None]
    ends = [nodes[k]["_end"] for k in pre if nodes[k]["_end"] is not None]
    live = any(nodes[k]["status"] == "running" for k in pre)
    span = (max(ends) - min(starts)) if starts and ends and max(ends) >= min(starts) else None
    unattr = {"input": 0, "output": 0}
    has_unattr = False
    for k in pre:
        u = nodes[k]["unattributed"]
        if u:
            has_unattr = True
            unattr["input"] += u["input"]
            unattr["output"] += u["output"]
    cov = {"nodes": len(pre), "usage_complete": 0, "usage_partial": 0, "usage_unknown": 0, "usage_na": 0,
           "priced": 0, "unknown_models": [], "missing_sources": 0}
    unknown_models = set()
    for k in pre:
        n = nodes[k]
        cov["usage_" + {"complete": "complete", "partial": "partial", "unknown": "unknown", "n/a": "na"}[n["usage_status"]]] += 1
        if n["cost"]["usd"] is not None:
            cov["priced"] += 1
        if any(c in n["notes"] for c in ("transcript_missing", "rollout_missing", "workflow_record_missing", "zstd_unavailable")):
            cov["missing_sources"] += 1
        for model, c in n["cost"]["by_model"].items():
            if c is None:
                unknown_models.add(model)
    cov["unknown_models"] = sorted(unknown_models)
    # "match" | "mismatch" | None: the tree's tokens against Claude Code's own session record
    cov["claude_cost_state"] = root.get("_cross_check")
    # finalize nodes to output form
    out_nodes, edges = [], []
    for k in pre:
        n = nodes[k]
        o = dict(n)
        o["id"] = ids[k]
        o["parent_id"] = ids[n["parent_id"]] if n["parent_id"] is not None else None
        o["children"] = [ids[c] for c in n["children"]]
        sub = dict(n["subtree"])
        sub.pop("_cost_items", None)
        sub.pop("_usage_items", None)
        o["subtree"] = sub
        o["source_id"] = n["source_id"]
        for private in [p for p in o if p.startswith("_")]:
            del o[private]
        out_nodes.append(o)
        ek = edge_kind_for(n, harness)
        if ek:
            edges.append({"from": ids[n["parent_id"]], "to": ids[k], "kind": ek})
    warnings = []
    for code, detail in tree.global_warnings:
        warnings.append({"code": code, "node_id": None, "message": WARN_TEXT.get(code, code) + (" (%s)" % detail if detail else "")})
    if prices.source == "none":
        warnings.append({"code": "no_prices", "node_id": None, "message": WARN_TEXT["no_prices"]})
    if now_stale:
        warnings.append({"code": "prices_stale", "node_id": None, "message": WARN_TEXT["prices_stale"]})
    for w in tree.warnings:
        warnings.append({"code": w["code"], "node_id": ids.get(w["node_id"]) if w["node_id"] is not None else None,
                         "message": w["message"]})
    by_model_list = []
    for model in sorted(by_model_tot):
        e = by_model_tot[model]
        by_model_list.append({"model": model, "provider": provider_of(model), "agents": e["agents"], "usage": e["usage"],
                              "cost_usd": round(e["_cost"], 10) if e["_known"] else None,
                              "usage_status": "unknown" if e["usage"] is None else (
                                  "partial" if e["_usage_partial"] else "complete"),
                              "cost_status": "unknown" if not e["_known"] else (
                                  "complete" if not e["_miss"] and not e["_partial"] else "partial")})
    totals = {
        "nodes": len(pre),
        "agents": rs["agents"],
        "models": sorted(models),
        "span_ms": span, "active_ms": rs["active_ms"], "active_status": rs["active_status"],
        "usage": rs["usage"], "usage_status": rs["usage_status"],
        "cost_usd": rs["cost_usd"], "cost_status": rs["cost_status"],
        "unattributed": unattr if has_unattr else None,
    }
    return {
        "nodes": out_nodes, "edges": edges, "totals": totals, "by_model": by_model_list,
        "coverage": cov, "warnings": warnings,
        "chat_times": {"started_at": iso(min(starts)) if starts else None, "ended_at": iso(max(ends)) if ends else None,
                       "live": live},
    }


# --------------------------------------------------------------------------
# HTML
# --------------------------------------------------------------------------

def fmt_ms(ms, status="complete"):
    if ms is None:
        return "unknown"
    ms = int(ms)
    if ms < 1000:
        text = "%.1fs" % (ms / 1000.0)
    else:
        s = ms // 1000
        m, sec = divmod(s, 60)
        h, mins = divmod(m, 60)
        text = "%ds" % s if s < 60 else ("%dm %02ds" % (m, sec) if m < 60 else "%dh %02dm" % (h, mins))
    return ("≥ " + text) if status == "partial" else text


def fmt_tok(n):
    if n is None:
        return "unknown"
    if n < 1000:
        return str(n)
    if n < 1000000:
        return "%.1fk" % (n / 1000.0)
    if n < 1000000000:
        return "%.1fM" % (n / 1e6)
    return "%.2fB" % (n / 1e9)


def fmt_int(n):
    return "unknown" if n is None else "{:,}".format(n)


def fmt_usd(v, status="complete"):
    if v is None:
        return "unknown"
    s = "$%.4f" % v if v < 0.1 else "${:,.2f}".format(v)
    return ("\u2265 " + s) if status == "partial" else s


def esc(s):
    return html.escape("" if s is None else str(s), quote=True)


def own_tokens_text(n):
    if n["usage_status"] == "n/a" and n["usage"]["total"] is None:
        return "n/a", None
    t = n["usage"]["total"]
    # codex-tokens-used is an exact total with an unknown split, not a lower bound
    more = "≥ " if n["usage_status"] == "partial" and n["usage_source"] != "codex-tokens-used" else ""
    return (more + fmt_tok(t["total"]) if t else "unknown"), (t["total"] if t else None)


def own_cost_text(n):
    c = n["cost"]
    if c["status"] == "n/a":
        return "n/a"
    return fmt_usd(c["usd"], c["status"])


def sub_cost_text(n):
    s = n["subtree"]
    return fmt_usd(s["cost_usd"], s["cost_status"]) if s["cost_status"] != "n/a" else "n/a"


def render_row(n, t0, span, by_id):
    s = n["subtree"]
    own_tok, own_tok_raw = own_tokens_text(n)
    sub_tok = (("≥ " if s["usage_status"] == "partial" else "") + fmt_tok(s["usage"]["total"])) if s["usage"] else "unknown"
    model = ", ".join(n["models"]) if n["models"] else "unknown"
    if n["effort"]:
        model += " \u00b7 " + n["effort"]
    bar = ""
    if n["started_at"] and n["ended_at"] and span:
        a, b = parse_ts(n["started_at"]), parse_ts(n["ended_at"])
        left = max(0.0, min(100.0, (a - t0) * 100.0 / span))
        width = max(0.6, min(100.0 - left, (b - a) * 100.0 / span))
        bar = '<span class="bar" aria-hidden="true"><i style="left:%.2f%%;width:%.2f%%"></i></span>' % (left, width)
    nk = len(n["children"])
    depth_chip = '<span class="chip depth">D%d</span>' % n["depth"] if n["depth"] > 8 else ""
    agg = ""
    if nk:
        agg = '<span class="agg">%d agents \u00b7 \u03a3 %s tok \u00b7 \u03a3 %s</span>' % (
            s["agents"] - (0 if n["kind"] == "workflow" else 1), sub_tok, esc(sub_cost_text(n)))
    status = n["status"]
    return (
        '<span class="cell label"><span class="mark st-%s" title="%s"></span>'
        '<span class="ttl" title="%s">%s</span>%s'
        '<span class="chip role">%s</span>%s%s</span>'
        '<span class="cell model" title="%s">%s</span>'
        '<span class="cell time"><span class="v">%s</span>%s</span>'
        '<span class="cell tok"><span class="own v" title="%s">%s</span><span class="sub v">%s</span></span>'
        '<span class="cell cost"><span class="own v">%s</span><span class="sub v">%s</span></span>'
    ) % (
        esc(status), esc(status), esc(n["title"]), esc(n["title"]), depth_chip,
        esc(n["role"] or n["kind"].replace("_", " ")),
        ('<span class="chip kind">%s</span>' % esc(n["kind"].replace("_", " "))) if n["role"] else "", agg,
        esc(model), esc(model),
        esc(fmt_ms(n["active_ms"])), bar,
        esc(fmt_int(own_tok_raw)), esc(own_tok), esc(sub_tok),
        esc(own_cost_text(n)), esc(sub_cost_text(n)),
    )


def usage_table(n):
    rows = []
    bm = n["usage"]["by_model"]
    for model in sorted(bm):
        u = bm[model]
        c = n["cost"]["by_model"].get(model)
        rows.append(
            "<tr><td>%s</td><td>%s</td><td>%s</td><td>%s</td><td>%s</td><td>%s</td><td>%s</td><td>%s</td></tr>" % (
                esc(model), fmt_int(u["input"]), fmt_int(u["cache_read"]),
                "%s / %s" % (fmt_int(u["cache_write_5m"]), fmt_int(u["cache_write_1h"])),
                fmt_int(u["output"]), fmt_int(u["reasoning"]), fmt_int(u["total"]),
                esc(fmt_usd(c) if c is not None else "unknown")))
    if not rows:
        t = n["usage"]["total"]
        if t and t.get("total") is not None:
            return ('<p class="note">Only the total is known: %s tokens. The split and cost are unknown.</p>'
                    % esc(fmt_int(t["total"])))
        return '<p class="note">No token record: usage is %s.</p>' % ("n/a" if n["usage_status"] == "n/a" else "unknown")
    return ('<div class="scroll"><table class="mini"><thead><tr><th>Model</th><th>Input</th><th>Cache read</th>'
            "<th>Cache write 5m / 1h</th><th>Output</th><th>Reasoning (in output)</th><th>Total</th><th>Cost</th></tr></thead>"
            "<tbody>%s</tbody></table></div>") % "".join(rows)


def render_info(n):
    notes = "".join("<li><code>%s</code> %s</li>" % (esc(c), esc(WARN_TEXT.get(c, ""))) for c in n["notes"])
    unattr = ""
    if n["unattributed"]:
        unattr = "<dt>Unattributed (not priced)</dt><dd>%s in / %s out</dd>" % (
            fmt_int(n["unattributed"]["input"]), fmt_int(n["unattributed"]["output"]))
    attempts = "<dt>Attempts</dt><dd>%d</dd>" % n["attempts"] if n["attempts"] else ""
    return (
        '<details class="info"><summary>Details</summary><div class="panel">'
        "<dl><dt>Status</dt><dd>%s</dd><dt>Origin</dt><dd>%s</dd><dt>Started</dt><dd>%s</dd><dt>Ended</dt><dd>%s</dd>"
        "<dt>Span</dt><dd>%s</dd><dt>Active</dt><dd>%s</dd><dt>Usage source</dt><dd>%s (%s)</dd>%s%s"
        '<dt>Id</dt><dd class="idv"><code>%s</code> <button type="button" class="copy" data-copy="%s" hidden>COPY</button></dd></dl>'
        "%s%s</div></details>"
    ) % (
        esc(n["status"]), esc(n["origin"]), esc(n["started_at"] or "unknown"), esc(n["ended_at"] or "unknown"),
        esc(fmt_ms(n["span_ms"])), esc(fmt_ms(n["active_ms"])), esc(n["usage_source"]), esc(n["usage_status"]),
        unattr, attempts, esc(n["source_id"]), esc(n["source_id"]), usage_table(n),
        ("<ul class=\"notes\">%s</ul>" % notes) if notes else "",
    )


def render_tree(nodes, t0, span):
    by_id = {n["id"]: n for n in nodes}

    def render(n):
        nk = len(n["children"])
        search = " ".join([n["title"], n["role"] or "", " ".join(n["models"]), n["kind"]]).lower()
        models = "|".join(n["models"])
        cost = n["subtree"]["cost_usd"] or 0
        toks = (n["subtree"]["usage"] or {"total": 0})["total"]
        start = n["_ms_start"]
        attrs = ('id="row-%s" data-id="%s" data-search="%s" data-models="%s" data-start="%s" data-cost="%s" '
                 'data-tok="%s" data-time="%s"') % (n["id"], n["id"], esc(search), esc(models), start, cost, toks,
                                                    n["active_ms"] or 0)
        if not nk:
            return '<li class="node" %s><div class="row" tabindex="0">%s</div>%s</li>' % (
                attrs, render_row(n, t0, span, by_id), render_info(n))
        opened = "" if (nk > 12 or n["depth"] >= 2) else " open"
        flat = " flat" if n["depth"] >= 8 else ""
        kids = "".join(render(by_id[c]) for c in n["children"])
        return ('<li class="node" %s><details class="grp"%s><summary class="row">%s</summary>'
                '<div class="body">%s<ul class="kids%s">%s</ul></div></details></li>') % (
            attrs, opened, render_row(n, t0, span, by_id), render_info(n), flat, kids)

    return '<ul class="tree">%s</ul>' % render(nodes[0])


def totals_html(model):
    t, c = model["totals"], model["coverage"]
    u = t["usage"]
    cw = (u["cache_write_5m"] + u["cache_write_1h"]) if u and u["cache_write_5m"] is not None else None

    def item(label, value, cls=""):
        return '<div class="stat %s"><dt>%s</dt><dd>%s</dd></div>' % (cls, esc(label), esc(value))

    def tokens(value):
        # some agents have no token record: the sum of the known ones is a lower bound
        if value is None:
            return "unknown"
        return ("≥ " if t["usage_status"] == "partial" else "") + fmt_int(value)

    parts = [
        item("Agents", "%d" % t["agents"]),
        item("Models", ", ".join(t["models"]) or "unknown"),
        item("Wall span", fmt_ms(t["span_ms"])),
        item("Agent time, summed", fmt_ms(t["active_ms"], t["active_status"])
             + (" (partial)" if t["active_status"] == "partial" else "")),
        item("Input", tokens(u["input"] if u else None)),
        item("Cache read", tokens(u["cache_read"] if u else None)),
        item("Cache write", tokens(cw)),
        item("Output", tokens(u["output"] if u else None)),
        item("API-price estimate", fmt_usd(t["cost_usd"], t["cost_status"]) + ("" if t["cost_status"] == "complete" else " (%s)" % t["cost_status"]), "wide hi"),
        item("Coverage", "priced %d/%d \u00b7 usage complete %d/%d" % (
            c["priced"], c["nodes"] - c["usage_na"], c["usage_complete"], c["nodes"] - c["usage_na"])),
    ]
    if t.get("unattributed"):
        parts.append(item("Unattributed (not priced)", "%s in / %s out" % (
            fmt_int(t["unattributed"]["input"]), fmt_int(t["unattributed"]["output"]))))
    return '<dl class="totals">%s</dl>' % "".join(parts)


def models_html(model):
    rows = []
    for m in model["by_model"]:
        u = m["usage"]
        more = "≥ " if m["usage_status"] == "partial" else ""
        rows.append('<tr><td>%s</td><td>%s</td><td class="num">%d</td><td class="num">%s</td>'
                    '<td class="num">%s</td><td class="num">%s</td></tr>' % (
            esc(m["model"]), esc(m["provider"]), m["agents"],
            esc(more + fmt_int(u["total"]) if u else "unknown"),
            esc(more + fmt_int(u["input"] + u["cache_read"] + u["cache_write_5m"] + u["cache_write_1h"]) if u else "unknown"),
            esc(fmt_usd(m["cost_usd"], m["cost_status"]))))
    if not rows:
        return '<p class="note">No model usage is known.</p>'
    return ('<div class="scroll"><table class="models"><thead><tr><th>Model</th><th>Provider</th><th>Agents</th>'
            "<th>Tokens</th><th>Of which input side</th><th>Cost</th></tr></thead><tbody>%s</tbody></table></div>") % "".join(rows)


def warnings_html(model):
    groups = {}
    for w in model["warnings"]:
        g = groups.setdefault(w["code"], {"n": 0, "msg": w["message"]})
        g["n"] += 1
    check = ""
    if model["coverage"].get("claude_cost_state") == "match":
        check = '<p class="note">Token totals match Claude Code\'s own record for this session.</p>'
    if not groups:
        return check + '<p class="note">No warnings.</p>'
    items = "".join('<li><code>%s</code> \u00d7%d <span>%s</span></li>' % (esc(code), g["n"], esc(WARN_TEXT.get(code, g["msg"])))
                    for code, g in sorted(groups.items(), key=lambda kv: (-kv[1]["n"], kv[0])))
    return check + '<ul class="warn">%s</ul>' % items


def embed_json(model):
    text = json.dumps(model, ensure_ascii=False, separators=(",", ":"))
    return (text.replace("&", "\\u0026").replace("<", "\\u003c").replace(">", "\\u003e")
            .replace("\u2028", "\\u2028").replace("\u2029", "\\u2029"))


def render_html(model):
    template = TEMPLATE_PATH.read_text(encoding="utf-8")
    nodes = model["nodes"]
    t0 = parse_ts(model["chat"]["started_at"])
    t1 = parse_ts(model["chat"]["ended_at"])
    span = (t1 - t0) if t0 is not None and t1 is not None and t1 > t0 else None
    for n in nodes:
        n["_ms_start"] = parse_ts(n["started_at"]) or 0
    tree_html = render_tree(nodes, t0 or 0, span)
    for n in nodes:
        del n["_ms_start"]
    chat = model["chat"]
    live = '<span class="badge live">LIVE SNAPSHOT</span>' if chat["live"] else ""
    pr = model["pricing"]
    price_line = "Prices: %s%s." % (esc(pr["label"] or "none"),
                                    (", fetched " + esc(pr["fetched_at"])) if pr.get("fetched_at") else "")
    body = (
        '<header class="top"><p class="brand">$dale-trace</p><h1>%s</h1>'
        '<p class="meta"><span>%s</span><span>root <code>%s</code></span><span>captured %s</span>%s</p></header>'
        "%s"
        '<section id="graph-section" aria-labelledby="gh"><h2 id="gh">Agent graph</h2>'
        '<div id="graph-root"><p class="note">The agent graph needs JavaScript. The agent tree below holds the same information.</p></div></section>'
        '<section aria-labelledby="mh"><h2 id="mh">Models</h2>%s</section>'
        '<section aria-labelledby="th"><h2 id="th">Agent tree</h2><div id="tree-tools"></div>'
        '<div class="treehead" aria-hidden="true"><span>Agent</span><span>Model</span><span>Active time</span><span>Tokens</span><span>Cost</span></div>%s</section>'
        '<section aria-labelledby="wh"><h2 id="wh">Coverage and warnings</h2>%s</section>'
        '<footer class="foot"><p><b>API-price equivalent (estimate) \u2014 not your bill.</b> %s Cost = input\u00d7input rate + cache read\u00d7cache-read rate + '
        "cache write 5m\u00d7cache-write rate + cache write 1h\u00d7its rate + output\u00d7output rate, at standard-tier rates; "
        "reasoning is already inside output. A figure marked \u2265 is a lower bound.</p>"
        "<p>Prompts, results and file paths are not included in this page.</p></footer>"
    ) % (
        esc(chat["title"]), esc(model["harness"].upper()), esc(chat["id"]), esc(model["captured_at"]), live,
        totals_html(model), models_html(model), tree_html, warnings_html(model), price_line,
    )
    parts = {"TITLE": esc("Dale Trace \u00b7 " + chat["title"]), "BODY": body, "DATA": embed_json(model)}
    return re.sub(r"\{\{(TITLE|BODY|DATA)\}\}", lambda m: parts[m.group(1)], template)


# --------------------------------------------------------------------------
# orchestration
# --------------------------------------------------------------------------

def claude_home_of(args):
    return Path(os.path.expanduser(args.claude_home or os.environ.get("CLAUDE_CONFIG_DIR") or "~/.claude"))


def codex_home_of(args):
    return Path(os.path.expanduser(args.codex_home or os.environ.get("CODEX_HOME") or "~/.codex"))


SAFE_ID = re.compile(r"^[A-Za-z0-9._:%@-]{1,512}$")


def locate_chat(args, chat, tree):
    """Return (harness, handle) where handle holds what the builders need."""
    order = ["t3", "codex", "claude"] if args.harness == "auto" else [args.harness]
    schema_error = None
    for h in order:
        if h == "t3":
            for home in candidate_t3_homes(args.t3_home):
                path = t3_db_path(home)
                if not path.is_file():
                    continue
                try:
                    conn, mode = open_ro(path, tree)
                except TraceError:
                    continue
                try:
                    t3_check_schema(conn)
                    if t3_has(conn, chat):
                        return "t3", {"conn": conn, "home": home, "mode": mode}
                except TraceError as exc:
                    schema_error = schema_error or exc
                except sqlite3.Error:
                    pass
                close_ro(conn)
        elif h == "codex":
            path = codex_db_path(codex_home_of(args))
            if path is None:
                continue
            try:
                conn, mode = open_ro(path, tree)
            except TraceError:
                continue
            try:
                codex_check_schema(conn)
                if conn.execute("SELECT 1 FROM threads WHERE id=?", (chat,)).fetchone():
                    return "codex", {"conn": conn, "home": codex_home_of(args), "mode": mode}
            except TraceError as exc:
                schema_error = schema_error or exc
            except sqlite3.Error:
                pass
            close_ro(conn)
        elif h == "claude":
            if find_claude_session_file(claude_home_of(args), chat):
                return "claude", {"home": claude_home_of(args)}
    if schema_error is not None:
        raise schema_error
    return None, None


def current_chat():
    for var, harness in (("T3CODE_THREAD_ID", "t3"), ("CODEX_THREAD_ID", "codex"), ("CLAUDE_CODE_SESSION_ID", "claude")):
        v = os.environ.get(var)
        if v:
            return harness, v
    return None, None


def run_list(args, tree):
    limit = max(1, args.list)
    rows = []
    harnesses = ["t3", "codex", "claude"] if args.harness == "auto" else [args.harness]
    for h in harnesses:
        if h == "t3":
            for home in candidate_t3_homes(args.t3_home):
                path = t3_db_path(home)
                if not path.is_file():
                    continue
                try:
                    conn, _ = open_ro(path, tree)
                except TraceError:
                    continue
                try:
                    rows += t3_list(conn, limit, tilde(home))
                except sqlite3.Error:
                    pass
                finally:
                    close_ro(conn)
        elif h == "codex":
            path = codex_db_path(codex_home_of(args))
            if path:
                try:
                    conn, _ = open_ro(path, tree)
                    try:
                        rows += codex_list(conn, limit, args.prompt_titles)
                    finally:
                        close_ro(conn)
                except (TraceError, sqlite3.Error):
                    pass
        else:
            rows += claude_list(claude_home_of(args), limit)
    rows.sort(key=lambda r: str(r["updated_at"] or ""), reverse=True)
    for r in rows[:limit]:
        print(json.dumps(r, ensure_ascii=False))
    return 0


def write_new(path, text):
    p = Path(os.path.expanduser(str(path)))
    p.parent.mkdir(parents=True, exist_ok=True)
    try:
        with open(str(p), "x", encoding="utf-8") as fh:
            fh.write(text)
    except FileExistsError:
        raise TraceError(5, "output already exists: %s" % tilde(p))
    except OSError as exc:
        raise TraceError(5, "cannot write %s: %s" % (tilde(p), exc))
    return p


def default_out(harness, root_id, now_ms):
    stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime(now_ms / 1000))
    return Path.home() / ".agents" / "dale" / "traces" / ("%s-%s-%s.html" % (harness, safe_id(root_id), stamp))


def build_model(args, now_ms):
    tree = Tree()
    chat, harness_hint = args.chat, None
    if not chat:
        harness_hint, chat = current_chat()
        if not chat:
            raise TraceError(2, "no current chat; pass --chat")
        if args.harness == "auto":
            args.harness = harness_hint
    if not SAFE_ID.match(chat):
        raise TraceError(2, "invalid chat id")
    harness, handle = locate_chat(args, chat, tree)
    if harness is None:
        raise TraceError(3, "chat not found: %s" % chat)
    resolved_from = None
    conn = handle.get("conn")
    try:
        if harness == "t3":
            root_id, resolved_from = t3_find_root(conn, chat)
            root = build_t3(conn, root_id, claude_home_of(args), tree)
            source = {"kind": "t3-db", "home": tilde(handle["home"]), "db_mode": handle["mode"]}
            title = root["title"]
        elif harness == "codex":
            root_id, resolved_from = codex_find_root(conn, chat)
            root = build_codex(conn, handle["home"], root_id, tree, getattr(args, "prompt_titles", False))
            source = {"kind": "codex-state+rollouts", "home": tilde(handle["home"]), "db_mode": handle["mode"]}
        else:
            root_id = chat
            root = build_claude(handle["home"], chat, tree)
            source = {"kind": "claude-transcripts", "home": tilde(handle["home"]), "db_mode": None}
    except sqlite3.Error as exc:
        raise TraceError(4, "database error: %s" % exc)
    finally:
        if conn is not None:
            close_ro(conn)
    prices = load_prices(args, tree)
    parts = finalize(tree, root, harness, prices, now_ms)
    root_out = parts["nodes"][0]
    model = {
        "schema": SCHEMA,
        "captured_at": iso(now_ms),
        "harness": harness,
        "source": source,
        "chat": {"id": root_id, "title": root_out["title"], "resolved_from": resolved_from,
                 "live": parts["chat_times"]["live"], "started_at": parts["chat_times"]["started_at"],
                 "ended_at": parts["chat_times"]["ended_at"]},
        "pricing": {"source": prices.source, "label": prices.label,
                    "fetched_at": iso(prices.fetched_ms), "note": PRICE_NOTE},
        "nodes": parts["nodes"],
        "edges": parts["edges"],
        "totals": parts["totals"],
        "by_model": parts["by_model"],
        "coverage": parts["coverage"],
        "warnings": parts["warnings"],
    }
    return model


def main(argv=None):
    ap = argparse.ArgumentParser(prog="dale_trace.py", description="Draw one chat's agent tree as a self-contained HTML page.")
    ap.add_argument("--chat")
    ap.add_argument("--harness", choices=["auto", "t3", "claude", "codex"], default="auto")
    ap.add_argument("--t3-home")
    ap.add_argument("--claude-home")
    ap.add_argument("--codex-home")
    ap.add_argument("--prices")
    ap.add_argument("--fetch-prices", action="store_true")
    ap.add_argument("--prompt-titles", action="store_true",
                    help="Codex only: title an unnamed chat with the start of its first prompt")
    ap.add_argument("--out")
    ap.add_argument("--json", dest="json_out")
    ap.add_argument("--no-html", action="store_true")
    ap.add_argument("--list", type=int, metavar="N")
    ap.add_argument("--now")
    try:
        args = ap.parse_args(argv)
    except SystemExit as exc:
        return 2 if exc.code else 0
    try:
        now_ms = parse_ts(args.now) if args.now else int(time.time() * 1000)
        if now_ms is None:
            raise TraceError(2, "--now must be an ISO 8601 time")
        if args.list is not None:
            return run_list(args, Tree())
        model = build_model(args, now_ms)
        out_path = None
        if not args.no_html:
            out_path = Path(os.path.expanduser(args.out)) if args.out else default_out(model["harness"], model["chat"]["id"], now_ms)
            if out_path.exists():
                raise TraceError(5, "output already exists: %s" % tilde(out_path))
        if args.json_out and Path(os.path.expanduser(args.json_out)).exists():
            raise TraceError(5, "output already exists: %s" % tilde(args.json_out))
        page = None if args.no_html else render_html(model)
        if page is not None:
            out_path = write_new(out_path, page)
        json_path = None
        if args.json_out:
            json_path = write_new(args.json_out, json.dumps(model, ensure_ascii=False, indent=1) + "\n")
    except TraceError as exc:
        sys.stderr.write("dale-trace: %s\n" % exc.message)
        return exc.code or 4
    groups = {}
    for w in model["warnings"]:
        g = groups.setdefault(w["code"], [0, w["message"]])
        g[0] += 1
    for code, (count, message) in sorted(groups.items(), key=lambda kv: (-kv[1][0], kv[0])):
        sys.stderr.write("warning %s x%d: %s\n" % (code, count, WARN_TEXT.get(code, message)))
    summary = {
        "output": str(out_path) if out_path else None,
        "json": str(json_path) if json_path else None,
        "harness": model["harness"],
        "chat": {"id": model["chat"]["id"], "title": model["chat"]["title"], "resolved_from": model["chat"]["resolved_from"]},
        "totals": model["totals"],
        "coverage": model["coverage"],
        "warnings_count": len(model["warnings"]),
    }
    sys.stdout.write(json.dumps(summary, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
