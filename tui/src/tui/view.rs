//! Pure rendering of the application state.

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, WidgetRef, Wrap};
use ratatui::Frame;

use super::state::{AgentsChoice, App, Item, ItemStatus, Row, Screen};
use super::update::planned_agents_mode;
use crate::install::{AgentsMdMode, StepState, RESTART_REMINDER};
use crate::payload::short_version;
use crate::plugin;

/// Dale brand green (#51C878).
pub const BRAND: Color = Color::Rgb(0x51, 0xC8, 0x78);

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 16;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    app.too_small = area.width < MIN_WIDTH || area.height < MIN_HEIGHT;
    if app.too_small {
        let advice = if app.screen == Screen::Installing {
            "Resize the window; an install is running (ctrl+c twice to abort)."
        } else {
            "Resize the window or press q to quit."
        };
        let msg = Paragraph::new(format!(
            "Terminal too small.\nNeed at least {MIN_WIDTH}x{MIN_HEIGHT}, \
             got {}x{}.\n{advice}",
            area.width, area.height
        ))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }
    let [content, bar] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    match app.screen {
        Screen::Splash => draw_splash(f, app, content),
        Screen::Picker => draw_picker(f, app, content),
        Screen::Confirm => draw_confirm(f, app, content),
        Screen::Installing => draw_installing(f, app, content),
        Screen::Done => draw_done(f, app, content),
    }
    draw_footer(f, app, bar);
}

/// Persistent bottom bar: brand mark on the left, the current screen's key
/// hints on the right. Hints are truncated first; the name never is.
fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let left = Line::from(vec![
        Span::styled("● Dale", Style::default().fg(BRAND).bold()),
        dim(" — Codex skill installer"),
    ]);
    let name_width = "● Dale — Codex skill installer".chars().count() as u16;
    if area.width <= name_width + 2 {
        f.render_widget(Paragraph::new(left), area);
        return;
    }
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Length(name_width), Constraint::Min(0)]).areas(area);
    f.render_widget(Paragraph::new(left), left_area);
    let hints = truncate_chars(
        footer_hints(app),
        right_area.width.saturating_sub(2) as usize,
    );
    f.render_widget(
        Paragraph::new(dim(hints)).alignment(Alignment::Right),
        right_area,
    );
}

fn footer_hints(app: &App) -> &'static str {
    match app.screen {
        Screen::Splash => {
            if app.scan_done {
                "any key continue · ctrl+c quit"
            } else {
                "scanning… · ctrl+c quit"
            }
        }
        Screen::Picker => {
            if app.filter_active {
                "type to filter · enter keep · esc clear · ↑/↓ move"
            } else {
                "space toggle · a all · ↑/↓ or j/k move · / filter · u update · \
                 enter continue · q quit"
            }
        }
        Screen::Confirm => {
            if app.agents_dilemma() {
                "enter install · r/a/s AGENTS.md action · j/k scroll diff · b/esc back · q quit"
            } else {
                "enter install · b/esc back · q quit"
            }
        }
        Screen::Installing => "installing — ctrl+c twice to abort",
        Screen::Done => "press any key to exit",
    }
}

/// Truncate to `max` characters, ellipsizing when something was cut.
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

/// Figlet-style DALE wordmark for the splash screen (no external dep).
const DALE_ART: [&str; 6] = [
    "██████╗  █████╗ ██╗     ███████╗",
    "██╔══██╗██╔══██╗██║     ██╔════╝",
    "██║  ██║███████║██║     █████╗  ",
    "██║  ██║██╔══██║██║     ██╔══╝  ",
    "██████╔╝██║  ██║███████╗███████╗",
    "╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝",
];

