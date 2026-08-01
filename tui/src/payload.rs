//! Payload model: the set of skills + AGENTS.md + version that an install
//! run operates on. A payload comes either from the embedded assets
//! ("bundled") or from a fetched release tarball ("remote").

use std::cmp::Ordering;

/// Skill groups shown in the picker, in display order.
pub const GROUPS: &[(&str, &[&str])] = &[
    (
        "Core",
        &[
            "dale-brainstorm",
            "dale-lenses",
            "dale-index",
            "dale-graph",
            "dale-proof",
            "dale-visualize",
        ],
    ),
    (
        "Loop",
        &[
            "dale-loop",
            "dale-loop-project",
            "dale-loop-pr",
            "dale-loop-repo",
            "dale-loop-watch",
            "dale-loop-goal",
        ],
    ),
    ("Max", &["dale-max"]),
];

/// Group name for skills not covered by [`GROUPS`].
pub const GROUP_OTHER: &str = "Other";

/// Group name used for the AGENTS.md config item in the picker.
pub const GROUP_CONFIG: &str = "Config";

/// Return the display group of a skill.
pub fn group_of(name: &str) -> &'static str {
    for (group, names) in GROUPS {
        if names.contains(&name) {
            return group;
        }
    }
    GROUP_OTHER
}

fn group_rank(name: &str) -> (usize, usize) {
    for (gi, (_, names)) in GROUPS.iter().enumerate() {
        if let Some(pos) = names.iter().position(|n| *n == name) {
            return (gi, pos);
        }
    }
    (GROUPS.len(), 0)
}

/// Sort skills into picker order: grouped, then declaration order, then name.
pub fn sort_skills(skills: &mut [SkillPayload]) {
    skills.sort_by(|a, b| {
        group_rank(&a.name)
            .cmp(&group_rank(&b.name))
            .then_with(|| a.name.cmp(&b.name))
    });
}

/// True when `s` is a single, safe path component: non-empty, no path
/// separators, and no `.`/`..` traversal.
pub fn is_safe_component(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && !s.contains('/')
        && !s.contains('\\')
        && !s.contains('\0')
}

/// True when `name` may be used as a dale skill directory name.
pub fn is_valid_skill_name(name: &str) -> bool {
    is_safe_component(name) && name.starts_with("dale-")
}

/// True when `rel` is a safe `/`-separated path relative to a skill dir.
pub fn is_safe_rel_path(rel: &str) -> bool {
    !rel.is_empty() && rel.split('/').all(is_safe_component)
}

/// One installable skill: its metadata plus every file under its directory.
#[derive(Debug, Clone)]
pub struct SkillPayload {
    pub name: String,
    pub description: String,
    /// Relative path inside the skill directory -> file bytes.
    pub files: Vec<(String, Vec<u8>)>,
}

/// A complete installable payload.
#[derive(Debug, Clone)]
pub struct Payload {
    /// Version string from `.codex-plugin/plugin.json`.
    pub version: String,
    /// `"bundled"` or `"remote"`.
    pub source: String,
    /// Content of the bundled global AGENTS.md.
    pub agents_md: String,
    pub skills: Vec<SkillPayload>,
}

impl Payload {
    pub fn skill(&self, name: &str) -> Option<&SkillPayload> {
        self.skills.iter().find(|s| s.name == name)
    }
}

/// Parse `X.Y.Z+meta` into comparable parts. Missing numbers become 0.
fn parse_version(v: &str) -> ((u64, u64, u64), &str) {
    let (core, meta) = v.split_once('+').unwrap_or((v, ""));
    let mut nums = core.split('.').map(|p| {
        p.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0)
    });
    let major = nums.next().unwrap_or(0);
    let minor = nums.next().unwrap_or(0);
    let patch = nums.next().unwrap_or(0);
    ((major, minor, patch), meta)
}

/// True when `a` is strictly newer than `b`.
///
/// Compares `major.minor.patch` first, then the build metadata string
/// (Dale build metadata embeds a sortable timestamp such as
/// `codex.20260731205247`).
pub fn version_newer(a: &str, b: &str) -> bool {
    let (ca, ma) = parse_version(a);
    let (cb, mb) = parse_version(b);
    match ca.cmp(&cb) {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => ma > mb,
    }
}

/// Core `X.Y.Z` part of a version, for compact display.
pub fn short_version(v: &str) -> &str {
    v.split('+').next().unwrap_or(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(version_newer("0.5.0", "0.4.9"));
        assert!(version_newer("1.0.0", "0.99.99"));
        assert!(!version_newer("0.4.0", "0.4.0"));
        assert!(!version_newer("0.3.0", "0.4.0"));
        assert!(version_newer(
            "0.4.0+codex.20260801000000",
            "0.4.0+codex.20260731205247"
        ));
        assert!(!version_newer(
            "0.4.0+codex.20260731205247",
            "0.4.0+codex.20260731205247"
        ));
        assert!(version_newer("99.0.0", "0.4.0+codex.20260731205247"));
        assert!(version_newer("0.4.0", "unknown"));
    }

    #[test]
    fn short_version_strips_metadata() {
        assert_eq!(short_version("0.4.0+codex.2026"), "0.4.0");
        assert_eq!(short_version("1.2.3"), "1.2.3");
    }

    #[test]
    fn skill_name_validation() {
        assert!(is_valid_skill_name("dale-brainstorm"));
        assert!(!is_valid_skill_name("notes"));
        assert!(!is_valid_skill_name(".."));
        assert!(!is_valid_skill_name("dale-x/../y"));
        assert!(!is_valid_skill_name("../dale-x"));
        assert!(!is_valid_skill_name(""));
    }

    #[test]
    fn rel_path_validation() {
        assert!(is_safe_rel_path("SKILL.md"));
        assert!(is_safe_rel_path("agents/loop.md"));
        assert!(!is_safe_rel_path("../evil.txt"));
        assert!(!is_safe_rel_path("a/../../evil.txt"));
        assert!(!is_safe_rel_path("/etc/passwd"));
        assert!(!is_safe_rel_path(""));
        assert!(!is_safe_rel_path("a//b"));
    }

    #[test]
    fn grouping() {
        assert_eq!(group_of("dale-brainstorm"), "Core");
        assert_eq!(group_of("dale-loop-pr"), "Loop");
        assert_eq!(group_of("dale-max"), "Max");
        assert_eq!(group_of("dale-unknown"), GROUP_OTHER);
    }

    #[test]
    fn sorting_follows_groups() {
        let mk = |n: &str| SkillPayload {
            name: n.to_string(),
            description: String::new(),
            files: Vec::new(),
        };
        let mut skills = vec![mk("dale-max"), mk("dale-loop"), mk("dale-brainstorm")];
        sort_skills(&mut skills);
        let names: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["dale-brainstorm", "dale-loop", "dale-max"]);
    }
}
