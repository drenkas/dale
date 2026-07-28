---
name: dale-proof
description: Verify an existing implementation, fix, migration, deployment, runtime behavior, compatibility boundary, or other technical claim by mapping atomic claims to direct, fresh, reproducible primary signals and returning scoped PROVEN, PARTIAL, UNPROVEN, or CONTRADICTED verdicts with a compact proof capsule. Use when the user invokes $dale-proof, selects Dale Proof through /skills, asks to prove, verify, confirm, or validate whether a technical result is actually true, or needs claimed success separated from available evidence. Do not use as a general code-review, implementation, remediation, independent-review, or orchestration workflow.
---

# Dale Proof

Establish what the available evidence actually proves. Work backward from a
technical claim to the shortest decisive observation; never promote source
inventory, green secondary checks, or plausible reasoning into runtime truth.

## Verify without repairing

- Treat the supplied result as an object to verify, not a request to fix it.
  Diagnose or implement only when the user separately asks.
- Default to read-only inspection and non-destructive validation. Do not deploy,
  restart, promote, mutate production data, alter configuration, or perform a
  real external transaction merely to obtain proof.
- Preserve dirty worktrees and live systems. Never mutate Git, expose secrets,
  print raw environment values, or capture sensitive payloads.
- Stay in the current task. Do not create Codex tasks, subagents, or review
  swarms merely to make the verdict look independent.
- Describe same-agent verification honestly. Dale Proof checks evidence but does
  not provide reviewer independence; use a separately authorized review workflow
  when independence itself is an acceptance criterion.
- Ask for new authority only when the decisive signal requires a sensitive,
  destructive, or externally mutating action. Otherwise gather the strongest
  safe evidence available and bound the verdict accordingly.

Read `references/proof-contract.md` completely before gathering evidence.

## Build the proof

### 1. Normalize the claim

Preserve the user's original claim before bounding it. Split it into atomic
material claims without dropping quantifiers, named environments, side effects,
user-visible outcomes, or forbidden outcomes. Map every material phrase from
the original claim to an atomic claim or an explicit uncovered obligation. For
each atomic claim record:

- exact subject and expected behavior;
- target environment, revision, identity, and time window;
- success and failure observations;
- forbidden side effects;
- strongest available primary signal;
- inaccessible surfaces that limit certainty.

Do not silently broaden “the tested case works” into “the feature always works,”
or “a deployment succeeded” into “that revision serves user traffic.” Do not
silently narrow an unbounded or universal claim to the tested slice. If a safe
verification can cover only a narrower scope, keep that evidence scoped and
make the original overall claim `PARTIAL` or `UNPROVEN`.

### 2. Choose the shortest sufficient evidence ladder

Start with the cheapest signal that can falsify the claim, then climb only while
a material obligation remains unresolved:

1. source and configuration;
2. focused static or type checks;
3. targeted behavioral tests or isolated reproduction;
4. producer-consumer or integration observation;
5. actual runtime, deployed target, or user-visible behavior.

Higher cost does not automatically mean stronger proof. Match the signal to the
claim. A targeted contract test may directly prove a library boundary; a green
pipeline cannot prove that production is serving its artifact.

Prefer existing repository commands and the narrowest safe test. Run a wider
suite only when coupling or regression scope justifies it. Treat every non-zero
exit, failed assertion, runtime error, or unavailable required surface as
material evidence, never as noise to hide.

### 3. Capture evidence as evidence

For every material observation retain the target, revision or identity,
environment, timestamp or freshness, exact non-secret command or source,
result, scope, and limitation. Redact credentials and private data before they
enter a command, log, screenshot, task, or final answer.

Distinguish:

- declared behavior in documentation or schemas;
- implemented behavior in code;
- exercised behavior in tests;
- observed behavior in an integration or runtime;
- user-visible outcome at the requested surface.

Repetition does not turn indirect evidence into a primary signal.

### 4. Reconcile contradictions

Do not vote across sources. Compare directness, target fit, revision identity,
freshness, reproducibility, and covered scope. Preserve contradictions that
cannot be resolved safely. A fresher production observation may describe the
live boundary while an authoritative schema still proves a declaration drift;
report both instead of forcing one global truth.

### 5. Assign scoped verdicts

Use only these claim verdicts:

- `PROVEN` — direct evidence satisfies the atomic claim within the stated scope
  and no material contradiction remains;
- `PARTIAL` — some material obligations are directly supported, but the full
  claim exceeds the observed scope;
- `UNPROVEN` — adequate evidence is absent or inaccessible; this does not mean
  the claim is false;
- `CONTRADICTED` — direct evidence falsifies a material part of the claim.

Compose the overall verdict deterministically:

1. return `CONTRADICTED` if any material atomic claim is contradicted;
2. otherwise return `PROVEN` only if every material atomic claim is proven;
3. otherwise return `PARTIAL` if at least one material atomic claim is proven or
   partial;
4. otherwise return `UNPROVEN`.

If the required runtime or user-visible signal is unavailable, never substitute
source inspection or local green tests for it; return `PARTIAL` or `UNPROVEN`
under this composition rule and name the missing observation exactly.

## Return a proof capsule

Keep the result in the conversation unless the user asks for a persisted
artifact. Use this compact contract:

```text
claim: <original claim, preserving material quantifiers>
verified scope: <exact narrower scope, if any>
target: <environment / revision / identity / time>
verdict: PROVEN | PARTIAL | UNPROVEN | CONTRADICTED

atomic claims:
- <claim> — <verdict> — <direct evidence or missing signal>

primary signal: <strongest observation and result>
supporting evidence: <useful but weaker observations>
contradictions: <preserved conflicts or none>
checks not passed: <failed, blocked, skipped, and unavailable checks>
coverage boundary: <what was not exercised>
reproduction: <exact safe command or observation path, when available>
next decisive check: <one minimal check, only when it could change the verdict>
```

Lead with the verdict and primary signal. State exact checks run and every
failed or blocked check. Avoid dumping raw logs when a precise result and source
pointer carry the evidence.

## Refuse false closure

Absence claims require an explicit search universe and detection capability;
otherwise they remain unproven. Scheduled delivery requires evidence of the
send or receive path, not merely a healthy scheduler. Deployment readiness
requires the actual serving identity and requested behavior, not only a passed
pipeline. A test inventory proves that tests exist, not that they passed.

End when each material atomic claim has a verdict, the primary signal is named,
and the remaining uncertainty is explicit. Do not continue into remediation or
another Dale workflow without a new user request.
