# Transport and lifecycle

Read live tool descriptions before using them; where they differ from this
file, the live schema wins.

## Choose the transport

- Use the harness's native subagent tool when it can run the exact selected
  model and options.
- Use T3 `delegate_task` for another provider's model, options the native tool
  cannot set, a separate checkout, or work that outlives the turn.
- If nothing can run the selected route, report it blocked. Do not replace it
  with another model, ordinary threads, or ad hoc CLI agents.

A child that may delegate must be told so in its brief: harnesses commonly
instruct children to do the work themselves unless the task says otherwise.
Set a nested child's model explicitly; it otherwise inherits its parent's.

## T3 Code

Tool names may carry a normalized MCP prefix such as `mcp__t3_code__`; the
operations below name their suffixes.

### Discover

Call `orchestrator_capabilities` once per conversation, before the first
`delegate_task`, for exact provider instance and model IDs, supported options,
constraints, and child-task support. Call it again only after a dispatch error
or when the user names a provider change. If the tool catalog does not expose
it, make one bounded direct attempt using its known name before reporting it
unavailable.

For ACP runtimes that omit injected MCP tools, check whether
`T3_ACP_MCP_NODE` exists. If present, use the supported transport:

```sh
ELECTRON_RUN_AS_NODE=1 "$T3_ACP_MCP_NODE" ${T3_ACP_MCP_ENTRYPOINT:+"$T3_ACP_MCP_ENTRYPOINT"} acp-mcp-call orchestrator_capabilities '{}'
```

Use the same transport with `delegate_task`, `task_status`, or `task_cancel` and
their JSON arguments. Prefer a structured tool call; shell arguments need real
shell quoting, not `JSON.stringify()` as shell escaping. Do not print unrelated
environment values.

### Dispatch

Use `delegate_task` with a bounded `task`, explicit
`target: {providerInstanceId, model}`, and only supported `target.options` when
needed. Pass the role name where the schema takes one; a role label does not
enforce read-only access, so put the read/write boundary in the child contract.
Keep runtime permissions inherited unless the user requests a supported change;
never broaden the task's authority.

Retain a unique `clientRequestId` per work unit or review round before dispatch,
stable across retries of that same request. Retain the returned `taskId` and
`childThreadId` immediately. The `childThreadId` is backing storage, not a target
for a new review through `t3_thread_send`. New review rounds use `delegate_task`
again with the original brief, prior findings, responses, and unresolved objections.

Use `isolation: "worktree"` for a separate checkout. T3 prepares it from the
parent's local commit; uncommitted changes are not copied. Never build a child's
checkout in the shell or in its `task`. `t3_thread_launch` and `create_threads`
create ordinary top-level conversations; they are not delegated children and
need a user request for separate conversations.

### Wait, recover, and cancel

Prefer `mode: "async"` for long work. Completion wakes the parent; when no useful
local work remains, yield the turn instead of polling or creating watchers.
Use `task_status({taskId})` when a result is needed during active work or after
resuming. Require a terminal task with no pending child runs; a completed turn
with live descendants is not completed work. A bounded `mode: "wait"` is also
available; its timeout ends only the wait, not the child task.

If `hasPendingQuestion` is set, inspect the pending request for that
`childThreadId` with the live pending-request tools and bring a user decision
back to the user. Do not answer a model substitution or permission request on
their behalf.

On timeout or ambiguous creation, retain the request identity and any returned
IDs. Inspect the existing task; never create a second task merely because its
result is missing. If no task ID can be recovered, report unresolved dispatch
and stop dependent work. Retry creation only when the tool's documented
idempotency guarantees recovery for that same request, or rejection proves no
task was created. Distinguish a retry from a new work unit.

On cancellation or conflicting scope changes, call `task_cancel` for affected
task IDs, including pending child runs, and preserve existing artifacts.

## Claude Code

The `Agent` tool runs Claude models only. Set `model` and `effort` on the call;
call values override the agent definition.

- Nesting: a subagent can start its own subagents, by default up to three
  layers below the main conversation. At the limit it has no `Agent` tool and
  does the work itself.
- Waiting: a background subagent notifies the parent when it finishes. Do not
  poll, and never predict its result.
- Worktree: `isolation: "worktree"` gives the child a temporary checkout that
  is branched by default from the repository's default branch, not from the
  session's `HEAD`. Name the base commit in the brief and have the child
  confirm it before it writes.

## Codex

`spawn_agent` runs GPT models only. Pass `model` and `reasoning_effort`
explicitly with fresh context (`fork_turns: "none"`) and the required inputs;
without them a subagent inherits its parent's model and effort. Do not supply
overrides on a full-history fork when the schema requires inheritance. A role
may fix its model or effort: pick a compatible role or the default one, and
never claim that an override changed a fixed setting.

- Nesting: the spawn tool decides. A child without it does the work itself.
- Concurrency: respect the configured cap on open subagent threads.
- Worktree: if the spawn tool offers no isolated checkout, run conflicting
  writers in sequence instead of improvising checkouts.

## Every transport

Keep each child's executor ID, selected route, target, and state. Wait for
every child and its descendants to reach a terminal state, or stop them, before
calling the work complete. On timeout or ambiguous creation, inspect what
exists; never start the same work twice because a result is missing.
