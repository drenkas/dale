"""Unit tests for skills/dale-trace/scripts/dale_trace.py.

Synthetic fixtures only: no real history is read. Run with
    python3 -m unittest discover -s tests/dale_trace
"""
import ast
import contextlib
import importlib.util
import io
import json
import os
import sqlite3
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "skills" / "dale-trace" / "scripts" / "dale_trace.py"
spec = importlib.util.spec_from_file_location("dale_trace", str(SCRIPT))
dt = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dt)

CANARY = "CANARY-SECRET-PROMPT-7731"
HOSTILE = '<img src=x onerror=alert(1)>'
NOW = "2026-11-19T12:00:00Z"

OPUS_RATES = {
    "input_cost_per_token": 4e-06, "output_cost_per_token": 2e-05,
    "cache_read_input_token_cost": 2e-07, "cache_creation_input_token_cost": 5e-06,
    "cache_creation_input_token_cost_above_1hr": 8e-06,
}
PRICES = {
    "claude-opus-5-5": OPUS_RATES,
    # no output rate: a node that outputs tokens on this model must be unknown, never 0
    "claude-sonnet-5-5": {"input_cost_per_token": 2e-06, "cache_read_input_token_cost": 1e-07,
                          "cache_creation_input_token_cost": 2.5e-06},
    "gpt-x-one": {"input_cost_per_token": 1e-06, "output_cost_per_token": 5e-06,
                  "cache_read_input_token_cost": 1e-07},
    "gpt-x-two": {"input_cost_per_token": 2e-06, "output_cost_per_token": 6e-06,
                  "cache_read_input_token_cost": 2e-07,
                  "input_cost_per_token_above_272k_tokens": 4e-06},
}

T3_SCHEMA = """
CREATE TABLE orchestration_v2_projection_threads (
  thread_id TEXT PRIMARY KEY, project_id TEXT NOT NULL, title TEXT NOT NULL, default_provider TEXT NOT NULL,
  runtime_mode TEXT NOT NULL, interaction_mode TEXT NOT NULL, active_provider_thread_id TEXT,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL, archived_at TEXT, deleted_at TEXT,
  payload_json TEXT NOT NULL, provider_instance_id TEXT);
CREATE TABLE orchestration_v2_projection_runs (
  run_id TEXT PRIMARY KEY, thread_id TEXT NOT NULL, ordinal INTEGER NOT NULL, provider TEXT NOT NULL,
  provider_thread_id TEXT, status TEXT NOT NULL, requested_at TEXT NOT NULL, completed_at TEXT,
  payload_json TEXT NOT NULL, provider_instance_id TEXT);
CREATE UNIQUE INDEX orchestration_v2_projection_runs_thread_ordinal_idx ON orchestration_v2_projection_runs(thread_id, ordinal);
CREATE TABLE orchestration_v2_projection_provider_threads (
  provider_thread_id TEXT PRIMARY KEY, thread_id TEXT, owner_node_id TEXT, provider TEXT NOT NULL,
  provider_session_id TEXT, status TEXT NOT NULL, first_run_ordinal INTEGER, last_run_ordinal INTEGER,
  updated_at TEXT NOT NULL, payload_json TEXT NOT NULL, driver TEXT, provider_instance_id TEXT);
CREATE INDEX orchestration_v2_projection_provider_threads_thread_idx ON orchestration_v2_projection_provider_threads(thread_id);
CREATE TABLE orchestration_v2_projection_provider_turns (
  provider_turn_id TEXT PRIMARY KEY, thread_id TEXT NOT NULL, provider_thread_id TEXT NOT NULL, node_id TEXT NOT NULL,
  run_attempt_id TEXT, ordinal INTEGER NOT NULL, status TEXT NOT NULL, started_at TEXT, completed_at TEXT,
  payload_json TEXT NOT NULL);
CREATE INDEX orchestration_v2_projection_provider_turns_thread_idx ON orchestration_v2_projection_provider_turns(thread_id);
CREATE TABLE orchestration_v2_projection_subagents (
  subagent_id TEXT PRIMARY KEY, thread_id TEXT NOT NULL, run_id TEXT, parent_node_id TEXT NOT NULL, provider TEXT NOT NULL,
  provider_thread_id TEXT, child_thread_id TEXT, origin TEXT NOT NULL, status TEXT NOT NULL, started_at TEXT,
  completed_at TEXT, updated_at TEXT NOT NULL, payload_json TEXT NOT NULL, driver TEXT, provider_instance_id TEXT,
  completion_delivery_state TEXT);
CREATE INDEX orchestration_v2_projection_subagents_thread_idx ON orchestration_v2_projection_subagents(thread_id, started_at, subagent_id);
CREATE INDEX orchestration_v2_projection_subagents_child_thread_idx ON orchestration_v2_projection_subagents(child_thread_id);
CREATE INDEX orchestration_v2_projection_threads_root_thread_idx ON orchestration_v2_projection_threads(
  CASE WHEN json_valid(payload_json) THEN json_extract(payload_json, '$.lineage.rootThreadId') END);
"""

