use crate::commands;
use crate::db::due_reminders::{self, DueReminder};
use crate::db::jobs::{self, DueStep};
use crate::db::{backups, now_ms, vault};
use crate::state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

const APP_NAME: &str = "KenChamiOs";
const EVERY: Duration = Duration::from_secs(60);
// more due tasks than this become one summary notification
const MAX_SEPARATE: usize = 3;
// edits in either place reach the other within about this long
const VAULT_EVERY: Duration = Duration::from_secs(30);

/// Background work: due task reminders and the daily backup once a minute, the vault sync more often.
pub fn start(app: &AppHandle) {
    start_vault(app);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            run_once(&app).await;
            tokio::time::sleep(EVERY).await;
        }
    });
}

fn start_vault(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            sync_vault(&app).await;
            tokio::time::sleep(VAULT_EVERY).await;
        }
    });
}

// only when the user turned it on and the folder is there
async fn sync_vault(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !matches!(vault::enabled(&state.db).await, Ok(true)) {
        return;
    }
    match commands::vault::sync_once(&state).await {
        Ok(report) if !report.changes.is_empty() => {
            let _ = app.emit("vault-synced", report);
        }
        Ok(_) => {}
        Err(error) => eprintln!("vault sync failed: {error}"),
    }
}

async fn run_once(app: &AppHandle) {
    let state = app.state::<AppState>();
    match due_reminders::take(&state.db, now_ms()).await {
        Ok(due) => notify_due(app, &due),
        Err(error) => eprintln!("could not read due tasks: {error}"),
    }
    match jobs::take_due_steps(&state.db, now_ms()).await {
        Ok(steps) => notify_steps(app, &steps),
        Err(error) => eprintln!("could not read job follow-ups: {error}"),
    }
    if let Err(error) = backups::run_if_due(&state.db, &state.default_backups_dir).await {
        eprintln!("daily backup failed: {error}");
    }
}

fn notify_due(app: &AppHandle, due: &[DueReminder]) {
    let bodies: Vec<String> = if due.len() > MAX_SEPARATE {
        vec![format!("{} tasks are due today", due.len())]
    } else {
        due.iter()
            .map(|task| format!("Due today: {} ({})", task.title, task.project_name))
            .collect()
    };
    show_all(app, bodies);
}

fn notify_steps(app: &AppHandle, steps: &[DueStep]) {
    let bodies: Vec<String> = if steps.len() > MAX_SEPARATE {
        vec![format!("{} job follow-ups are due", steps.len())]
    } else {
        steps
            .iter()
            .map(|step| {
                let what = if step.next_step.is_empty() {
                    "Follow up"
                } else {
                    step.next_step.as_str()
                };
                format!("{what}: {} at {}", step.role, step.company)
            })
            .collect()
    };
    show_all(app, bodies);
}

fn show_all(app: &AppHandle, bodies: Vec<String>) {
    for body in bodies {
        let result = app
            .notification()
            .builder()
            .title(APP_NAME)
            .body(body)
            .show();
        if let Err(error) = result {
            eprintln!("could not show the notification: {error}");
        }
    }
}
