---
name: dale-team
description: Show, create, or change the saved Dale team, the model each role runs on in Dale Max and Dale Cross. Use when the user asks to set up, view, edit, or clear their team or roles.
---

# Dale Team

Manage the user's saved team: which model runs which role. Dale Max and Dale
Cross read it instead of asking each time. This skill only edits the team; it
dispatches no work and changes no harness setting.

Read [team.md](references/team.md) for the file location, format, and rules.

## Show

Read the file and present each role with its description, models, and any
stored options. Say plainly when there is no team yet. Check the routes against
the models runnable here and mark any that are not; do not remove them, since
they may run in another harness.

## Create or change

1. Find the models that can run a child here from the live tool schema or
   catalog. Never invent or hard-code a model ID. Name families that are
   unavailable instead of guessing at them.
2. Propose roles. Start from the usual four (architect, researcher,
   implementer, reviewer) when the user names none, and keep roles they already
   have. Give a one-sentence description and a recommended model for each: a
   stronger model where judgment decides the result, a cheaper one for reading
   and gathering.
3. Ask the user to choose, in one round, in their language. Show the full
   catalog; never trim it to fit an option limit. A recommendation is not a
   choice, and neither is silence.
4. A role may take up to three models for independent perspectives. Offer this
   only where a second opinion can change the outcome, usually review.
5. Store options only when the user picks them. Keep each provider's own
   option names and values.
6. Show the complete resulting team and write the file once the user confirms.

A request such as "reviewer = this model" changes that role and keeps the rest.
Removing a role or clearing the team follows the same show-then-confirm step.

## In each harness

Read the live tool schema first; it wins over these notes.

- **T3 Code.** Call `orchestrator_capabilities` once. Offer models only from
  providers whose `canRunChildTask` is true, and name a provider with a
  constraint as unavailable. The list is short: use its `search` argument for
  a model the user names that is not shown. Store `providerInstanceId` as
  `provider`, and option ids and values as that model lists them. T3 may keep a
  team of its own; do not read it as this one or change it.
- **Claude Code.** Only Claude models can run, through the `Agent` tool; its
  `model` and `effort` parameters show what is accepted. Offer each as its full
  model ID. Other families need T3 Code: keep such roles in the file, marked as
  not runnable here. `AskUserQuestion` takes at most four options per question,
  so with more models than that, print the numbered catalog and take the answer
  in chat.
- **Codex.** Only GPT models can run, through `spawn_agent`; its `model` and
  `reasoning_effort` parameters show what is accepted. Roles for other families
  stay in the file, marked as not runnable here.

In every harness, write `~/.agents/dale/team.json` with an absolute path. A
permission prompt for a file outside the project is expected.

## After writing

Confirm the path and the saved roles. Note any role that cannot run in the
current harness. Running work still needs its own request.