CODEX_SCHEMA = """
CREATE TABLE threads (
  id TEXT PRIMARY KEY, rollout_path TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
  title TEXT NOT NULL, tokens_used INTEGER NOT NULL DEFAULT 0, model TEXT, reasoning_effort TEXT,
  agent_nickname TEXT, agent_role TEXT, agent_path TEXT, thread_source TEXT, created_at_ms INTEGER,
  updated_at_ms INTEGER, name TEXT, recency_at_ms INTEGER NOT NULL DEFAULT 0);
CREATE TABLE thread_spawn_edges (
  parent_thread_id TEXT NOT NULL, child_thread_id TEXT NOT NULL PRIMARY KEY, status TEXT NOT NULL);
CREATE INDEX idx_thread_spawn_edges_parent_status ON thread_spawn_edges(parent_thread_id, status);
"""


def run_main(argv):
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = dt.main(argv)
    return code, out.getvalue(), err.getvalue()


def write_prices(tmp):
    p = Path(tmp) / "prices.json"
    p.write_text(json.dumps({"fetchedAtMs": 1795000000000, "document": PRICES}), encoding="utf-8")
    return str(p)


def make_t3(tmp, root_title="Root chat"):
    home = Path(tmp) / "t3home"
    (home / "userdata").mkdir(parents=True)
    conn = sqlite3.connect(str(home / "userdata" / "statev2.sqlite"))
    conn.executescript(T3_SCHEMA)

    def thread(tid, title, lineage, model, effort="high"):
        payload = {"lineage": lineage, "prompt": CANARY, "result": CANARY,
                   "modelSelection": {"instanceId": "claudeAgent", "model": model,
                                      "options": [{"id": "effort", "value": effort}]}}
        conn.execute("INSERT INTO orchestration_v2_projection_threads VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
                     (tid, "p", title, "claudeAgent", "full", "default", None, "2026-11-19T10:00:00.000Z",
                      "2026-11-19T11:00:00.000Z", None, None, json.dumps(payload), "claudeAgent"))

    thread("ROOT", root_title, {"parentThreadId": None, "relationshipToParent": None, "rootThreadId": "ROOT"},
           "claude-opus-5-5")
    thread("CHILD", "Child agent", {"parentThreadId": "ROOT", "relationshipToParent": "subagent",
                                    "rootThreadId": "ROOT", "role": "implementer"}, "claude-sonnet-5-5", "medium")
    thread("ORPHAN", "Orphan agent", {"parentThreadId": "MISSING", "relationshipToParent": "subagent",
                                      "rootThreadId": "ROOT", "role": "reviewer"}, "claude-opus-5-5")
    conn.execute("INSERT INTO orchestration_v2_projection_subagents VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                 ("sub1", "ROOT", None, "node", "claudeAgent", None, "CHILD", "app_owned", "completed",
                  "2026-11-19T10:10:00.000Z", "2026-11-19T10:20:00.000Z", "2026-11-19T10:20:00.000Z",
                  json.dumps({"title": "Child agent", "model": "claude-sonnet-5-5", "prompt": CANARY,
                              "result": CANARY, "nativeTaskRef": None}),
                 "claudeAgent", "claudeAgent", None))

    def turn(tid, n, start, end, model, usage, cache_ttl_ms=None):
        payload = {"model": model, "turnTokenUsage": usage, "prompt": CANARY}
        if cache_ttl_ms is not None:
            payload["tokenUsage"] = {"promptCacheTtlMs": cache_ttl_ms}
        conn.execute("INSERT INTO orchestration_v2_projection_provider_turns VALUES (?,?,?,?,?,?,?,?,?,?)",
                     ("%s-t%d" % (tid, n), tid, "pt", "node", None, n, "completed", start, end, json.dumps(payload)))

    # inputTokens (100) is smaller than cached + creation (160): the uncached part clamps at 0.
    # No cache lifetime is recorded for this turn, so its cache writes are a lower-bound price.
    turn("ROOT", 1, "2026-11-19T10:00:00.000Z", "2026-11-19T10:01:00.000Z", "claude-opus-5-5",
         {"inputTokens": 100, "cachedInputTokens": 150, "cacheCreationTokens": 10, "outputTokens": 5,
          "reasoningTokens": 2, "usageStatus": "complete", "otherUsage": {"inputTokens": 7, "outputTokens": 1}})
    turn("CHILD", 1, "2026-11-19T10:10:00.000Z", "2026-11-19T10:20:00.000Z", "claude-sonnet-5-5",
         {"inputTokens": 1000, "cachedInputTokens": 400, "cacheCreationTokens": 100, "outputTokens": 50,
          "reasoningTokens": 0, "usageStatus": "complete"}, cache_ttl_ms=3600000)
    turn("ORPHAN", 1, "2026-11-19T10:30:00.000Z", "2026-11-19T10:31:00.000Z", "claude-opus-5-5",
         {"inputTokens": 20, "cachedInputTokens": 0, "cacheCreationTokens": 0, "outputTokens": 3,
          "usageStatus": "complete"})
    conn.commit()
    conn.close()
    return str(home)