fn draw_splash(f: &mut Frame, app: &mut App, area: Rect) {
    let art_width = DALE_ART
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0) as u16;
    // Degrade to a plain wordmark when the terminal is too tight.
    let use_art = area.width > art_width && area.height >= DALE_ART.len() as u16 + 6;

    let mut lines: Vec<Line> = Vec::new();
    if use_art {
        for row in DALE_ART {
            lines.push(Line::from(Span::styled(
                row,
                Style::default().fg(BRAND).bold(),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "Dale",
            Style::default().fg(BRAND).bold(),
        )));
    }
    lines.push(Line::default());
    lines.push(Line::from("Codex skill installer"));
    lines.push(Line::from(dim(format!(
        "v{} ({})",
        app.payload.version, app.payload.source
    ))));
    lines.push(Line::default());
    lines.push(if app.scan_done {
        Line::from(Span::styled(
            "✓ environment scanned",
            Style::default().fg(BRAND),
        ))
    } else {
        Line::from(vec![
            Span::styled(spinner_frame(app), Style::default().fg(BRAND)),
            Span::raw(" scanning environment…"),
        ])
    });

    let height = (lines.len() as u16).min(area.height);
    let top = area.y + (area.height - height) / 2;
    let centered = Rect {
        x: area.x,
        y: top,
        width: area.width,
        height,
    };
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), centered);
}

fn spinner_frame(app: &App) -> &'static str {
    SPINNER[app.spinner % SPINNER.len()]
}

fn dim(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(Color::DarkGray))
}

fn status_span(item: &Item) -> Span<'static> {
    if item.is_agents {
        return match item.status {
            ItemStatus::Installed { .. } => dim("present"),
            _ => dim("missing"),
        };
    }
    let plugin_tag = |plugin: bool| if plugin { " · plugin" } else { "" };
    match &item.status {
        ItemStatus::New => dim("new"),
        ItemStatus::Installed { version, plugin } => Span::styled(
            format!(
                "installed v{}{}",
                short_version(version),
                plugin_tag(*plugin)
            ),
            Style::default().fg(BRAND),
        ),
        ItemStatus::Update { from, to, plugin } => Span::styled(
            format!(
                "update v{} → v{}{}",
                short_version(from),
                short_version(to),
                plugin_tag(*plugin)
            ),
            Style::default().fg(Color::Yellow),
        ),
    }
}

fn draw_picker(f: &mut Frame, app: &mut App, area: Rect) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .areas(area);

    // Header.
    let plan = app.plan();
    let mut env_line = vec![
        dim("CODEX_HOME "),
        Span::raw(app.home.display().to_string()),
        dim(format!(
            "   installed {}/{}   plan +{} ~{} -{}",
            app.installed_count(),
            app.payload.skills.len(),
            plan.install.len(),
            plan.update.len(),
            plan.remove.len()
        )),
    ];
    if let Some(pi) = plugin::effective(&app.plugin_installs) {
        env_line.push(Span::styled(
            format!(
                "   plugin dale@{} v{}",
                pi.marketplace,
                short_version(&pi.version)
            ),
            Style::default().fg(BRAND),
        ));
    }
    let mut lines = vec![
        Line::from(vec![
            Span::styled("● Dale", Style::default().fg(BRAND).bold()),
            Span::raw("  Codex skill installer  "),
            Span::styled(
                format!("v{}", app.payload.version),
                Style::default().fg(BRAND),
            ),
            dim(format!("  ({})", app.payload.source)),
        ]),
        Line::from(env_line),
    ];
    let mut status_line = if app.fetching {
        Line::from(vec![
            Span::styled(spinner_frame(app), Style::default().fg(BRAND)),
            Span::raw(" checking for updates…"),
        ])
    } else if let Some(err) = &app.remote_error {
        Line::from(vec![
            Span::styled(
                format!("update check failed: {err}"),
                Style::default().fg(Color::Red),
            ),
            dim("  — embedded payload still available"),
        ])
    } else if let Some(remote) = &app.remote_version {
        Line::from(vec![
            Span::styled(
                format!("remote payload v{remote} active"),
                Style::default().fg(Color::Yellow),
            ),
            dim(format!(
                "  (bundled v{})",
                short_version(&app.bundled_version)
            )),
        ])
    } else {
        Line::from(dim("press u to check for updates"))
    };
    if let Some(version) = &app.newer_binary {
        status_line.push_span(Span::styled(
            format!("  ⟳ newer dale binary v{version} available"),
            Style::default().fg(Color::Yellow),
        ));
    }
    lines.push(status_line);
    f.render_widget(Paragraph::new(lines), header);

    // Body: list + description.
    let [list_area, desc_area] =
        Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)]).areas(body);

    let list_items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| match row {
            Row::Header(group) => ListItem::new(Line::from(Span::styled(
                group.to_string(),
                Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
            ))),
            Row::Item(i) => {
                let item = &app.items[*i];
                let checkbox = if item.selected { "[x] " } else { "[ ] " };
                ListItem::new(Line::from(vec![
                    Span::raw(format!("  {checkbox}")),
                    Span::raw(format!("{:<18} ", item.name)),
                    status_span(item),
                ]))
            }
        })
        .collect();
    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(" Skills "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("");
    let mut state = ListState::default().with_selected(Some(app.cursor));
    f.render_stateful_widget(list, list_area, &mut state);

    let (title, description) = match app.current_item() {
        Some(item) if item.is_agents => (" AGENTS.md ".to_string(), agents_description(app)),
        Some(item) => (format!(" {} ", item.name), item.description.clone()),
        None => (" Dale ".to_string(), String::new()),
    };
    let desc = Paragraph::new(description)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(desc, desc_area);

    // Footer line: filter input or flash (key hints live in the bottom bar).
    if app.filter_active || !app.filter.text().is_empty() {
        let [label, input] =
            Layout::horizontal([Constraint::Length(8), Constraint::Min(1)]).areas(footer);
        f.render_widget(
            Paragraph::new(Span::styled("filter ❯", Style::default().fg(BRAND))),
            label,
        );
        (&app.filter).render_ref(input, f.buffer_mut());
        if app.filter_active {
            if let Some((x, y)) = app.filter.cursor_pos(input) {
                f.set_cursor_position((x, y));
            }
        }
    } else if let Some(flash) = &app.flash {
        f.render_widget(
            Paragraph::new(Span::styled(
                flash.clone(),
                Style::default().fg(Color::Yellow),
            )),
            footer,
        );
    }
}

