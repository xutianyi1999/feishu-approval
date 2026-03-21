use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn read_json_file(path: &Path) -> Result<Value> {
    let s = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&s).with_context(|| format!("invalid JSON in {}", path.display()))
}

pub fn read_string_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .with_context(|| format!("read {}", path.display()))
}

/// Shallow-merge: top-level keys from `patch` overwrite `base`. If `patch` is not an object, returns error.
pub fn merge_object(base: &mut Map<String, Value>, patch: Value) -> Result<()> {
    let obj = patch
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("merge JSON must be a JSON object"))?;
    for (k, v) in obj {
        base.insert(k.clone(), v.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_object_overwrites_and_rejects_non_object_patch() {
        let mut m = Map::new();
        m.insert("a".into(), json!(1));
        merge_object(&mut m, json!({"a": 2, "b": 3})).unwrap();
        assert_eq!(m.get("a").unwrap(), &json!(2));
        assert_eq!(m.get("b").unwrap(), &json!(3));
        let err = merge_object(&mut Map::new(), json!([1])).unwrap_err();
        assert!(err.to_string().contains("JSON object"));
    }
}
