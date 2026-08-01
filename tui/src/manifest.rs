//! The install manifest at `$CODEX_HOME/skills/.dale-manifest.json` plus
//! small time-formatting helpers (no external time crates).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const MANIFEST_FILE: &str = ".dale-manifest.json";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub source: String,
    #[serde(default, rename = "installedAt")]
    pub installed_at: String,
    #[serde(default)]
    pub skills: BTreeMap<String, String>,
}

pub fn manifest_path(home: &Path) -> PathBuf {
    home.join("skills").join(MANIFEST_FILE)
}

/// Load the manifest; missing or corrupt files yield an empty manifest.
pub fn load(home: &Path) -> Manifest {
    fs::read_to_string(manifest_path(home))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Atomically write the manifest (temp file + rename).
pub fn save(home: &Path, manifest: &Manifest) -> io::Result<()> {
    let path = manifest_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(manifest).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json + "\n")?;
    fs::rename(&tmp, &path)
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn civil_parts(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, mo, d) = civil_from_days(days);
    let h = (rem / 3600) as u32;
    let mi = ((rem % 3600) / 60) as u32;
    let s = (rem % 60) as u32;
    (y, mo, d, h, mi, s)
}

/// Current UTC time as RFC 3339, e.g. `2026-07-31T20:52:47Z`.
pub fn rfc3339_now() -> String {
    let (y, mo, d, h, mi, s) = civil_parts(now_unix());
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Current UTC time compacted for directory names, e.g. `20260731-205247`.
pub fn compact_timestamp() -> String {
    let (y, mo, d, h, mi, s) = civil_parts(now_unix());
    format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = Manifest {
            version: "0.4.0+codex.1".to_string(),
            source: "bundled".to_string(),
            installed_at: "2026-07-31T00:00:00Z".to_string(),
            skills: BTreeMap::new(),
        };
        m.skills
            .insert("dale-brainstorm".to_string(), "0.4.0+codex.1".to_string());
        save(dir.path(), &m).unwrap();
        assert_eq!(load(dir.path()), m);

        // Field names on disk match the documented schema.
        let raw = fs::read_to_string(manifest_path(dir.path())).unwrap();
        assert!(raw.contains("\"installedAt\""));
        assert!(raw.contains("\"skills\""));
    }

    #[test]
    fn missing_or_corrupt_manifest_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()), Manifest::default());
        fs::create_dir_all(dir.path().join("skills")).unwrap();
        fs::write(manifest_path(dir.path()), "{ not json").unwrap();
        assert_eq!(load(dir.path()), Manifest::default());
    }

    #[test]
    fn civil_conversion_known_date() {
        // 2026-07-31 20:52:47 UTC == 1785531167.
        let (y, mo, d, h, mi, s) = civil_parts(1_785_531_167);
        assert_eq!((y, mo, d, h, mi, s), (2026, 7, 31, 20, 52, 47));
        let (y, mo, d, ..) = civil_parts(0);
        assert_eq!((y, mo, d), (1970, 1, 1));
    }
}