fn agents_description(app: &App) -> String {
    let base = "Global working agreements for every Codex session, maintained by Dale. \
                Toggle to install or update it alongside the skills.";
    match &app.agents_existing {
        Some(_) => format!(
            "{base}\n\nYou already have an AGENTS.md at {}. The confirm step offers \
             backup+replace, a managed append (only the <!-- dale:begin/end --> section \
             is touched), or skip.",
            crate::install::agents_md_path(&app.home).display()
        ),
        None => format!(
            "{base}\n\nNo AGENTS.md found at {} — it would be created.",
            crate::install::agents_md_path(&app.home).display()
        ),
    }
}

fn draw_confirm(f: &mut Frame, app: &mut App, area: Rect) {
    let dilemma = app.agents_dilemma();
    let plan = app.plan();
    let mut actions: Vec<Line> = Vec::new();
    let header = |actions: &mut Vec<Line>, title: &str| {
        actions.push(Line::from(Span::styled(
            format!("  {title}"),
            Style::default().fg(BRAND).bold(),
        )));
    };
    if !plan.install.is_empty() {
        header(&mut actions, "Install");
        for name in &plan.install {
            actions.push(Line::from(vec![
                Span::styled("    • ", Style::default().fg(BRAND)),
                Span::raw(format!("{name} v{}", short_version(&app.payload.version))),
                dim(format!("  [{} payload]", app.payload.source)),
            ]));
        }
    }
    if !plan.update.is_empty() {
        header(&mut actions, "Update");
        for name in &plan.update {
            let from = match app
                .items
                .iter()
                .find(|i| &i.name == name)
                .map(|i| &i.status)
            {
                Some(ItemStatus::Update { from, .. }) => short_version(from).to_string(),
                _ => "?".to_string(),
            };
            actions.push(Line::from(vec![
                Span::styled("    • ", Style::default().fg(Color::Yellow)),
                Span::raw(format!(
                    "{name} v{from} → v{}",
                    short_version(&app.payload.version)
                )),
                dim(format!("  [{} payload]", app.payload.source)),
            ]));
        }
    }
    if !plan.remove.is_empty() {
        header(&mut actions, "Remove");
        for name in &plan.remove {
            let (version, plugin) = app
                .installed
                .get(name)
                .map(|info| {
                    (
                        short_version(&info.version).to_string(),
                        info.marketplace.is_some(),
                    )
                })
                .unwrap_or_else(|| ("?".to_string(), false));
            let note = if plugin {
                "new plugin version dir without it; old dir kept as rollback"
            } else {
                "backed up first"
            };
            actions.push(Line::from(vec![
                Span::styled("    • ", Style::default().fg(Color::Red)),
                Span::raw(format!("{name} v{version}")),
                dim(format!("  ({note})")),
            ]));
        }
    }
    match planned_agents_mode(app) {
        AgentsMdMode::Skip => {
            if app.agents_selected() {
                actions.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(BRAND)),
                    Span::raw("AGENTS.md: skip (leave your file untouched)"),
                ]));
            } else if app.agents_existing.is_some() && !app.agents_has_managed() {
                actions.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(BRAND)),
                    Span::raw("AGENTS.md: no dale-managed section — nothing to remove"),
                ]));
            }
        }
        AgentsMdMode::Install => actions.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(BRAND)),
            Span::raw("AGENTS.md: create (no existing file)"),
        ])),
        AgentsMdMode::Replace => actions.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(BRAND)),
            Span::raw("AGENTS.md: back up current file, then replace"),
        ])),
        AgentsMdMode::Append => actions.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(BRAND)),
            Span::raw("AGENTS.md: insert/update the managed dale section"),
        ])),
        AgentsMdMode::RemoveSection => actions.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::Red)),
            Span::raw("AGENTS.md: remove the dale-managed section (backup kept)"),
        ])),
    }
    if let Some(pi) = plugin::effective(&app.plugin_installs) {
        actions.push(Line::from(dim(format!(
            "  Plugin install detected (dale@{}): skills go into a new version dir \
             v{}; v{} stays as rollback",
            pi.marketplace,
            short_version(&app.payload.version),
            short_version(&pi.version)
        ))));
    }
    actions.push(Line::from(dim(format!(
        "  Overwritten items are backed up to {}/backups/dale-<timestamp>/",
        app.home.display()
    ))));

    let actions_height = (actions.len() as u16 + 2).min(area.height / 2);
    let mut constraints = vec![Constraint::Length(1), Constraint::Length(actions_height)];
    if dilemma {
        constraints.push(Constraint::Length(1)); // Choice line.
        constraints.push(Constraint::Min(3)); // Diff.
    } else {
        constraints.push(Constraint::Min(0));
    }
    let chunks = Layout::vertical(constraints).split(area);

    f.render_widget(
        Paragraph::new(Span::styled(
            "Confirm planned actions",
            Style::default().fg(BRAND).bold(),
        )),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(actions).block(Block::default().borders(Borders::ALL).title(" Plan ")),
        chunks[1],
    );

    if dilemma {
        let choice = |label: &str, active: bool| {
            if active {
                Span::styled(
                    format!("[{label}]"),
                    Style::default().fg(Color::Black).bg(BRAND),
                )
            } else {
                Span::raw(format!(" {label} "))
            }
        };
        let line = Line::from(vec![
            Span::raw("AGENTS.md exists — choose: "),
            choice("r replace", app.agents_choice == AgentsChoice::Replace),
            Span::raw(" "),
            choice(
                "a append managed",
                app.agents_choice == AgentsChoice::Append,
            ),
            Span::raw(" "),
            choice("s skip", app.agents_choice == AgentsChoice::Skip),
        ]);
        f.render_widget(Paragraph::new(line), chunks[2]);

        let diff_lines: Vec<Line> = app
            .diff
            .iter()
            .map(|l| {
                let style = if l.starts_with('+') {
                    Style::default().fg(BRAND)
                } else if l.starts_with('-') {
                    Style::default().fg(Color::Red)
                } else if l.starts_with('@') {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                };
                Line::from(Span::styled(l.clone(), style))
            })
            .collect();
        let diff = Paragraph::new(diff_lines)
            .scroll((app.diff_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" AGENTS.md diff preview (j/k scroll) "),
            );
        f.render_widget(diff, chunks[3]);
    }
}

