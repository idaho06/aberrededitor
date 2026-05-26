//! ECS observers for all entity component mutations.
//!
//! Organized by concern:
//! - `transform`  — MapPosition, ZIndex, Rotation, Scale, multi-selection bulk moves.
//! - `visual`     — Sprite, BoxCollider, Tint, DynamicText, Animation, LuaSetup.
//! - `metadata`   — Group, ParticleEmitter, Ttl/Timer/Phase/Persistent removes.
//! - `lifecycle`  — create, clone, remove entity, add-component.
//! - `tilemap`    — remove/bake tilemap.
//! - `registration` — register/unregister entity in WorldSignals.
//!
//! All observers are registered in `main.rs` via `.add_observer()` or the
//! per-subsystem `register(builder)` helper. Every mutation observer ends by
//! re-triggering `InspectEntityRequested` so the GUI snapshot refreshes within
//! the same frame's command queue.
#[macro_use]
mod macros;
pub mod lifecycle;
pub mod metadata;
pub mod registration;
pub mod tilemap;
pub mod transform;
pub mod visual;

use super::entity_inspector::InspectEntityRequested;
use aberredengine::bevy_ecs::prelude::{Commands, Entity};
use aberredengine::resources::worldsignals::WorldSignals;
use log::{debug, warn};

// ── Events ──────────────────────────────────────────────────────────────────

use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::Event;

#[derive(Event)]
pub struct UpdateMapPositionRequested {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
}

#[derive(Event)]
pub struct UpdateZIndexRequested {
    pub entity: Entity,
    pub z_index: f32,
}

#[derive(Event)]
pub struct MoveMultiSelectionRequested {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Event)]
pub struct AdjustMultiSelectionZRequested {
    pub delta: f32,
}

#[derive(Event)]
pub struct UpdateGroupRequested {
    pub entity: Entity,
    pub group: String,
}

#[derive(Event)]
pub struct UpdateRotationRequested {
    pub entity: Entity,
    pub rotation_deg: f32,
}

#[derive(Event)]
pub struct UpdateScaleRequested {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
}

#[derive(Event)]
pub struct UpdateSpriteRequested {
    pub entity: Entity,
    pub tex_key: String,
    pub width: f32,
    pub height: f32,
    pub offset: [f32; 2],
    pub origin: [f32; 2],
    pub flip_h: bool,
    pub flip_v: bool,
}

#[derive(Event)]
pub struct UpdateBoxColliderRequested {
    pub entity: Entity,
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub origin: [f32; 2],
}

/// Set the active animation key on `entity`. Resets frame index and elapsed time.
#[derive(Event)]
pub struct UpdateAnimationRequested {
    pub entity: Entity,
    pub animation_key: String,
}

#[derive(Event)]
pub struct RemoveMapPositionRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveZIndexRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveGroupRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveSpriteRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveBoxColliderRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveRotationRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveScaleRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveAnimationRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveTtlRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveTimerRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemovePhaseRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemovePersistentRequested {
    pub entity: Entity,
}
/// Removes `TileMap`, cleans up `MapData`/`TextureStore` entries, despawns the entity,
/// and clears all its `WorldSignals` registrations.
#[derive(Event)]
pub struct RemoveTileMapRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveTintRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveLuaSetupRequested {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveDynamicTextRequested {
    pub entity: Entity,
}
/// Despawn `entity` and remove all its `WorldSignals` entity registrations.
#[derive(Event)]
pub struct RemoveEntityRequested {
    pub entity: Entity,
}
/// Bake a tilemap root: convert tile children into standalone `MapEntity` entries and despawn
/// the tilemap root. The baked texture entry is added to `MapData.textures`.
#[derive(Event)]
pub struct BakeTilemapRequested {
    pub entity: Entity,
}

#[derive(Event)]
pub struct UpdateTintRequested {
    pub entity: Entity,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Event)]
pub struct UpdateDynamicTextRequested {
    pub entity: Entity,
    pub text: String,
    pub font_key: String,
    pub font_size: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Event)]
pub struct RemoveParticleEmitterRequested {
    pub entity: Entity,
}

#[derive(Event)]
pub struct UpdateParticleEmitterRequested {
    pub entity: Entity,
    pub template_keys: Vec<String>,
    pub shape: crate::editor_types::EmitterShapeKind,
    pub shape_rect_w: f32,
    pub shape_rect_h: f32,
    pub offset: [f32; 2],
    pub particles_per_emission: u32,
    pub emissions_per_second: f32,
    /// `u32::MAX` means unlimited.
    pub emissions_remaining: u32,
    pub arc_min_deg: f32,
    pub arc_max_deg: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    pub ttl_kind: crate::editor_types::TtlKind,
    pub ttl_fixed: f32,
    pub ttl_min: f32,
    pub ttl_max: f32,
}