def make_claude(tmp, title=HOSTILE):
    home = Path(tmp) / "claudehome"
    proj = home / "projects" / "-p"
    sess = proj / "SESS"
    (sess / "subagents").mkdir(parents=True)

    def asst(mid, out, model="claude-opus-5-5", inp=10, cr=100, cw=20, side=False, extra=None):
        line = {"type": "assistant", "timestamp": "2026-11-19T10:00:%02d.000Z" % min(out, 59),
                "isSidechain": side,
                "message": {"id": mid, "model": model, "role": "assistant", "content": extra or [],
                            "usage": {"input_tokens": inp, "cache_read_input_tokens": cr,
                                      "cache_creation_input_tokens": cw, "output_tokens": out,
                                      "cache_creation": {"ephemeral_1h_input_tokens": 5}}}}
        return json.dumps(line)

    lines = [
        json.dumps({"type": "custom-title", "customTitle": title, "sessionId": "SESS"}),
        json.dumps({"type": "api-request", "params": {"prompt": CANARY}}),
        asst("m1", 10), asst("m1", 20), asst("m1", 20),  # the same message written three times
        asst("m2", 30, extra=[{"type": "tool_use", "id": "toolu_1", "name": "Agent", "input": {"prompt": CANARY}}]),
        asst("side", 40, side=True),  # sidechain lines of the main file are dropped
        json.dumps({"type": "assistant", "timestamp": "2026-11-19T10:00:50.000Z",
                    "message": {"id": "syn", "model": "<synthetic>",
                                "usage": {"input_tokens": 999, "output_tokens": 999}}}),
        json.dumps({"type": "user", "timestamp": "2026-11-19T10:01:00.000Z",
                    "toolUseResult": {"agentId": "a1", "totalDurationMs": 5000, "prompt": CANARY}}),
        "this line is not json",
        # Claude Code's own session record: main + subagent, each message once
        json.dumps({"type": "cost-state", "modelUsage": {"claude-opus-5-5": {
            "inputTokens": 30, "outputTokens": 59, "cacheReadInputTokens": 300, "cacheCreationInputTokens": 60}}}),
    ]
    (proj / "SESS.jsonl").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (sess / "subagents" / "agent-a1.meta.json").write_text(json.dumps(
        {"agentType": "general-purpose", "description": "Child agent", "toolUseId": "toolu_1", "finished": True,
         "prompt": CANARY}), encoding="utf-8")
    (sess / "subagents" / "agent-a1.jsonl").write_text(
        "\n".join([asst("c1", 7, side=True), asst("c1", 9, side=True)]) + "\n", encoding="utf-8")
    return str(home)


