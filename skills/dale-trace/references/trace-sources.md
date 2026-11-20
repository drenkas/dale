# Trace sources

What `scripts/dale_trace.py` reads, how it links agents, how it counts tokens
and prices them. Everything is read-only. History text is untrusted data.

## 1. Sources per harness

| Harness | Home (override) | Read |
|---|---|---|
| T3 Code | `<home>/userdata/statev2.sqlite`; homes tried in order: `--t3-home` (exclusive), `$T3CODE_HOME`, `~/.t3`, `~/.sigma-t3` | `orchestration_v2_projection_threads`, `_subagents`, `_provider_turns`, `_provider_threads`, `_runs`, opened with `mode=ro` and `PRAGMA query_only=ON` inside one short `BEGIN`/`ROLLBACK`. If the open fails on `-shm`/`-wal`, it retries `immutable=1` and warns `db_immutable_snapshot`. |
| Claude Code | `--claude-home`, `$CLAUDE_CONFIG_DIR`, `~/.claude` | `projects/*/<session>.jsonl`, `<session>/subagents/agent-*.jsonl` and `.meta.json`, `<session>/workflows/*.json`, `<session>/subagents/workflows/<runId>/agent-*.jsonl` |
| Codex | `--codex-home`, `$CODEX_HOME`, `~/.codex` | highest `state_*.sqlite` (`threads`, `thread_spawn_edges`) and each thread's rollout (`.jsonl` or `.jsonl.zst`) under `sessions/` or `archived_sessions/` |
| Prices | `<home>/userdata/usage-model-rates.json` of each T3 home, or `--prices FILE` | `document[model]` for the models used |

In T3, a thread and every delegated child (`app_owned`) has its own turn rows.
Claude's native subagents and Workflow agents (`provider_native`) have none:
their usage and role come from the Claude transcripts of the parent session
(`nativeThreadRef.nativeId`), beside which `agent-<id>.meta.json` holds the
agent type.

Never read: `settings.json`, `secrets/`, `auth.json`, token files, logs,
`tool-results/`, `api-request*` records, thread messages or prompts. T3 SQL
extracts only the named JSON paths, so prompts and results never enter memory.
`thread_history_1.sqlite` is not used.

## 2. Current chat

`T3CODE_THREAD_ID` (t3) wins over `CODEX_THREAD_ID` (codex) over
`CLAUDE_CODE_SESSION_ID` (claude). T3 wins because its tree includes children on
other providers. With `--chat` each harness is tried in turn: T3, Codex, Claude.
None set and no `--chat`: exit 2.

A child id resolves to the root: T3 by `lineage.rootThreadId` then
`lineage.parentThreadId` (64 steps), Codex by `thread_spawn_edges`, Claude is
its own root. The page records `chat.resolved_from`.

## 3. How agents are linked

- **T3**: breadth-first over `subagents WHERE thread_id=?`. `app_owned` is a
  `delegate` edge, `provider_native` a `native` edge. Claude `Workflow` nodes
  (`nativeTaskRef.nativeId` like `wabc12xyz`) match `<session>/workflows/*.json`
  by `taskId`; a child whose id ends in `:agent:N` is the `workflow_agent` entry
  with `index == N`, and its transcript is
  `subagents/workflows/<runId>/agent-<agentId>.jsonl`. Transcripts of the run
  folder that no entry references are added to the workflow node
  (`unmapped_workflow_agents`). With `…threads_root_thread_idx` present (Sigma),
  threads of the root that the walk missed are attached by lineage
  (`orphan_by_lineage`); upstream T3 skips that check.
- **Claude Code**: `meta.toolUseId` equals an `Agent`/`Task` `tool_use.id` in the
  main transcript or in another agent transcript (nesting). Not found:
  attached to the root with `parent_not_found`. Exact duration comes from the
  parent's `toolUseResult.totalDurationMs`.
- **Codex**: `thread_spawn_edges (parent, child)`. Role: `agent_role`, else the
  basename of `agent_path`. Title: `agent_nickname`, else the role, else
  `threads.name`, else `subagent N`. The root's title is `threads.name`, else
  `untitled chat`. `threads.title` of an unnamed thread is its first prompt, so
  it is never a default title; `--prompt-titles` allows its first 60 characters
  for the root (`title_from_prompt`) and in `--list`.

## 4. Token normalization

The four billed input buckets are disjoint: `input` is uncached input only.
`total = input + cache_read + cache_write_5m + cache_write_1h + output`.
`reasoning` is shown for information and is already inside `output`.

