//! Remote payload fetching: download (or read from disk) a repository
//! tarball and turn its `skills/`, `codex/AGENTS.md`, and
//! `.codex-plugin/plugin.json` into a [`Payload`].

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use flate2::read::GzDecoder;

use crate::assets::version_from_plugin_json;
use crate::frontmatter;
use crate::payload::{self, Payload, SkillPayload};

pub const DEFAULT_UPDATE_URL: &str =
    "https://codeload.github.com/lubluniky/dale/tar.gz/refs/heads/main";

/// GitHub "latest release" URL used by the best-effort self-version check.
pub const RELEASES_LATEST_URL: &str = "https://github.com/lubluniky/dale/releases/latest";

/// One-liner the user reruns to get a newer dale binary.
pub const INSTALL_ONE_LINER: &str = "curl -fsSL https://borkiss.net/dale-install.sh | sh";

/// Best-effort self-version check: the version of the newest published dale
/// release, when it is strictly newer than this build. Any network or parse
/// hiccup yields `None` — this must never fail an update.
pub fn newer_binary_version() -> Option<String> {
    latest_release_tag().filter(|v| payload::version_newer(v, env!("CARGO_PKG_VERSION")))
}

/// Resolve the latest release tag by requesting the `releases/latest` page
/// with redirects disabled and parsing the `Location` header
/// (`…/releases/tag/v0.4.0` → `0.4.0`).
fn latest_release_tag() -> Option<String> {
    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let response = match agent
        .get(RELEASES_LATEST_URL)
        .timeout(Duration::from_secs(10))
        .set("User-Agent", "dale-installer")
        .call()
    {
        Ok(r) => r,
        // With redirects disabled some ureq versions surface 3xx as errors.
        Err(ureq::Error::Status(code, r)) if (300..400).contains(&code) => r,
        Err(_) => return None,
    };
    let location = response.header("Location")?;
    let (_, tag) = location.rsplit_once("/tag/")?;
    let version = tag.trim_start_matches('v').trim_end_matches('/');
    if version.is_empty() {
        None
    } else {
        Some(version.to_string())
    }
}

/// Cap tarball size to keep a hostile or misconfigured source from OOMing us.
const MAX_TARBALL_BYTES: u64 = 100 * 1024 * 1024;

/// Fetch and parse a payload from a URL or a local `.tar.gz` path.
pub fn fetch_payload(source: &str) -> Result<Payload, String> {
    let bytes = load_bytes(source)?;
    parse_tarball(&bytes)
}

fn load_bytes(source: &str) -> Result<Vec<u8>, String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let response = ureq::get(source)
            .timeout(Duration::from_secs(60))
            .set("User-Agent", "dale-installer")
            .call()
            .map_err(|e| format!("download failed: {e}"))?;
        let mut buf = Vec::new();
        response
            .into_reader()
            .take(MAX_TARBALL_BYTES)
            .read_to_end(&mut buf)
            .map_err(|e| format!("download failed while reading body: {e}"))?;
        Ok(buf)
    } else {
        let path = Path::new(source);
        if !path.is_file() {
            return Err(format!("update source not found: {source}"));
        }
        std::fs::read(path).map_err(|e| format!("cannot read {source}: {e}"))
    }
}

