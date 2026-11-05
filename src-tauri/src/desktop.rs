use crate::{
    ai::{self, AiConfig, AiInput, Provider},
    git::{Action, Backend, Commit, ConflictFile, FileChange, Snapshot, WorktreeParent},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{Manager, State};
#[derive(Clone)]
pub struct AppState {
    backend: Arc<Mutex<Backend>>,
    requests: crate::ai_requests::Requests,
    provider_settings: Arc<PathBuf>,
}
fn locked<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    mutex
        .lock()
        .map_err(|_| "Internal state unavailable. Restart Ramo.".into())
}
async fn task<T: Send + 'static>(
    run: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(run)
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn github_repositories(
    host: String,
    organization: String,
    page: u32,
) -> Result<Vec<crate::hosting::Repository>, String> {
    task(move || crate::hosting::github_repositories(&host, &organization, page)).await
}
#[tauri::command]
async fn repository_remotes(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<Vec<crate::remotes::Remote>, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.remote_list(&repo_id)).await
}
#[tauri::command]
async fn recent_repositories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(locked(&state.backend)?.preferences.recent.clone())
}
#[tauri::command]
async fn open_repository(state: State<'_, AppState>) -> Result<Option<Snapshot>, String> {
    let state = state.inner().clone();
    task(move || {
        let Some(folder) = rfd::FileDialog::new()
            .set_title("Open Git repository or worktree")
            .pick_folder()
        else {
            return Ok(None);
        };
        locked(&state.backend)?.register(&folder).map(Some)
    })
    .await
}
#[tauri::command]
async fn choose_clone_folder(
    state: State<'_, AppState>,
    source: bool,
) -> Result<Option<crate::cloning::Grant>, String> {
    let state = state.inner().clone();
    task(move || {
        let Some(path) = rfd::FileDialog::new()
            .set_title(if source {
                "Choose local source repository"
            } else {
                "Choose parent folder for clone"
            })
            .pick_folder()
        else {
            return Ok(None);
        };
        let mut backend = locked(&state.backend)?;
        if source {
            backend.authorize_clone_source(&path).map(Some)
        } else {
            backend.authorize_clone_parent(&path).map(Some)
        }
    })
    .await
}
#[tauri::command]
async fn prepare_clone(
    state: State<'_, AppState>,
    parent_id: String,
    directory: String,
    source: crate::cloning::Source,
) -> Result<crate::cloning::Plan, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.prepare_clone(&parent_id, &directory, source)).await
}
#[tauri::command]
async fn clone_repository(state: State<'_, AppState>, plan_id: String) -> Result<Snapshot, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.clone_repository(&plan_id)).await
}
#[tauri::command]
async fn open_recent(state: State<'_, AppState>, path: String) -> Result<Snapshot, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.recent(&path)).await
}
#[tauri::command]
async fn close_repository(state: State<'_, AppState>, repo_id: String) -> Result<(), String> {
    let state = state.inner().clone();
    task(move || {
        locked(&state.backend)?.close_repository(&repo_id);
        Ok(())
    })
    .await
}
#[tauri::command]
async fn open_worktree(
    state: State<'_, AppState>,
    repo_id: String,
    path: String,
) -> Result<Snapshot, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.open_worktree(&repo_id, &path)).await
}
#[tauri::command]
async fn choose_worktree_parent(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<Option<WorktreeParent>, String> {
    let state = state.inner().clone();
    task(move || {
        locked(&state.backend)?.repository(&repo_id)?;
        let Some(parent) = rfd::FileDialog::new()
            .set_title("Choose parent folder for a new worktree")
            .pick_folder()
        else {
            return Ok(None);
        };
        locked(&state.backend)?
            .authorize_worktree_parent(&repo_id, &parent)
            .map(Some)
    })
    .await
}
#[tauri::command]
async fn trust_repository(state: State<'_, AppState>, repo_id: String) -> Result<Snapshot, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.trust(&repo_id)).await
}
#[tauri::command]
async fn repository_snapshot(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<Snapshot, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.snapshot(&repo_id)).await
}
#[tauri::command]
async fn repository_history(
    state: State<'_, AppState>,
    repo_id: String,
    skip: usize,
) -> Result<Vec<Commit>, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.history(&repo_id, skip)).await
}
#[tauri::command]
async fn amend_message(
    state: State<'_, AppState>,
    repo_id: String,
    head: String,
) -> Result<String, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.amend_message(&repo_id, &head)).await
}
#[tauri::command]
async fn commit_files(
    state: State<'_, AppState>,
    repo_id: String,
    commit: String,
) -> Result<Vec<FileChange>, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.commit_files(&repo_id, &commit)).await
}
#[tauri::command]
async fn file_diff(
    state: State<'_, AppState>,
    repo_id: String,
    path: String,
    commit: Option<String>,
    staged: bool,
) -> Result<String, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.file_diff(&repo_id, &path, commit.as_deref(), staged))
        .await
}
#[tauri::command]
async fn compare_revisions(
    state: State<'_, AppState>,
    repo_id: String,
    from: String,
    to: String,
    kind: crate::compare::Kind,
) -> Result<crate::compare::Comparison, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.compare_revisions(&repo_id, &from, &to, kind)).await
}
#[tauri::command]
async fn comparison_diff(
    state: State<'_, AppState>,
    repo_id: String,
    from: String,
    to: String,
    kind: crate::compare::Kind,
    path: String,
) -> Result<String, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.comparison_diff(&repo_id, &from, &to, kind, &path)).await
}
#[tauri::command]
async fn stash_details(
    state: State<'_, AppState>,
    repo_id: String,
    index: usize,
    oid: String,
) -> Result<crate::stash::Details, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.stash_details(&repo_id, index, &oid)).await
}
#[tauri::command]
async fn stash_diff(
    state: State<'_, AppState>,
    repo_id: String,
    index: usize,
    oid: String,
    path: String,
    untracked: bool,
) -> Result<String, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.stash_diff(&repo_id, index, &oid, &path, untracked)).await
}
#[tauri::command]
async fn conflict_file(
    state: State<'_, AppState>,
    repo_id: String,
    expected: String,
    path: String,
) -> Result<ConflictFile, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.conflict_file(&repo_id, &expected, &path)).await
}
#[tauri::command]
async fn git_action(
    state: State<'_, AppState>,
    repo_id: String,
    expected: String,
    action: Action,
) -> Result<Snapshot, String> {
    let state = state.inner().clone();
    task(move || locked(&state.backend)?.mutate(&repo_id, &expected, action)).await
}
#[tauri::command]
async fn ai_providers(state: State<'_, AppState>) -> Result<Vec<Provider>, String> {
    let state = state.inner().clone();
    task(move || {
        let path = crate::provider_settings::codex_path(&state.provider_settings)?;
        Ok(ai::providers(path.as_deref()))
    })
    .await
}
#[tauri::command]
async fn codex_cli_path(state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(
        crate::provider_settings::codex_path(&state.provider_settings)?
            .map(|p| p.display().to_string()),
    )
}
#[tauri::command]
async fn choose_codex_cli(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let state = state.inner().clone();
    task(move || {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Choose installed Codex CLI 0.166.0")
            .pick_file()
        else {
            return Ok(None);
        };
        let program = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let temporary = tempfile::tempdir().map_err(|e| e.to_string())?;
        ai::require_codex_version(&program, temporary.path())?;
        crate::provider_settings::save_codex(&state.provider_settings, program.clone())?;
        Ok(Some(program.display().to_string()))
    })
    .await
}
#[tauri::command]
async fn ollama_models() -> Result<Vec<String>, String> {
    task(ai::ollama_models).await
}
#[tauri::command]
async fn prepare_ai(
    state: State<'_, AppState>,
    repo_id: String,
    kind: String,
    path: Option<String>,
    commit: Option<String>,
    staged: bool,
) -> Result<AiInput, String> {
    let state = state.inner().clone();
    task(move || {
        let input = ai::prepare(
            &*locked(&state.backend)?,
            &repo_id,
            &kind,
            path.as_deref(),
            commit.as_deref(),
            staged,
        )?;
        state.requests.insert(input.clone())?;
        Ok(input)
    })
    .await
}
#[tauri::command]
async fn generate_ai(
    state: State<'_, AppState>,
    input_id: String,
    config: AiConfig,
) -> Result<String, String> {
    let state = state.inner().clone();
    // Claim before dispatching the blocking worker. Cancel can also consume an
    // unclaimed preview, covering either command arrival order.
    let running = state.requests.begin(&input_id)?;
    task(move || {
        running.cancellation.check()?;
        let input = &running.input;
        ai::ensure_current(input, &locked(&state.backend)?.snapshot(&input.repo_id)?)?;
        let program = if config.provider == "codex" {
            crate::provider_settings::codex_path(&state.provider_settings)?
        } else {
            None
        };
        let answer =
            ai::generate_cancellable(input, &config, program.as_deref(), &running.cancellation)?;
        running.cancellation.check()?;
        ai::ensure_current(input, &locked(&state.backend)?.snapshot(&input.repo_id)?)?;
        running.cancellation.check()?;
        Ok(answer)
    })
    .await
}
#[tauri::command]
async fn cancel_ai(state: State<'_, AppState>, input_id: String) -> Result<(), String> {
    state.requests.cancel(&input_id)
}
#[tauri::command]
async fn startup_repository(state: State<'_, AppState>) -> Result<Option<Snapshot>, String> {
    let state = state.inner().clone();
    task(move || {
        if let Some(path) = std::env::args()
            .skip(1)
            .find(|s| !s.starts_with('-') && PathBuf::from(s).is_dir())
        {
            locked(&state.backend)?
                .register(&PathBuf::from(path))
                .map(Some)
        } else {
            Ok(None)
        }
    })
    .await
}
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let folder = app.path().app_config_dir()?;
            let prefs = folder.join("preferences.json");
            app.manage(AppState {
                backend: Arc::new(Mutex::new(Backend::new(prefs))),
                requests: crate::ai_requests::Requests::default(),
                provider_settings: Arc::new(folder.join("provider-cli.json")),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            recent_repositories,
            repository_remotes,
            github_repositories,
            open_repository,
            open_recent,
            choose_clone_folder,
            prepare_clone,
            clone_repository,
            open_worktree,
            close_repository,
            choose_worktree_parent,
            trust_repository,
            repository_snapshot,
            repository_history,
            amend_message,
            commit_files,
            file_diff,
            git_action,
            conflict_file,
            compare_revisions,
            comparison_diff,
            stash_details,
            stash_diff,
            ai_providers,
            codex_cli_path,
            choose_codex_cli,
            ollama_models,
            prepare_ai,
            generate_ai,
            cancel_ai,
            startup_repository
        ])
        .run(tauri::generate_context!())
        .expect("Could not start Ramo");
}
