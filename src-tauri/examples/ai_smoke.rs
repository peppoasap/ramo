//! Deliberate live check. Never run in CI. Requires explicit user authorization.
use ramo_core::{
    ai::{self, AiConfig},
    git::Backend,
};
fn main() -> Result<(), String> {
    let root = std::env::args()
        .nth(1)
        .ok_or("Pass a synthetic Meadow fixture path")?;
    let temporary = tempfile::tempdir().map_err(|e| e.to_string())?;
    let mut backend = Backend::new(temporary.path().join("preferences.json"));
    let snapshot = backend.register(std::path::Path::new(&root))?;
    let input = ai::prepare(&backend, &snapshot.repo.id, "commit", None, None, true)?;
    if input.paths != vec!["src/search.ts"]
        || input.text.contains("ANTHROPIC")
        || input.text.contains("PRIVATE_SENTINEL")
    {
        return Err("Live check only accepts the approved single-file fixture.".into());
    }
    let model = std::env::var("RAMO_SMOKE_MODEL")
        .unwrap_or_else(|_| ai::codex_preferred_model().unwrap_or_default());
    let answer = ai::generate(
        &input,
        &AiConfig {
            provider: "codex".into(),
            model: model.clone(),
        },
    )?;
    let evidence = serde_json::json!({"date":"2026-11-03","provider":"Codex CLI","cliVersion":ai::CODEX_CLI_VERSION,"model":model,"scope":"Synthetic Meadow staged diff only","paths":input.paths,"inputBytes":input.text.len(),"result":answer,"successfulGenerations":1});
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence).map_err(|e| e.to_string())?
    );
    Ok(())
}
