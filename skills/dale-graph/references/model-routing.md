# Luna and Astra node routing

Dale Graph chooses a model for the work owned by each node, not one model for
the whole graph. Its automatic catalog contains only Luna and Astra.

## Runtime catalog

Immediately before creating tasks, inspect the current `create_thread` schema.
It is authoritative for availability and supported reasoning efforts on the
calling host; the destination host validates the final combination.

Route only among these Luna and Astra roles when they are currently available:

| Route | Model ID | Use it for | Normal thinking range |
|---|---|---|---|
| Luna standard | `gpt-5.6-luna` | Fast, bounded exploration; repository inventory; mechanical extraction; narrow checks; routine independent work | `high` |
| Luna deep | `gpt-5.6-luna` | Everyday implementation; debugging; integration; normal code review; work that needs balanced reasoning | `high` or `max` |
| Astra | `gpt-6-astra` | Genuinely ambiguous architecture; difficult cross-layer synthesis; high-stakes proof where stronger reasoning can change the conclusion | `high` |

Dale routes Luna only at `high` or `max`. Validate the chosen effort against
the live schema. Astra efforts are also validated against the live schema. If no allowed
Luna or Astra route is available, omit overrides and report the fallback instead of
inventing a model id.

## Selection procedure

For every ready node:

1. Apply any explicit Luna or Astra model or effort constraint from the user's current
   request. If it is unavailable in the live schema, report the conflict rather
   than silently replacing it.
2. Classify the node by its owned deliverable, evidence risk, ambiguity,
   coupling, and cost of a wrong result.
3. Start with Luna at `high`; raise the effort to `max`
   only when the node needs substantial implementation or reasoning, and
   promote to Astra only when stronger capability would materially reduce
   confidence or create likely rework.
4. Pick effort separately: `high` for mechanical work and ordinary
   reasoning, and `max` for complex work or independent proof.
5. Record a one-line reason that names the workload property which justified the
   route. Do not use generic claims such as "best model".

Use Astra `xhigh` only when the node has an unusually difficult, consequential
reasoning bottleneck that can be stated concretely. Use `max` only after a
cheaper route failed or when a uniquely high-stakes proof obligation clearly
justifies it. Route `ultra` only to Astra and only when the live schema confirms
that exact combination. Never use higher effort to compensate for a vague node
contract.

## Cost and concurrency guardrails

- Parallel discovery nodes default to Luna `high`, not Astra.
- Ordinary implementation defaults to Luna `high`; use `max` only for a stated
  reason.
- Never assign Astra `xhigh`, `max`, or `ultra` merely because the coordinator is
  running at that setting.
- Run at most one Astra node above `high` at a time unless distinct critical paths
  each have a concrete justification.
- Verification does not automatically require Astra. Match the model to the proof
  difficulty and consequence of a false result.
- A clear contract plus a cheaper model is preferable to an underspecified
  contract plus more reasoning.

## Runtime changes

Model choice is part of the mutable graph. New evidence may change the route for
an undispatched node or a later follow-up. Use `send_message_to_thread` with a
new supported `model` and `thinking` only when the work changed materially and
record why. Do not claim to change an already running turn.
