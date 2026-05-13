//! Entity selection: hit-testing, rectangle requests, group selection, registry-key selection.
//!
//! Five observers handle different selection sources:
//! - `entity_pick_observer` — spatial hit-test at a world-space click point (left mouse click).
//! - `entity_rect_pick_observer` — receives a world-space rectangle selection request and routes
//!   the result set into single- or multi-selection UI.
//! - `select_group_observer` — selects all entities in a named group.
//! - `select_registered_entity_observer` — selects an entity by its `WorldSignals` key.
//! - `select_entity_observer` — resolves a row index from the selector panel into an entity.
//!
//! Results are stored in `RenderableSelectorMutex` (an `AppState` Mutex cache). The GUI reads
//! this cache to render the entity selector panel. A single entity auto-selected on a non-empty
//! pick triggers `InspectEntityRequested` to populate the entity editor.
use super::entity_inspector::InspectEntityRequested;
use super::group_selector::{GroupListCache, GroupListMutex};
use super::utils::{display_group_name, entity_label};
use crate::components::map_entity::MapEntity;
use crate::editor_types::{ComponentSnapshot, SelectionCorners};
use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query, ResMut, With};
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::dynamictext::DynamicText;
use aberredengine::components::globaltransform2d::GlobalTransform2D;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::persistent::Persistent;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::prelude::Vector2;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::render::geometry::{compute_sprite_geometry, resolve_world_transform};
use log::{debug, warn};

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Hit-test all renderable `MapEntity` entities at world-space `(x, y)`.
///
/// Results are sorted topmost-first (by `ZIndex` descending, then entity index). The topmost
/// hit is auto-selected; all hits are written to `RenderableSelectorMutex` for the panel.
#[derive(Event)]
pub struct PickEntitiesAtPointRequested {
    pub x: f32,
    pub y: f32,
}

/// Request hit-testing all renderable `MapEntity` entities touched by a world-space rectangle.
///
/// Handled by `entity_rect_pick_observer`, which tests each entity's visual bounds (sprite quad,
/// collider AABB, or dynamic-text rect) against the rectangle via SAT. Multiple hits open the
/// multi-entity selector; a single hit behaves like a click pick.
#[derive(Event)]
pub struct PickEntitiesInRectRequested {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Resolve `index` from the selector panel's hit list into a full entity selection.
#[derive(Event)]
pub struct SelectEntityRequested {
    pub index: usize,
}

/// Select all entities whose `Group` component matches `group`.
#[derive(Event)]
pub struct SelectGroupRequested {
    pub group: String,
}

/// Select the entity registered under `key` in `WorldSignals.entities`.
#[derive(Event)]
pub struct SelectRegisteredEntityRequested {
    pub key: String,
}

// ---------------------------------------------------------------------------
// Cache resource
// ---------------------------------------------------------------------------

/// Describes how the current selector result set was produced.
#[derive(Clone, Default)]
pub enum SelectorSource {
    #[default]
    None,
    /// Result of a spatial click at `(x, y)` in world space.
    Click { x: f32, y: f32 },
    /// Result of a rectangle selection in world space.
    Rectangle {
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    },
    /// Result of selecting all entities in a named group.
    Group { display_name: String },
    /// Result of selecting a single registered entity by key.
    Registry { key: String },
}

/// Cached hit list from the most recent entity selection operation.
///
/// Written by selection observers; read by `draw_entity_selector`. Stored in `AppState` as
/// [`RenderableSelectorMutex`]. Indices in all `Vec` fields are aligned — `hits[i]`,
/// `labels[i]`, `z_indices[i]`, and `corner_sets[i]` all describe the same entity.
#[derive(Default)]
pub struct RenderableSelectorCache {
    pub hits: Vec<Entity>,
    pub labels: Vec<String>,
    pub z_indices: Vec<f32>,
    /// World-space corners for each hit: TL, TR, BR, BL (clockwise). `None` if the entity has
    /// no pickable visual bounds (no Sprite, BoxCollider, or DynamicText).
    pub corner_sets: Vec<Option<[[f32; 2]; 4]>>,
    pub source: SelectorSource,
}

/// `AppState` key for the selector hit-list cache. Acquired via `app_state.get::<RenderableSelectorMutex>()`.
pub type RenderableSelectorMutex = std::sync::Mutex<RenderableSelectorCache>;

/// Cached multi-selection result set for the dedicated multi-entity UI.
#[derive(Default)]
pub struct MultiEntitySelectionCache {
    pub hits: Vec<Entity>,
    pub labels: Vec<String>,
    /// World-space corners for each hit (parallel to `hits`): TL, TR, BR, BL.
    /// `None` if the entity has no pickable visual bounds.
    pub corner_sets: Vec<Option<[[f32; 2]; 4]>>,
    pub source: SelectorSource,
    pub bulk_edit: MultiEntityBulkEditState,
}

/// `AppState` key for the multi-selection result cache.
pub type MultiEntitySelectionMutex = std::sync::Mutex<MultiEntitySelectionCache>;

/// Transient modal buffers and pending apply requests for multi-selection bulk edits.
#[derive(Default)]
pub struct MultiEntityBulkEditState {
    pub move_dx: f32,
    pub move_dy: f32,
    pub pending_move_request: Option<[f32; 2]>,
    pub z_delta: f32,
    pub pending_z_request: Option<f32>,
}

impl MultiEntityBulkEditState {
    pub fn reset_move_buffer(&mut self) {
        self.move_dx = 0.0;
        self.move_dy = 0.0;
    }

