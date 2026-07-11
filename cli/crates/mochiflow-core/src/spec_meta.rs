//! spec.yaml metadata types.
//!
//! Uses a minimal YAML subset parser (no PyYAML dependency) matching the Python
//! reference implementation's `parse_yaml_subset`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecMetaError {
    #[error("spec.yaml not found: {0}")]
    NotFound(PathBuf),
    #[error("spec.yaml parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    #[error("spec.yaml invalid: {0}")]
    Invalid(String),
}

/// Parsed spec.yaml metadata.
#[derive(Debug, Clone)]
pub struct SpecMeta {
    pub path: PathBuf,
    pub data: BTreeMap<String, YamlValue>,
}

/// Minimal YAML value matching the supported subset.
#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    Str(String),
    Bool(bool),
    Int(i64),
    Null,
    List(Vec<YamlValue>),
    Map(BTreeMap<String, YamlValue>),
}

impl YamlValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[YamlValue]> {
        match self {
            Self::List(v) => Some(v),
            _ => None,
        }
    }
}

impl SpecMeta {
    pub fn slug(&self) -> &str {
        self.get_str("slug").unwrap_or_else(|| {
            self.path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("")
        })
    }

    pub fn title(&self) -> &str {
        self.get_str("title").unwrap_or_else(|| self.slug())
    }

    pub fn spec_type(&self) -> &str {
        self.get_str("type").unwrap_or("feature")
    }

    pub fn module(&self) -> Option<&str> {
        self.get_str("module")
    }

