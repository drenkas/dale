---
name: dale-max
description: Orchestrate consequential work through bounded subagents, independent review, evidence-based acceptance, and focused revision. Use for consequential work where a wrong result is costly and independent verification is worth its price.
---

# Dale Max

For consequential work, the calling agent is the orchestrator and planner: own decomposition, dependencies, integration, and the
final gate. Delegate substantial research, design, implementation, and
verification to bounded subagents, using maximum useful available concurrency
without filler or duplicated work. If delegation is unavailable, make useful
progress locally and disclose the unavailable delegation gate.

## Size each unit

Max works through subagents; do not ask whether to split. Size each work unit
instead. Do it yourself when briefing, waiting for, and checking
a child would cost more than the work. Delegate when the unit is substantial,
can progress independently, or needs an independent view. Never edit files a
running reviewer is inspecting.

## Establish the acceptance contract

Define the outcome, authority, criteria, exact target, direct evidence, and
material failure modes. Respect user-specified resource limits. Do not impose
unlimited review or a fixed number of reviewers. Read
[contracts.md](references/contracts.md) for worker and reviewer contracts.

Take each role's model from the saved team; read [team.md](references/team.md).
Use the catalog in [model-routing.md](references/model-routing.md) only for
roles the team does not cover. Explicit user settings and the live dispatch
schema take precedence. Read [transport.md](references/transport.md) for the
harness in use.

## Implement and review

Assign substantive implementation to bounded workers. Several writers,
the coordinator included, may share a checkout when they cannot conflict:

- each has an exclusive write scope stated in its contract;
- shared files such as lockfiles, generated output, and common configuration
  have one owner;
- workers leave Git state alone;
- acceptance evidence is taken after every writer of the affected area finishes.

When scopes cannot be separated, run the writers in sequence or give each its
own worktree from the same commit. Uncommitted changes do not reach a worktree:
if a child needs them, tell the user instead of committing or dropping them.
Compare the results, integrate one coherent change into the parent checkout,
then verify and review that. Preserve unrelated changes. Reuse the implementer
for focused revisions. Follow explicit user limits and never expand scope.

Delegation may nest. A child may hand part of its unit to its own child when
that saves cost or time, such as an architect sending source reading to a
researcher. State the allowance in the child's contract. A nested child takes
its route from its own role, never from its parent's model or effort, and stays
inside its parent's authority and write scope. The parent waits for it and
reports it; a unit is not complete while any descendant is running.

Run a fresh independent reviewer against the original requirement, raw artifact,
and evidence, without the implementer's intended verdict. Review arranged by
the maker of the artifact is not independent. Delegate distinct review
questions only when each covers a material failure mode that could change
acceptance. A reviewer role with several models runs on all of them. A separate
preflight review is useful for costly or hard-to-reverse plans; do not require
it for every cycle. The coordinator performs complementary work while review
runs.

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

On revision, send failed criteria and supporting evidence to the owning worker,
or make a small correction yourself. Recheck affected risks; reuse still-valid
evidence for unchanged artifacts. A fresh reviewer is warranted when
independence or changed failure modes require it, not merely because a cycle
counter advanced. If two revisions fail to move the same primary signal,
re-plan instead of repeating them.

Stop after acceptance, a rejected approach with no credible alternative, a real
blocker, a user limit, or interruption. Additional review needs a named uncovered
risk; passing required checks is not an invitation to broaden tests indefinitely.
Ask for new authority only when needed; existing authorization persists.

Before completion, finish or interrupt all delegated workers and their
descendants, and reconcile any uncertain dispatch. Report the result, decisive
evidence, unresolved risks, and any unmet gate. Distinguish code/static, test,
browser, and deployed/runtime evidence so one proof level cannot imply another.
Keep detailed cycle and agent accounting for a requested audit, not every reply.
The legacy image in `assets/` illustrates an older thread topology and is not
an execution requirement.

## Cancellation and changed scope

On cancellation or revoked scope, interrupt affected workers immediately before
other work, invalidate pending outputs, and preserve existing changes. Update
or disable related scheduled work only within the user's cancellation scope.
For a correction, stop conflicting work first, then send the revised contract;
accept subsequent results only against the current request and artifact state.
