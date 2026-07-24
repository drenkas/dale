---
name: dale-max
description: Run a maximum-assurance resident-worker orchestration loop with persistent forked Codex threads, thread-local task-derived subagents, plan review, synthesis, an explicit pass gate, and progress-gated revision feedback without preset cost, token, reviewer-count, or cycle budgets. Use only when the user explicitly invokes $dale-max, selects Dale Max through /skills, or directly asks to run Dale Max for consequential implementation, debugging, migration, audit, or delivery work.
---

# Dale Max

Execute the fixed control loop shown in `assets/dale-max-graph.jpg`. Dale Max is
the deliberate, high-assurance companion to Dale Graph: Dale Graph creates a
unique topology from the task; Dale Max keeps this review-and-revision topology
while generating its reviewer lenses, evidence obligations, and routes from the
task at runtime.

## Require explicit invocation

- Run only after explicit `$dale-max` selection, selection through `/skills`, or
  a direct request to run Dale Max. Do not activate it implicitly for an
  ordinary difficult task.
- Explicit use authorizes visible Codex thread creation, thread-local
  subagents, and per-node GPT-5.6 routing. It does not authorize new
  permissions, destructive actions, external writes, Git mutation, or ignored
  repository instructions.
- If Codex thread tools or resident-thread `spawn_agent` capability are
  unavailable, report the blocker. Do not simulate resident threads inside one
  response or replace them with coordinator-local subagents.

## Map the graph to Codex

Keep two green resident Codex threads across feedback cycles. A resident node
must be created with `fork_thread` or the `create_thread` fallback; never create
it with `spawn_agent`:

1. **Worker** — a visible fork of the calling thread that owns the implementation
   or primary artifact.
2. **Reviewer 1 / Lead Reviewer** — a separate visible fork that owns
   adversarial review and dispatches fresh, task-derived reviewer subagents.

Run the white nodes ephemerally:

- **Planner, Synthesizer, Pass Gate, and Final Compressor** are phases owned by
  the calling coordinator.
- **Reviewer 2..N** are fresh `spawn_agent` subagents of Lead Reviewer. Each
  receives the Worker's raw artifact directly as input so the logical
  Worker-to-reviewer edges in the diagram remain intact. Create only the lenses
  justified by the current artifact and failure modes; there is no stock role
  list and no target reviewer count.
- **Plan Reviewer** is a fresh coordinator-local `spawn_agent` subagent. Run a
  preflight review as soon as Planner produces the plan, then run a fresh result
  review for each material cycle. It compares plan, result, and evidence without
  inheriting the coordinator's intended verdict.
- Worker and Lead Reviewer may each spawn one bounded layer of node-local
  subagents when their own work contains genuinely independent parts. No
  subagent may create another agent or Codex thread.

Read `references/contracts.md` before dispatching any node. Read
`references/model-routing.md` before selecting models or reasoning effort.

## Run the control loop

### 1. Plan against observable proof

Preserve the user's exact intent and wording, but replace any credential, token,
secret, raw `.env` value, or private key material with an explicit redaction
before storing or forwarding it. Define the outcome, acceptance criteria,
forbidden outcomes, authority boundary, artifacts, direct primary evidence,
and failure conditions. Do not impose a cost, token, reviewer-count, dispatch,
or cycle budget. Generate every independent review lens that could materially
change the verdict and schedule all of them in as many batches as live capacity
requires.

Do not ask for confirmation unless the plan needs new authority or a material
user choice. Show the compact execution contract and routing before dispatch:

```text
DALE MAX
outcome: <observable result>
stop: <evidence-based terminal and re-plan conditions>
resident worker: <scope> — <GPT-5.6 model / thinking> — <reason>
resident reviewer: <scope> — <GPT-5.6 model / thinking> — <reason>
review lenses: generated after inspecting the artifact; no preset count
proof: <criterion -> direct signal>
```

### 2. Review the plan and fork the resident threads

Spawn the first ephemeral Plan Reviewer as soon as the plan exists. Run it in
parallel with resident setup when safe; if it finds a material plan flaw before
Worker proceeds irreversibly, revise or steer the plan first. This preserves the
diagram's direct `Planner -> Plan Reviewer` edge instead of reviewing the plan
only after implementation.

Prefer `fork_thread` without `threadId` so both resident nodes inherit the
calling thread's completed history and project context. An active user turn and
unfinished response are not inherited, so send the current request with secret
values redacted plus the complete role contract immediately with
`send_message_to_thread`.

- Use a same-directory fork for read-only work and for one designated writer
  when no concurrent node can touch the same files.
- Use a worktree for isolated mutation when the live tool supports it.
- Use `create_thread` only when forking is unavailable; pass all context
  explicitly and label it as a fresh fallback.
- Title nodes `Dale Max · <run> · Worker` and
  `Dale Max · <run> · Reviewer 1`.
