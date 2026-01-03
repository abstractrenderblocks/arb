mod errors;
mod package;
mod schema;
mod validate;
mod template;

pub use errors::{ArbError, ValidationError};
pub use schema::{SchemaNode, SchemaType};

use std::fs;
use std::path::{Path, PathBuf};

fn read_yaml_file(path: &Path) -> Result<serde_yaml::Value, ArbError> {
    let text = fs::read_to_string(path)?;
    serde_yaml::from_str::<serde_yaml::Value>(&text).map_err(|e| ArbError::Yaml {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

fn read_schema(path: &Path) -> Result<SchemaNode, ArbError> {
    let text = fs::read_to_string(path)?;
    serde_yaml::from_str::<SchemaNode>(&text).map_err(|e| ArbError::Yaml {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

/// Validate a data file against a package schema.
/// `project_root` is typically the current working directory.
pub fn validate_command(
    project_root: &Path,
    package: &str,
    data_file: &Path,
) -> Result<Vec<ValidationError>, ArbError> {
    let pkg_root: PathBuf = package::resolve_package(package, project_root)?;
    let schema_path = pkg_root.join("schema.yaml");

    if !schema_path.is_file() {
        return Err(ArbError::Other(format!(
            "package schema not found: {}",
            schema_path.display()
        )));
    }

    let schema = read_schema(&schema_path)?;
    let data = read_yaml_file(data_file)?;

    Ok(validate::validate(&schema, &data))
}

use std::io::Write;
use walkdir::WalkDir;

pub fn compile_command(
    project_root: &Path,
    package: &str,
    data_file: &Path,
    out_dir: &Path,
) -> Result<(), ArbError> {
    // 1) Resolve package
    let pkg_root: PathBuf = package::resolve_package(package, project_root)?;

    // 2) Load schema + validate (must happen before any rendering)
    let schema_path = pkg_root.join("schema.yaml");
    if !schema_path.is_file() {
        return Err(ArbError::Other(format!(
            "package schema not found: {}",
            schema_path.display()
        )));
    }
    let schema = read_schema(&schema_path)?;
    let data = read_yaml_file(data_file)?;
    let errs = validate::validate(&schema, &data);
    if !errs.is_empty() {
        return Err(ArbError::ValidationFailed(errs.len()));
    }

    // 3) Walk templates/
    let templates_dir = pkg_root.join("templates");
    if !templates_dir.is_dir() {
        return Err(ArbError::Other(format!(
            "package templates directory not found: {}",
            templates_dir.display()
        )));
    }

    // --- Atomic output: render into temp dir, swap on success ---
    let out_parent = out_dir.parent().unwrap_or_else(|| Path::new("."));
    let out_base = out_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("out")
        .to_string();

    let tmp_dir = out_parent.join(format!("{out_base}.__arb_tmp__"));
    let backup_dir = out_parent.join(format!("{out_base}.__arb_old__"));

    // Clean any stale temp dir (best effort)
    if tmp_dir.exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
    std::fs::create_dir_all(&tmp_dir)?;

    // Do ALL rendering/copying into tmp_dir
    let render_result: Result<(), ArbError> = (|| {
        for entry in WalkDir::new(&templates_dir) {
            let entry = entry.map_err(|e| ArbError::Io(e.to_string()))?;
            if entry.file_type().is_dir() {
                continue;
            }

            let src_path = entry.path();
            let rel = src_path.strip_prefix(&templates_dir).map_err(|_| {
                ArbError::Other("internal error: failed to compute template relative path".to_string())
            })?;

            let rel_str = rel.to_string_lossy().replace('\\', "/");

            // Copy assets (non-.arb) verbatim
            if !rel_str.ends_with(".arb") {
                let dst_path = tmp_dir.join(rel);
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(src_path, &dst_path)?;
                continue;
            }

            let file_name = rel
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| ArbError::Other("invalid template file name".to_string()))?;

            // Skip “partial” templates (include-only convention)
            if file_name.starts_with('_') {
                continue;
            }

            // Render .arb → output without .arb suffix
            let mut out_rel = rel.to_path_buf();
            let out_name: String = file_name.trim_end_matches(".arb").to_string();
            out_rel.set_file_name(out_name);

            let dst_path = tmp_dir.join(out_rel);
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let text = std::fs::read_to_string(src_path)?;
            let rendered =
                template::render_var_if_rep_inc(&templates_dir, &rel_str, &text, &data)?;

            // Enforce output size limit (10 MB default per spec)
            let max_bytes: usize = 10 * 1024 * 1024;
            if rendered.as_bytes().len() > max_bytes {
                return Err(ArbError::Other(format!(
                    "output exceeds size limit (10 MB): {}",
                    dst_path.display()
                )));
            }

            let mut f = std::fs::File::create(&dst_path)?;
            f.write_all(rendered.as_bytes())?;
        }
        Ok(())
    })();

    // If anything failed, delete tmp and leave out_dir untouched
    if let Err(e) = render_result {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(e);
    }

    // Swap into place:
    // 1) out -> backup (if exists)
    // 2) tmp -> out
    // 3) delete backup
    if backup_dir.exists() {
        let _ = std::fs::remove_dir_all(&backup_dir);
    }

    if out_dir.exists() {
        std::fs::rename(out_dir, &backup_dir)?;
    }

    // Move tmp into place
    match std::fs::rename(&tmp_dir, out_dir) {
        Ok(()) => {
            if backup_dir.exists() {
                let _ = std::fs::remove_dir_all(&backup_dir);
            }
            Ok(())
        }
        Err(e) => {
            // rollback best-effort
            let _ = std::fs::remove_dir_all(out_dir);
            if backup_dir.exists() {
                let _ = std::fs::rename(&backup_dir, out_dir);
            }
            Err(ArbError::Io(e.to_string()))
        }
    }

}
