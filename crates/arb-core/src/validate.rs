use crate::errors::ValidationError;
use crate::schema::{SchemaNode, SchemaType};
use serde_yaml::Value;

pub fn validate(schema: &SchemaNode, data: &Value) -> Vec<ValidationError> {
    let mut errs = Vec::new();
    validate_node(schema, data, "$", &mut errs);
    errs
}

fn validate_node(schema: &SchemaNode, data: &Value, path: &str, errs: &mut Vec<ValidationError>) {
    match schema.node_type {
        SchemaType::Object => {
            let map = match data.as_mapping() {
                Some(m) => m,
                None => {
                    errs.push(ValidationError {
                        path: path.to_string(),
                        message: "expected object".to_string(),
                    });
                    return;
                }
            };

            // required fields
            for req in &schema.required {
                let key = Value::String(req.clone());
                if !map.contains_key(&key) {
                    errs.push(ValidationError {
                        path: format!("{path}.{req}"),
                        message: "missing required field".to_string(),
                    });
                }
            }

            // validate known properties (extra keys allowed in v1)
            for (k, v) in map {
                let key_str = match k.as_str() {
                    Some(s) => s,
                    None => continue, // ignore non-string keys
                };

                if let Some(prop_schema) = schema.properties.get(key_str) {
                    let child_path = format!("{path}.{key_str}");
                    validate_node(prop_schema, v, &child_path, errs);
                }
            }
        }

        SchemaType::List => {
            let seq = match data.as_sequence() {
                Some(s) => s,
                None => {
                    errs.push(ValidationError {
                        path: path.to_string(),
                        message: "expected list".to_string(),
                    });
                    return;
                }
            };

            let item_schema = match &schema.items {
                Some(i) => i.as_ref(),
                None => {
                    // spec allows list type; items should exist, but if missing, treat as schema error
                    errs.push(ValidationError {
                        path: path.to_string(),
                        message: "schema error: list missing 'items'".to_string(),
                    });
                    return;
                }
            };

            for (i, item) in seq.iter().enumerate() {
                let child_path = format!("{path}[{i}]");
                validate_node(item_schema, item, &child_path, errs);
            }
        }

        SchemaType::String => {
            if !data.is_string() {
                errs.push(ValidationError {
                    path: path.to_string(),
                    message: "expected string".to_string(),
                });
            }
        }

        SchemaType::Number => {
            if !data.is_number() {
                errs.push(ValidationError {
                    path: path.to_string(),
                    message: "expected number".to_string(),
                });
            }
        }

        SchemaType::Boolean => {
            if !data.is_bool() {
                errs.push(ValidationError {
                    path: path.to_string(),
                    message: "expected boolean".to_string(),
                });
            }
        }
    }
}