- Keep threads visible and user-owned. Never archive them unless asked.
- Never commit, merge, stash, reset, push, or delete unless explicitly asked.

In the first `send_message_to_thread` prompt for **each** resident thread,
include this authority block:

```text
RESIDENT CODEX THREAD
You are a visible resident Codex thread, not a coordinator-local subagent.
You are explicitly authorized to use your local `spawn_agent` tool for
independent work inside your role, read/write scope, and current permissions.
Subagent use is adaptive, but the capability must be available. Inspect your
live tool surface before work. If `spawn_agent` is unavailable or multi-agent
work is disabled, return `SUBAGENT_CAPABILITY: UNAVAILABLE` and `STATUS:
BLOCKED`; do not simulate delegation. Every spawned subagent is one level deep
and must create neither subagents nor Codex threads.
```

Do not infer this authority from inherited history alone. Send it explicitly to
Worker and Reviewer 1 on their first turn and preserve it on later feedback
turns. Record `SUBAGENT_CAPABILITY: AVAILABLE | UNAVAILABLE` for both resident
thread IDs before accepting their artifacts.

For an isolated Worker, require exact diffs, changed artifacts, and validation.
Review may mark that artifact provisionally acceptable, but that is not the
final gate. Apply only the provisionally accepted changes in the destination
checkout without a Git merge, run the primary validation there, and only then
issue `PASS`. Failed integration or validation returns `REVISE`; if safe
integration is impossible within current authority, return `BLOCKED`. A
worker-local success is not a delivered result.

Create Reviewer 1 at the start, but send it the candidate artifact only
after Worker returns one. The reviewer must not be primed with the desired
conclusion.

### 3. Produce, fan out, and synthesize

Worker returns one artifact plus exact evidence. Forward that raw artifact,
without coordinator synthesis or a desired verdict, to Reviewer 1. Reviewer 1
derives material failure modes and creates Reviewer 2..N with
`spawn_agent` only when independent lenses improve confidence. It passes the
same raw Worker artifact to each reviewer and integrates their raw reports
without majority voting. Live concurrency limits only scheduling, not coverage:
queue additional batches until every material lens has completed or reached a
real `BLOCKED` state. Never drop a material lens merely to save time, tokens, or
model spend.

In parallel with artifact review when capacity allows, spawn a fresh result-mode
Plan Reviewer with the original plan, preflight feedback, actual result,
validation, and discovered constraints. It identifies plan drift, missing
obligations, and unnecessary work; it does not rewrite the deliverable.

The coordinator then synthesizes:

- worker artifact and primary-signal evidence;
- Reviewer 1 report and raw ephemeral reviews;
- Plan Reviewer verdict;
- repository/runtime evidence gathered directly by the coordinator.

Resolve contradictions by directness, freshness, source quality, and fit. Never
average reviewer opinions or hide unresolved uncertainty.

### 4. Gate and revise

Evaluate every acceptance criterion and return exactly one internal verdict:

- `PASS` — direct evidence satisfies every material criterion;
- `REVISE` — a bounded correction by the resident Worker can plausibly pass;
- `REJECT` — the approach is structurally wrong or unsafe;
- `BLOCKED` — required authority, input, dependency, or runtime is unavailable.

On `REVISE`, send precise failed criteria, evidence, and the smallest required
change back to the same resident Worker. Reuse Reviewer 1 for continuity;
spawn fresh ephemeral reviewers when the artifact or failure modes changed.
Spawn a fresh Plan Reviewer for the next material cycle.

If two revisions fail to move the same primary signal, stop patching harder and
re-plan with a different evidence path or approach. If re-planning cannot
produce a credible way to move that signal, return `REJECT` or `BLOCKED` rather
than repeating the same loop. Continue otherwise until `PASS`, `REJECT`,
`BLOCKED`, or explicit user interruption.

### 5. Compress the passed result

The image's “Turn into haiku” node means editorial compression by default, not
a literal 5-7-5 poem. After `PASS`, produce the shortest complete outcome-first
handoff that still states:

- what changed or was proved;
- the exact primary signal and status;
- remaining risk, if any;
- created resident thread directives.

Write a literal haiku only when the user asks for one. Do not let compression
erase evidence, blockers, safety notes, or requested detail.

## Keep a run ledger

Track the state from `references/contracts.md` after every node completion.
Wait in bounded intervals, reuse task wait cursors, and re-evaluate as soon as a
resident node needs attention. A running turn cannot be rerouted in place;
change model or thinking only on a later follow-up when its workload materially
changed.

## Completion contract

Report the plan, resident thread IDs and subagent capability checks, ephemeral
reviewers, cycle verdicts, rejected findings, accepted artifact, exact final
validation, and remaining risk. Include one `::created-thread` directive per
visible resident thread using its exact `threadId` or `clientThreadId`. Never
emit a thread directive for a subagent.
