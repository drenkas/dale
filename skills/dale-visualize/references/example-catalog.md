# HTML example catalog

Use this catalog to choose a small number of source patterns. All examples live
under `../assets/examples/` relative to this reference's parent skill.

Source: [anthropics/html-effectiveness](https://github.com/anthropics/html-effectiveness)
at commit `58c305be97f47b26b678f2c07dec01d4242268ec`. The bundled files are licensed
under MIT; preserve `assets/examples/LICENSE` and the attribution comment
required by `SKILL.md` when substantially adapting code.

## Routing table

| Example | Use when | Reusable pattern |
|---|---|---|
| `01-exploration-code-approaches.html` | Comparing implementation strategies | Parallel alternatives, inline code, explicit tradeoffs, recommendation |
| `02-exploration-visual-designs.html` | Exploring visual directions | Live variants, theme controls, comparable canvases |
| `03-code-review-pr.html` | Reviewing a change | Annotated diff, severity markers, jump navigation, checklist |
| `04-code-understanding.html` | Explaining an unfamiliar code path | Module map, highlighted hot path, entrypoints, linked details |
| `05-design-system.html` | Documenting an existing design system | Token swatches, type and spacing scales, component states |
| `06-component-variants.html` | Reviewing a component API | Variant matrix, adjustable controls, state and intent comparison |
| `07-prototype-animation.html` | Tuning motion | Isolated micro-interaction, easing and duration controls, replay |
| `08-prototype-interaction.html` | Testing a short product flow | Clickable states, direct manipulation, minimal functional prototype |
| `09-slide-deck.html` | Presenting a paced narrative | Full-screen sections, keyboard navigation, progress indication |
| `10-svg-illustrations.html` | Producing reusable technical figures | Inline SVG figure sheet and per-figure export |
| `11-status-report.html` | Communicating current delivery state | Scan-first summary, shipped/slipped sections, compact chart |
| `12-incident-report.html` | Explaining an operational incident | Impact summary, minute timeline, logs, contributing factors, follow-ups |
| `13-flowchart-diagram.html` | Showing a process with branches and failures | Annotated SVG flowchart, clickable steps, decision paths |
| `14-research-feature-explainer.html` | Explaining how a repository feature works | TL;DR, request path, collapsible steps, tabbed source, FAQ |
| `15-research-concept-explainer.html` | Teaching an abstract concept | Interactive model, adjustable inputs, comparison table, glossary |
| `16-implementation-plan.html` | Handing off a substantial implementation | Milestones, data flow, mockups, risky code, test and risk tables |
| `17-pr-writeup.html` | Preparing reviewers for a pull request | Motivation, before/after, file tour, focus areas, rollout |
| `18-editor-triage-board.html` | Prioritizing or ordering a set | Drag board, filters, explicit buckets, Markdown export |
| `19-editor-feature-flags.html` | Editing dependent configuration | Grouped toggles, prerequisite warnings, changed-state diff export |
| `20-editor-prompt-tuner.html` | Tuning a reusable text template | Editable slots, live sample rendering, copy and reset |
| `index.html` | Building a browsable gallery | Categorized catalog, thumbnails, dense but scan-friendly navigation |

## Useful combinations

- Architecture or runtime explanation: `04` as the page skeleton plus `13` for
  branching flow details.
- Technical decision: `01` for alternatives plus a focused diagram technique
  from `10` or `13`.
- Repository feature explainer: `14` plus the module-map vocabulary from `04`.
- Delivery plan: `16` plus a narrow interactive prototype from `08` only when
  behavior must be felt.
- Operational review: `12` for chronology plus the compact status summary from
  `11`.
- Decision interface: choose exactly one of `18`, `19`, or `20` based on whether
  the user is ordering items, toggling dependent state, or editing a template.

Default to one example. A combination must have a concrete reason; visual
variety alone is not sufficient.
