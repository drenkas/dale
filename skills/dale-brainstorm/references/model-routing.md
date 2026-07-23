# GPT-5.6 routing for brainstorm evidence

Dale Brainstorm stays one-on-one. This router applies only after a specific,
separable evidence task becomes necessary and before the user approves that
single visible task. Its automatic catalog contains only GPT-5.6 models.

Immediately before proposing the task, inspect the live `create_thread` schema.
It is authoritative for available models and supported efforts.

| Model | Evidence-task use | Normal thinking range |
|---|---|---|
| `gpt-5.6-luna` | Bounded lookup, repository sweep, extraction, or narrow measurement | `low` to `medium` |
| `gpt-5.6-terra` | Coupled investigation, technical comparison, or evidence that needs substantial interpretation | `medium` to `high` |
| `gpt-5.6-sol` | Rare, consequential ambiguity where deep cross-domain reasoning can materially change the evidence conclusion | `high` |

At authoring time, all three accept `low`, `medium`, `high`, `xhigh`, `max`, and
`ultra`; the live schema outranks this snapshot. If no allowed GPT-5.6 route is
available, omit overrides and report the fallback rather than inventing an id.

Choose the cheapest sufficient route. Start with Luna, move to Terra for real
coupling or interpretation, and use Sol only for a concrete high-consequence
reason. Use `low` for mechanical work, `medium` for normal investigation, and
`high` for complex evidence analysis. Never inherit Sol or elevated effort from
the coordinator by default.

Use `xhigh` only for a named, unusually difficult reasoning bottleneck. Use
`max` or `ultra` only after a cheaper route failed or a uniquely high-stakes
proof obligation clearly justifies it. Present the chosen model, effort, and
reason with the evidence-task proposal so the user can override or decline it.
