#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod drafts;
mod editor;
mod graph;
mod merge;
mod theme;
mod worker;

use std::path::PathBuf;

fn main() -> eframe::Result {
    let mut args = std::env::args_os().skip(1);
    let mut repository = None;
    let mut state = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--repository") => repository = Some(required_path(args.next(), "--repository")),
            Some("--state-dir") => state = Some(required_path(args.next(), "--state-dir")),
            Some("--help") => {
                println!(
                    "Ramo Native development preview\n  --repository PATH   Open a Git repository\n  --state-dir PATH    Isolated app preferences and explicit merge drafts\nGit writes require repository trust. No automatic AI inference."
                );
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument. Run with --help.");
                std::process::exit(2);
            }
        }
    }
    let state = state.or_else(default_state).unwrap_or_else(|| {
        eprintln!("A private application state directory is required. Pass --state-dir PATH.");
        std::process::exit(2);
    });
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Ramo Native")
            .with_inner_size([1680., 960.])
            .with_min_inner_size([1000., 680.]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Ramo Native",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, state, repository)))),
    )
}
fn required_path(value: Option<std::ffi::OsString>, flag: &str) -> PathBuf {
    let Some(value) = value.filter(|v| !v.is_empty() && !v.to_string_lossy().starts_with("--"))
    else {
        eprintln!("{flag} requires a path.");
        std::process::exit(2);
    };
    PathBuf::from(value)
}
fn default_state() -> Option<PathBuf> {
    let directory = |name: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    };
    #[cfg(target_os = "macos")]
    if let Some(home) = directory("HOME") {
        return Some(home.join("Library/Application Support/dev.ramo.native"));
    }
    #[cfg(target_os = "windows")]
    if let Some(appdata) = directory("LOCALAPPDATA") {
        return Some(appdata.join("RamoNative"));
    }
    directory("XDG_CONFIG_HOME")
        .map(|p| p.join("ramo-native"))
        .or_else(|| directory("HOME").map(|p| p.join(".config/ramo-native")))
}
