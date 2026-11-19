# Dale team

The team is the user's saved answer to "which model runs which role". It is one
global file, the same in every harness, read by Dale Max and Dale Cross and
edited by Dale Team:

```text
~/.agents/dale/team.json
```

There is no per-project team. A team supplied by the harness itself does not
replace this file.

## Format

```json
{
  "version": 1,
  "roles": [
    {
      "role": "architect",
      "description": "Plans the approach, decomposes work, and makes design decisions.",
      "targets": [
        {
          "provider": "<provider or provider instance>",
          "model": "<model id>",
          "options": { "<option id>": "<value>" }
        }
      ]
    }
  ]
}
```

- `role`: a unique name. The usual roles are `architect` (plans and decides),
  `researcher` (gathers evidence, read-only), `implementer` (writes the
  change), and `reviewer` (independently inspects the result, read-only). They
  are a starting point, not a required set; the user may name others.
- `description`: one sentence saying what the role does. Work units are matched
  to roles by it.
- `targets`: one to three models. The first is the role's model; more than one
  means the role runs on each for independent perspectives.
- `model`: the full model ID, never a harness alias such as `opus`, so the
  file reads the same everywhere.
- `provider`: who runs the model; see the table below. Needed because one
  model ID can be offered by more than one provider.
- `options`: only settings the user picked, stored under the names the saving
  harness uses.

## One file, three harnesses

| Harness | Can run | `provider` to store |
|---|---|---|
| T3 Code | every provider instance that can run a child task | its `providerInstanceId` |
| Claude Code | Claude models, through `Agent` | `claudeAgent` |
| Codex | GPT models, through `spawn_agent` | `codex` |

Claude Code and Codex use T3's default instance names so a team saved in one
harness reads in another. A target whose provider the current harness cannot
reach is unavailable here, even when the model ID looks familiar; it stays in
the file for the harness that can run it.

The same provider's setting may carry different names per transport, such as
`reasoningEffort` in T3 and `reasoning_effort` in Codex. Map it only when it is
plainly the same setting and the value is supported; otherwise ask.

## Read the team

Read the file once at the start of a run, before choosing routes. A missing
file means there is no team. If the file is malformed or has an unknown
`version`, say so, do not repair or guess, and continue as if there were no team.

Choose each route in this order:

1. A model the user names for this task.
2. The team role that fits the work unit.
3. The skill's own fallback for a role the team does not cover.

Check every target against the live dispatch schema: exact model, provider, and
options. Never translate one provider's option name or value into another
provider's. When a role stores no effort, the skill chooses it.

A target that cannot run here is not replaced by a similar one. Ask the user
for a replacement for this run, or for leave to continue without it. A
replacement for one run does not change the file unless the user asks.

## Several models for one role

Write the brief once and send the identical brief to every target, each as its
own child, launched together. Wait for all of them. Report each target's
findings, where they agree, where they disagree, and any failed or unavailable
target as missing coverage. Do not substitute a model or retry unasked.

Read-only roles share the checkout. A writing role with several targets needs
one worktree per target, all from the same commit; compare the results and
integrate one coherent change before review.

## Write the team

Write only when the user asks, or accepts an offer to save. Take provider and
model IDs from the live catalog or the user's exact words; never invent one.
Show the complete team first, then write the whole file, creating
`~/.agents/dale/` when needed and keeping roles the user did not change. Store
no secrets. Delete the file only on an explicit request.
