---
name: dale-graph
description: Synthesize and run a unique graph of visible Codex tasks, each able to orchestrate its own bounded node-local subagents, directly from the current request, repository boundaries, dependencies, evidence needs, and runtime discoveries. Use when the user invokes $dale-graph, selects Dale Graph through /skills, explicitly asks for a task graph or multiple Codex threads, or wants complex work decomposed into adaptive roles and relationships created on the fly instead of selected from predefined graph patterns.
---

# Dale Graph

Construct the graph from the task itself. Do not classify the request into a
named graph pattern and do not choose from a catalog of fixed shapes. Generate
the nodes, roles, edges, gates, and execution order from the work that must
actually happen.

## Use visible Codex tasks with node-local subagents

- Treat explicit `$dale-graph` selection, a direct request to run Dale Graph, or
  an explicit request for a task graph as authorization to create visible Codex
  tasks.
- If this skill matched implicitly from an ordinary task, explain that Dale
  Graph creates visible tasks and get consent before calling `fork_thread` or
  `create_thread`.
- Use `fork_thread`, `list_threads`, `list_projects`, `create_thread`,
  `set_thread_title`, `wait_threads`, `read_thread`, and
  `send_message_to_thread`.
- Use `handoff_thread` only when the user explicitly asks to move or take over a
  task. Never hand off the calling task.
- Keep top-level graph nodes as visible Codex tasks. Do not replace them with
  coordinator-local subagents or hidden delegation.
- Authorize every created task to use its own node-local `spawn_agent`
  orchestration when useful. Permission is universal; spawning is adaptive, not
  mandatory.
- Read `references/node-subagents.md` before dispatch. A node may create
  subagents only inside its owned deliverable and scopes. It remains responsible
  for their coordination, verification, and one integrated result.
- Node-local subagents must not create Codex tasks or further subagents. Only the
  top-level coordinator may change the visible graph.
- Keep every created task user-owned and inspectable. Do not archive it unless
  the user asks.
- Explicit `$dale-graph` selection or a direct request to run Dale Graph also
  delegates per-node GPT-5.6 model and reasoning selection to this skill. For an
  implicit match, get consent for both visible task creation and automatic
  routing; otherwise omit model and reasoning overrides.
- Read `references/model-routing.md` and route each node independently. Explicit
  GPT-5.6 model or reasoning constraints in the current request override
  automatic routing. If a requested route falls outside the skill's GPT-5.6
  boundary or the live tool schema, state that conflict instead of silently
  substituting a different model.
- If task tools are unavailable, report the blocker. Do not imitate a graph
  inside one task and claim multiple agents ran.

## Synthesize the graph from first principles

Read `references/graph-synthesis.md`, `references/model-routing.md`, and
`references/node-subagents.md` before dispatch. Derive the graph in this order:

1. Preserve the user's request verbatim.
2. Define the observable outcome, acceptance criteria, forbidden outcomes, and
   authority boundary.
3. Derive the artifacts and evidence required to prove that outcome.
4. Turn distinct work units into nodes only when they have a separate
   deliverable, evidence boundary, expertise lens, or mutation owner.
5. Generate each role from its work unit. Do not pull roles from a stock list.
6. Add an edge only when one node consumes another node's artifact, must wait
   for its decision, conflicts over a resource, or receives its failed output
   for revision.
7. Insert proof nodes at the exact boundaries where independent verification is
   needed. Do not force one global verifier or a fixed verification stage.
8. Merge nodes that need the same context and cannot progress independently.
9. Split nodes when their inputs and outputs are genuinely independent.

Do not target a default number of tasks. The number must fall out of the
dependency and ownership analysis. Minimize nodes without collapsing distinct
evidence or write boundaries.

Treat the synthesized graph as current state, not a frozen plan. Add, remove,
merge, split, or reroute nodes when runtime evidence changes what must happen.

## Expose the generated graph

Before dispatch, show only the graph justified by current evidence. It may be a
partial graph containing the first ready nodes plus provisional downstream work
whose shape depends on their results:

```text
NODES
<id> — <generated role> — <deliverable> — <read/write scope> — ready | provisional
  context: fork of calling task | fresh fallback
  route: <GPT-5.6 model> / <thinking> — <one-line reason>
  inner: node-local subagents allowed — <likely split trigger or direct-work reason>

EDGES
<from> -> <to> — <artifact, decision, conflict, or revision reason>

READY
<nodes whose dependencies are satisfied>

PROOF
<acceptance criterion> -> <how and where it will be verified>
```

Do not label the graph with a named pattern. Explain why each node exists and
why each edge is necessary. Do not present provisional work as a commitment.
This is a status update, not a confirmation request, unless the graph needs new
authority or a material user choice.

If synthesis produces only one coherent work unit, say that a multi-task graph
would add ceremony without independent evidence. Offer to continue directly or
create one visible task; do not manufacture nodes to justify the skill name.

## Resolve execution environments

Prefer `fork_thread` so every visible node inherits the calling task's completed
history and project context. Always fork the calling coordinator by omitting
`threadId`; do not fork predecessor nodes. Use `create_thread` only when
`fork_thread` is unavailable or the source cannot be forked. For that fallback,
call `list_projects`, select the exact saved project, and use a projectless
target only for work with no repository.