def make_codex(tmp, with_rollout=True, named=True, big_request=False):
    home = Path(tmp) / "codexhome"
    day = home / "sessions" / "2026" / "11" / "01"
    day.mkdir(parents=True)
    rollout = day / "rollout-2026-11-01T00-00-00-CODEX1.jsonl"
    conn = sqlite3.connect(str(home / "state_5.sqlite"))
    conn.executescript(CODEX_SCHEMA)
    conn.execute(
        "INSERT INTO threads (id,rollout_path,created_at,updated_at,title,tokens_used,model,reasoning_effort,"
        "agent_nickname,agent_role,agent_path,thread_source,created_at_ms,updated_at_ms,name,recency_at_ms) "
        "VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        ("CODEX1", str(rollout), 1, 2, CANARY, 300230 if big_request else 230, "gpt-x-one", "medium", None, None,
         None, "user", 1793000000000, 1793000100000, "Codex root" if named else None, 5))
    # an unnamed child with no nickname or role: its title column is its first prompt
    conn.execute(
        "INSERT INTO threads (id,rollout_path,created_at,updated_at,title,tokens_used,model,reasoning_effort,"
        "agent_nickname,agent_role,agent_path,thread_source,created_at_ms,updated_at_ms,name,recency_at_ms) "
        "VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        ("CODEX2", str(day / "rollout-missing-CODEX2.jsonl"), 1, 2, CANARY, 5, "gpt-x-one", "medium", None, None,
         None, "subagent", 1793000010000, 1793000020000, None, 5))
    conn.execute("INSERT INTO thread_spawn_edges VALUES (?,?,?)", ("CODEX1", "CODEX2", "open"))
    conn.commit()
    conn.close()

    def tc(i, o, c, total, ts):
        return json.dumps({"timestamp": ts, "type": "event_msg", "payload": {"type": "token_count", "info": {
            "total_token_usage": {"input_tokens": i, "cached_input_tokens": c, "cache_write_input_tokens": 0,
                                  "output_tokens": o, "reasoning_output_tokens": 0, "total_tokens": total}}}})

    lines = [
        json.dumps({"timestamp": "2026-11-01T00:00:00.000Z", "type": "session_meta", "payload": {"id": "CODEX1"}}),
        json.dumps({"timestamp": "2026-11-01T00:00:01.000Z", "type": "turn_context",
                    "payload": {"model": "gpt-x-one", "effort": "medium"}}),
        json.dumps({"timestamp": "2026-11-01T00:00:02.000Z", "type": "event_msg",
                    "payload": {"type": "task_started", "turn_id": "t1", "started_at": 1793000001}}),
        tc(100, 10, 40, 110, "2026-11-01T00:00:03.000Z"),
        tc(100, 10, 40, 110, "2026-11-01T00:00:03.500Z"),  # duplicate event
        json.dumps({"timestamp": "2026-11-01T00:00:04.000Z", "type": "turn_context",
                    "payload": {"model": "gpt-x-two", "effort": "high"}}),
        # with big_request one request has more than 272k input tokens, past gpt-x-two's price tier
        tc(300200 if big_request else 200, 30, 90, 300230 if big_request else 230, "2026-11-01T00:00:05.000Z"),
        json.dumps({"timestamp": "2026-11-01T00:00:06.000Z", "type": "event_msg",
                    "payload": {"type": "task_complete", "turn_id": "t1", "duration_ms": 4000}}),
        json.dumps({"timestamp": "2026-11-01T00:00:07.000Z", "type": "response_item",
                    "payload": {"type": "message", "text": CANARY}}),
    ]
    if with_rollout:
        rollout.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return str(home)


