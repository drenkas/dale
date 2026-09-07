# Luna and Astra routing for brainstorm evidence

Dale Brainstorm stays one-on-one. This router applies only after a specific,
separable evidence task becomes necessary and before the user approves that
single visible task. Its automatic catalog contains only Luna and Astra models.

Immediately before proposing the task, inspect the live `create_thread` schema.
It is authoritative for available models and supported efforts.

| Route | Model ID | Evidence-task use | Normal thinking range |
|---|---|---|---|
| Luna standard | `gpt-5.6-luna` | Bounded lookup, repository sweep, extraction, or narrow measurement | `low` to `medium` |
| Luna deep | `gpt-5.6-luna` | Coupled investigation, technical comparison, or evidence that needs substantial interpretation | `xhigh` to `max` |
| Astra | `gpt-6-astra` | Rare, consequential ambiguity where deep cross-domain reasoning can materially change the evidence conclusion | `high` |

At authoring time, Luna accepts `low`, `medium`, `high`, `xhigh`, and `max`;
Astra accepts the same efforts plus `ultra`. The live schema outranks this snapshot. If no allowed
Luna or Astra route is available, omit overrides and report the fallback rather than
inventing an id.

Choose the cheapest sufficient route. Start with Luna, raise it to `xhigh` or
`max` for real coupling or interpretation, and use Astra only for a concrete
high-consequence reason. Use `low` for mechanical work, `medium` for normal
investigation, and `xhigh` or `max` for complex evidence analysis. Never
inherit Astra or elevated effort from the coordinator by default.

Use `xhigh` only for a named, unusually difficult reasoning bottleneck. Use
`max` only after a cheaper route failed or a uniquely high-stakes proof
obligation clearly justifies it. Route `ultra` only to Astra and only when the
live schema confirms that exact combination. Present the chosen model, effort,
and reason with the evidence-task proposal so the user can override or decline
it.
