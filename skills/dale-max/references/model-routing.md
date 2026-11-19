# Fallback routing for Dale Max

The saved team decides each role's model; see [team.md](team.md). This catalog
covers only roles the team leaves out, or a run with no team. Dale Max routes
each worker and independent reviewer separately. The live dispatch schema is
authoritative for models and supported reasoning efforts.

## Catalog

| Route | Model ID | Best fit | Normal thinking |
|---|---|---|---|
| Terra standard | `gpt-6.1-terra` | Research, mechanical extraction, narrow checks, fresh low-risk reviewer lenses | `high` |
| Terra deep | `gpt-6.1-terra` | Everyday implementation, debugging, integration | `high` or `max` |
| Astra | `gpt-6.2-astra` | Demanding roles: architecture, planning, independent review | `high` or `max` |
| Sol | `gpt-6.2-sol` | Consequential ambiguity, difficult cross-layer synthesis, adversarial proof | `high` |

By role, the fallback is Astra for an architect or reviewer and Terra for a
researcher or implementer.

The exact model IDs and accepted efforts can change. Inspect the live schema
immediately before dispatch; [transport.md](transport.md) describes each
harness. If no allowed route is available, continue useful local work and
report the missing delegation route. Keep the independent-review gate blocked
when it cannot be satisfied; never silently inherit a model outside the team
and this catalog.

Automatic Terra and Astra routes use only `high` or `max`; explicit user
settings take precedence. Use `high` for bounded work and `max` when deeper
reasoning benefits the result.

## Claude catalog

When the coordinator runs on Claude, use this catalog instead of the GPT
catalog above. Every rule below applies with Terra read as Sonnet, and Astra
and Sol read as Opus.

| Route | Model ID | Replaces | Normal thinking |
|---|---|---|---|
| Sonnet standard | `claude-sonnet-5-5` | Terra standard | `medium` |
| Sonnet deep | `claude-sonnet-5-5` | Terra deep | `medium` or `high` |
| Opus | `claude-opus-5-5` | Astra and Sol | `medium` or `high` |

Map effort as GPT `high` to Claude `medium` and GPT `max` to Claude `high`.
Automatic Claude routes use only `medium` or `high`; `high` is the ceiling,
including where the GPT rules allow Sol `xhigh` or `ultra`. Explicit user
settings still take precedence. Never mix catalogs automatically: a Claude
coordinator does not route to Terra, Astra, or Sol unless the user or the team
asks.

## Selection

1. Obey an explicit user-approved model and effort first, then the team role,
   including a supported route outside this catalog. If the live schema cannot
   support it, report the conflict and do not substitute without the user's
   resolution.
2. Use Terra for bounded mechanical work and research whose correctness does
   not depend on deeper synthesis.
3. Use Terra with `high` or `max` reasoning for substantial implementation or
   coupled reasoning.
4. Use Astra for architecture, planning, and the independent review of
   consequential work.
5. Use Sol when stronger reasoning can materially change a consequential
   conclusion.
6. Pick effort separately and state the concrete reason before dispatch. A
   team role that stores an effort keeps it.

These are starting points, not defaults that outrank the task.

## Routing rules

- Do not optimize Dale Max for cost or token spend. Route for correctness,
  difficulty, and consequence.
- Do not select a weaker route merely to save tokens, and do not select a
  stronger route as ceremony when its capability cannot improve the result.
- Never inherit a model or effort merely because the coordinator or a parent
  uses it. A nested child gets the route of its own role.
- Use Sol `xhigh` only for a named, unusually difficult reasoning bottleneck.
- Use `max` when the owned reasoning bottleneck or proof obligation genuinely
  benefits from it; no prior cheaper failure is required. Route `ultra` only to
  Sol and only when the live schema confirms that exact combination.
- Additional reviewers receive the model and effort best matched to their owned
  question. Verification alone does not imply Sol.
- Change undispatched routes only when the workload materially changes. A
  message or follow-up to a running child does not change its model; never
  claim to reroute a running turn.
