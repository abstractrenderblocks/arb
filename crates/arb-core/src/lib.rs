mod errors;
mod package;
mod schema;
mod validate;

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