    pub fn reset_z_buffer(&mut self) {
        self.z_delta = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Pick observer — internal types
// ---------------------------------------------------------------------------

struct PickResult {
    entity: Entity,
    label: String,
    zindex: f32,
    corners: Option<[[f32; 2]; 4]>,
}

// ---------------------------------------------------------------------------
// Pick observer
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
pub fn entity_pick_observer(
    trigger: On<PickEntitiesAtPointRequested>,
    query: Query<
        (
            Entity,
            &MapPosition,
            Option<&BoxCollider>,
            Option<&Sprite>,
            Option<&DynamicText>,
            Option<&Rotation>,
            Option<&Scale>,
            Option<&ZIndex>,
            Option<&GlobalTransform2D>,
            Option<&Group>,
            Option<&Persistent>,
        ),
        With<MapEntity>,
    >,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut commands: Commands,
) {
    let click_x = trigger.event().x;
    let click_y = trigger.event().y;
    let click = Vector2 {
        x: click_x,
        y: click_y,
    };

    let mut hits: Vec<PickResult> = Vec::new();

    for (
        entity,
        pos,
        maybe_collider,
        maybe_sprite,
        maybe_dynamic_text,
        maybe_rot,
        maybe_scale,
        maybe_zindex,
        maybe_gt,
        maybe_group,
        maybe_persistent,
    ) in query.iter()
    {
        let (resolved_pos, resolved_scale, resolved_rot) = resolve_world_transform(
            *pos,
            maybe_scale.copied(),
            maybe_rot.copied(),
            maybe_gt.copied(),
        );

        let hit = if let Some(collider) = maybe_collider {
            // BoxCollider takes priority — axis-aligned, ignores sprite rotation
            collider.contains_point(resolved_pos.pos, click)
        } else if let Some(sprite) = maybe_sprite {
            point_in_sprite(
                click,
                &resolved_pos,
                sprite,
                resolved_scale.as_ref(),
                resolved_rot.as_ref(),
            )
        } else if let Some(dynamic_text) = maybe_dynamic_text {
            let hit = point_in_dynamic_text(click, &resolved_pos, dynamic_text);
            if hit {
                let size = dynamic_text.size();
                debug!(
                    "entity_pick_observer: DynamicText hit entity {} click=({:.2}, {:.2}) rect=({:.2}, {:.2}, {:.2}, {:.2}) text='{}'",
                    entity.to_bits(),
                    click.x,
                    click.y,
                    resolved_pos.pos.x,
                    resolved_pos.pos.y,
                    size.x,
                    size.y,
                    dynamic_text.text,
                );
            }
            hit
        } else {
            false
        };

        if hit {
            let zindex = maybe_zindex.map_or(0.0, |z| z.0);
            let label = entity_label(entity, maybe_group, maybe_persistent);
            let corners = compute_corners(
                &resolved_pos,
                maybe_collider,
                maybe_sprite,
                maybe_dynamic_text,
                resolved_scale.as_ref(),
                resolved_rot.as_ref(),
            );
            hits.push(PickResult {
                entity,
                label,
                zindex,
                corners: Some(corners),
            });
        }
    }

    // Sort topmost-first: higher ZIndex = rendered last = visually on top
    hits.sort_by(|a, b| {
        b.zindex
            .partial_cmp(&a.zindex)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.entity.index().cmp(&b.entity.index()))
    });

    apply_single_selection_results(
        hits,
        SelectorSource::Click {
            x: click_x,
            y: click_y,
        },
        &mut world_signals,
        &mut app_state,
        &mut commands,
    );
}

#[allow(clippy::type_complexity)]
pub fn entity_rect_pick_observer(
    trigger: On<PickEntitiesInRectRequested>,
    query: Query<
        (
            Entity,
            &MapPosition,
            Option<&BoxCollider>,
            Option<&Sprite>,
            Option<&DynamicText>,
            Option<&Rotation>,
            Option<&Scale>,
            Option<&ZIndex>,
            Option<&GlobalTransform2D>,
            Option<&Group>,
            Option<&Persistent>,
        ),
        With<MapEntity>,
    >,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    let mut hits: Vec<PickResult> = Vec::new();
    for (
        entity,
        pos,
        maybe_collider,
        maybe_sprite,
        maybe_dynamic_text,
        maybe_rot,
        maybe_scale,
        maybe_zindex,
        maybe_gt,
        maybe_group,
        maybe_persistent,
    ) in query.iter()
    {
        let corners = compute_group_corners(
            Some(pos),
            maybe_collider,
            maybe_sprite,
            maybe_dynamic_text,
            maybe_scale,
            maybe_rot,
            maybe_gt,
        );

        let included = match corners {
            Some(c) => quad_overlaps_rect(c, ev.min_x, ev.min_y, ev.max_x, ev.max_y),
            // No visual bounds — fall back to origin-point test.
            None => {
                pos.pos.x >= ev.min_x
                    && pos.pos.x <= ev.max_x
                    && pos.pos.y >= ev.min_y
                    && pos.pos.y <= ev.max_y
            }
        };
        if !included {
            continue;
        }

        hits.push(PickResult {
            entity,
            label: entity_label(entity, maybe_group, maybe_persistent),
            zindex: maybe_zindex.map_or(0.0, |z| z.0),
            corners,
        });
    }
    hits.sort_by(|a, b| {
        a.label
            .cmp(&b.label)
            .then_with(|| a.entity.index().cmp(&b.entity.index()))
    });

    let source = SelectorSource::Rectangle {
        min_x: ev.min_x,
        min_y: ev.min_y,
        max_x: ev.max_x,
        max_y: ev.max_y,
    };
    if hits.len() > 1 {
        apply_multi_selection_results(hits, source, &mut world_signals, &mut app_state);
    } else {
        apply_single_selection_results(
            hits,
            source,
            &mut world_signals,
            &mut app_state,
            &mut commands,
        );
    }
}

#[allow(clippy::type_complexity)]
pub fn select_group_observer(
    trigger: On<SelectGroupRequested>,
    query: Query<(
        Entity,
        Option<&MapPosition>,
        Option<&BoxCollider>,
        Option<&Sprite>,
        Option<&DynamicText>,
        Option<&Rotation>,
        Option<&Scale>,
        Option<&ZIndex>,
        Option<&GlobalTransform2D>,
        &Group,
        Option<&Persistent>,
    )>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut commands: Commands,
) {
    let selected_group = trigger.event().group.as_str();
    let mut hits: Vec<PickResult> = Vec::new();

    for (
        entity,
        maybe_pos,
        maybe_collider,
        maybe_sprite,
        maybe_dynamic_text,
        maybe_rot,
        maybe_scale,
        maybe_zindex,
        maybe_gt,
        group,
        maybe_persistent,
    ) in query.iter()
    {
        if group.name() != selected_group {
            continue;
        }

        let zindex = maybe_zindex.map_or(0.0, |z| z.0);
        hits.push(PickResult {
            entity,
            label: entity_label(entity, Some(group), maybe_persistent),
            zindex,
            corners: compute_group_corners(
                maybe_pos,
                maybe_collider,
                maybe_sprite,
                maybe_dynamic_text,
                maybe_scale,
                maybe_rot,
                maybe_gt,
            ),
        });
    }

    hits.sort_by(|a, b| {
        a.label
            .cmp(&b.label)
            .then_with(|| a.entity.index().cmp(&b.entity.index()))
    });

    apply_single_selection_results(
        hits,
        SelectorSource::Group {
            display_name: display_group_name(&trigger.event().group).to_owned(),
        },
        &mut world_signals,
        &mut app_state,
        &mut commands,
    );
}

#[allow(clippy::type_complexity)]
pub fn select_registered_entity_observer(
    trigger: On<SelectRegisteredEntityRequested>,
    query: Query<(
        Option<&MapPosition>,
        Option<&BoxCollider>,
        Option<&Sprite>,
        Option<&DynamicText>,
        Option<&Rotation>,
        Option<&Scale>,
        Option<&ZIndex>,
        Option<&GlobalTransform2D>,
        Option<&Group>,
        Option<&Persistent>,
    )>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut commands: Commands,
) {
    let key = trigger.event().key.as_str();
    let Some(entity) = world_signals.entities.get(key).copied() else {
        warn!(
            "select_registered_entity_observer: key '{}' not found in WorldSignals.entities",
            key
        );
        return;
    };

    let Ok((
        maybe_pos,
        maybe_collider,
        maybe_sprite,
        maybe_dynamic_text,
        maybe_rot,
        maybe_scale,
        maybe_zindex,
        maybe_gt,
        maybe_group,
        maybe_persistent,
    )) = query.get(entity)
    else {
        world_signals.remove_entity(key);
        warn!(
            "select_registered_entity_observer: entity {} for key '{}' is unavailable; removed stale registration",
            entity.to_bits(),
            key
        );
        return;
    };

    let label = entity_label(entity, maybe_group, maybe_persistent);
    let zindex = maybe_zindex.map_or(0.0, |z| z.0);
    let corners = compute_group_corners(
        maybe_pos,
        maybe_collider,
        maybe_sprite,
        maybe_dynamic_text,
        maybe_scale,
        maybe_rot,
        maybe_gt,
    );

    apply_single_selection_results(
        vec![PickResult {
            entity,
            label,
            zindex,
            corners,
        }],
        SelectorSource::Registry {
            key: key.to_owned(),
        },
        &mut world_signals,
        &mut app_state,
        &mut commands,
    );
}

// ---------------------------------------------------------------------------
// Selection resolve observer
// ---------------------------------------------------------------------------

pub fn select_entity_observer(
    trigger: On<SelectEntityRequested>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
    mut commands: Commands,
) {
    let index = trigger.event().index;
    let hit = {
        let mutex = app_state
            .get::<RenderableSelectorMutex>()
            .expect("RenderableSelectorMutex not in AppState");
        let cache = mutex.lock().unwrap();
        cache
            .hits
            .get(index)
            .map(|&entity| {
                let label = cache.labels.get(index).cloned();
                let corners = cache.corner_sets.get(index).copied().flatten();
                (entity, label, corners)
            })
            .ok_or(cache.hits.len())
    };
    match hit {
        Ok((entity, label, corners)) => {
            if let Some(label) = label {
                apply_selection(
                    entity,
                    &label,
                    corners,
                    &mut world_signals,
                    &mut app_state,
                    &mut commands,
                );
            }
        }
        Err(cache_len) => {
            warn!(
                "select_entity_observer: index {} out of range (cache has {} hits)",
                index, cache_len
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: compute world-space quad corners for selection outline
// ---------------------------------------------------------------------------

/// Returns the 4 world-space corners of the entity's pickable bounds in order
/// TL → TR → BR → BL (clockwise, matching Raylib's rotation convention).
///
/// - BoxCollider present: axis-aligned AABB, no rotation.
/// - Sprite only: rotated quad using the same anchor/origin math as the renderer.
/// - Neither: degenerate zero-size quad at the entity's world position.
fn compute_corners(
    pos: &MapPosition,
    maybe_collider: Option<&BoxCollider>,
    maybe_sprite: Option<&Sprite>,
    maybe_dynamic_text: Option<&DynamicText>,
    scale: Option<&Scale>,
    rot: Option<&Rotation>,
) -> [[f32; 2]; 4] {
    if let Some(collider) = maybe_collider {
        let (min, max) = collider.aabb(pos.pos);
        [
            [min.x, min.y],
            [max.x, min.y],
            [max.x, max.y],
            [min.x, max.y],
        ]
    } else if let Some(sprite) = maybe_sprite {
        let geom = compute_sprite_geometry(pos, sprite, scale, rot);
        let angle = geom.rotation.to_radians();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let ax = geom.dest.x;
        let ay = geom.dest.y;
        let w = geom.dest.width;
        let h = geom.dest.height;
        let ox = geom.origin.x;
        let oy = geom.origin.y;
        // Local offsets relative to anchor (TL, TR, BR, BL)
        let locals: [(f32, f32); 4] = [(-ox, -oy), (w - ox, -oy), (w - ox, h - oy), (-ox, h - oy)];
        let mut corners = [[0.0f32; 2]; 4];
        for (i, (lx, ly)) in locals.iter().enumerate() {
            corners[i] = [ax + lx * cos_a - ly * sin_a, ay + lx * sin_a + ly * cos_a];
        }
        corners
    } else if let Some(dynamic_text) = maybe_dynamic_text {
        let p = pos.pos;
        let size = dynamic_text.size();
        [
            [p.x, p.y],
            [p.x + size.x, p.y],
            [p.x + size.x, p.y + size.y],
            [p.x, p.y + size.y],
        ]
    } else {
        // No pickable bounds — degenerate quad
        let p = pos.pos;
        [[p.x, p.y]; 4]
    }
}

// ---------------------------------------------------------------------------
// Helper: point-in-sprite with rotation
// ---------------------------------------------------------------------------

/// Returns `true` if `click` falls inside the sprite's visible bounds, accounting
/// for scale and rotation (same transform math as the renderer).
fn point_in_sprite(
    click: Vector2,
    pos: &MapPosition,
    sprite: &Sprite,
    scale: Option<&Scale>,
    rot: Option<&Rotation>,
) -> bool {
    let geom = compute_sprite_geometry(pos, sprite, scale, rot);

    if geom.rotation.abs() < f32::EPSILON {
        // Axis-aligned: direct AABB test
        let left = geom.dest.x - geom.origin.x;
        let top = geom.dest.y - geom.origin.y;
        click.x >= left
            && click.x <= left + geom.dest.width
            && click.y >= top
            && click.y <= top + geom.dest.height
    } else {
        // Rotated: inverse-transform click into sprite-local space, then AABB test
        let dx = click.x - geom.dest.x;
        let dy = click.y - geom.dest.y;
        let angle = (-geom.rotation).to_radians();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let local_x = dx * cos_a - dy * sin_a;
        let local_y = dx * sin_a + dy * cos_a;
        local_x >= -geom.origin.x
            && local_x <= geom.dest.width - geom.origin.x
            && local_y >= -geom.origin.y
            && local_y <= geom.dest.height - geom.origin.y
    }
}

fn point_in_dynamic_text(click: Vector2, pos: &MapPosition, dynamic_text: &DynamicText) -> bool {
    let size = dynamic_text.size();
    click.x >= pos.pos.x
        && click.x <= pos.pos.x + size.x
        && click.y >= pos.pos.y
        && click.y <= pos.pos.y + size.y
}

fn compute_group_corners(
    maybe_pos: Option<&MapPosition>,
    maybe_collider: Option<&BoxCollider>,
    maybe_sprite: Option<&Sprite>,
    maybe_dynamic_text: Option<&DynamicText>,
    maybe_scale: Option<&Scale>,
    maybe_rot: Option<&Rotation>,
    maybe_gt: Option<&GlobalTransform2D>,
) -> Option<[[f32; 2]; 4]> {
    let pos = maybe_pos?;
    if maybe_collider.is_none() && maybe_sprite.is_none() && maybe_dynamic_text.is_none() {
        return None;
    }

    let (resolved_pos, resolved_scale, resolved_rot) = resolve_world_transform(
        *pos,
        maybe_scale.copied(),
        maybe_rot.copied(),
        maybe_gt.copied(),
    );
    Some(compute_corners(
        &resolved_pos,
        maybe_collider,
        maybe_sprite,
        maybe_dynamic_text,
        resolved_scale.as_ref(),
        resolved_rot.as_ref(),
    ))
}

fn populate_selector_cache(
    cache: &mut RenderableSelectorCache,
    hits: Vec<PickResult>,
    source: SelectorSource,
) {
    cache.hits.clear();
    cache.labels.clear();
    cache.z_indices.clear();
    cache.corner_sets.clear();
    for hit in hits {
        cache.hits.push(hit.entity);
        cache.labels.push(hit.label);
        cache.z_indices.push(hit.zindex);
        cache.corner_sets.push(hit.corners);
    }
    cache.source = source;
}

fn populate_multi_selection_cache(
    cache: &mut MultiEntitySelectionCache,
    hits: Vec<PickResult>,
    source: SelectorSource,
) {
    cache.hits.clear();
    cache.labels.clear();
    cache.corner_sets.clear();
    cache.bulk_edit = MultiEntityBulkEditState::default();
    for hit in hits {
        cache.hits.push(hit.entity);
        cache.labels.push(hit.label);
        cache.corner_sets.push(hit.corners);
    }
    cache.source = source;
}

fn apply_single_selection_results(
    hits: Vec<PickResult>,
    source: SelectorSource,
    world_signals: &mut WorldSignals,
    app_state: &mut AppState,
    commands: &mut Commands,
) {
    clear_multi_selection_state(world_signals, app_state);
    let top_hit = {
        let mutex = app_state
            .get::<RenderableSelectorMutex>()
            .expect("RenderableSelectorMutex not in AppState");
        let mut cache = mutex.lock().unwrap();
        populate_selector_cache(&mut cache, hits, source);
        cache
            .hits
            .first()
            .map(|&entity| (entity, cache.labels[0].clone(), cache.corner_sets[0]))
    };

    world_signals.set_flag(sig::UI_ENTITY_SELECTOR_OPEN);
    if let Some((top, top_label, top_corners)) = top_hit {
        apply_selection(
            top,
            &top_label,
            top_corners,
            world_signals,
            app_state,
            commands,
        );
    } else {
        clear_active_selection(world_signals, app_state);
    }
}

fn apply_multi_selection_results(
    hits: Vec<PickResult>,
    source: SelectorSource,
    world_signals: &mut WorldSignals,
    app_state: &mut AppState,
) {
    clear_active_selection(world_signals, app_state);
    world_signals.clear_flag(sig::UI_ENTITY_SELECTOR_OPEN);
    world_signals.clear_integer(sig::ES_SELECTED_ROW);
    let mutex = app_state
        .get::<MultiEntitySelectionMutex>()
        .expect("MultiEntitySelectionMutex not in AppState");
    let mut cache = mutex.lock().unwrap();
    populate_multi_selection_cache(&mut cache, hits, source);
    world_signals.set_flag(sig::UI_MULTI_ENTITY_SELECTOR_OPEN);
}

fn clear_multi_selection_state(world_signals: &mut WorldSignals, app_state: &mut AppState) {
    world_signals.clear_flag(sig::UI_MULTI_ENTITY_SELECTOR_OPEN);
    if let Some(mutex) = app_state.get::<MultiEntitySelectionMutex>() {
        *mutex.lock().unwrap() = MultiEntitySelectionCache::default();
    }
}

fn clear_active_selection(world_signals: &mut WorldSignals, app_state: &mut AppState) {
    clear_multi_selection_state(world_signals, app_state);
    world_signals.remove_entity(sig::ES_SELECTED_ENTITY);
    world_signals.remove_string(sig::ES_SELECTED_LABEL);
    world_signals.clear_flag(sig::UI_ENTITY_EDITOR_OPEN);
    app_state.remove::<SelectionCorners>();
    app_state.remove::<ComponentSnapshot>();
}

pub(crate) fn apply_selection(
    entity: Entity,
    label: &str,
    corners: Option<[[f32; 2]; 4]>,
    world_signals: &mut WorldSignals,
    app_state: &mut AppState,
    commands: &mut Commands,
) {
    clear_multi_selection_state(world_signals, app_state);
    world_signals.set_entity(sig::ES_SELECTED_ENTITY, entity);
    world_signals.set_string(sig::ES_SELECTED_LABEL, label);
    app_state.remove::<ComponentSnapshot>();
    if let Some(corners) = corners {
        app_state.insert(SelectionCorners(corners));
    } else {
        app_state.remove::<SelectionCorners>();
    }
    commands.trigger(InspectEntityRequested { entity });
}

// ---------------------------------------------------------------------------
// Lifecycle cleanup helpers
// ---------------------------------------------------------------------------

/// Clear WorldSignals keys, AppState entries, and the selector cache.
/// Call on new-map or load-map operations.
pub fn clear_selector_state(world_signals: &mut WorldSignals, app_state: &mut AppState) {
    if let Some(m) = app_state.get::<RenderableSelectorMutex>() {
        *m.lock().unwrap() = RenderableSelectorCache::default();
    }
    if let Some(m) = app_state.get::<MultiEntitySelectionMutex>() {
        *m.lock().unwrap() = MultiEntitySelectionCache::default();
    }
    if let Some(m) = app_state.get::<GroupListMutex>() {
        *m.lock().unwrap() = GroupListCache::default();
    }
    world_signals.clear_integer(sig::ES_SELECTED_ROW);
    world_signals.remove_string(sig::ENTITY_REGISTRY_SELECTED_KEY);
    world_signals.remove_string(sig::GROUPS_SELECTED_GROUP);
    clear_active_selection(world_signals, app_state);
}

/// SAT overlap test for a world-space quad (4 corners, TL→TR→BR→BL) against an AABB.
/// Returns `true` if any part of the quad overlaps the rectangle [min_x, max_x] × [min_y, max_y].
fn quad_overlaps_rect(
    corners: [[f32; 2]; 4],
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
) -> bool {
    // Quick rejection: test AABB of the OBB against the selection rect (equivalent to projecting
    // both shapes onto the world X and Y axes — the first two SAT axes for an AABB opponent).
    let (obb_min_x, obb_max_x, obb_min_y, obb_max_y) = corners.iter().fold(
        (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ),
        |(mnx, mxx, mny, mxy), c| (mnx.min(c[0]), mxx.max(c[0]), mny.min(c[1]), mxy.max(c[1])),
    );
    if obb_max_x < min_x || obb_min_x > max_x || obb_max_y < min_y || obb_min_y > max_y {
        return false;
    }

    // Remaining two SAT axes: the OBB's own edge normals (2 unique directions for a rectangle).
    let rect_corners = [
        [min_x, min_y],
        [max_x, min_y],
        [max_x, max_y],
        [min_x, max_y],
    ];
    for i in 0..2 {
        let dx = corners[i + 1][0] - corners[i][0];
        let dy = corners[i + 1][1] - corners[i][1];
        let (nx, ny) = (-dy, dx); // perpendicular edge normal

        let proj = |c: &[f32; 2]| c[0] * nx + c[1] * ny;
        let (obb_min, obb_max) = corners
            .iter()
            .map(proj)
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(mn, mx), v| {
                (mn.min(v), mx.max(v))
            });
        let (rect_min, rect_max) = rect_corners
            .iter()
            .map(proj)
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(mn, mx), v| {
                (mn.min(v), mx.max(v))
            });
        if obb_max < rect_min || obb_min > rect_max {
            return false;
        }
    }

    true
}
