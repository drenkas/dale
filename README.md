# Dale

![Codex Dale — graph orchestration for your GPT fam models](assets/dale-banner.png)

A compact development toolbox for Codex: think one-on-one, build a trusted
repository index, synthesize a task graph that adapts while it runs, put a
consequential task through a resident review loop, or turn complex material
into an effective standalone HTML visualization.

## Skills

### `$dale-brainstorm`

![Dale Brainstorm architecture](assets/dale-brainstorm.png)

A focused conversation between you and the current Codex agent. It keeps the
work in exploration mode instead of rushing into a plan or implementation.

- stays one-on-one by default;
- separates evidence, assumptions, preferences, and unknowns;
- challenges the strongest option instead of agreeing automatically;
- proposes at most one visible evidence task, only when a separable research
  question emerges and you explicitly approve it;
- routes an approved evidence task to the cheapest sufficient GPT-5.6 model.

### `$dale-index`

![Dale Index architecture](assets/dale-index.png)

Turns the current repository into six evidence-backed project primitives:

| Primitive | Source of truth for |
|---|---|
| `REQ.md` | Product intent, scope, requirements, and acceptance criteria |
| `CONTEXT.md` | Architecture, repository map, contracts, and constraints |
| `STATE.md` | Current objective, active work, blockers, risks, and next actions |
| `TDD.md` | Test strategy, commands, quality gates, and missing coverage |
| `DESIGN.md` | Visual language, primitives, states, and accessibility |
| `DECISIONS.md` | Durable decisions, alternatives, evidence, and consequences |

Dale launches three parallel, read-only discovery tasks, then sends their raw
reports through an adversarial evidence verifier. The calling task is the sole
integrator and writes only claims that survive verification.

Each visible task is routed independently across GPT-5.6 Luna, Terra, and Sol.
Repository inventory does not default to frontier-level reasoning.

### `$dale-graph`

![Dale Graph architecture](assets/dale-graph.png)

Builds a unique execution graph directly from the request. It does not choose a
fixed diamond, pipeline, debate, or other remembered template.

1. Derives observable outcomes, artifacts, evidence boundaries, and ownership.
2. Creates only the nodes and edges justified by the work.
3. Forks the calling Codex task when available, inheriting its completed
   history and project context.
4. Sends the active request and exact node contract into every fork because an
   unfinished turn is not part of inherited history.
5. Routes every node to the cheapest sufficient GPT-5.6 model and reasoning
   effort.
6. Lets every visible node orchestrate one bounded layer of node-local
   subagents inside its own authority and write scope.
7. Rebuilds the graph when runtime evidence changes dependencies, proof needs,
   or required work.
8. Verifies, rejects, revises, and integrates surviving artifacts into one
   result.

The outer graph remains visible and user-owned. Node-local subagents cannot
create Codex tasks, expand authority, or spawn another agent layer.

### `$dale-max`

Runs a fixed high-assurance control loop for consequential work. Unlike Dale
Graph, which creates its topology on the fly, Dale Max preserves the topology
below while generating reviewer lenses, proof obligations, and model routes
from the actual task.

![Dale Max resident-worker review loop](skills/dale-max/assets/dale-max-graph.png)

1. The coordinator plans the outcome and direct proof.
2. A persistent forked Codex Worker thread produces the artifact, receives every
   revision, and may spawn its own task-local subagents.
3. A persistent forked Codex Reviewer 1 / Lead Reviewer thread creates fresh
   Reviewer 2..N subagents with `spawn_agent` only for material, task-specific
   failure modes.
4. A fresh Plan Reviewer checks the initial plan, then another fresh instance
   checks plan drift against each actual result.
5. The coordinator synthesizes evidence and applies a `PASS`, `REVISE`,
   `REJECT`, or `BLOCKED` gate.
6. Failed criteria loop back to the same Worker; a passed result is compressed
   into the shortest complete handoff.

Green nodes in the diagram are resident forked Codex threads; white nodes are
ephemeral phases or `spawn_agent` subagents. Every green thread receives
explicit authority to spawn one level of task-local subagents and must report
that the capability is available before its artifact can pass. “Turn into
haiku” means concise editorial compression, not
a literal poem unless requested. Dale Max is explicit-only because it runs an
exhaustive review loop with no preset cost, token, reviewer-count, or cycle
budget.

### `$dale-visualize`

![Dale Visualize architecture](assets/dale-visualize.png)

Turns code, architecture, research, plans, comparisons, reports, incidents,
design systems, concepts, and editable decisions into a self-contained HTML
artifact.

- selects one primary pattern and at most two supporting patterns from
  Anthropic's MIT-licensed `html-effectiveness` examples;
- adapts the information shape instead of copying fictional sample content;
- defaults to standalone HTML with inline CSS, SVG, and minimal JavaScript;
- uses interaction only when it improves understanding or enables a decision;
- preserves source evidence and labels unknowns instead of inventing data;
- verifies desktop, mobile, keyboard, interactions, and runtime errors in a real
  browser before delivery.

The bundled gallery includes patterns for comparisons, code review, module
maps, design systems, prototypes, SVG figures, flowcharts, explainers, plans,
reports, incidents, slide decks, and small editing interfaces.

## GPT-5.6 routing

| Model | Default role |
|---|---|
| Luna | Bounded discovery, inventory, extraction, and narrow checks |
| Terra | Everyday implementation, debugging, integration, and normal proof |
| Sol | Difficult cross-layer synthesis and consequential ambiguity |

The live Codex tool schema is authoritative. Dale shows the selected model,
reasoning effort, and concrete routing reason before dispatch. Sol at `xhigh`,
`max`, or `ultra` is never inherited merely because the coordinator uses it.

## Principles

- visible Codex tasks for top-level ownership;
- node-local subagents for bounded inner parallelism;
- runtime graph synthesis instead of predefined orchestration shapes;
- explicit read/write scopes and conflict edges;
- evidence before synthesis;
- verify first, integrate second;
- no Git mutation unless the user requested it.

## Repository layout

```text
.codex-plugin/plugin.json
skills/
  dale-brainstorm/
  dale-index/
  dale-graph/
  dale-max/
  dale-visualize/
assets/
  dale-banner.png
  dale-icon.png
  dale-logo.png
  dale-brainstorm.png
  dale-index.png
  dale-graph.png
  dale-visualize.png
```

Select a skill from Codex or invoke it explicitly with `$dale-brainstorm`,
`$dale-index`, `$dale-graph`, `$dale-max`, or `$dale-visualize`.

## License

Dale is licensed solely under the [Mozilla Public License 2.0](LICENSE).
Bundled third-party examples retain the license notices shipped with their
source distributions.
