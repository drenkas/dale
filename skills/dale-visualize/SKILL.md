---
name: dale-visualize
description: Create a self-contained HTML visualization when spatial layout or interaction materially clarifies technical material or decisions. Skip production apps and prose-sufficient tasks.
---

# Dale Visualize

Create a tailored HTML artifact that clarifies a relationship or decision.
Define its question, audience, intended understanding, evidence, uncertainties,
and spatial relationship (comparison, hierarchy, sequence, dependency, time,
state, or manipulation). Add interaction only when it improves comprehension.

Do not invent facts, metrics, events, citations, behavior, or evidence. Label
inferences and unknowns; exclude secrets, credentials, private environment values,
and unrelated user data. Prefer
prose, a table, or a repository-native component when clearer, unless the user
explicitly requires HTML.

## Choose the form

- Code changes or paths: default to a linked code walkthrough. Read
  [code-walkthrough.md](references/code-walkthrough.md) before tracing source:
  real snippets, rationale, and connected execution/data flow. Preserve an
  explicitly requested high-level overview.
- Static explainer: scanning and understanding.
- Inline SVG: meaningful geometry in flows, architecture, timelines or graphs.
- Cards or matrices: alternatives and variants.
- Controls: inputs that teach or support a decision.
- Small editor: reorder, tune, toggle, or export user-controlled output.
- Slides: when pacing and sequential presentation matter.

Read [example-catalog.md](references/example-catalog.md) only if examples help;
inspect one primary and at most two supporting HTML examples. Borrow techniques,
not fictional Acme content, arbitrary palettes, or entire compositions. Use the
user's material and the owning project's visual language. Do not add ornamental
tabs, charts, animation, drag-and-drop, or filters.

## Build

Unless another target is requested, write one self-contained `.html` file with
inline CSS/SVG and minimal vanilla JavaScript, no build step or production
dependency. Default to no remote fonts/scripts, analytics, network, storage,
or trackers.

Use the requested path; otherwise `visualizations/<slug>.html` for durable
repository artifacts or `.scratch/dale-visualize/<slug>.html` for throwaway work.
Never overwrite an artifact without a replacement request.

Require semantic structure, meaningful title and heading order, desktop/mobile
responsiveness, keyboard operation, visible focus, sufficient contrast, no
clipping/overlap, and reduced-motion support for animation. Insert untrusted
text as text, not executable markup. Keep the page useful without JavaScript
unless interaction is its core deliverable.

For substantial code or styling adapted from bundled examples, preserve:

```html
<!-- Adapted from anthropics/html-effectiveness (MIT), commit 58c305be97f47b26b678f2c07dec01d4242268ec. -->
```

## Verify and deliver

Inspect source for placeholders, fictional sample data, secret-like values,
broken local links, and unintended external URLs. Render with browser control
or Playwright and verify:

1. The question is answered above the fold or through an obvious first action.
2. Desktop and narrow-mobile layouts have no overflow, clipping, or unreadable labels.
3. Keyboard navigation and every material interaction work without console/runtime errors.
4. Applicable loading, empty, error, selected, and changed states exist.
5. Facts match evidence; editor export/copy returns the exact user-controlled result.
6. Code walkthroughs pass the source/navigation checks in their reference;
   a prose module diagram alone is insufficient.

If `file://` is blocked, use an isolated local server; do not weaken browser
security or interfere with existing processes. Without browser rendering,
report static validation as partial.

Deliver the tailored artifact, exact path, selected examples and borrowed
patterns, checks run, and evidence/browser-coverage gaps. Do not deliver the
bundled examples themselves.
