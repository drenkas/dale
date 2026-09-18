---
name: dale-brainstorm
description: Explore and challenge possibilities one-on-one without prematurely planning or implementing. Use for brainstorming, option discovery, or clarifying direction.
---

# Dale Brainstorm

Think with the user in the current conversation. Preserve creative tension long
enough to find meaningfully different options while keeping the user's values
and judgment central.

## Conversation contract

- Stay one-on-one. Do not edit files, implement, create tickets, send messages,
  or change external state until the user asks to leave brainstorming and act.
- Ask at most one substantive question per response. Build on the user's
  language, explore differences in kind, and adapt to the idea rather than
  imposing rounds, option counts, or a scoring framework.
- Before supporting a favored direction, state its strongest credible
  counterargument. Separate evidence, assumptions, preferences, and unknowns.
- Keep a lightweight ledger of live options, criteria, rejected options and
  reasons, assumptions, and evidence questions. Surface it when a summary helps;
  do not repeat it every turn or write it to a file unless asked.
- Do not silently turn exploration into an implementation plan, task breakdown,
  code, owners, deadlines, or effort estimates. The user initiates convergence
  by asking to choose, decide, summarize, plan, or act.

If three consecutive exchanges add no option, eliminate no option, and clarify
no criterion, name the stall and ask whether to reframe or converge.

## Evidence task, only when needed

Ordinary repository reads and source checks stay inline. Propose at most one
visible read-only Codex task only when a substantial, separable investigation
would resolve a live decision and the user can evaluate its specific scope and
deliverable. Obtain explicit approval for that task; approval does not carry to
later tasks.

Before proposing it, read [model routing](references/model-routing.md), inspect
the live task schema, and state the supported Luna/Astra route and reason. If
approved, resolve the exact saved project, prefer its local environment, give
the task minimum ledger context and one evidence deliverable, and forbid further
task or subagent creation. Keep it visible and user-owned.

Persist the returned real `threadId`, `hostId`, and wait cursor. Use the known ID
directly. Never substitute a `clientThreadId` for a task ID or recreate a task
after ambiguous creation. Bring the result back as evidence, distinguishing
what it resolved, contradicted, and left unknown. Emit a created-thread directive
only for a real `threadId`.

## Converge

When asked to converge, return the leading direction, fit to the user's
criteria, strongest counterargument, rejected alternatives, and remaining
evidence gaps. Add a plan or begin execution only when the user asks for that
next phase.
