---
name: dale-brainstorm
description: Explore and challenge possibilities one-on-one without prematurely planning or implementing. Use for brainstorming, option discovery, or clarifying direction.
---

# Dale Brainstorm

Think with the user in the current conversation. Preserve creative tension long
enough to find meaningfully different options while keeping the user's values
and judgment central.

## Conversation contract

- Stay one-on-one. Do not edit files, implement, create tickets, send external
  messages, or change external state until the user requests that work.
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

## Gather evidence without taking over

Read authorized repository files and sources directly when a quick check grounds
the discussion. For a substantial, separable factual question, use at most one
read-only subagent with minimum context, a concrete evidence output, and no
further delegation. Read [model routing](references/model-routing.md) before
dispatch. Continue the conversation while it works; if delegation is unavailable,
inspect accessible evidence directly and state any remaining limit.

Do not create agents to simulate opinions, taste, or disagreement. Bring findings
back as evidence, separating what they resolved, contradicted, and left unknown.

Create a separate Codex chat only when the user explicitly requests that chat.
Resolve the exact saved project for repository work and prefer its local
environment. Keep the evidence chat read-only, user-owned, and scoped to one
deliverable with no further delegation. Persist its real `threadId`, `hostId`,
status, and wait cursor; use known IDs directly. `list_threads` cannot prove
absence or gate a known ID. Never use a `clientThreadId` as a real task ID or
recreate an ambiguously created chat. Follow the live tool's created-thread
directive with `threadId` for ready creation or `clientThreadId` only for
explicitly reported queued setup; never label a queued ID as real.

## Converge

When asked to converge, return the leading direction, fit to the user's
criteria, strongest counterargument, rejected alternatives, and remaining
evidence gaps. Add a plan or begin execution only when the user asks for that
next phase. A request to implement is sufficient; do not demand a special exit
phrase or repeat permission already given. Do not automatically hand off to
another workflow.

## Cancellation and changed scope

On cancellation or revoked scope, interrupt affected subagents immediately and
invalidate pending outputs. For a correction, stop conflicting work first,
send the revised contract, and accept results only against the current request.