/// Parse a repo tarball (with or without the GitHub `repo-ref/` root prefix).
pub fn parse_tarball(bytes: &[u8]) -> Result<Payload, String> {
    let mut archive = tar::Archive::new(GzDecoder::new(bytes));
    let mut skill_files: BTreeMap<String, Vec<(String, Vec<u8>)>> = BTreeMap::new();
    let mut agents_md: Option<String> = None;
    let mut version: Option<String> = None;

    let entries = archive
        .entries()
        .map_err(|e| format!("invalid tarball: {e}"))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("invalid tarball entry: {e}"))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let raw_path = entry
            .path()
            .map_err(|e| format!("invalid tarball path: {e}"))?;
        let comps: Vec<String> = raw_path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        // GitHub tarballs wrap everything in a `repo-ref/` directory; local
        // fixtures may not. Accept both layouts.
        let known_root = |c: &str| matches!(c, "skills" | "codex" | ".codex-plugin");
        let rel: &[String] = match comps.first() {
            Some(first) if known_root(first) => &comps,
            _ if comps.len() > 1 && known_root(&comps[1]) => &comps[1..],
            _ => continue,
        };

        let mut read_bytes = || -> Result<Vec<u8>, String> {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("cannot read tarball entry: {e}"))?;
            Ok(buf)
        };

        match rel[0].as_str() {
            "skills" if rel.len() >= 3 => {
                let name = rel[1].clone();
                let rel_path = rel[2..].join("/");
                // Fail closed on anything that could escape the skill
                // directory when the payload is written to disk.
                if !payload::is_valid_skill_name(&name) || !payload::is_safe_rel_path(&rel_path) {
                    return Err(format!("unsafe path in tarball: {}", comps.join("/")));
                }
                let data = read_bytes()?;
                skill_files.entry(name).or_default().push((rel_path, data));
            }
            "codex" if rel.len() == 2 && rel[1] == "AGENTS.md" => {
                agents_md = Some(String::from_utf8_lossy(&read_bytes()?).into_owned());
            }
            ".codex-plugin" if rel.len() == 2 && rel[1] == "plugin.json" => {
                version = Some(version_from_plugin_json(&read_bytes()?));
            }
            _ => {}
        }
    }

    let mut skills = Vec::new();
    for (name, mut files) in skill_files {
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let Some(skill_md) = files
            .iter()
            .find(|(rel, _)| rel == "SKILL.md")
            .and_then(|(_, data)| std::str::from_utf8(data).ok())
        else {
            continue; // Not a skill directory.
        };
        let description = frontmatter::parse(skill_md)
            .map(|fm| fm.description)
            .unwrap_or_default();
        skills.push(SkillPayload {
            name,
            description,
            files,
        });
    }
    payload::sort_skills(&mut skills);

    let version = version.ok_or("payload is missing .codex-plugin/plugin.json")?;
    let agents_md = agents_md.ok_or("payload is missing codex/AGENTS.md")?;
    if skills.is_empty() {
        return Err("payload contains no skills".to_string());
    }
    Ok(Payload {
        version,
        source: "remote".to_string(),
        agents_md,
        skills,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;

    fn tarball(entries: &[(&str, &str)]) -> Vec<u8> {
        // Write the header name bytes directly: `Builder::append_data`
        // refuses `..` components, but a hostile archive has no such
        // scruples, so the fixtures must not either.
        let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
        for (path, content) in entries {
            let mut header = tar::Header::new_gnu();
            {
                let name = &mut header.as_old_mut().name;
                assert!(path.len() <= name.len(), "fixture path too long: {path}");
                name[..path.len()].copy_from_slice(path.as_bytes());
            }
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, content.as_bytes()).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn parses_github_style_tarball() {
        let bytes = tarball(&[
            (
                "dale-main/skills/dale-brainstorm/SKILL.md",
                "---\nname: dale-brainstorm\ndescription: Remote desc.\n---\nBody\n",
            ),
            ("dale-main/skills/dale-brainstorm/extra.md", "extra\n"),
            ("dale-main/codex/AGENTS.md", "# Remote AGENTS\n"),
            (
                "dale-main/.codex-plugin/plugin.json",
                r#"{"version":"9.9.9"}"#,
            ),
            ("dale-main/README.md", "ignored\n"),
        ]);
        let p = parse_tarball(&bytes).unwrap();
        assert_eq!(p.version, "9.9.9");
        assert_eq!(p.source, "remote");
        assert_eq!(p.agents_md, "# Remote AGENTS\n");
        assert_eq!(p.skills.len(), 1);
        let s = p.skill("dale-brainstorm").unwrap();
        assert_eq!(s.description, "Remote desc.");
        assert_eq!(s.files.len(), 2);
    }

    #[test]
    fn parses_rootless_tarball() {
        let bytes = tarball(&[
            (
                "skills/dale-max/SKILL.md",
                "---\nname: dale-max\ndescription: d\n---\n",
            ),
            ("codex/AGENTS.md", "a"),
            (".codex-plugin/plugin.json", r#"{"version":"1.0.0"}"#),
        ]);
        let p = parse_tarball(&bytes).unwrap();
        assert_eq!(p.version, "1.0.0");
        assert!(p.skill("dale-max").is_some());
    }

    #[test]
    fn incomplete_tarball_is_rejected() {
        let bytes = tarball(&[("dale-main/README.md", "nothing useful")]);
        let err = parse_tarball(&bytes).unwrap_err();
        assert!(err.contains("missing"));
    }

    #[test]
    fn traversal_file_path_is_rejected() {
        let bytes = tarball(&[
            (
                "dale-main/skills/dale-brainstorm/SKILL.md",
                "---\nname: dale-brainstorm\ndescription: x\n---\n",
            ),
            (
                "dale-main/skills/dale-brainstorm/../../../victim.txt",
                "PWNED\n",
            ),
            ("dale-main/codex/AGENTS.md", "# a\n"),
            (
                "dale-main/.codex-plugin/plugin.json",
                r#"{"version":"9.9.9"}"#,
            ),
        ]);
        let err = parse_tarball(&bytes).unwrap_err();
        assert!(err.contains("unsafe path"), "err: {err}");
    }

    #[test]
    fn traversal_skill_name_is_rejected() {
        let bytes = tarball(&[
            ("dale-main/skills/../evil/SKILL.md", "---\nname: e\n---\n"),
            ("dale-main/codex/AGENTS.md", "# a\n"),
            (
                "dale-main/.codex-plugin/plugin.json",
                r#"{"version":"9.9.9"}"#,
            ),
        ]);
        let err = parse_tarball(&bytes).unwrap_err();
        assert!(err.contains("unsafe path"), "err: {err}");
    }

    #[test]
    fn non_dale_skill_dir_is_rejected() {
        let bytes = tarball(&[
            ("dale-main/skills/notes/SKILL.md", "---\nname: notes\n---\n"),
            ("dale-main/codex/AGENTS.md", "# a\n"),
            (
                "dale-main/.codex-plugin/plugin.json",
                r#"{"version":"9.9.9"}"#,
            ),
        ]);
        let err = parse_tarball(&bytes).unwrap_err();
        assert!(err.contains("unsafe path"), "err: {err}");
    }

    #[test]
    fn missing_local_path_is_a_readable_error() {
        let err = fetch_payload("/definitely/not/here.tar.gz").unwrap_err();
        assert!(err.contains("not found"));
    }
}
