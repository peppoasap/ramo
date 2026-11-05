//! Explicit native draft storage, separate from repositories and released WebView data.
use ramo_core::git::hash;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const MAX_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scope {
    pub repository: String,
    pub path: String,
    pub fingerprint: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub version: u32,
    pub scope: Scope,
    pub text: String,
}
#[derive(Debug, Clone)]
pub struct Loaded {
    pub record: Record,
    pub revision: String,
}

pub fn permitted(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    ![
        ".env",
        "credentials",
        "id_rsa",
        "id_ed25519",
        ".npmrc",
        ".netrc",
    ]
    .iter()
    .any(|s| name == *s || name.starts_with(&format!("{s}.")))
        && !["pem", "key", "p12", "pfx"].contains(
            &Path::new(&name)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or(""),
        )
}

fn destination(root: &Path, scope: &Scope) -> PathBuf {
    root.join(format!(
        "{}.json",
        hash(format!("{}\0{}", scope.repository, scope.path))
    ))
}

pub fn load(root: &Path, scope: &Scope) -> Result<Option<Loaded>, String> {
    if !permitted(&scope.path) {
        return Err("Draft storage is disabled for this sensitive filename.".into());
    }
    let path = destination(root, scope);
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > MAX_BYTES {
        return Err("Draft is not a bounded regular file. Existing data was preserved.".into());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let record: Record = serde_json::from_slice(&bytes)
        .map_err(|_| "Draft is unreadable. Existing data was preserved.")?;
    if record.version != 1
        || record.scope.repository != scope.repository
        || record.scope.path != scope.path
    {
        return Err("Draft scope/version is unsupported. Existing data was preserved.".into());
    }
    Ok(Some(Loaded {
        record,
        revision: hash(bytes),
    }))
}

struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn save(
    root: &Path,
    scope: Scope,
    text: String,
    expected: Option<String>,
) -> Result<Loaded, String> {
    if !permitted(&scope.path) {
        return Err("Draft storage is disabled for this sensitive filename.".into());
    }
    if text.len() > 1_048_576 || text.contains('\0') {
        return Err("Draft must be UTF-8 text up to 1 MiB.".into());
    }
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let path = destination(root, &scope);
    let lock = path.with_extension("lock");
    let handle=fs::OpenOptions::new().write(true).create_new(true).open(&lock)
        .map_err(|_|"Another draft write or interrupted writer holds this draft. Existing data was preserved.")?;
    let _guard = Lock(lock);
    drop(handle);
    if load(root, &scope)?.map(|s| s.revision) != expected {
        return Err(
            "Another window changed this draft. Existing and in-memory copies were preserved."
                .into(),
        );
    }
    let record = Record {
        version: 1,
        scope,
        text,
    };
    let bytes = serde_json::to_vec(&record).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Encoded draft exceeds storage limit.".into());
    }
    let mut tmp = tempfile::NamedTempFile::new_in(root).map_err(|e| e.to_string())?;
    tmp.write_all(&bytes)
        .and_then(|_| tmp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    tmp.persist(path).map_err(|e| e.to_string())?;
    Ok(Loaded {
        record,
        revision: hash(bytes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scope() -> Scope {
        Scope {
            repository: "synthetic".into(),
            path: "src/search.rs".into(),
            fingerprint: "before".into(),
        }
    }
    #[test]
    fn roundtrip_and_competing_writer_preserve_the_winning_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let text = "日本語\r\nline without EOF";
        let first = save(dir.path(), scope(), text.into(), None).unwrap();
        assert_eq!(
            load(dir.path(), &scope()).unwrap().unwrap().record.text,
            text
        );
        let second = save(
            dir.path(),
            scope(),
            "new".into(),
            Some(first.revision.clone()),
        )
        .unwrap();
        assert!(save(dir.path(), scope(), "stale".into(), Some(first.revision)).is_err());
        assert_eq!(
            load(dir.path(), &scope()).unwrap().unwrap().revision,
            second.revision
        );
    }
    #[test]
    fn malformed_locked_and_sensitive_records_fail_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let p = destination(dir.path(), &scope());
        fs::write(&p, b"malformed").unwrap();
        assert!(save(dir.path(), scope(), "x".into(), None).is_err());
        assert_eq!(fs::read(&p).unwrap(), b"malformed");
        fs::remove_file(&p).unwrap();
        fs::write(p.with_extension("lock"), b"").unwrap();
        assert!(save(dir.path(), scope(), "x".into(), None).is_err());
        assert!(!permitted("config/.env.local"));
        assert!(!permitted("id_rsa"));
        assert!(permitted("src/key_bindings.rs"));
    }
}
