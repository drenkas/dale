//! AGENTS.md managed-section merging.
//!
//! Append mode wraps the Dale content in marker comments and inserts or
//! replaces only the marked region, never touching user text outside it.

pub const BEGIN_MARKER: &str = "<!-- dale:begin -->";
pub const END_MARKER: &str = "<!-- dale:end -->";

/// Wrap incoming content in the Dale managed-section markers.
pub fn managed_block(incoming: &str) -> String {
    format!("{BEGIN_MARKER}\n{}\n{END_MARKER}", incoming.trim_end())
}

/// True when `text` contains a complete managed-section marker pair.
pub fn has_managed(text: &str) -> bool {
    text.find(BEGIN_MARKER)
        .and_then(|begin| text[begin..].find(END_MARKER))
        .is_some()
}

/// Remove the managed section from `existing`, returning the document
/// without it. `None` when there is no complete marker pair — user text is
/// never touched in that case. Only the marked region and the blank lines
/// directly around it are removed.
pub fn remove_managed(existing: &str) -> Option<String> {
    let begin = existing.find(BEGIN_MARKER)?;
    let end = begin + existing[begin..].find(END_MARKER)? + END_MARKER.len();
    let before = existing[..begin].trim_end();
    let after = existing[end..].trim_start_matches('\n');
    Some(match (before.is_empty(), after.trim().is_empty()) {
        (true, true) => String::new(),
        (true, false) => after.to_string(),
        (false, true) => format!("{before}\n"),
        (false, false) => format!("{before}\n\n{after}"),
    })
}

/// Merge `incoming` into `existing` as a managed section.
///
/// If a marked region exists it is replaced in place; otherwise the block is
/// appended after the user's content. Idempotent: merging the same content
/// twice yields the same document.
pub fn merge_managed(existing: &str, incoming: &str) -> String {
    let block = managed_block(incoming);
    if let Some(begin) = existing.find(BEGIN_MARKER) {
        if let Some(end_rel) = existing[begin..].find(END_MARKER) {
            let end = begin + end_rel + END_MARKER.len();
            let mut out = String::with_capacity(existing.len() + block.len());
            out.push_str(&existing[..begin]);
            out.push_str(&block);
            out.push_str(&existing[end..]);
            return out;
        }
    }
    if existing.trim().is_empty() {
        format!("{block}\n")
    } else {
        format!("{}\n\n{block}\n", existing.trim_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INCOMING: &str = "# Dale rules\n\n- Be rigorous.\n";

    #[test]
    fn fresh_append_preserves_user_text() {
        let existing = "# My rules\n\nDo not break prod.\n";
        let merged = merge_managed(existing, INCOMING);
        assert!(merged.starts_with("# My rules\n\nDo not break prod."));
        assert_eq!(merged.matches(BEGIN_MARKER).count(), 1);
        assert_eq!(merged.matches(END_MARKER).count(), 1);
        assert!(merged.contains("- Be rigorous."));
    }

    #[test]
    fn append_to_empty_produces_bare_block() {
        let merged = merge_managed("", INCOMING);
        assert!(merged.starts_with(BEGIN_MARKER));
        assert!(merged.trim_end().ends_with(END_MARKER));
    }

    #[test]
    fn reappend_is_idempotent() {
        let existing = "# My rules\n";
        let once = merge_managed(existing, INCOMING);
        let twice = merge_managed(&once, INCOMING);
        assert_eq!(once, twice);
    }

    #[test]
    fn replaces_only_marked_region() {
        let existing = format!(
            "before text\n\n{BEGIN_MARKER}\nold dale content\n{END_MARKER}\n\nafter text\n"
        );
        let merged = merge_managed(&existing, "new dale content");
        assert!(merged.starts_with("before text\n\n"));
        assert!(merged.ends_with("\n\nafter text\n"));
        assert!(merged.contains("new dale content"));
        assert!(!merged.contains("old dale content"));
        assert_eq!(merged.matches(BEGIN_MARKER).count(), 1);
    }

    #[test]
    fn remove_managed_inverts_merge_and_preserves_user_text() {
        let existing = "# My rules\n\nDo not break prod.\n";
        let merged = merge_managed(existing, INCOMING);
        let removed = remove_managed(&merged).expect("managed section present");
        assert_eq!(removed, existing);
    }

    #[test]
    fn remove_managed_keeps_text_on_both_sides() {
        let existing =
            format!("before text\n\n{BEGIN_MARKER}\ndale content\n{END_MARKER}\n\nafter text\n");
        let removed = remove_managed(&existing).unwrap();
        assert_eq!(removed, "before text\n\nafter text\n");
    }

    #[test]
    fn remove_managed_without_complete_markers_is_none() {
        assert_eq!(remove_managed("# just user text\n"), None);
        let unclosed = format!("user text\n{BEGIN_MARKER}\nbroken");
        assert_eq!(remove_managed(&unclosed), None);
        assert!(!has_managed(&unclosed));
        assert!(has_managed(&managed_block("x")));
    }

    #[test]
    fn remove_managed_on_bare_block_yields_empty_document() {
        let merged = merge_managed("", INCOMING);
        assert_eq!(remove_managed(&merged).unwrap(), "");
    }

    #[test]
    fn unclosed_marker_falls_back_to_append() {
        let existing = format!("user text\n{BEGIN_MARKER}\nbroken");
        let merged = merge_managed(&existing, INCOMING);
        // The broken region is left untouched; a complete block is appended.
        assert!(merged.contains("broken"));
        assert_eq!(merged.matches(END_MARKER).count(), 1);
    }
}