fn draw_installing(f: &mut Frame, app: &mut App, area: Rect) {
    let [title, list_area, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(area);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(spinner_frame(app), Style::default().fg(BRAND)),
            Span::styled(" Installing…", Style::default().fg(BRAND).bold()),
        ])),
        title,
    );
    let lines: Vec<Line> = app
        .progress
        .iter()
        .map(|(name, state)| {
            let (icon, detail, style) = match state {
                StepState::Pending => ("·", String::new(), Style::default().fg(Color::DarkGray)),
                StepState::Started => (
                    spinner_frame(app),
                    String::new(),
                    Style::default().fg(Color::Yellow),
                ),
                StepState::Done(msg) => ("✓", format!("  {msg}"), Style::default().fg(BRAND)),
                StepState::Failed(err) => {
                    ("✗", format!("  {err}"), Style::default().fg(Color::Red))
                }
            };
            Line::from(vec![
                Span::styled(format!(" {icon} "), style),
                Span::raw(format!("{name:<18}")),
                Span::styled(detail, style),
            ])
        })
        .collect();
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Progress ")),
        list_area,
    );
    if let Some(flash) = &app.flash {
        f.render_widget(
            Paragraph::new(Span::styled(
                flash.clone(),
                Style::default().fg(Color::Yellow),
            )),
            footer,
        );
    }
}