- Use a same-directory fork for read-only nodes and for one designated writer
  when no concurrent node can touch the same files.
- Use separate worktree forks for independent mutating nodes.
- Do not specify `startingState` unless the user explicitly asked to start from
  the current working tree or a named existing branch in the fresh-thread
  fallback. `fork_thread` owns its own source-state semantics.
- Represent overlapping write scopes as conflicts and never run those nodes
  concurrently.
- Never commit, merge, stash, reset, push, or delete unless the user explicitly
  requested that Git action.

When worktree outputs must be integrated without an authorized Git merge, make
the node return exact diffs, artifacts, and validation. Apply only accepted
changes in the destination checkout.

## Dispatch just in time

Give every node the contract from `references/graph-synthesis.md`, including:

- its generated role and unique `Dale Graph · <run> · <role>` title;
- exact inputs and satisfied dependencies;
- one observable deliverable;
- read and write ownership;
- forbidden actions and applicable `AGENTS.md`;
- required evidence and stop conditions;
- permission to spawn bounded node-local subagents under
  `references/node-subagents.md`;
- `Do not create Codex tasks. Do not let your subagents create agents or Codex
  tasks.`

Immediately before dispatch, read the current `fork_thread`,
`send_message_to_thread`, and fallback `create_thread` schemas and intersect
their supported combinations with the GPT-5.6 routing catalog. Choose the
cheapest sufficient model and effort for each ready node. Record and show the
routing reason; never silently inherit the coordinator's expensive defaults.

For the preferred fork path:

1. Call `fork_thread` without `threadId` so the source is the calling task. Omit
   `environment` for a same-directory fork; request `worktree` only when the
   node needs isolated mutation.
2. Remember that a fork copies completed history only. The active user turn and
   unfinished coordinator response are not inherited.
3. Send the child a follow-up containing the current request verbatim plus its
   complete node contract. Set the selected `model` and `thinking` on this
   `send_message_to_thread` call.
4. Title the child after its `threadId` exists. A queued worktree fork may first
   return only `clientThreadId`; use recent `list_threads` results to resolve the
   created child before messaging, titling, or waiting on it.

For the fresh fallback, pass the node prompt, selected `model`, and `thinking`
directly to `create_thread`. Mark the node as fresh context and include every
required input explicitly.

Create only nodes that are ready. Create independent ready nodes without
waiting between calls, then title them with `set_thread_title`. Retain
`threadId`, `hostId`, model, thinking, routing reason, status, dependencies,
context origin, source thread, `clientThreadId` when queued, outputs, node-local
subagent summary, and wait cursor in the graph state.

If more than eight nodes are ready, dispatch and wait in concurrency batches of
at most eight. This is a tool limit, not a property of the graph. Do not invent
dependencies merely to reduce concurrency.

## Rebuild while running

Use one bounded `wait_threads` call for current ready nodes. Reuse `afterCursor`
values. The first completion or attention request triggers graph re-evaluation;
do not wait for every node in the current concurrency batch before rebuilding.
Use `read_thread` only when compact progress omits material evidence.

After every completed node:

1. Attach its raw artifact and evidence to the graph.
2. Re-evaluate downstream assumptions and acceptance coverage.
3. Unlock nodes whose real dependencies are now satisfied.
4. Add a new node when the result reveals necessary work that was not knowable
   earlier.
5. Remove a node whose purpose disappeared.
6. Merge newly redundant nodes.
7. Add a conflict-resolution node when evidence contradicts and neither source
   dominates.
8. Add or move a proof node when risk or the primary signal changes.
9. Route a focused revision back to the owning node when correction is
   plausible.
10. Rebuild the affected downstream graph when a shared premise fails.

Whenever the graph grows, compare every added node with the baseline of doing
the work in the coordinator. Remove nodes whose independence or evidence value
does not justify their coordination cost.

Pass predecessor artifacts verbatim with `send_message_to_thread` or in a newly
created task prompt. Preserve the receiving task's current model settings unless
new evidence changes the node's workload enough to justify rerouting. A
follow-up may set a new supported GPT-5.6 `model` and `thinking`, but an already
running turn cannot be changed in place.

If two revisions fail to move the same primary signal, reject that route and
derive a different work unit instead of retrying harder.

## Verify and integrate

Map every acceptance criterion to direct evidence. Generate independent proof
tasks for material correctness, security, freshness, destructive actions, or
cross-layer behavior. Let the coordinator perform cheap low-risk checks when a
separate task would add no independent evidence.

Accept, revise, or reject each artifact independently. Resolve conflicts by
directness, freshness, source quality, and fit—not by majority vote. Merge only
accepted artifacts and preserve unresolved uncertainty.

For repository changes, validate the integrated user-visible or contract-level
outcome in the destination checkout. Worker-local green checks are supporting
evidence, not final proof.

## Report completion

Report the generated nodes and edges, node-local subagent trees and their
contributions, runtime graph mutations, rejected routes, merged result, exact
primary-signal validation, and remaining risks. Include one `::created-thread`
directive per created task, using the exact `threadId` or `clientThreadId`
returned by `create_thread`. Do not emit created-thread directives for
node-local subagents.
