//! Shared payload types that cross the ECS ↔ GUI boundary.
//!
//! These types cannot query ECS directly inside the GUI callback, so data is serialised into
//! plain Rust structs here and stored in `AppState` for the GUI to read. See `patterns.md §5`
//! (ComponentSnapshot serialization) for the full flow.

// ---------------------------------------------------------------------------
// Component kind selector (for Add-component feature)
// ---------------------------------------------------------------------------

/// Discriminant for the "Add Component" dropdown in the entity inspector.
///
/// Each variant maps to one engine component type. Add a variant here when introducing a new
/// editable component; the rest of the wiring is described in `docs/recipes/add-component.md`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ComponentKind {
    MapPosition,
    ZIndex,
    Group,
    Rotation,
    Scale,
    Sprite,
    BoxCollider,
    Animation,
    Ttl,
    Persistent,
    Tint,
    LuaSetup,
    DynamicText,
}

// ---------------------------------------------------------------------------
// Entity selector payloads
// ---------------------------------------------------------------------------

/// World-space quad corners for the active entity selection outline: TL → TR → BR → BL.
///
/// Stored in `AppState` by `entity_selector` observers; consumed by `overlay::draw_selection_outline`
/// to project the quad to screen space and render it as a yellow border.
#[derive(Debug, Clone, Copy)]
pub struct SelectionCorners(pub [[f32; 2]; 4]);

// ---------------------------------------------------------------------------
// Entity inspector payloads
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SpriteSnapshot {
    pub tex_key: String,
    pub width: f32,
    pub height: f32,
    /// Draw offset from the entity's world position, in world units.
    pub offset: [f32; 2],
    /// Rotation/scale pivot, in sprite-local pixels from the top-left corner.
    pub origin: [f32; 2],
    pub flip_h: bool,
    pub flip_v: bool,
}

#[derive(Clone)]
pub struct ColliderSnapshot {
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub origin: [f32; 2],
}

/// Snapshot of an `Animation` component.
#[derive(Clone)]
pub struct AnimationSnapshot {
    /// Key into `AnimationStore` identifying the animation clip.
    pub animation_key: String,
}

#[derive(Clone)]
pub struct TtlSnapshot {
    pub remaining: f32,
}

#[derive(Clone)]
pub struct TimerSnapshot {
    pub duration: f32,
    pub elapsed: f32,
}

/// Snapshot of a `Tint` component (RGBA u8).
#[derive(Clone)]
pub struct TintSnapshot {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl TintSnapshot {
    /// Returns the RGBA channels normalised to `[0.0, 1.0]` for use with ImGui colour editors.
    pub fn color_normalized(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

#[derive(Clone)]
pub struct DynamicTextSnapshot {
    pub text: String,
    /// Key into `FontStore`.
    pub font_key: String,
    pub font_size: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl DynamicTextSnapshot {
    /// Returns the RGBA channels normalised to `[0.0, 1.0]` for use with ImGui colour editors.
    pub fn color_normalized(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

/// Snapshot of a `Phase` state-machine component.
#[derive(Clone)]
pub struct PhaseSnapshot {
    pub current: String,
    pub previous: Option<String>,
    pub next: Option<String>,
    pub time_in_phase: f32,
    /// Sorted list of all defined phase names (for display in the inspector).
    pub phase_names: Vec<String>,
}

/// Complete snapshot of an entity's components at a single point in time.
///
/// Built by `entity_inspect_observer` in response to `InspectEntityRequested` and stored in
/// `AppState`. GUI panels read this each frame.
///
/// `Option<T>` fields: `None` means the entity does not have that component; `Some(v)` means it
/// does. `bool` flags (`persistent`) are always present.
#[derive(Clone)]
pub struct ComponentSnapshot {
    /// `Entity::to_bits()` — stored as `u64` because `Entity` cannot safely cross the
    /// `AppState` boundary (different ECS worlds, potential pointer issues).
    /// Reconstruct with `Entity::from_bits(snapshot.entity_bits)`.
    pub entity_bits: u64,
    /// `WorldSignals.entities` keys whose value equals this entity (user-visible names).
    pub world_signal_keys: Vec<String>,
    pub map_position: Option<[f32; 2]>,
    pub z_index: Option<f32>,
    pub group: Option<String>,
    pub sprite: Option<SpriteSnapshot>,
    pub box_collider: Option<ColliderSnapshot>,
    pub rotation_deg: Option<f32>,
    pub scale: Option<[f32; 2]>,
    pub animation: Option<AnimationSnapshot>,
    pub ttl: Option<TtlSnapshot>,
    pub timer: Option<TimerSnapshot>,
    pub phase: Option<PhaseSnapshot>,
    pub persistent: bool,
    pub tilemap_path: Option<String>,
    /// `u64` bits because `Entity` cannot cross the `AppState` boundary.
    pub tilemap_parent: Option<u64>,
    pub tint: Option<TintSnapshot>,
    pub lua_setup: Option<String>,
    pub dynamic_text: Option<DynamicTextSnapshot>,
}
