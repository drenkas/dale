//! Minimal YAML frontmatter parsing for SKILL.md files.
//!
//! Dale skill frontmatter is a flat map with single-line scalar values
//! (`name`, `description`), so a full YAML parser is not needed.

/// Parsed SKILL.md frontmatter fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter {
    pub name: String,
    pub description: String,
}

/// Parse the leading `---` frontmatter block of a SKILL.md document.
///
/// Returns `None` when the document has no well-formed frontmatter or the
/// mandatory `name` key is missing.
pub fn parse(content: &str) -> Option<Frontmatter> {
    let rest = content.strip_prefix("---")?;
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))?;

    let mut name = None;
    let mut description = None;
    let mut closed = false;
    for line in rest.lines() {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        if let Some(v) = line.strip_prefix("name:") {
            name = Some(clean_scalar(v));
        } else if let Some(v) = line.strip_prefix("description:") {
            description = Some(clean_scalar(v));
        }
    }
    if !closed {
        return None;
    }
    Some(Frontmatter {
        name: name?,
        description: description.unwrap_or_default(),
    })
}

/// Trim whitespace and one layer of matching quotes from a scalar value.
fn clean_scalar(v: &str) -> String {
    let v = v.trim();
    for quote in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(quote) && v.ends_with(quote) {
            return v[1..v.len() - 1].to_string();
        }
    }
    v.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_and_description() {
        let doc = "---\nname: dale-brainstorm\ndescription: Facilitate focused brainstorming.\n---\n\n# Body\n";
        let fm = parse(doc).unwrap();
        assert_eq!(fm.name, "dale-brainstorm");
        assert_eq!(fm.description, "Facilitate focused brainstorming.");
    }

    #[test]
    fn parses_quoted_values() {
        let doc = "---\nname: \"dale-x\"\ndescription: 'quoted: value'\n---\nbody";
        let fm = parse(doc).unwrap();
        assert_eq!(fm.name, "dale-x");
        assert_eq!(fm.description, "quoted: value");
    }

    #[test]
    fn missing_frontmatter_returns_none() {
        assert!(parse("# Just a heading\n").is_none());
        assert!(parse("---\nname: x\nno closing fence").is_none());
    }

    #[test]
    fn missing_name_returns_none() {
        assert!(parse("---\ndescription: only\n---\n").is_none());
    }

    #[test]
    fn description_defaults_to_empty() {
        let fm = parse("---\nname: dale-y\n---\n").unwrap();
        assert_eq!(fm.description, "");
    }
}