fn draw_done(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    match &app.report {
        Some(Ok(report)) => {
            let ok = report.failures.is_empty();
            lines.push(Line::from(Span::styled(
                if ok {
                    "Done."
                } else {
                    "Finished with failures."
                },
                Style::default()
                    .fg(if ok { BRAND } else { Color::Red })
                    .bold(),
            )));
            lines.push(Line::default());
            if !report.installed.is_empty() {
                lines.push(Line::from(format!(
                    "Installed {} skill(s) at v{}:",
                    report.installed.len(),
                    short_version(&app.payload.version)
                )));
                for (name, _) in &report.installed {
                    lines.push(Line::from(vec![
                        Span::styled("  ✓ ", Style::default().fg(BRAND)),
                        Span::raw(name.clone()),
                    ]));
                }
            }
            if !report.removed.is_empty() {
                lines.push(Line::from(format!(
                    "Removed {} skill(s):",
                    report.removed.len()
                )));
                for name in &report.removed {
                    lines.push(Line::from(vec![
                        Span::styled("  − ", Style::default().fg(Color::Red)),
                        Span::raw(name.clone()),
                    ]));
                }
            }
            if let Some(action) = &report.agents_action {
                lines.push(Line::from(format!("AGENTS.md: {action}")));
            }
            if let Some(dir) = &report.plugin_dir {
                lines.push(Line::from(dim(format!(
                    "Plugin version dir: {}",
                    dir.display()
                ))));
            }
            if let Some(dir) = &report.backup_dir {
                lines.push(Line::from(dim(format!("Backups: {}", dir.display()))));
            }
            for (item, err) in &report.failures {
                lines.push(Line::from(Span::styled(
                    format!("  ✗ {item}: {err}"),
                    Style::default().fg(Color::Red),
                )));
            }
            if report.changed {
                lines.push(Line::default());
                lines.push(Line::from(Span::styled(
                    RESTART_REMINDER,
                    Style::default().fg(Color::Yellow).bold(),
                )));
            }
            if !report.installed.is_empty() {
                lines.push(Line::default());
                lines.push(Line::from(
                    "Use the skills from any Codex session, for example:",
                ));
                for hint in ["$dale-brainstorm", "$dale-graph", "$dale-loop", "$dale-max"] {
                    lines.push(Line::from(Span::styled(
                        format!("  {hint}"),
                        Style::default().fg(BRAND),
                    )));
                }
            }
        }
        Some(Err(e)) => {
            lines.push(Line::from(Span::styled(
                format!("Install failed: {e}"),
                Style::default().fg(Color::Red).bold(),
            )));
        }
        None => lines.push(Line::from("Nothing happened.")),
    }
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title(" Dale ")),
        area,
    );
}
