# Luna and Astra routing for Dale Max

Dale Max routes each resident thread and ephemeral reviewer independently. The
live task, messaging, and subagent tool schemas are authoritative. Route only
among available Luna and Astra models and supported reasoning efforts.

## Catalog

| Route | Model ID | Best fit | Normal thinking |
|---|---|---|---|
| Luna standard | `gpt-5.6-luna` | Mechanical extraction, narrow checks, fresh low-risk reviewer lenses | `high` |
| Luna deep | `gpt-5.6-luna` | Everyday implementation, debugging, integration, ordinary review | `high` or `max` |
| Astra | `gpt-6-astra` | Consequential ambiguity, difficult cross-layer synthesis, adversarial proof | `high` |

The exact model IDs and accepted efforts can change. Inspect the live schema
immediately before dispatch. `fork_thread` does not select a route; set model
and thinking on the first `send_message_to_thread` follow-up. If no allowed
Luna or Astra route is available, return `BLOCKED`. Never omit the override when that
could silently inherit a model outside the Luna and Astra boundary.

Dale routes Luna only at `high` or `max`; use `high` for bounded work and
`max` when deeper reasoning benefits the result.

## Selection

1. Obey explicit user constraints that fit the live Luna and Astra catalog.
2. Use Luna for bounded mechanical work whose correctness does not depend on
   deeper synthesis.
3. Use Luna with `high` or `max` reasoning for substantial implementation or
   coupled reasoning.
4. Use Astra when stronger reasoning can materially change a consequential
   conclusion.
5. Pick effort separately and state the concrete reason before dispatch.

Typical starting routes are Luna `high` for the resident Worker, Astra `high`
for resident Reviewer 1 only when the proof is genuinely difficult, and Luna
at the effort appropriate to the owned question for ephemeral reviewers.
These are starting points, not defaults that outrank the task.

## Routing rules

- Do not optimize Dale Max for cost or token spend. Route for correctness,
  difficulty, and consequence.
- Do not select a weaker route merely to save tokens, and do not select a
  stronger route as ceremony when its capability cannot improve the result.
- Never inherit Astra or extreme effort merely because the coordinator uses it.
- Use Astra `xhigh` only for a named, unusually difficult reasoning bottleneck.
- Use `max` when the owned reasoning bottleneck or proof obligation genuinely
  benefits from it; no prior cheaper failure is required. Route `ultra` only to
  Astra and only when the live schema confirms that exact combination.
- Reviewer 2..N each receive the model and effort best matched to their owned
  question. Verification alone does not imply Astra.
- A later follow-up may change route if the workload materially changed. Never
  claim to reroute a turn already running.