class Base(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = self._tmp.name
        self.prices = write_prices(self.tmp)

    def tearDown(self):
        self._tmp.cleanup()

    def trace(self, extra, name="x"):
        out = os.path.join(self.tmp, name + ".html")
        js = os.path.join(self.tmp, name + ".json")
        code, so, se = run_main(extra + ["--prices", self.prices, "--out", out, "--json", js, "--now", NOW])
        self.assertEqual(code, 0, se)
        return json.loads(Path(js).read_text(encoding="utf-8")), Path(out).read_text(encoding="utf-8"), Path(js).read_text(encoding="utf-8")


class SyntaxTest(unittest.TestCase):
    def test_python_39_syntax(self):
        ast.parse(SCRIPT.read_text(encoding="utf-8"), feature_version=(3, 9))


class HelperTest(unittest.TestCase):
    def test_timestamps(self):
        self.assertEqual(dt.parse_ts("1970-01-01T00:00:01.5Z"), 1500)
        self.assertEqual(dt.parse_ts("2026-11-19T10:00:00+02:00"), dt.parse_ts("2026-11-19T08:00:00Z"))
        self.assertEqual(dt.parse_ts(1793000000), 1793000000000)
        self.assertIsNone(dt.parse_ts("nonsense"))
        self.assertEqual(dt.iso(1500), "1970-01-01T00:00:01.500Z")

    def test_unknown_is_never_zero(self):
        self.assertEqual(dt.fmt_ms(None), "unknown")
        self.assertEqual(dt.fmt_tok(None), "unknown")
        self.assertEqual(dt.fmt_usd(None), "unknown")
        self.assertEqual(dt.fmt_usd(0.0), "$0.0000")
        self.assertEqual(dt.fmt_usd(1.7177126), "$1.72")
        self.assertEqual(dt.fmt_usd(0.05, "partial"), "≥ $0.0500")

    def test_price_lookup_and_missing_rate(self):
        p = dt.Prices(PRICES, "file", "f")
        self.assertIsNotNone(p.lookup("claude-opus-5-5[1m]"))
        self.assertIsNotNone(p.lookup("claude-opus-5-5-20261101"))
        usd, notes = p.cost("claude-opus-5-5", dict(dt.zero_usage(), input=1000000, total=1000000))
        self.assertAlmostEqual(usd, 4.0)
        usd, notes = p.cost("claude-sonnet-5-5", dict(dt.zero_usage(), output=10, total=10))
        self.assertIsNone(usd)  # missing output rate: unknown, never 0
        self.assertEqual(notes[0][0], "rate_missing")
        usd, notes = p.cost("no-such-model", dict(dt.zero_usage(), input=1, total=1))
        self.assertIsNone(usd)
        self.assertEqual(notes[0][0], "price_missing")
        usd, notes = p.cost("gpt-x-two", dict(dt.zero_usage(), input=1, total=1))
        self.assertEqual([n[0] for n in notes], ["tiered_price_ignored"])

    def test_no_price_source_is_none(self):
        usd_cost = dt.Prices().lookup("claude-opus-5-5")
        self.assertIsNone(usd_cost)


class ClaudeTest(Base):
    def test_dedup_totals_and_privacy(self):
        home = make_claude(self.tmp)
        model, page, raw = self.trace(["--harness", "claude", "--chat", "SESS", "--claude-home", home])
        nodes = model["nodes"]
        self.assertEqual(len(nodes), 2)
        root, child = nodes
        # m1 counted once (max output 20), m2 once (30), sidechain + synthetic dropped
        self.assertEqual(root["usage"]["total"]["output"], 50)
        self.assertEqual(root["usage"]["total"]["input"], 20)
        self.assertEqual(root["usage"]["total"]["cache_read"], 200)
        self.assertEqual(root["usage"]["total"]["cache_write_1h"], 10)
        self.assertEqual(root["usage"]["total"]["cache_write_5m"], 30)
        self.assertEqual(child["parent_id"], root["id"])
        self.assertEqual(child["usage"]["total"]["output"], 9)
        self.assertEqual(child["active_ms"], 5000)
        self.assertEqual(child["role"], "general-purpose")
        self.assertEqual(model["edges"], [{"from": "n0", "to": "n1", "kind": "agent_tool"}])
        self.assertEqual(model["coverage"]["claude_cost_state"], "match")
        self.assertIn("match Claude Code", page)
        # the root has no turn-duration record: the summed time covers the child only and says so
        self.assertIsNone(root["active_ms"])
        self.assertEqual((model["totals"]["active_ms"], model["totals"]["active_status"]), (5000, "partial"))
        self.assertIn("≥ 5s (partial)", page)
        for text in (page, raw):
            self.assertNotIn(CANARY, text)

    def test_hostile_title_is_escaped(self):
        home = make_claude(self.tmp)
        model, page, raw = self.trace(["--harness", "claude", "--chat", "SESS", "--claude-home", home])
        self.assertNotIn(HOSTILE, page)
        self.assertIn("&lt;img src=x onerror=alert(1)&gt;", page)
        self.assertEqual(json.loads(raw)["chat"]["title"], HOSTILE)  # the data file keeps the exact text
        self.assertIn("\\u003cimg", page)  # inside the embedded JSON the markup characters are escaped

    def test_cost_is_exact(self):
        home = make_claude(self.tmp)
        model, _, _ = self.trace(["--harness", "claude", "--chat", "SESS", "--claude-home", home])
        root = model["nodes"][0]
        want = 20 * 4e-06 + 200 * 2e-07 + 30 * 5e-06 + 10 * 8e-06 + 50 * 2e-05
        self.assertAlmostEqual(root["cost"]["usd"], want, places=9)
        self.assertEqual(root["cost"]["status"], "complete")


class T3Test(Base):
    def test_tree_normalization_and_unattributed(self):
        home = make_t3(self.tmp)
        model, page, raw = self.trace(["--chat", "CHILD", "--t3-home", home])
        self.assertEqual(model["chat"]["id"], "ROOT")
        self.assertEqual(model["chat"]["resolved_from"], "CHILD")
        by_src = dict((n["source_id"], n) for n in model["nodes"])
        self.assertEqual(set(by_src), {"ROOT", "CHILD", "ORPHAN"})
        root = by_src["ROOT"]["usage"]["total"]
        # inputTokens 100 < cached 150 + creation 10: uncached input clamps at 0
        self.assertEqual((root["input"], root["cache_read"], root["cache_write_5m"], root["output"]), (0, 150, 10, 5))
        self.assertEqual(root["total"], 165)
        child = by_src["CHILD"]
        self.assertEqual(child["role"], "implementer")
        self.assertEqual(child["usage"]["total"]["input"], 500)
        self.assertEqual(child["active_ms"], 600000)
        self.assertEqual(by_src["ROOT"]["unattributed"], {"input": 7, "output": 1})
        self.assertEqual(model["totals"]["unattributed"], {"input": 7, "output": 1})
        self.assertEqual(model["totals"]["usage"]["input"], 0 + 500 + 20)  # unattributed is not added
        self.assertIn("orphan_by_lineage", by_src["ORPHAN"]["notes"])
        # no recorded cache lifetime: priced at the 5-minute rate and shown as a lower bound
        self.assertIn("cache_ttl_unsplit", by_src["ROOT"]["notes"])
        self.assertEqual(by_src["ROOT"]["cost"]["status"], "partial")
        # a recorded 1-hour lifetime moves the turn's cache writes to the 1-hour bucket
        self.assertEqual((child["usage"]["total"]["cache_write_5m"], child["usage"]["total"]["cache_write_1h"]), (0, 100))
        self.assertNotIn("cache_ttl_unsplit", child["notes"])
        opus = [m for m in model["by_model"] if m["model"] == "claude-opus-5-5"][0]
        self.assertEqual(opus["cost_status"], "partial")  # the Models row keeps the lower-bound marker
        edge_kinds = dict((e["to"], e["kind"]) for e in model["edges"])
        self.assertEqual(edge_kinds[child["id"]], "delegate")
        for text in (page, raw):
            self.assertNotIn(CANARY, text)

    def test_missing_rate_gives_unknown_cost_not_zero(self):
        home = make_t3(self.tmp)
        model, _, _ = self.trace(["--chat", "ROOT", "--t3-home", home])
        child = [n for n in model["nodes"] if n["source_id"] == "CHILD"][0]
        self.assertIsNone(child["cost"]["usd"])  # sonnet has no output rate
        self.assertEqual(child["cost"]["status"], "unknown")
        self.assertEqual(model["totals"]["cost_status"], "partial")
        self.assertIn("rate_missing", [w["code"] for w in model["warnings"]])
        self.assertEqual(model["coverage"]["priced"], 2)

    def test_unknown_model_price_missing(self):
        home = make_t3(self.tmp)
        os.remove(self.prices)
        Path(self.prices).write_text(json.dumps({"document": {}}), encoding="utf-8")
        model, _, _ = self.trace(["--chat", "ROOT", "--t3-home", home])
        self.assertIn("price_missing", [w["code"] for w in model["warnings"]])
        self.assertIsNone(model["totals"]["cost_usd"])

    def test_not_found_exit_3(self):
        home = make_t3(self.tmp)
        code, _, err = run_main(["--chat", "NOPE", "--t3-home", home, "--prices", self.prices,
                                 "--out", os.path.join(self.tmp, "n.html")])
        self.assertEqual(code, 3)

    def test_schema_mismatch_exit_4(self):
        home = Path(self.tmp) / "badhome"
        (home / "userdata").mkdir(parents=True)
        conn = sqlite3.connect(str(home / "userdata" / "statev2.sqlite"))
        conn.execute("CREATE TABLE orchestration_v2_projection_threads (thread_id TEXT PRIMARY KEY, title TEXT)")
        conn.commit()
        conn.close()
        code, _, err = run_main(["--chat", "X", "--t3-home", str(home), "--out", os.path.join(self.tmp, "b.html")])
        self.assertEqual(code, 4)
        self.assertIn("orchestration_v2_projection_threads", err)

    def test_list(self):
        home = make_t3(self.tmp)
        code, out, _ = run_main(["--list", "5", "--harness", "t3", "--t3-home", home])
        self.assertEqual(code, 0)
        rows = [json.loads(l) for l in out.splitlines()]
        self.assertEqual([r["id"] for r in rows], ["ROOT"])

    def test_query_plans_use_indexes(self):
        home = make_t3(self.tmp)
        conn = sqlite3.connect(os.path.join(home, "userdata", "statev2.sqlite"))
        for name, sql in dt.T3_QUERIES.items():
            params = ("x",)
            plan = " ".join(r[3] for r in conn.execute("EXPLAIN QUERY PLAN " + sql, params))
            self.assertTrue("USING INDEX" in plan or "USING COVERING INDEX" in plan or "PRIMARY KEY" in plan,
                            "%s: %s" % (name, plan))
            for line in [r[3] for r in conn.execute("EXPLAIN QUERY PLAN " + sql, params)]:
                self.assertFalse(line.startswith("SCAN orchestration_v2"), "%s scans: %s" % (name, line))
        conn.close()

    def test_database_is_not_modified(self):
        home = make_t3(self.tmp)
        db = os.path.join(home, "userdata", "statev2.sqlite")
        before = Path(db).read_bytes()
        self.trace(["--chat", "ROOT", "--t3-home", home])
        self.assertEqual(Path(db).read_bytes(), before)


class CodexTest(Base):
    def test_delta_dedup_and_model_split(self):
        home = make_codex(self.tmp)
        model, page, raw = self.trace(["--harness", "codex", "--chat", "CODEX1", "--codex-home", home])
        n = model["nodes"][0]
        bm = n["usage"]["by_model"]
        self.assertEqual((bm["gpt-x-one"]["input"], bm["gpt-x-one"]["cache_read"], bm["gpt-x-one"]["output"]), (60, 40, 10))
        self.assertEqual((bm["gpt-x-two"]["input"], bm["gpt-x-two"]["cache_read"], bm["gpt-x-two"]["output"]), (50, 50, 20))
        self.assertEqual(n["usage"]["total"]["total"], 230)
        self.assertEqual(n["active_ms"], 4000)
        self.assertNotIn("codex_tokens_mismatch", n["notes"])
        # no request reached gpt-x-two's 272k tier, so base rates are exact
        self.assertNotIn("tiered_price_ignored", n["notes"])
        self.assertEqual(n["cost"]["status"], "complete")
        for text in (page, raw):
            self.assertNotIn(CANARY, text)

    def test_price_tier_makes_cost_a_lower_bound(self):
        home = make_codex(self.tmp, big_request=True)
        model, page, _ = self.trace(["--harness", "codex", "--chat", "CODEX1", "--codex-home", home])
        n = model["nodes"][0]
        self.assertIn("tiered_price_ignored", n["notes"])
        self.assertEqual(n["cost"]["status"], "partial")
        self.assertEqual(model["totals"]["cost_status"], "partial")
        self.assertIn("≥ $", page)

    def test_prompt_is_never_a_default_title(self):
        home = make_codex(self.tmp, named=False)
        model, page, raw = self.trace(["--harness", "codex", "--chat", "CODEX1", "--codex-home", home])
        self.assertEqual(model["chat"]["title"], "untitled chat")
        self.assertEqual(model["nodes"][1]["title"], "subagent 2")
        for text in (page, raw):
            self.assertNotIn(CANARY, text)
        code, out, _ = run_main(["--list", "5", "--harness", "codex", "--codex-home", home])
        self.assertEqual(code, 0)
        self.assertNotIn(CANARY, out)

    def test_prompt_title_only_on_request(self):
        home = make_codex(self.tmp, named=False)
        model, _, _ = self.trace(["--harness", "codex", "--chat", "CODEX1", "--codex-home", home, "--prompt-titles"])
        self.assertEqual(model["chat"]["title"], CANARY)
        self.assertIn("title_from_prompt", model["nodes"][0]["notes"])
        self.assertEqual(model["nodes"][1]["title"], "subagent 2")  # a child never takes a prompt title

    def test_missing_rollout_falls_back_to_total(self):
        home = make_codex(self.tmp, with_rollout=False)
        model, _, _ = self.trace(["--harness", "codex", "--chat", "CODEX1", "--codex-home", home])
        n = model["nodes"][0]
        self.assertEqual(n["usage_source"], "codex-tokens-used")
        self.assertEqual(n["usage"]["total"]["total"], 230)
        self.assertIsNone(n["usage"]["total"]["input"])
        self.assertIsNone(n["cost"]["usd"])
        self.assertIsNone(model["totals"]["usage"]["input"])
        self.assertIn("rollout_missing", n["notes"])


class OutputTest(Base):
    def test_refuses_to_overwrite(self):
        home = make_claude(self.tmp)
        args = ["--harness", "claude", "--chat", "SESS", "--claude-home", home, "--prices", self.prices,
                "--out", os.path.join(self.tmp, "once.html"), "--now", NOW]
        self.assertEqual(run_main(args)[0], 0)
        code, _, err = run_main(args)
        self.assertEqual(code, 5)
        self.assertIn("exists", err)

    def test_deterministic_with_now(self):
        home = make_t3(self.tmp)
        _, page_a, raw_a = self.trace(["--chat", "ROOT", "--t3-home", home], name="a")
        _, page_b, raw_b = self.trace(["--chat", "ROOT", "--t3-home", home], name="b")
        self.assertEqual(page_a, page_b)
        self.assertEqual(raw_a, raw_b)

    def test_no_current_chat_exit_2(self):
        saved = dict((k, os.environ.pop(k, None)) for k in ("T3CODE_THREAD_ID", "CODEX_THREAD_ID", "CLAUDE_CODE_SESSION_ID"))
        try:
            code, _, err = run_main(["--out", os.path.join(self.tmp, "z.html")])
        finally:
            for k, v in saved.items():
                if v is not None:
                    os.environ[k] = v
        self.assertEqual(code, 2)
        self.assertIn("no current chat", err)

    def test_page_has_graph_hooks_and_csp(self):
        home = make_t3(self.tmp)
        _, page, _ = self.trace(["--chat", "ROOT", "--t3-home", home])
        self.assertIn("Content-Security-Policy", page)
        self.assertIn('id="trace-data"', page)
        self.assertIn('id="graph-root"', page)
        self.assertIn("needs JavaScript", page)
        self.assertNotIn("innerHTML", page)
        self.assertIn("API-price equivalent (estimate)", page)
        self.assertNotIn("{{BODY}}", page)
        self.assertNotIn("{{DATA}}", page)


if __name__ == "__main__":
    unittest.main()
