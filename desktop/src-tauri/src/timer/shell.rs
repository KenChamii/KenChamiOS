use super::machine::{Phase, Status, TimerView};
use super::service::PhaseChange;
use crate::db::now_ms;
use crate::state::AppState;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Window, WindowEvent};
use tauri_plugin_notification::NotificationExt;

pub const TICK_EVENT: &str = "timer://tick";
const TRAY_ID: &str = "timer";
const APP_NAME: &str = "KenChamiOs";

/// Tray icon with its menu, and the loop that ticks the timer every second.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", format!("Open {APP_NAME}"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(APP_NAME)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    let app = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            tick(&app);
        }
    });
    Ok(())
}

/// Sends the timer to the ui and the tray tooltip.
pub fn publish(app: &AppHandle, view: Option<&TimerView>) {
    if let Err(error) = app.emit(TICK_EVENT, view) {
        eprintln!("could not send the timer to the window: {error}");
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID)
        && let Err(error) = tray.set_tooltip(Some(tooltip(view)))
    {
        eprintln!("could not update the tray tooltip: {error}");
    }
}

/// Closing the window while a timer exists hides it, so the timer keeps running in the tray.
pub fn keep_timer_alive(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let state = window.state::<AppState>();
    if tauri::async_runtime::block_on(state.timer.view(now_ms())).is_some() {
        api.prevent_close();
        if let Err(error) = window.hide() {
            eprintln!("could not hide the window: {error}");
        }
    }
}

fn tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    match tauri::async_runtime::block_on(state.timer.tick(&state.db, now_ms())) {
        Ok(tick) => {
            if tick.view.is_some() {
                publish(app, tick.view.as_ref());
            }
            if let Some(change) = tick.change {
                notify(app, change);
            }
        }
        Err(error) => eprintln!("timer tick failed: {error}"),
    }
}

fn notify(app: &AppHandle, change: PhaseChange) {
    let result = app
        .notification()
        .builder()
        .title(APP_NAME)
        .body(message(change))
        .show();
    if let Err(error) = result {
        eprintln!("could not show the notification: {error}");
    }
}

fn show_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Err(error) = window.show().and_then(|()| window.set_focus()) {
        eprintln!("could not show the window: {error}");
    }
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Focus => "Focus",
        Phase::Break => "Break",
        Phase::LongBreak => "Long break",
    }
}

fn message(change: PhaseChange) -> String {
    match change.finished {
        Phase::Focus => format!("Focus session done. {} time.", phase_label(change.next)),
        Phase::Break | Phase::LongBreak => "Break over. Back to focus.".to_string(),
    }
}

fn tooltip(view: Option<&TimerView>) -> String {
    let Some(view) = view else {
        return APP_NAME.to_string();
    };
    let paused = if view.status == Status::Paused {
        " (paused)"
    } else {
        ""
    };
    format!(
        "{} {:02}:{:02}{paused}",
        phase_label(view.phase),
        view.remaining_s / 60,
        view.remaining_s % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(phase: Phase, status: Status, remaining_s: u32) -> TimerView {
        TimerView {
            project_id: "p".into(),
            task_id: "t".into(),
            phase,
            status,
            round: 0,
            length_s: 1500,
            remaining_s,
        }
    }

    #[test]
    fn tooltip_shows_phase_and_remaining_time() {
        let running = view(Phase::Focus, Status::Running, 1449);
        assert_eq!(tooltip(Some(&running)), "Focus 24:09");
        let paused = view(Phase::LongBreak, Status::Paused, 5);
        assert_eq!(tooltip(Some(&paused)), "Long break 00:05 (paused)");
        assert_eq!(tooltip(None), "KenChamiOs");
    }

    #[test]
    fn messages_name_the_next_phase() {
        let after_focus = PhaseChange {
            finished: Phase::Focus,
            next: Phase::LongBreak,
        };
        assert_eq!(message(after_focus), "Focus session done. Long break time.");
        let after_break = PhaseChange {
            finished: Phase::Break,
            next: Phase::Focus,
        };
        assert_eq!(message(after_break), "Break over. Back to focus.");
    }
}
