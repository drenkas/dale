# GPT-5.6 node routing

Dale Graph chooses a model for the work owned by each node, not one model for
the whole graph. Its automatic catalog contains only the GPT-5.6 generation.

## Runtime catalog

Immediately before creating tasks, inspect the current
`send_message_to_thread` schema for forked nodes and the `create_thread` schema
for fresh fallbacks. They are authoritative for availability and supported
reasoning efforts on the calling host; the destination host validates the final
combination. `fork_thread` itself does not select a model, so apply the route on
the first follow-up sent to the fork.

Route only among these GPT-5.6 roles when they are currently available:

| Model | Use it for | Normal thinking range |
|---|---|---|
| `gpt-5.6-luna` | Fast, bounded exploration; repository inventory; mechanical extraction; narrow checks; routine independent work | `low` to `medium` |
| `gpt-5.6-terra` | Everyday implementation; debugging; integration; normal code review; work that needs balanced reasoning | `medium` to `high` |
| `gpt-5.6-sol` | Genuinely ambiguous architecture; difficult cross-layer synthesis; high-stakes proof where stronger reasoning can change the conclusion | `high` |

At the time this skill was authored, all three accept `low`, `medium`, `high`,
`xhigh`, `max`, and `ultra`. Do not assume that snapshot outranks the live tool
schema. If no allowed GPT-5.6 route is available, omit overrides and report the
fallback instead of inventing a model id.

## Selection procedure

For every ready node:

1. Apply any explicit GPT-5.6 model or effort constraint from the user's current
   request. If it is unavailable in the live schema, report the conflict rather
   than silently replacing it.
2. Classify the node by its owned deliverable, evidence risk, ambiguity,
   coupling, and cost of a wrong result.
3. Start with Luna, promote to Terra only when the node needs substantial
   implementation or reasoning, and promote to Sol only when Terra would
   materially reduce confidence or create likely rework.
4. Pick effort separately: `low` for mechanical work, `medium` for ordinary
   reasoning, and `high` for complex work or independent proof.
5. Record a one-line reason that names the workload property which justified the
   route. Do not use generic claims such as "best model".

Use `xhigh` only when the node has an unusually difficult, consequential
reasoning bottleneck that can be stated concretely. Use `max` or `ultra` only
after a cheaper route failed or when a uniquely high-stakes proof obligation
clearly justifies it. Never use higher effort to compensate for a vague node
contract.

## Cost and concurrency guardrails

- Parallel discovery nodes default to Luna `low` or `medium`, not Sol.
- Ordinary implementation defaults to Terra `medium`; increase capability only
  for a stated reason.
- Never assign Sol `xhigh`, `max`, or `ultra` merely because the coordinator is
  running at that setting.
- Run at most one Sol node above `high` at a time unless distinct critical paths
  each have a concrete justification.
- Verification does not automatically require Sol. Match the model to the proof
  difficulty and consequence of a false result.
- A clear contract plus a cheaper model is preferable to an underspecified
  contract plus more reasoning.

## Runtime changes

Model choice is part of the mutable graph. New evidence may change the route for
an undispatched node or a later follow-up. Use `send_message_to_thread` with a
new supported `model` and `thinking` only when the work changed materially and
record why. Do not claim to change an already running turn.
