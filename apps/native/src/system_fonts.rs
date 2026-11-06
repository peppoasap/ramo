//! Optional installed-font fallback. No system font is copied into Ramo's bundle.
use ab_glyph::{Font, FontRef};
use eframe::egui::FontData;
use std::path::PathBuf;

fn candidates() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    return vec![
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc".into(),
        "/System/Library/Fonts/Hiragino Sans GB.ttc".into(),
    ];
    #[cfg(target_os = "windows")]
    return std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .map_or_else(Vec::new, |root| {
            ["YuGothM.ttc", "msgothic.ttc", "meiryo.ttc"]
                .iter()
                .map(|name| root.join("Fonts").join(name))
                .collect()
        });
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    vec![
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc".into(),
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc".into(),
    ]
}
fn usable(bytes: Vec<u8>) -> Option<FontData> {
    let font = FontRef::try_from_slice(&bytes).ok()?;
    if !"日本語あア".chars().all(|c| font.glyph_id(c).0 != 0) {
        return None;
    }
    Some(FontData::from_owned(bytes))
}
pub fn installed_cjk() -> Option<FontData> {
    for path in candidates() {
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() || meta.len() > 32 * 1024 * 1024 {
            continue;
        }
        if let Ok(bytes) = std::fs::read(path)
            && let Some(font) = usable(bytes)
        {
            return Some(font);
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_and_latin_only_fonts_are_not_installed_as_cjk_fallback() {
        assert!(usable(vec![0, 1, 2, 3]).is_none());
        assert!(usable(include_bytes!("../assets/fonts/NotoSans-Regular.ttf").to_vec()).is_none());
    }
}
