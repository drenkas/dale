# Linked code walkthrough

Make the code itself the main reading surface. The reader should be able to
follow a concrete scenario through real snippets, understand each change, and
choose which connected fragment to inspect next. A short overview can orient
the reader, but should not push the first code fragment behind a long essay.

## Trace before drawing

Establish the requested scope and source versions: repository, base and target
revision, or explicitly named staged/unstaged working-tree changes. Use the
conversation to resolve the comparison; if it is unspecified, inspect the
available diff and state the chosen scope. Keep staged and unstaged evidence
distinct, and include relevant untracked files only when they belong to the
task. Do not attribute every working-tree change to the current agent.

Read the diff and surrounding source, then follow the actual entrypoint,
callers, callees, contracts, state writes, and consumers needed to explain the
behavior. Order the walkthrough by execution or data flow, rather than diff
file order. Include unchanged connecting code as **context**, so the chain does
not jump over the mechanism that carries the change.

Cover every meaningful changed fragment in scope. Several hunks may share a
node when they implement one responsibility, but retain their individual source
locations and explanations. List omitted mechanical changes or an explicitly
bounded subset so a short tour does not imply complete diff coverage.

Use only inspected source for snippets and connections. With no base version,
show the available code and mark the missing comparison; do not reconstruct a
fictional "before". Proposed code must be labeled proposed. Explanations of
unchanged code describe its role, without claiming it was changed. Source
tracing establishes a code path, not proof that it ran or was deployed.

## Build each code node

Give every fragment a stable anchor shared by the graph and the reading order.
Keep this information together beside its code:

- **Location:** repository/file, symbol or section, source version, and real
  line numbers. Distinguish base-side and target-side lines for diffs. Link to
  verified source locations when the viewing environment supports them; keep
  the location readable when a source link cannot open.
- **Code:** a focused, verbatim excerpt with enough surrounding context to
  understand the condition, call, or assignment. For changes, show a compact
  diff or before/after pair with textual addition/removal markers as well as
  color. Identify omissions without silently joining nonadjacent source lines.
- **Why:** the specific problem or requirement this change addresses and the
  resulting behavior. Explain the effect, not just "added a check" or a
  paraphrase of the syntax; label inferred intent when it is not documented.
- **Handoff:** what input reaches this fragment, what value/event/state leaves
  it, and which connected fragment consumes it, including relevant branch
  conditions. Terminal paths should say where they stop.

Keep explanation outside the verbatim excerpt. Escape source text when
embedding it in HTML or inline script data so code, comments, and strings
cannot become executable page markup. Syntax highlighting is optional and
must preserve the displayed source without requiring a remote library.

## Connect the fragments

Use a directed graph or connected sequence of code cards, with a clearly
selected fragment and highlighted incoming/outgoing connections. Label edges
with their actual relationship, such as `calls validate(input)`, `emits saved`,
`writes status`, or `renders result`. Show the value or contract crossing a
boundary when that explains how the change propagates.

Distinguish calls, data dependencies, async messages, and returns when relevant.
A reading-order link is not an execution edge. Do not draw an invented direct
call across an HTTP endpoint, queue, subscription, or persistence boundary;
show the producer/consumer contract and mark any unverified gap.

Preserve branches, guards, early returns, and fan-out rather than pretending
every fragment always runs in one line. Label branch conditions and make the
alternative targets selectable. For cycles or shared consumers, link back to
the existing node. Keep test/assertion nodes visibly separate from runtime
flow, with "verifies" links to the behavior they cover.

Group related fragments by behavior, layer, or module when it improves scanning.
Give groups a purpose label and keep cross-group connections visible. Collapsed
groups show a summary and fragment count; selecting a hidden fragment expands
its group. Do not require grouping for a short, simple path.

## Make it a walkthrough

Provide both a guided reading order and direct graph navigation:

- Open with the scenario, compact map, and first actual code fragment visible
  or reachable through an obvious first action.
- Provide Previous/Next controls and a position indicator. Name the next
  fragment and label these controls as tour order when a branch or return means
  they are not the next runtime step. Disable them at the tour boundaries.
- Selecting a graph node reveals or scrolls to its code and rationale; selecting
  a code card highlights the same graph node. Keep selection, progress, and
  expanded groups synchronized, including after a direct jump.
- Give outgoing connections explicit targets so the reader can follow a branch
  or jump across groups. Use stable fragment anchors for direct navigation.
- Use native links/buttons or equivalent keyboard semantics with visible focus
  and an accessible current-step indication. On narrow screens, stack map and
  code or use a compact connected list; keep code readable at normal size.
  Long code lines may scroll inside their panel, never widen the whole page.

Use the existing `04-code-understanding.html` pattern for source walkthroughs,
`03-code-review-pr.html` for change excerpts, and `13-flowchart-diagram.html` for
branch navigation as needed. Keep the catalog's limit of one primary and at
most two supporting examples. The requested flow determines the composition.

## Check the result

Alongside the shared browser checks in `SKILL.md`, verify:

- Every displayed excerpt matches its named source version and line numbers;
  changed and context nodes agree with the diff. Redactions and omissions are
  explicit, and descriptions are tied to the shown lines.
- Every meaningful in-scope hunk is represented or its omission is disclosed.
  Each graph edge has an inspected call site, contract, or consumer as evidence;
  uncertain connections are visibly labeled.
- Following the tour from start to finish explains the scenario's outcome.
  Exercise Previous/Next, a direct graph jump, an alternative branch when
  present, and collapse/reopen behavior when groups are used. Confirm graph,
  code, current step, and focus remain usable and consistent.
- Fragment anchors resolve, including targets inside collapsed groups. Verify
  source links when supported, keyboard operation, and narrow-screen code
  scrolling without page overflow. Code containing HTML-like text stays inert
  and is displayed faithfully.

Report the comparison scope and any missing source or runtime evidence with the
artifact. Browser interaction checks validate the visualization; they do not
prove the behavior of the software it depicts.
