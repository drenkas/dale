# dale — TUI installer for the Dale skill family

Interactive terminal installer that puts the Dale skills into
`$CODEX_HOME/skills` (default `~/.codex/skills`) and can manage the global
`AGENTS.md`. Built on ratatui with the grok-build pager stack
(`xai-ratatui-textarea` for the filter input, `xai-ratatui-inline` for the
terminal).

## Install

```sh
curl -fsSL https://borkiss.net/dale-install.sh | sh
```

From source:

```sh
cargo install --path tui
```

## Usage

- `dale` — open the interactive picker (TTY only). Checkboxes express the
  desired state: installed skills start checked, unchecking one plans its
  removal, checking a new one plans its install. The confirm screen groups
  the plan into install / update / remove.
- Existing plugin installs (`$CODEX_HOME/plugins/cache/…/dale/<version>/`)
  are detected automatically: changes always land in a **new** version dir;
  the old dir is never touched and stays as rollback material.
- Everything an install overwrites or removes is first backed up to
  `$CODEX_HOME/backups/dale-<timestamp>/`; the manifest lives at
  `$CODEX_HOME/skills/.dale-manifest.json`.
- After any run that changed something, restart Codex (app or CLI session)
  so it reloads skills and config — both the TUI and the CLI remind you.

## Headless mode (for agents/CI)

Every operation is scriptable; nothing prompts (mutating commands require
`--yes`).

| Command | What it does |
|---|---|
| `dale install --all --yes` | Install every bundled skill. |
| `dale install dale-brainstorm dale-graph --yes` | Install specific skills. |
| `dale install --all --yes --agents-md append` | Also merge the managed `<!-- dale:begin/end -->` section into `AGENTS.md` (`replace` and `skip` exist too). |
| `dale update --yes` | Fetch the remote payload, update installed skills that are older, install skills new in the payload ("added"), then print `Summary: updated N, added M, up-to-date K.` Also best-effort checks GitHub for a newer `dale` binary (default update source only). |
| `dale list` | Human-readable status of every skill and `AGENTS.md`. |
| `dale list --json` | Machine-readable status: `{codexHome, bundledVersion, pluginInstall: {marketplace, version, path}\|null, skills: [{name, installed, source: "loose"\|"plugin"\|null, installedVersion, availableVersion, updateAvailable}], agentsMd: {present, daleManaged}}`. |
| `dale uninstall dale-graph --yes` | Remove a skill. Loose installs are backed up then deleted; plugin installs get a new version dir without the skill (old dir kept as rollback). |
| `dale uninstall --all --yes` | Remove every installed dale skill. |
| `dale uninstall --agents-md --yes` | Remove only the dale-managed section from `AGENTS.md` (backup first; the file is never deleted; a no-op without markers). |

Global flags: `--codex-home <path>` (or `$CODEX_HOME`) and
`--update-url <url-or-.tar.gz-path>` (or `$DALE_UPDATE_URL`). The self-version
check is skipped for `--update-url` overrides, so update runs against local
tarballs stay fully offline.

One-line example against a throwaway home:

```sh
dale --codex-home /tmp/codex install dale-brainstorm --yes && dale --codex-home /tmp/codex list --json
```

## Development

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo run
```

Tests never touch the network; the update path is covered with a local
`.tar.gz` fixture passed via `--update-url`.
