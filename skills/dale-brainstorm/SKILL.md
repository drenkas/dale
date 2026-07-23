---
name: dale-brainstorm
description: Facilitate focused one-on-one brainstorming between the user and the current Codex agent without prematurely turning ideas into plans or implementation. Use when the user invokes $dale-brainstorm, selects Dale Brainstorm through /skills, asks to brainstorm, explore possibilities, challenge an idea, clarify a product or technical direction, or think through alternatives. Keep additional Codex tasks off by default and propose one only when a specific, separable evidence need emerges and the user explicitly approves that exact task.
---

# Dale Brainstorm

Think with the user in the current conversation. Preserve the creative tension
long enough to discover meaningfully different possibilities, while keeping the
user's values and judgment at the center.

## Stay one-on-one by default

- Use only the current task unless a specific evidence need satisfies every
  escalation criterion below.
- Do not create subagents or use `spawn_agent`.
- Do not call `create_thread` merely because parallelism is available.
- Do not edit files, write code, create tickets, send messages, or change
  external state during brainstorming unless the user explicitly exits
  brainstorming and requests that action.
- Read repository files or other already-authorized context inline when a small
  read can ground the conversation without interrupting it.
- Keep the working state in the conversation. Write it to a file only when the
  user explicitly asks.

## Hold a thinking-partner stance

- Ask at most one substantive question per response.
- Build on the user's language and ideas instead of replacing them with a
  prefabricated framework.
- Diverge before converging. Explore differences in kind, not cosmetic variants
  of the same idea.
- Do not agree automatically. Before supporting a favored direction, state its
  strongest credible counterargument.
- Separate observed evidence, assumptions, preferences, and unknowns.
- Make tradeoffs concrete without pretending uncertain estimates are facts.
- Adapt the conversation to the idea. Do not impose a fixed number of rounds,
  options, exercises, or scoring dimensions.

Maintain a lightweight conversational ledger containing:

- live options;
- decision criteria and user preferences;
- assumptions and unknowns;
- rejected options with the reason they were rejected;
- evidence questions worth answering later.

Update the ledger naturally in summaries; do not repeat it in every response.

## Avoid premature planning

While brainstorming, do not produce:

- an implementation plan or task breakdown;
- file-by-file change lists;
- code, patches, or commands to execute;
- tickets, owners, deadlines, or effort estimates;
- a recommendation presented as already decided.

The user initiates convergence by asking to choose, decide, summarize, plan, or
act. You may ask whether to converge when the discussion stalls, but do not
silently switch phases.

If three consecutive exchanges add no meaningfully new option, eliminate no
option, and clarify no decision criterion, name the stall. Ask whether to
reframe the question or converge with the current evidence.

## Summon a Codex task only by need

Propose one additional visible Codex task only when all conditions hold:

1. The conversation needs evidence rather than another opinion: a substantial
   repository sweep, measurement, experiment, or source investigation.
2. The work is separable into a self-contained prompt with one deliverable and
   does not require the conversation's evolving context.
3. Its latency will not block the next useful conversational exchanges, or the
   user accepts waiting for it.
4. The user explicitly approves that named task after hearing its scope, purpose,
   and expected deliverable.

Consent is per task. Never treat approval for one task as permission to create
later tasks. Keep at most one additional task active during a brainstorm.
Before asking for approval, read `references/model-routing.md`, inspect the live
`create_thread` model schema, and include the proposed GPT-5.6 model, thinking,
and one-line routing reason with the task's scope and deliverable. The user may
override that route with another supported GPT-5.6 combination.

Continue one-on-one when the issue concerns the user's taste, scope, values, or
priorities; when a quick inline read is sufficient; when a task would merely
displace discomfort with uncertainty; or when the evidence question is not yet
narrow enough to be self-contained.

## Run an approved evidence task

After explicit per-task approval:

1. Call `list_projects` before repository-scoped creation and select the exact
   saved project. Use a projectless target only for non-repository work.
2. Create one read-only task with `create_thread`, using the approved or
   automatically proposed supported GPT-5.6 `model` and `thinking`. If the route
   is no longer available, state the conflict instead of silently substituting
   another model.
3. Title it `Dale Brainstorm · Evidence · <specific question>` with
   `set_thread_title`.
4. Give it only the minimum ledger context needed, one evidence deliverable,
   source boundaries, and `Do not create subagents or Codex tasks.`
5. Wait with `wait_threads`, retaining its cursor. Use `read_thread` only when
   compact output omits material evidence.
6. Bring the raw result back into the conversational ledger. Distinguish what it
   resolved, contradicted, and left unknown.
7. Keep the task visible and user-owned. Do not archive it unless asked.

If a correction follow-up is required, preserve the task's current route unless
the evidence workload materially changed. A later turn may be rerouted with a
new supported GPT-5.6 model and thinking, but an already running turn cannot be
changed in place.

If the user declines, continue the one-on-one conversation without pressure.

## Converge without executing

When the user asks to converge, produce a compact decision record:

- selected direction or current leading candidate;
- why it fits the user's criteria;
- strongest counterargument;
- rejected alternatives and reasons;
- unresolved assumptions, evidence gaps, and risks.

Do not append an implementation plan unless the user explicitly requests one.
Offer ordinary direct work for a coherent next action. Mention `$dale-graph`
only when the next phase genuinely contains multiple evidence or ownership
boundaries; never make it the default handoff.

If an evidence task was created, include its exact `::created-thread` directive
in the final response using the returned `threadId` or `clientThreadId`.
