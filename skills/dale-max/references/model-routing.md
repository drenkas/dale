# GPT-5.6 routing for Dale Max

Dale Max routes each resident thread and ephemeral reviewer independently. The
live task, messaging, and subagent tool schemas are authoritative. Route only
among available GPT-5.6 models and supported reasoning efforts.

## Catalog

| Model | Best fit | Normal thinking |
|---|---|---|
| `gpt-5.6-luna` | Mechanical extraction, narrow checks, fresh low-risk reviewer lenses | `low` to `medium` |
| `gpt-5.6-terra` | Everyday implementation, debugging, integration, ordinary review | `medium` to `high` |
| `gpt-5.6-sol` | Consequential ambiguity, difficult cross-layer synthesis, adversarial proof | `high` |

The exact model IDs and accepted efforts can change. Inspect the live schema
immediately before dispatch. `fork_thread` does not select a route; set model
and thinking on the first `send_message_to_thread` follow-up. If no allowed
GPT-5.6 route is available, return `BLOCKED`. Never omit the override when that
could silently inherit a model outside the GPT-5.6 boundary.

## Selection

1. Obey explicit user constraints that fit the live GPT-5.6 catalog.
2. Use Luna for bounded mechanical work whose correctness does not depend on
   deeper synthesis.
3. Use Terra for substantial implementation or coupled reasoning.
4. Use Sol when stronger reasoning can materially change a consequential
   conclusion.
5. Pick effort separately and state the concrete reason before dispatch.

Typical starting routes are Terra `high` for the resident Worker, Sol `high`
for resident Reviewer 1 only when the proof is genuinely difficult, and
Luna or Terra for ephemeral reviewers. These are starting points, not defaults
that outrank the task.

## Routing rules

- Do not optimize Dale Max for cost or token spend. Route for correctness,
  difficulty, and consequence.
- Do not select a weaker route merely to save tokens, and do not select a
  stronger route as ceremony when its capability cannot improve the result.
- Never inherit Sol or extreme effort merely because the coordinator uses it.
- Use `xhigh` only for a named, unusually difficult reasoning bottleneck.
- Use `max` or `ultra` when the owned reasoning bottleneck or proof obligation
  genuinely benefits from it; no prior cheaper failure is required.
- Reviewer 2..N each receive the model and effort best matched to their owned
  question. Verification alone does not imply Sol.
- A later follow-up may change route if the workload materially changed. Never
  claim to reroute a turn already running.