    pub fn surfaces(&self) -> Vec<&str> {
        self.data
            .get("surfaces")
            .and_then(|v| v.as_list())
            .map(|list| list.iter().filter_map(|item| item.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn integration(&self) -> &str {
        self.get_str("integration").unwrap_or("none")
    }

    pub fn risk(&self) -> &str {
        self.get_str("risk").unwrap_or("standard")
    }

    pub fn status(&self) -> &str {
        self.get_str("status").unwrap_or("draft")
    }

    pub fn updated(&self) -> Option<&str> {
        self.get_str("updated")
    }

    /// Completion timestamp (ISO 8601 UTC), set once when status becomes `done`.
    /// Optional; legacy specs predate this field.
    pub fn completed(&self) -> Option<&str> {
        self.get_str("completed")
    }

    fn get_str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.as_str())
    }
}

/// Read and parse spec.yaml from a spec directory.
pub fn read_spec_metadata(spec_dir: &Path) -> Result<SpecMeta, SpecMetaError> {
    let path = spec_dir.join("spec.yaml");
    if !path.exists() {
        return Err(SpecMetaError::NotFound(path));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| SpecMetaError::Invalid(format!("cannot read: {e}")))?;
    let data = parse_yaml_subset(&text)?;
    Ok(SpecMeta { path, data })
}

/// Minimal YAML subset parser matching the Python reference.
pub fn parse_yaml_subset(text: &str) -> Result<BTreeMap<String, YamlValue>, SpecMetaError> {
    let mut root: BTreeMap<String, YamlValue> = BTreeMap::new();
    // Stack: (indent_level, container reference key path)
    // We use a simpler approach: collect non-empty non-comment lines, then parse.
    let lines: Vec<(usize, usize, &str)> = text
        .lines()
        .enumerate()
        .filter_map(|(lineno, raw)| {
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let indent = raw.len() - raw.trim_start_matches(' ').len();
            Some((lineno + 1, indent, trimmed))
        })
        .collect();

    parse_map(&lines, &mut 0, 0, &mut root)?;
    Ok(root)
}

fn parse_map(
    lines: &[(usize, usize, &str)],
    pos: &mut usize,
    min_indent: usize,
    map: &mut BTreeMap<String, YamlValue>,
) -> Result<(), SpecMetaError> {
    while *pos < lines.len() {
        let (lineno, indent, line) = lines[*pos];
        if indent < min_indent {
            break;
        }

        if line.starts_with("- ") {
            break; // list items belong to parent
        }

        let colon_pos = line.find(':').ok_or_else(|| SpecMetaError::Parse {
            line: lineno,
            message: "expected key: value".to_string(),
        })?;
        let key = line[..colon_pos].trim().to_string();
        let value_part = line[colon_pos + 1..].trim();

        if !value_part.is_empty() {
            map.insert(key, parse_scalar(value_part, lineno)?);
            *pos += 1;
        } else {
            // Check next line to decide: list or nested map
            *pos += 1;
            if *pos < lines.len() && lines[*pos].1 > indent {
                let next_line = lines[*pos].2;
                if next_line.starts_with("- ") {
                    let mut list = Vec::new();
                    parse_list(lines, pos, lines[*pos].1, &mut list)?;
                    map.insert(key, YamlValue::List(list));
                } else {
                    let mut nested = BTreeMap::new();
                    parse_map(lines, pos, lines[*pos].1, &mut nested)?;
                    map.insert(key, YamlValue::Map(nested));
                }
            } else {
                map.insert(key, YamlValue::Null);
            }
        }
    }
    Ok(())
}

fn parse_list(
    lines: &[(usize, usize, &str)],
    pos: &mut usize,
    min_indent: usize,
    list: &mut Vec<YamlValue>,
) -> Result<(), SpecMetaError> {
    while *pos < lines.len() {
        let (lineno, indent, line) = lines[*pos];
        if indent < min_indent {
            break;
        }
        if !line.starts_with("- ") {
            break;
        }
        let item = line[2..].trim();
        list.push(parse_scalar(item, lineno)?);
        *pos += 1;
    }
    Ok(())
}

fn parse_scalar(value: &str, line: usize) -> Result<YamlValue, SpecMetaError> {
    let parsed = match value {
        "true" => YamlValue::Bool(true),
        "false" => YamlValue::Bool(false),
        "null" => YamlValue::Null,
        _ => {
            // Strip inline comment
            let v = if let Some(idx) = value.find(" #") {
                value[..idx].trim()
            } else {
                value
            };
            // Quoted string
            if v.starts_with(['"', '\'']) || v.ends_with(['"', '\'']) {
                let quote = v.as_bytes().first().copied();
                if v.len() < 2 || quote != v.as_bytes().last().copied() {
                    return Err(SpecMetaError::Parse {
                        line,
                        message: "unterminated quoted scalar".to_string(),
                    });
                }
                return Ok(YamlValue::Str(v[1..v.len() - 1].to_string()));
            }
            // Inline list
            if v.starts_with('[') && v.ends_with(']') {
                let inner = v[1..v.len() - 1].trim();
                if inner.is_empty() {
                    return Ok(YamlValue::List(Vec::new()));
                }
                let items: Vec<YamlValue> = inner
                    .split(',')
                    .map(|part| parse_scalar(part.trim(), line))
                    .collect::<Result<_, _>>()?;
                return Ok(YamlValue::List(items));
            }
            // Integer
            if let Ok(n) = v.parse::<i64>() {
                return Ok(YamlValue::Int(n));
            }
            YamlValue::Str(v.to_string())
        }
    };
    Ok(parsed)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_spec_yaml() {
        let yaml = "\
version: 1
slug: test-spec
title: Test Spec
type: feature
surfaces:
  - cli
integration: none
risk: standard
status: draft
created: 2026-01-01
";
        let data = parse_yaml_subset(yaml).unwrap();
        assert_eq!(data.get("slug").and_then(|v| v.as_str()), Some("test-spec"));
        assert_eq!(data.get("version"), Some(&YamlValue::Int(1)));
        let surfaces = data.get("surfaces").and_then(|v| v.as_list()).unwrap();
        assert_eq!(surfaces.len(), 1);
        assert_eq!(surfaces[0].as_str(), Some("cli"));
    }

    #[test]
    fn test_completed_accessor_present_and_absent() {
        let with = "\
version: 1
slug: s
title: S
type: feature
surfaces:
  - cli
integration: none
risk: standard
status: done
completed: 2026-06-21T22:16:03Z
";
        let meta = SpecMeta {
            path: PathBuf::from("s/spec.yaml"),
            data: parse_yaml_subset(with).unwrap(),
        };
        assert_eq!(meta.completed(), Some("2026-06-21T22:16:03Z"));

        let without = "\
version: 1
slug: s
title: S
type: feature
surfaces:
  - cli
integration: none
risk: standard
status: done
";
        let meta = SpecMeta {
            path: PathBuf::from("s/spec.yaml"),
            data: parse_yaml_subset(without).unwrap(),
        };
        assert_eq!(meta.completed(), None);
    }

    #[test]
    fn malformed_quoted_scalars_return_errors_without_panicking() {
        for value in ["'", "\"", "'unterminated", "\"unterminated", "value'"] {
            let yaml = format!("title: {value}\n");
            let result = std::panic::catch_unwind(|| parse_yaml_subset(&yaml));
            assert!(result.is_ok(), "parser panicked for {value:?}");
            assert!(result.unwrap().is_err(), "accepted {value:?}");
        }
        for value in ["''", "\"\""] {
            let yaml = format!("title: {value}\n");
            assert_eq!(
                parse_yaml_subset(&yaml)
                    .unwrap()
                    .get("title")
                    .and_then(YamlValue::as_str),
                Some("")
            );
        }
    }
}
