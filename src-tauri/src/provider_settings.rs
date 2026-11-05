//! Non-secret CLI preferences, set only after the native file picker.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Default, Serialize, Deserialize)]
struct Settings {
    codex_cli: Option<PathBuf>,
}

pub fn codex_path(file: &Path) -> Result<Option<PathBuf>, String> {
    match fs::read(file) {
        Ok(bytes) => serde_json::from_slice::<Settings>(&bytes)
            .map(|s| s.codex_cli)
            .map_err(|_| "Cannot read saved Codex CLI selection. Choose the CLI again.".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("Cannot read saved Codex CLI selection".into()),
    }
}

pub fn save_codex(file: &Path, program: PathBuf) -> Result<(), String> {
    let parent = file.parent().ok_or("Invalid provider settings folder")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&Settings {
        codex_cli: Some(program),
    })
    .map_err(|e| e.to_string())?;
    temporary.write_all(&bytes).map_err(|e| e.to_string())?;
    temporary.persist(file).map_err(|e| e.to_string())?;
    Ok(())
}
