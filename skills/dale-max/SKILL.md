---
name: dale-max
description: Orchestrate consequential work through bounded subagents, independent review, evidence-based acceptance, and focused revision. Use only for explicit $dale-max or a direct request to run Dale Max.
---

# Dale Max

For explicitly requested consequential work, the calling agent is the
orchestrator and planner: own decomposition, dependencies, integration, and the
final gate. Proactively delegate substantial research, design, implementation,
and verification to bounded subagents when useful, using maximum useful
available concurrency without filler or duplicated work.
Keep tiny corrections and orchestration work local; do not make substantive
implementation the coordinator's default. If delegation is unavailable, make
useful progress locally and disclose the unavailable delegation gate. Separate
Codex tasks are optional only when the user explicitly requests them.

## Establish the acceptance contract

Define the outcome, authority, criteria, exact target, direct evidence, and
material failure modes. Respect user-specified resource limits. Do not impose
unlimited review or a fixed number of reviewers. Read
[contracts.md](references/contracts.md) for worker and reviewer contracts.
Read [model-routing.md](references/model-routing.md) when selecting routes.
Keep established Terra/Sol routing; explicit user settings and the live dispatch
schema take precedence.

## Implement and review

Assign substantive implementation to a bounded worker with exclusive write
scope. Use live collaboration tools and maximum useful available capacity. Allow
one mutating writer per shared checkout, including the coordinator and
disjoint-file workers; read-only work may run in parallel. Workers do not spawn
more agents or separate tasks. Preserve unrelated changes. Reuse the implementer
for focused revisions. Follow explicit user limits and never expand scope.

Run a fresh independent reviewer against the original requirement, raw artifact,
and evidence, without the implementer's intended verdict. Delegate distinct
review questions only when each covers a material failure mode that could change
acceptance. A separate preflight review is useful for costly or hard-to-reverse
plans; do not require it for every cycle. The coordinator performs complementary
work while review runs.

If independent review is unavailable, continue implementation and direct checks
that remain useful, but report the Max independence gate as unavailable. Do not
claim a same-agent review is independent or issue final PASS without the gate.

## Accept or revise

Resolve findings by direct evidence, target identity, freshness, and coverage.
Keep a compact criterion-to-evidence record. Integrate accepted changes and run
required checks plus sufficient behavioral validation in the destination.
Shared-checkout changes already exist; inspect the actual diff and never replay
a reported patch. An isolated worker's success does not prove delivery.

Use these internal outcomes:

- PASS: material criteria have direct evidence and independent review is complete.
- REVISE: a concrete bounded correction could satisfy failed criteria.
- REJECT: the approach cannot credibly satisfy the outcome.
- BLOCKED: required input, authority, independence, or evidence is inaccessible.

On revision, send failed criteria and supporting evidence to the owning worker.
Recheck affected risks; reuse still-valid evidence for unchanged artifacts.
A fresh reviewer is warranted when independence or changed failure modes require
it, not merely because a cycle counter advanced. If two revisions fail to move
the same primary signal, re-plan instead of repeating them.

Stop after acceptance, a rejected approach with no credible alternative, a real
blocker, a user limit, or interruption. Additional review needs a named uncovered
risk; passing required checks is not an invitation to broaden tests indefinitely.
Ask for new authority only when needed; existing authorization persists.

Before completion, finish or interrupt all delegated workers and reconcile any
uncertain dispatch. Report the result, decisive evidence, unresolved risks, and
any unmet gate. Distinguish code/static, test, browser, and deployed/runtime
evidence so one proof level cannot imply another. Keep detailed cycle and agent accounting for a requested audit, not every reply.
The legacy image in `assets/` illustrates an older thread topology and is not
an execution requirement.

## Optional separate tasks

Use only when the user explicitly requests separate Codex tasks or a fork;
selecting this skill alone authorizes subagent work. Do not create a task merely
because delegation is unavailable. Resolve the exact project with `list_projects`
and use its local environment unless the user explicitly requests a worktree.
The calling coordinator
alone creates, messages, or manages tasks within the user's authorization.

Before creation, retain a sanitized invocation/run/node identity, dispatch key,
unique title, project/host, ownership, and `dispatching` state in a conversation
manifest. Recover that state on continuation. Record each exact returned
`threadId`, `hostId`, status, and wait cursor immediately. Known real IDs are
authoritative; listings cannot prove absence or justify recreation. An unexpected
`clientThreadId` is diagnostic only. Timeout or ambiguous creation fails closed:
never recreate unless a response proves pre-creation schema rejection. Validate
a user-supplied actual ID once with `read_thread` for the specified scope. Adopt
a discovered ID only when readable identity and retained creation evidence give
an unambiguous mapping. Capture concurrent creation results separately.

Use fresh sanitized leaf contracts, retained IDs and bounded `wait_threads`
with cursors. Do not infer identity from title or recency, replay shared-checkout
patches, or allow nested task creation. Created user-owned tasks remain available;
archive or hand off only when explicitly asked. Before completion, every created
or discovered child task requires a real ID, terminal state, reason and report;
unresolved creation prevents PASS. Follow the live tool's created-thread
directive with `threadId` for ready creation or `clientThreadId` only for
explicitly reported queued setup. Never label a queued ID as real or treat the
directive as resolving uncertainty; emit none for subagents or ambiguous results.

## Cancellation and changed scope

On cancellation or revoked scope, interrupt affected workers immediately before
other work, invalidate pending outputs, and preserve existing changes. Update
or disable related scheduled work only within the user's cancellation scope.
For a correction, stop conflicting work first, then send the revised contract;
accept subsequent results only against the current request and artifact state.
