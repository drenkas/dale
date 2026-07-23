# Runtime graph synthesis

Derive a graph from required work, not from remembered graph shapes.

## Synthesis procedure

### 1. Derive proof obligations

Convert the request into observable acceptance criteria. For each criterion,
identify:

- the artifact that could satisfy it;
- the evidence that could prove it;
- the source or runtime that owns the truth;
- the authority needed to read or mutate it;
- the failure that would disprove completion.

Unknown proof obligations become explicit discovery work. Do not hide them in a
generic research role.

### 2. Generate work units

Create a candidate work unit for each distinct artifact or evidence boundary.
Split a unit only when its result can be produced and evaluated without another
unit's unfinished context. Merge units when separating them would require
constant cross-reading or duplicate the same decision.

Generate the role name from the owned outcome:

```text
<system or artifact> + <action or lens>
```

Examples are deliberately omitted. The task vocabulary must supply the role.

### 3. Generate edges

Every edge must carry a reason:

| Edge reason | Meaning |
|---|---|
| artifact | The destination consumes a concrete output from the source |
| decision | The destination cannot proceed until the source resolves a choice |
| conflict | The nodes share a mutation target and must not run together |
| revision | Failed proof returns an artifact to its owning work unit |

Do not add order merely because a sequence feels tidy. Nodes without a real
dependency belong in the same ready frontier.

### 4. Generate proof boundaries

For each artifact, decide whether its evidence can be trusted from the producing
node. Create separate proof work when independence changes confidence,
especially for:

- high-impact or irreversible changes;
- authentication, authorization, privacy, or secrets;
- user-visible and contract-level behavior;
- cross-layer consistency;
- unstable external facts;
- claims based on interpretation rather than direct output.

Proof work may occur before, between, or after production nodes. Multiple
artifacts may share a proof node only when the same evidence and failure modes
apply.

### 5. Normalize

Repeat until stable:

1. Remove nodes with no distinct deliverable.
2. Merge nodes with inseparable context.
3. Split nodes with independent outputs or write owners.
4. Remove unjustified edges.
5. Add missing artifact, decision, conflict, and revision edges.
6. Confirm every acceptance criterion has an evidence path.
7. Confirm every mutating node has exclusive ownership while running.
8. Leave downstream nodes provisional when their role or deliverable depends on
   evidence that does not exist yet.
9. If only one coherent unit survives, return a direct-work recommendation
   instead of manufacturing a graph.

## Graph state

Track:

```yaml
objective: verbatim user-visible outcome
criteria:
  - id: criterion-id
    proof: direct evidence required
nodes:
  - id: short-stable-id
    title: Dale Graph · <run> · <generated-role>
    role: generated from owned work
    deliverable: one observable artifact
    depends_on: []
    context_origin: fork | fresh-fallback
    source_thread: calling coordinator or none
    client_thread_id: queued worktree id or none
    read_scope: exact scope
    write_scope: none or exclusive paths
    model: supported GPT-5.6 model selected for this node
    thinking: supported effort selected for this node
    routing_reason: why this is the cheapest sufficient route
    subagent_policy: allowed, adaptive, node-local, one level deep
    subagents: []
    status: provisional | planned | ready | running | passed | revise | rejected | blocked
    evidence: []
edges:
  - from: node-id
    to: node-id
    reason: artifact | decision | conflict | revision
    payload: exact artifact, choice, resource, or failed check
```

## Worker contract

```text
You are node <id> in a visible Dale Graph.

Context origin: fork of the calling task or fresh fallback
Generated role: <role>
Owned deliverable: <deliverable>
Inputs and satisfied dependencies: <artifacts>
Read scope: <scope>
Write scope: <scope or none>
Applicable instructions: <AGENTS.md and user constraints>
Forbidden actions: <list>
Required evidence: <proof obligation>
Stop when: <blocked or disproven condition>

You may spawn node-local subagents when independent work inside this node makes
that useful. Follow the supplied node-subagent policy. Do not create additional
Codex tasks, and require every subagent to create neither agents nor Codex
tasks. Stay inside your ownership boundary. You own orchestration, conflict
resolution, validation, and the single integrated result. Distinguish verified
evidence from inference. Do not claim completion without the required primary
signal.

Return:
STATUS: PASS | REVISE | BLOCKED | REJECT
ARTIFACT: <exact result or path>
EVIDENCE: <paths, commands, outputs, citations>
CHANGES: <files or none>
VALIDATION: <exact checks and exits>
SUBAGENTS: <name, scope, status, contribution, or none>
DISCOVERED WORK: <new necessary work, or none>
INVALIDATED ASSUMPTIONS: <items, or none>
RISKS: <remaining risks>
QUESTIONS FOR USER: <specific required input and why, or none>
```

## Proof contract

```text
You own proof obligation <criterion-id> in a visible Dale Graph.

Criterion: <observable acceptance criterion>
Candidate artifact: <raw artifact or diff>
Required evidence: <direct signal>
Relevant source or runtime: <scope>

You may spawn node-local subagents for independent adversarial lenses inside
this proof scope. Follow the supplied node-subagent policy. Do not create Codex
tasks, and require every subagent to create neither agents nor Codex tasks. Do
not inherit the producer's conclusion. You own the final verdict. Try to
disprove the criterion and inspect coupled layers that could make a local result
misleading.

Return:
VERDICT: PASS | REVISE | REJECT | BLOCKED
EVIDENCE: <direct support or contradiction>
SUBAGENTS: <name, lens, status, contribution, or none>
UNCOVERED FAILURE MODE: <item or none>
MINIMAL NEXT WORK: <new work unit, revision, or none>
QUESTIONS FOR USER: <specific required input and why, or none>
```

## Runtime mutation rules

- Add work when a result exposes a new necessary artifact or proof obligation.
- Prune work when its output can no longer affect any acceptance criterion.
- Split work when a newly discovered boundary allows independent ownership.
- Merge work when new evidence makes separate nodes redundant.
- Reroute dependents to the artifact that now owns the truth.
- Insert conflict resolution when credible evidence disagrees.
- Move proof closer to the risky artifact when late failure would waste work.
- Re-evaluate immediately when any running node finishes or needs attention;
  never wait for a whole concurrency batch solely for symmetry.
- Challenge every graph expansion against the cheaper baseline of direct work.
- Keep node-local decomposition inside its parent node; promote newly discovered
  work to the visible graph only when it crosses the node's deliverable,
  evidence, authority, or mutation boundary.
- Re-route only future turns when a node's actual workload materially differs
  from the workload used for its current model choice.
- Never preserve the original graph for aesthetic consistency.
