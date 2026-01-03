use crate::errors::ArbError;
use std::path::{Path, PathBuf};

fn default_cache_dir() -> Option<PathBuf> {
    // Per spec: user-specific, arb-managed
    dirs::data_local_dir().map(|p| p.join("arb").join("packages"))
}

fn looks_like_path(s: &str) -> bool {
    // Treat anything with a separator or a drive prefix as a path.
    s.contains('\\') || s.contains('/') || s.contains(':')
}

pub fn resolve_package(package: &str, project_root: &Path) -> Result<PathBuf, ArbError> {
    // 1) Explicit path (if it exists)
    if looks_like_path(package) {
        let p = PathBuf::from(package);
        if p.is_dir() {
            return Ok(p);
        }
        return Err(ArbError::PackageNotFound(format!(
            "explicit package path does not exist: {}",
            p.display()
        )));
    }

    // 2) Local packages/<name>
    let local = project_root.join("packages").join(package);
    if local.is_dir() {
        return Ok(local);
    }

    // 3) Cache packages/<name>
    if let Some(cache_root) = default_cache_dir() {
        let cached = cache_root.join(package);
        if cached.is_dir() {
            return Ok(cached);
        }
    }

    Err(ArbError::PackageNotFound(package.to_string()))
}
