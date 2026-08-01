//! Embedded payload: skills, the bundled global AGENTS.md, and plugin.json
//! are compiled into the binary with rust-embed.

use std::collections::BTreeMap;

use rust_embed::RustEmbed;

use crate::frontmatter;
use crate::payload::{self, Payload, SkillPayload};

#[derive(RustEmbed)]
#[folder = "../skills/"]
struct SkillAssets;

#[derive(RustEmbed)]
#[folder = "../codex/"]
#[include = "AGENTS.md"]
struct CodexAssets;

#[derive(RustEmbed)]
#[folder = "../.codex-plugin/"]
#[include = "plugin.json"]
struct PluginAssets;

/// Extract the `version` field from a plugin.json document.
pub fn version_from_plugin_json(bytes: &[u8]) -> String {
    serde_json::from_slice::<serde_json::Value>(bytes)
        .ok()
        .and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from))
        .unwrap_or_else(|| "0.0.0".to_string())
}

/// Build the payload embedded in this binary at compile time.
pub fn bundled_payload() -> Payload {
    let mut by_skill: BTreeMap<String, Vec<(String, Vec<u8>)>> = BTreeMap::new();
    for path in SkillAssets::iter() {
        let path = path.as_ref();
        let Some((name, rel)) = path.split_once('/') else {
            continue;
        };
        if rel.is_empty() {
            continue;
        }
        let data = SkillAssets::get(path)
            .map(|f| f.data.into_owned())
            .unwrap_or_default();
        by_skill
            .entry(name.to_string())
            .or_default()
            .push((rel.to_string(), data));
    }

    let mut skills = Vec::new();
    for (name, mut files) in by_skill {
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let description = files
            .iter()
            .find(|(rel, _)| rel == "SKILL.md")
            .and_then(|(_, data)| std::str::from_utf8(data).ok())
            .and_then(frontmatter::parse)
            .map(|fm| fm.description)
            .unwrap_or_default();
        skills.push(SkillPayload {
            name,
            description,
            files,
        });
    }
    payload::sort_skills(&mut skills);

    let agents_md = CodexAssets::get("AGENTS.md")
        .and_then(|f| String::from_utf8(f.data.into_owned()).ok())
        .unwrap_or_default();
    let version = PluginAssets::get("plugin.json")
        .map(|f| version_from_plugin_json(&f.data))
        .unwrap_or_else(|| "0.0.0".to_string());

    Payload {
        version,
        source: "bundled".to_string(),
        agents_md,
        skills,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_payload_is_complete() {
        let p = bundled_payload();
        assert!(!p.version.is_empty());
        assert_eq!(p.source, "bundled");
        assert!(p.agents_md.contains("AGENTS.md"));
        assert!(p.skills.len() >= 13);
        let brainstorm = p.skill("dale-brainstorm").expect("dale-brainstorm");
        assert!(!brainstorm.description.is_empty());
        assert!(brainstorm.files.iter().any(|(rel, _)| rel == "SKILL.md"));
    }

    #[test]
    fn plugin_json_version_parses() {
        assert_eq!(version_from_plugin_json(br#"{"version":"1.2.3"}"#), "1.2.3");
        assert_eq!(version_from_plugin_json(b"not json"), "0.0.0");
    }
}
