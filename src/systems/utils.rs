//! Shared utility helpers used across multiple systems.
//!
//! - [`entity_label`] — builds a display string for the entity selector.
//! - [`display_group_name`] — returns `"(empty)"` for blank group names, otherwise the name.
//! - [`sprite_to_entry`] — converts a `Sprite` component to a serialisable `SpriteEntry`.
//! - [`tilemap_tex_path`] / [`tilemap_stem`] — derive texture paths from tilemap folder paths.
//! - [`to_relative`] — converts an absolute path (e.g., from `rfd`) to a CWD-relative path.
//! - [`find_texture`]/[`find_texture_mut`]/[`default_texture_key`] and their font equivalents —
//!   `MapData.textures`/`.fonts` lookups. `MapData` is the logic-side source of truth for which
//!   texture/font keys exist (the render-side `TextureStore`/`FontStore` aren't queryable from
//!   logic code); use these instead of re-deriving `.iter().find(...)` at each call site.
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::core::components::boxcollider::BoxCollider;
use aberredengine::core::components::group::Group;
use aberredengine::core::components::persistent::Persistent;
use aberredengine::core::components::sprite::Sprite;
use aberredengine::core::resources::mapdata::{
    BoxColliderEntry, FontEntry, MapData, SpriteEntry, TextureEntry,
};

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
        offset: nonzero_vec2(s.offset.x, s.offset.y),
        origin: nonzero_vec2(s.origin.x, s.origin.y),
        flip_h: s.flip_h,
        flip_v: s.flip_v,
    }
}

/// Converts a `BoxCollider` component to its serializable map entry.
pub fn collider_to_entry(collider: &BoxCollider) -> BoxColliderEntry {
    BoxColliderEntry {
        size: [collider.size.x, collider.size.y],
        offset: nonzero_vec2(collider.offset.x, collider.offset.y),
        origin: nonzero_vec2(collider.origin.x, collider.origin.y),
    }
}

/// Returns `None` for a zero vector (the engine's canonical serialization of `(0, 0)`),
/// otherwise `Some([x, y])`. Safe for exact zero comparisons — these values are
/// default-initialized or stored without intermediate arithmetic.
fn nonzero_vec2(x: f32, y: f32) -> Option<[f32; 2]> {
    if x == 0.0 && y == 0.0 {
        None
    } else {
        Some([x, y])
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

/// Looks up a texture entry in `MapData.textures` by key.
pub fn find_texture<'a>(map_data: &'a MapData, key: &str) -> Option<&'a TextureEntry> {
    map_data.textures.iter().find(|e| e.key == key)
}

/// Looks up a mutable texture entry in `MapData.textures` by key.
pub fn find_texture_mut<'a>(map_data: &'a mut MapData, key: &str) -> Option<&'a mut TextureEntry> {
    map_data.textures.iter_mut().find(|e| e.key == key)
}

/// Returns the alphabetically-first texture key in `MapData.textures`, used as a sane
/// default when adding a new `Sprite` component.
pub fn default_texture_key(map_data: &MapData) -> Option<&str> {
    map_data.textures.iter().map(|e| e.key.as_str()).min()
}

/// Looks up a font entry in `MapData.fonts` by key.
pub fn find_font<'a>(map_data: &'a MapData, key: &str) -> Option<&'a FontEntry> {
    map_data.fonts.iter().find(|e| e.key == key)
}

/// Looks up a mutable font entry in `MapData.fonts` by key.
pub fn find_font_mut<'a>(map_data: &'a mut MapData, key: &str) -> Option<&'a mut FontEntry> {
    map_data.fonts.iter_mut().find(|e| e.key == key)
}

/// Returns the alphabetically-first font key in `MapData.fonts`, used as a sane default
/// when adding a new `DynamicText` component.
pub fn default_font_key(map_data: &MapData) -> Option<&str> {
    map_data.fonts.iter().map(|e| e.key.as_str()).min()
}
