---
name: dale-visualize
description: Create self-contained HTML visualizations, including code walkthroughs with actual snippets, change rationale, and linked execution or data flow. Use when the user invokes $dale-visualize, selects Dale Visualize through /skills, or asks to turn code changes, architecture, research, plans, comparisons, reports, incidents, workflows, design systems, concepts, or editable decisions into a browser-readable visual artifact. Do not use for a production application, an image-only deliverable, or prose that gains nothing from spatial layout or interaction.
---

# Dale Visualize

Turn material into a standalone HTML artifact that makes an important
relationship easier to see, inspect, compare, or manipulate. Adapt patterns to
the task; never replace the task with a generic dashboard.

## Frame the visual question

Before writing HTML, identify:

- the one question the artifact should answer;
- the audience and the decision or understanding it should enable;
- the verified facts, code, data, and uncertainties available;
- the relationship best expressed spatially: comparison, hierarchy, sequence,
  dependency, change over time, state, or direct manipulation;
- whether interaction materially improves comprehension.

Do not invent metrics, events, citations, product behavior, or source evidence.
Never embed secrets, credentials, private environment values, or unrelated user
data. Label unknown or inferred material honestly.

If a compact table, short prose, or an existing repository-native component is
clearer than HTML, say so and use the simpler form unless the user explicitly
requires an HTML artifact.

## Select examples deliberately

Read `references/example-catalog.md` before implementation. Select one primary
example whose information shape matches the task, then at most two secondary
examples for a specific interaction or visual device. Inspect only those HTML
files under `assets/examples/`.

Borrow structure, interaction ideas, and implementation techniques—not the
fictional Acme content, arbitrary palette, or entire page composition. Generate
the final hierarchy from the user's material and the owning project's visual
language when one exists.

## Choose the artifact form

- For code changes or code-path explanations, default to a linked code
  walkthrough: actual snippet, why it changed or matters, and where execution
  or data goes next. Read [references/code-walkthrough.md](references/code-walkthrough.md)
  before tracing the source and building the page. Keep an explicitly requested
  high-level overview high-level.
- Use a static explainer when the reader mainly needs to scan and understand.
- Use inline SVG for flows, architecture, timelines, call graphs, and custom
  figures whose geometry carries meaning.
- Use side-by-side cards or matrices for genuine alternatives and variants.
- Use controls only when changing an input teaches something or enables a real
  decision.
- Use a small editor when the output must be reordered, tuned, toggled, or
  exported back into the user's workflow.
- Use a slide deck only when pacing and sequential presentation matter.

Do not add tabs, charts, animation, drag-and-drop, or filters merely to make the
artifact feel interactive.

## Build a standalone page

Unless the user requests another integration target, create one self-contained
`.html` file with inline CSS, inline SVG, and minimal vanilla JavaScript. Require
no build step and no production dependency. Avoid remote fonts, scripts,
analytics, network calls, storage, and trackers by default.

When the user names a destination, write there. For a durable repository
artifact without a named path, use `visualizations/<descriptive-slug>.html`.
For a throwaway exploration, use `.scratch/dale-visualize/<slug>.html`. Never
overwrite an existing artifact unless the user asked for replacement.

Keep the page:

- semantically structured with a meaningful title and heading order;
- responsive at desktop and narrow mobile widths;
- keyboard usable with visible focus states;
- readable with sufficient contrast and no clipped or overlapping content;
- compatible with `prefers-reduced-motion` when animation exists;
- safe with untrusted text inserted via text content rather than executable
  markup;
- useful without JavaScript when interaction is not the core deliverable.

If substantial code or styling is adapted from the bundled examples, preserve
this source comment in the output:

```html
<!-- Adapted from anthropics/html-effectiveness (MIT), commit 58c305be97f47b26b678f2c07dec01d4242268ec. -->
```

## Verify in a real browser

Inspect the source first for placeholders, fictional sample data, secret-like
values, broken local links, and unintended external URLs. Then use available
browser control or Playwright tooling to render the page.

Validate at minimum:

1. the primary visual question is answered above the fold or through an obvious
   first action;
2. desktop and narrow-mobile layouts have no overflow, clipping, or unreadable
   labels;
3. keyboard navigation and every material interaction work;
4. browser console and page runtime show no errors;
5. loading, empty, error, selected, and changed states exist when applicable;
6. displayed facts match the supplied evidence;
7. export or copy actions return the exact user-controlled result when the page
   is an editor.

For a code walkthrough, also run the source and navigation checks in
`references/code-walkthrough.md`; a module diagram with prose alone does not
satisfy that form.

If direct `file://` loading is blocked, use an isolated local server rather than
weakening browser security or interfering with an existing process. If browser
rendering is unavailable, report static validation as partial rather than
claiming visual success.

## Deliver

Return the artifact as the main result. State:

- its exact path;
- the primary and secondary examples selected and what pattern each supplied;
- the visual and interaction checks run;
- any evidence gaps or browser coverage not completed.

Do not ship the bundled examples themselves as the answer. The result must be a
new artifact tailored to the user's question.
