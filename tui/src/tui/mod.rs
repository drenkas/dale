//! TUI runtime: terminal setup/teardown, the event loop, and side-effect
//! execution. Rendering and state transitions live in `view` and `update`.

pub mod state;
pub mod update;
pub mod view;

use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use ratatui::backend::CrosstermBackend;
use xai_ratatui_inline::Terminal;

use crate::cli::Ctx;
use crate::{install, remote};
use state::{App, Screen};
use update::{Cmd, Msg};

/// Best-effort terminal restore; safe to call more than once.
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
}

/// Run the interactive installer until the user quits.
pub fn run(ctx: Ctx) -> io::Result<()> {
    // Restore the terminal on a *main-thread* panic so the shell stays
    // usable. Worker-thread panics are caught in `spawn_cmd` and reported
    // through the message channel; restoring from them would leave the
    // still-running event loop drawing over the user's shell.
    let main_thread = thread::current().id();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if thread::current().id() == main_thread {
            restore_terminal();
        }
        default_hook(info);
    }));

    enable_raw_mode()?;
    if let Err(e) = execute!(io::stdout(), EnterAlternateScreen) {
        // Do not leak raw mode when entering the alternate screen fails.
        restore_terminal();
        return Err(e);
    }
    let result = run_app(ctx);
    restore_terminal();
    result
}

/// Event loop, split out so `run` owns terminal setup/teardown.
fn run_app(ctx: Ctx) -> io::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel::<Msg>();

    // Input thread: forward key/resize events into the message channel.
    {
        let tx = tx.clone();
        thread::spawn(move || loop {
            match crossterm::event::read() {
                Ok(Event::Key(key)) if key.kind != KeyEventKind::Release => {
                    if tx.send(Msg::Key(key)).is_err() {
                        break;
                    }
                }
                Ok(Event::Resize(..)) => {
                    if tx.send(Msg::Tick).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        });
    }

    let mut app = App::new(ctx);
    // Kick off the environment scan shown behind the splash screen.
    spawn_cmd(
        Cmd::Scan {
            home: app.home.clone(),
        },
        tx.clone(),
    );
    loop {
        terminal.draw(|f| view::draw(f, &mut app))?;
        let msg = match rx.recv_timeout(Duration::from_millis(120)) {
            Ok(msg) => msg,
            Err(mpsc::RecvTimeoutError::Timeout) => Msg::Tick,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        if let Some(cmd) = update::update(&mut app, msg) {
            spawn_cmd(cmd, tx.clone());
        }
        if app.quit {
            // Never abandon a running install silently; a double ctrl+c
            // sets `force_quit` as the explicit escape hatch.
            if app.screen == Screen::Installing && !app.force_quit {
                app.quit = false;
                continue;
            }
            break;
        }
    }
    Ok(())
}

/// Human-readable message from a caught panic payload.
fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

/// Execute a side effect on its own thread, reporting back over the channel.
/// Effect bodies are wrapped in `catch_unwind` so a panic still produces a
/// result message instead of leaving the UI waiting forever.
fn spawn_cmd(cmd: Cmd, tx: mpsc::Sender<Msg>) {
    match cmd {
        Cmd::Scan { home } => {
            thread::spawn(move || {
                // A crashed scan still advances the splash (empty result).
                let scan =
                    catch_unwind(AssertUnwindSafe(|| install::scan(&home))).unwrap_or_default();
                let _ = tx.send(Msg::Scanned(Box::new(scan)));
            });
        }
        Cmd::Fetch(url) => {
            thread::spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| remote::fetch_payload(&url)))
                    .unwrap_or_else(|p| Err(format!("update check crashed: {}", panic_message(p))));
                // Best-effort self-version check, only against the default
                // update source (never for --update-url overrides).
                let newer = if url == remote::DEFAULT_UPDATE_URL {
                    catch_unwind(AssertUnwindSafe(remote::newer_binary_version)).unwrap_or(None)
                } else {
                    None
                };
                let _ = tx.send(Msg::RemoteFetched(Box::new((result, newer))));
            });
        }
        Cmd::Install {
            home,
            payload,
            skills,
            remove,
            agents,
        } => {
            thread::spawn(move || {
                let progress_tx = tx.clone();
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let mut on_progress = move |p: install::InstallProgress| {
                        let _ = progress_tx.send(Msg::Progress(p));
                    };
                    install::install(&home, &payload, &skills, &remove, agents, &mut on_progress)
                }))
                .unwrap_or_else(|p| Err(format!("install crashed: {}", panic_message(p))));
                let _ = tx.send(Msg::InstallDone(Box::new(result)));
            });
        }
    }
}
