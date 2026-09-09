# Luna and Astra routing for brainstorm evidence

Dale Brainstorm stays one-on-one. This router applies only after a specific,
separable evidence task becomes necessary and before the user approves that
single visible task. Its automatic catalog contains only Luna and Astra models.

Immediately before proposing the task, inspect the live `create_thread` schema.
It is authoritative for available models and supported efforts.

| Route | Model ID | Evidence-task use | Normal thinking range |
|---|---|---|---|
| Luna standard | `gpt-5.6-luna` | Bounded lookup, repository sweep, extraction, or narrow measurement | `high` |
| Luna deep | `gpt-5.6-luna` | Coupled investigation, technical comparison, or evidence that needs substantial interpretation | `high` or `max` |
| Astra | `gpt-6-astra` | Rare, consequential ambiguity where deep cross-domain reasoning can materially change the evidence conclusion | `high` |

Dale routes Luna only at `high` or `max`. Validate the chosen effort against
the live schema. Astra efforts are also validated against the live schema. If no allowed
Luna or Astra route is available, omit overrides and report the fallback rather than
inventing an id.

Choose the cheapest sufficient route. Start with Luna, raise it to `max` for real coupling or interpretation, and use Astra only for a concrete
high-consequence reason. Use `high` for mechanical work and normal investigation, and `max` for
complex evidence analysis that benefits from deeper reasoning. Never
inherit Astra or elevated effort from the coordinator by default.

Use Astra `xhigh` only for a named, unusually difficult reasoning bottleneck. Use
`max` only after a cheaper route failed or a uniquely high-stakes proof
obligation clearly justifies it. Route `ultra` only to Astra and only when the
live schema confirms that exact combination. Present the chosen model, effort,
and reason with the evidence-task proposal so the user can override or decline
it.