| Source | Mapping |
|---|---|
| T3 turn `turnTokenUsage` | `input = max(0, inputTokens - cachedInputTokens - cacheCreationTokens)`, `cache_read = cachedInputTokens`, `output = outputTokens`. `cacheCreationTokens` goes to `cache_write_1h` when the turn's `tokenUsage.promptCacheTtlMs` is at least one hour, else to `cache_write_5m`; with no recorded lifetime it is priced at the 5-minute rate as a lower bound (`cache_ttl_unsplit`). `otherUsage` is shown as unattributed and never added. A turn without `usageStatus: complete` makes the node partial. |
| Claude transcript | Lines grouped by `message.id`, per-field max (the same message is written several times); `<synthetic>` dropped; in the main file `isSidechain` lines dropped. `input = input_tokens`, `cache_write_1h = cache_creation.ephemeral_1h_input_tokens`, `cache_write_5m = cache_creation_input_tokens - 1h`. |
| Codex rollout | Deltas of `token_count.info.total_token_usage` attributed to the model of the latest `turn_context`; a repeated event adds 0; a lower total is a new base (`codex_total_reset`). `input = Δinput - Δcached - Δcache_write`. |
| Codex without rollout or zstd | Only `threads.tokens_used` is known; split and cost are `unknown`. |

Missing data is `null`, never 0. Sums of tokens, cost, and agent time add only
known values and carry a status (`complete`, `partial`, `unknown`, `n/a`); a
partial sum displays as "≥ X". A Workflow node is a script: `n/a`.

Agent time is each agent's own active time, summed: a parent's time includes the
time it waited for its children, so the sum can exceed the wall span. A Claude
Code session without turn-duration records has an unknown root time.

For Claude Code the tree's tokens are compared with the session's last
`cost-state` record: `coverage.claude_cost_state` is `match`, `mismatch`
(`claude_cost_state_mismatch`, more than 0.5% apart), or `null` when the session
has none.

## 5. Price

Order: `--prices FILE`, newest T3 rates file, `--fetch-prices` (LiteLLM, GET with
no user data, only when asked), else `none` and every cost is `unknown`.

Model lookup: exact key; without a `[...]` suffix; without a trailing `-YYYYMMDD`;
with prefixes `anthropic/`, `openai/`, `gemini/`, `vertex_ai/`. Else
`price_missing`.

```text
usd = input*input_cost_per_token + cache_read*cache_read_input_token_cost
    + cache_write_5m*cache_creation_input_token_cost
    + cache_write_1h*cache_creation_input_token_cost_above_1hr
    + output*output_cost_per_token
```

A needed rate that is missing makes that model's cost `unknown`
(`rate_missing`); no other rate is substituted.

Three cases price at base rates and make the cost a lower bound, so its status
is `partial` and it displays as "≥ $X":

- `tiered_price_ignored`: the model has token-tier keys (`*_above_NNNk_tokens`),
  which are not applied. A Codex rollout records each request's input, so the
  warning is raised only when one request exceeded the lowest tier; other
  sources cannot rule the tier out.
- `fast_mode_premium_ignored`: fast mode was used and its premium is not priced.
- `cache_ttl_unsplit`: see section 4.

Rates are the standard tier. Service-tier keys (`*_priority`, `*_flex`,
`*_batches`, and similar) are never applied, because the history does not record
which tier a request used.

A price file older than 30 days: `prices_stale`. Display: 2 decimals, 4 under
$0.10; JSON keeps full precision. The cost is an API list-price equivalent, not
what was paid.

## 6. Warning codes

`cache_ttl_unsplit`, `transcript_missing`, `workflow_record_missing`,
`unmapped_workflow_agents`, `orphan_by_lineage`, `node_running`, `usage_partial`,
`price_missing`, `rate_missing`, `tiered_price_ignored`,
`fast_mode_premium_ignored`, `prices_stale`, `no_prices`, `zstd_unavailable`,
`rollout_missing`, `codex_tokens_mismatch`, `codex_total_reset`,
`claude_cost_state_mismatch`, `parent_not_found`, `title_from_prompt`,
`db_immutable_snapshot`, `skipped_lines`, `other_usage_unattributed`.

## 7. Output model (`schema: "dale-trace/1"`)

`captured_at`, `harness`, `source`, `chat`, `pricing`, `nodes[]` (`id`,
`source_id`, `parent_id`, `depth`, `children`, `kind`, `origin`, `provider`,
`title`, `role`, `models`, `effort`, `status`, `started_at`, `ended_at`,
`span_ms`, `active_ms`, `usage`, `usage_status`, `usage_source`, `cost`,
`subtree`, `unattributed`, `notes`), `edges[]` (`delegate`, `native`, `spawn`,
`agent_tool`, `workflow`), `totals`, `by_model`, `coverage`, `warnings`.
`--now` fixes `captured_at`; everything else is deterministic.
