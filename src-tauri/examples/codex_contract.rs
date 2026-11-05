//! Supplies the real adapter arguments to the offline synthetic-server probe.
fn main() -> Result<(), String> {
    let output = std::env::args()
        .nth(1)
        .ok_or("Pass a temporary output path")?;
    println!(
        "{}",
        serde_json::json!({
            "binary":ramo_core::process::executable("codex")?,
            "version":ramo_core::ai::CODEX_CLI_VERSION,
            "args":ramo_core::ai::codex_arguments("gpt-6.1-sol", std::path::Path::new(&output))
        })
    );
    Ok(())
}
