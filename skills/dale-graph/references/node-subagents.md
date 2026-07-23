# Node-local subagent orchestration

Every visible Dale Graph node may orchestrate subagents. This is permission, not
a fixed fan-out. The visible task remains the sole owner of its deliverable.

## When to spawn

Spawn only when the node contains work that can progress independently and the
coordination cost is justified, such as separate evidence surfaces, disjoint
implementation scopes, competing approaches, or an independent attempt to
falsify a material claim. Continue directly when the work is tightly coupled,
small, sequential, or requires one shared chain of thought.

Derive subagent roles from the node's actual work. Do not choose from a catalog
of stock roles and do not target a default count. Respect the live collaboration
capacity; do not queue speculative agents merely to fill available slots.

## Authority and depth

- A subagent inherits the parent node's read scope, write scope, forbidden
  actions, repository instructions, and evidence requirements. Narrow its scope
  further in the spawn prompt.
- Subagents may not expand authority, create Codex tasks, or spawn additional
  agents. Keep the orchestration exactly one level below each visible node.
- The visible node may not use subagents to bypass an outer graph dependency,
  conflict edge, approval, or environment boundary.
- Work discovered outside the node contract must be reported to the top-level
  Dale coordinator, which alone decides whether to mutate the visible graph.

## Mutation ownership

For a read-only node, all subagents remain read-only. For a mutating node,
parallel subagents may write only to explicitly disjoint paths inside the
parent's exclusive write scope. Otherwise designate one writer and keep other
subagents read-only, or run conflicting work sequentially. The parent node must
inspect the integrated diff and validate the final state.

## Model routing

Subagents inherit the visible node's model and reasoning by default. A node may
select another GPT-5.6 route only when the live `spawn_agent` schema exposes a
supported combination and the subagent's workload materially differs. Apply the
same cheapest-sufficient rules as `model-routing.md`, record the reason, and
never introduce a non-GPT-5.6 model automatically.

## Spawn contract

Every spawn prompt must include:

```text
Parent Dale node: <node id and role>
Generated subagent role: <role derived from owned subwork>
Owned output: <one observable result>
Inputs: <minimum required context>
Read scope: <narrow scope>
Write scope: <disjoint paths or none>
Applicable instructions: <AGENTS.md and user constraints>
Forbidden actions: <list>
Required evidence: <direct proof>
Stop when: <blocked, disproven, or complete condition>

Do not create subagents or Codex tasks. Stay inside this delegated scope.
Distinguish evidence from inference and return the exact artifact, validation,
risks, and newly discovered work to the parent node.
```

Give subagents the minimum context needed. Do not leak the parent's intended
conclusion to an independent verifier. Run independent ready subagents in
parallel, wait for their outputs, and inspect the raw evidence before accepting
it.

## Parent integration

The visible node must resolve contradictions by directness, freshness, source
quality, and fit rather than majority vote. It merges only accepted outputs,
performs the node-level primary validation, and returns one result. Its final
report lists every spawned subagent with generated role, scope, status,
contribution, rejected output, and remaining uncertainty. Subagent success alone
does not prove the visible node complete.
