# Terra and Sol routing for Dale Max

Dale Max routes each worker and independent reviewer separately. The
live task, messaging, and subagent tool schemas are authoritative. Its automatic
catalog contains available Terra and Sol models and supported reasoning efforts.

## Catalog

| Route | Model ID | Best fit | Normal thinking |
|---|---|---|---|
| Terra standard | `gpt-6.1-terra` | Mechanical extraction, narrow checks, fresh low-risk reviewer lenses | `high` |
| Terra deep | `gpt-6.1-terra` | Everyday implementation, debugging, integration, ordinary review | `high` or `max` |
| Sol | `gpt-6.1-sol` | Consequential ambiguity, difficult cross-layer synthesis, adversarial proof | `high` |

The exact model IDs and accepted efforts can change. Inspect the live schema
immediately before dispatch. For `spawn_agent`, use `model` and
`reasoning_effort` with fresh context (`fork_turns: "none"`) and explicit required
inputs. Do not supply overrides on a full-history fork when the schema requires
inheritance. A role may fix model/effort: select a compatible role or the default
role, never claim that an override changes a fixed setting. Task-only `thinking`
applies to explicitly requested separate tasks and their follow-ups; obey their
live routing and authorization rules. If no allowed Terra or Sol route is
available, continue useful local work and report the missing
delegation route. Keep the independent-review gate blocked when it cannot be
satisfied; never silently inherit a model outside this automatic catalog.

Automatic Terra routes use only `high` or `max`; explicit user settings take
precedence. Use `high` for bounded work and `max` when deeper reasoning benefits
the result.

## Selection

1. Obey explicit user-approved model and effort first, including a supported
   route outside this automatic catalog. If the live schema cannot support it,
   report the conflict and do not substitute without the user's resolution.
2. Use Terra for bounded mechanical work whose correctness does not depend on
   deeper synthesis.
3. Use Terra with `high` or `max` reasoning for substantial implementation or
   coupled reasoning.
4. Use Sol when stronger reasoning can materially change a consequential
   conclusion.
5. Pick effort separately and state the concrete reason before dispatch.

Typical starting routes are Terra `high` for implementation, Sol `high`
for independent review only when the proof is genuinely difficult, and Terra
at the effort appropriate to the owned question for ephemeral reviewers.
These are starting points, not defaults that outrank the task.

## Routing rules

- Do not optimize Dale Max for cost or token spend. Route for correctness,
  difficulty, and consequence.
- Do not select a weaker route merely to save tokens, and do not select a
  stronger route as ceremony when its capability cannot improve the result.
- Never inherit Sol or extreme effort merely because the coordinator uses it.
- Use Sol `xhigh` only for a named, unusually difficult reasoning bottleneck.
- Use `max` when the owned reasoning bottleneck or proof obligation genuinely
  benefits from it; no prior cheaper failure is required. Route `ultra` only to
  Sol and only when the live schema confirms that exact combination.
- Additional reviewers receive the model and effort best matched to their owned
  question. Verification alone does not imply Sol.
- Change undispatched routes only when the workload materially changes. A
  subagent message or follow-up does not establish a model change. Separate-task
  follow-ups may change route only when the live tool supports it; never claim
  to reroute a running turn.
