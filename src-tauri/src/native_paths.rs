//! Keep filesystem identity checks separate from Git for Windows path syntax.
use std::path::Path;

pub(crate) fn git_path(path: &Path) -> Result<String, String> {
    let value = path.to_str().ok_or("Git paths must be UTF-8")?;
    #[cfg(windows)]
    return windows_git_path(value);
    #[cfg(not(windows))]
    Ok(value.to_owned())
}

pub(crate) fn same_path(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    return matches!((git_path(left), git_path(right)), (Ok(a), Ok(b)) if a == b);
    #[cfg(not(windows))]
    (left == right)
}

// Only call for native filesystem paths, never messages, URLs or arbitrary args.
#[cfg(any(windows, test))]
fn windows_git_path(value: &str) -> Result<String, String> {
    let value = value.replace('\\', "/");
    if let Some(rest) = value.strip_prefix("//?/") {
        if let Some(unc) = rest.strip_prefix("UNC/") {
            if unc.split('/').filter(|part| !part.is_empty()).count() >= 2 {
                return Ok(format!("//{unc}"));
            }
        } else if rest.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
            && rest.as_bytes().get(1) == Some(&b':')
            && rest.as_bytes().get(2) == Some(&b'/')
        {
            return Ok(rest.to_owned());
        }
        return Err("This Windows device path cannot be passed safely to Git. Choose a drive or UNC folder.".into());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn git_for_windows_receives_drive_and_unc_paths_not_device_namespaces() {
        assert_eq!(
            windows_git_path(r"\\?\C:\Test folder\café").unwrap(),
            "C:/Test folder/café"
        );
        assert_eq!(
            windows_git_path(r"\\?\UNC\server\share\repo").unwrap(),
            "//server/share/repo"
        );
        assert_eq!(
            windows_git_path(r"C:\Test folder\repo").unwrap(),
            "C:/Test folder/repo"
        );
        assert!(windows_git_path(r"\\?\GLOBALROOT\Device\HarddiskVolume1").is_err());
        assert!(windows_git_path(r"\\?\Volume{fixture}\repo").is_err());
        assert!(windows_git_path(r"\\?\UNC\server").is_err());
    }
    #[test]
    fn native_paths_do_not_erase_real_location_differences() {
        assert!(same_path(Path::new("folder"), Path::new("folder")));
        assert!(!same_path(Path::new("first"), Path::new("second")));
        #[cfg(windows)]
        {
            assert!(same_path(
                Path::new(r"\\?\C:\Test\repo"),
                Path::new("C:/Test/repo")
            ));
            assert!(!same_path(
                Path::new(r"\\?\C:\Test\repo"),
                Path::new("D:/Test/repo")
            ));
        }
    }
}
