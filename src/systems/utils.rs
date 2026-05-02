//! Shared utility helpers used across multiple systems.
//!
//! - [`entity_label`] — builds a display string for the entity selector.
//! - [`display_group_name`] — returns `"(empty)"` for blank group names, otherwise the name.
//! - [`sprite_to_entry`] — converts a `Sprite` component to a serialisable `SpriteEntry`.
//! - [`tilemap_tex_path`] / [`tilemap_stem`] — derive texture paths from tilemap folder paths.
//! - [`to_relative`] — converts an absolute path (e.g., from `rfd`) to a CWD-relative path.
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::components::group::Group;
use aberredengine::components::persistent::Persistent;
use aberredengine::components::sprite::Sprite;
use aberredengine::resources::mapdata::SpriteEntry;

/// Build a display label for an entity: `Entity #<id> [group] [Persistent]`.
pub fn entity_label(
    entity: Entity,
    group: Option<&Group>,
    persistent: Option<&Persistent>,
) -> String {
    let group_suffix = group.map(|g| format!(" [{}]", g.0)).unwrap_or_default();
    let persistent_tag = if persistent.is_some() {
        " [Persistent]"
    } else {
        ""
    };
    format!(
        "Entity #{}{}{}",
        entity.index(),
        group_suffix,
        persistent_tag
    )
}

pub fn display_group_name(group: &str) -> &str {
    if group.is_empty() { "(empty)" } else { group }
}

/// Converts a `Sprite` component to its `SpriteEntry` serialization form.
pub fn sprite_to_entry(s: &Sprite) -> SpriteEntry {
    SpriteEntry {
        texture_key: s.tex_key.to_string(),
        width: s.width,
        height: s.height,
        offset: Some([s.offset.x, s.offset.y]),
        origin: Some([s.origin.x, s.origin.y]),
        flip_h: s.flip_h,
        flip_v: s.flip_v,
    }
}

/// Returns the relative path to a tilemap's texture PNG: `<dir>/<stem>.png`.
pub fn tilemap_tex_path(dir: &str, stem: &str) -> String {
    to_relative(&format!("{}/{}.png", dir, stem))
}

/// Returns the directory name (stem) of a tilemap path.
/// E.g. `"assets/tilemaps/forest"` → `"forest"`.
pub fn tilemap_stem(path: &str) -> &str {
    std::path::Path::new(path.trim_end_matches('/'))
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
}

/// Convert an absolute path to a path relative to the current working directory.
/// Works across directory boundaries (produces `../` traversals when needed).
/// Falls back to the original path if canonicalization fails.
pub fn to_relative(path: &str) -> String {
    let make_relative = || -> Option<String> {
        let p = std::path::Path::new(path);
        // canonicalize() requires the path to exist; for new files (e.g. Save As),
        // canonicalize the parent directory and re-append the filename.
        let canon_path = if p.exists() {
            p.canonicalize().ok()?
        } else {
            p.parent()?.canonicalize().ok()?.join(p.file_name()?)
        };
        let canon_cwd = std::env::current_dir().ok()?.canonicalize().ok()?;

        let path_parts: Vec<_> = canon_path.components().collect();
        let cwd_parts: Vec<_> = canon_cwd.components().collect();

        let common = path_parts
            .iter()
            .zip(cwd_parts.iter())
            .take_while(|(a, b)| a == b)
            .count();

        let mut result = std::path::PathBuf::new();
        for _ in 0..(cwd_parts.len() - common) {
            result.push("..");
        }
        for part in &path_parts[common..] {
            result.push(part);
        }
        Some(result.to_string_lossy().into_owned())
    };
    make_relative().unwrap_or_else(|| path.to_owned())
}