#[derive(Event)]
pub struct UpdateLuaSetupRequested {
    pub entity: Entity,
    pub callback: String,
}

/// Register `entity` under `key` in `WorldSignals.entities`.
///
/// If `old_key` is `Some`, the old registration is removed first. Triggers
/// `InspectEntityRequested` to refresh the inspector's key list.
#[derive(Event)]
pub struct RegisterEntityRequested {
    pub entity: Entity,
    pub key: String,
    pub old_key: Option<String>,
}

/// Remove the `key → entity` registration from `WorldSignals.entities`.
#[derive(Event)]
pub struct UnregisterEntityRequested {
    pub entity: Entity,
    pub key: String,
}

/// Insert a default-valued component of the given `kind` onto `entity`.
#[derive(Event)]
pub struct AddComponentRequested {
    pub entity: Entity,
    pub kind: crate::editor_types::ComponentKind,
}

/// Spawn a bare `MapEntity` with `MapPosition` at `(x, y)` and select it.
#[derive(Event)]
pub struct CreateBlankEntityRequested {
    pub x: f32,
    pub y: f32,
}

/// Spawn a `MapEntity` with `MapPosition` at `(x, y)` and `BoxCollider` of `(width, height)` and select it.
#[derive(Event)]
pub struct CreateColliderEntityRequested {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Deep-clone `entity` and place the clone at `(x, y)`. Selects the new entity.
#[derive(Event)]
pub struct CloneEntityRequested {
    pub entity: Entity,
    pub x: f32,
    pub y: f32,
}

// ── Shared helpers (used by submodules via `super::`) ───────────────────────

pub(super) fn refresh_inspector(commands: &mut Commands, entity: Entity) {
    commands.trigger(InspectEntityRequested { entity });
}

pub(super) fn remove_entity_registrations(world_signals: &mut WorldSignals, entity: Entity) {
    let keys_to_remove: Vec<String> = world_signals
        .entities
        .iter()
        .filter(|(_, e)| **e == entity)
        .map(|(k, _)| k.clone())
        .collect();
    for key in &keys_to_remove {
        world_signals.remove_entity(key);
    }
    if !keys_to_remove.is_empty() {
        debug!(
            "remove_entity_registrations: removed keys [{}] for entity {}",
            keys_to_remove.join(", "),
            entity.to_bits()
        );
    }
}

pub(super) fn warn_missing_component(observer: &str, entity: Entity, component: &str) {
    warn!(
        "{}: entity {} missing {}",
        observer,
        entity.to_bits(),
        component
    );
}

// ── Per-subsystem register helper (Part C) ───────────────────────────────────

use aberredengine::engine_app::EngineBuilder;

pub fn register(builder: EngineBuilder) -> EngineBuilder {
    builder
        .add_observer(transform::update_map_position_observer)
        .add_observer(transform::update_z_index_observer)
        .add_observer(transform::move_multi_selection_observer)
        .add_observer(transform::adjust_multi_selection_z_observer)
        .add_observer(transform::update_rotation_observer)
        .add_observer(transform::update_scale_observer)
        .add_observer(transform::remove_map_position_observer)
        .add_observer(transform::remove_z_index_observer)
        .add_observer(transform::remove_rotation_observer)
        .add_observer(transform::remove_scale_observer)
        .add_observer(metadata::update_group_observer)
        .add_observer(metadata::remove_group_observer)
        .add_observer(visual::update_sprite_observer)
        .add_observer(visual::remove_sprite_observer)
        .add_observer(visual::update_box_collider_observer)
        .add_observer(visual::remove_box_collider_observer)
        .add_observer(visual::update_animation_observer)
        .add_observer(visual::remove_animation_observer)
        .add_observer(metadata::remove_ttl_observer)
        .add_observer(metadata::remove_timer_observer)
        .add_observer(metadata::remove_phase_observer)
        .add_observer(metadata::remove_persistent_observer)
        .add_observer(visual::update_tint_observer)
        .add_observer(visual::remove_tint_observer)
        .add_observer(visual::update_lua_setup_observer)
        .add_observer(visual::remove_lua_setup_observer)
        .add_observer(visual::update_dynamic_text_observer)
        .add_observer(visual::remove_dynamic_text_observer)
        .add_observer(tilemap::remove_tilemap_observer)
        .add_observer(lifecycle::remove_entity_observer)
        .add_observer(tilemap::bake_tilemap_observer)
        .add_observer(registration::register_entity_observer)
        .add_observer(registration::unregister_entity_observer)
        .add_observer(lifecycle::add_component_observer)
        .add_observer(lifecycle::create_blank_entity_observer)
        .add_observer(lifecycle::create_collider_entity_observer)
        .add_observer(lifecycle::clone_entity_observer)
        .add_observer(metadata::remove_particle_emitter_observer)
        .add_observer(metadata::update_particle_emitter_observer)
}
