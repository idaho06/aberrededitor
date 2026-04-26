use super::entity_inspector::InspectEntityRequested;
use crate::editor_types::{ComponentSnapshot, SelectionCorners};
use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query, ResMut};
use super::utils::{display_group_name, entity_label};
use super::group_selector::{GroupListCache, GroupListMutex};
use aberredengine::components::boxcollider::BoxCollider;
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
use log::warn;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct PickEntitiesAtPointRequested {
    pub x: f32,
    pub y: f32,
}

#[derive(Event)]
pub struct SelectEntityRequested {
    pub index: usize,
}

#[derive(Event)]
pub struct SelectGroupRequested {
    pub group: String,
}

#[derive(Event)]
pub struct SelectRegisteredEntityRequested {
    pub key: String,
}

// ---------------------------------------------------------------------------
// Cache resource
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub enum SelectorSource {
    #[default]
    None,
    Click { x: f32, y: f32 },
    Group { display_name: String },
    Registry { key: String },
}

#[derive(Default)]
pub struct RenderableSelectorCache {
    pub hits: Vec<Entity>,
    pub labels: Vec<String>,
    pub z_indices: Vec<f32>,
    /// World-space corners for each hit: TL, TR, BR, BL (clockwise).
    pub corner_sets: Vec<Option<[[f32; 2]; 4]>>,
    pub source: SelectorSource,
}

pub type RenderableSelectorMutex = std::sync::Mutex<RenderableSelectorCache>;

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
    query: Query<(
        Entity,
        &MapPosition,
        Option<&BoxCollider>,
        Option<&Sprite>,
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

    let (is_empty, top_hit) = {
        let mutex = app_state.get::<RenderableSelectorMutex>().expect("RenderableSelectorMutex not in AppState");
        let mut cache = mutex.lock().unwrap();
        populate_selector_cache(
            &mut cache,
            hits,
            SelectorSource::Click {
                x: click_x,
                y: click_y,
            },
        );
        let top = cache.hits.first().map(|&e| {
            (
                e,
                cache.labels[0].clone(),
                cache.corner_sets[0],
            )
        });
        (cache.hits.is_empty(), top)
    };

    world_signals.set_flag(sig::UI_ENTITY_SELECTOR_OPEN);

    // Empty click — clear active selection and outline; otherwise auto-select topmost
    if is_empty {
        clear_active_selection(&mut world_signals, &mut app_state);
    } else if let Some((top, top_label, top_corners)) = top_hit {
        apply_selection(
            top,
            &top_label,
            top_corners,
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

    let (is_empty, top_hit) = {
        let mutex = app_state
            .get::<RenderableSelectorMutex>()
            .expect("RenderableSelectorMutex not in AppState");
        let mut cache = mutex.lock().unwrap();
        populate_selector_cache(
            &mut cache,
            hits,
            SelectorSource::Group {
                display_name: display_group_name(&trigger.event().group).to_owned(),
            },
        );
        let top = cache.hits.first().map(|&entity| {
            (
                entity,
                cache.labels[0].clone(),
                cache.corner_sets[0],
            )
        });
        (cache.hits.is_empty(), top)
    };

    world_signals.set_flag(sig::UI_ENTITY_SELECTOR_OPEN);

    if is_empty {
        clear_active_selection(&mut world_signals, &mut app_state);
    } else if let Some((top, top_label, top_corners)) = top_hit {
        apply_selection(
            top,
            &top_label,
            top_corners,
            &mut world_signals,
            &mut app_state,
            &mut commands,
        );
    }
}

#[allow(clippy::type_complexity)]
pub fn select_registered_entity_observer(
    trigger: On<SelectRegisteredEntityRequested>,
    query: Query<(
        Option<&MapPosition>,
        Option<&BoxCollider>,
        Option<&Sprite>,
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
        maybe_scale,
        maybe_rot,
        maybe_gt,
    );

    {
        let mutex = app_state
            .get::<RenderableSelectorMutex>()
            .expect("RenderableSelectorMutex not in AppState");
        let mut cache = mutex.lock().unwrap();
        populate_selector_cache(
            &mut cache,
            vec![PickResult {
                entity,
                label: label.clone(),
                zindex,
                corners,
            }],
            SelectorSource::Registry {
                key: key.to_owned(),
            },
        );
    }

    world_signals.set_flag(sig::UI_ENTITY_SELECTOR_OPEN);
    apply_selection(
        entity,
        &label,
        corners,
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
        let mutex = app_state.get::<RenderableSelectorMutex>().expect("RenderableSelectorMutex not in AppState");
        let cache = mutex.lock().unwrap();
        cache.hits.get(index).map(|&entity| {
            let label = cache.labels.get(index).cloned();
            let corners = cache.corner_sets.get(index).copied().flatten();
            (entity, label, corners)
        }).ok_or(cache.hits.len())
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

fn compute_group_corners(
    maybe_pos: Option<&MapPosition>,
    maybe_collider: Option<&BoxCollider>,
    maybe_sprite: Option<&Sprite>,
    maybe_scale: Option<&Scale>,
    maybe_rot: Option<&Rotation>,
    maybe_gt: Option<&GlobalTransform2D>,
) -> Option<[[f32; 2]; 4]> {
    let pos = maybe_pos?;
    if maybe_collider.is_none() && maybe_sprite.is_none() {
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

fn clear_active_selection(world_signals: &mut WorldSignals, app_state: &mut AppState) {
    world_signals.remove_entity(sig::ES_SELECTED_ENTITY);
    world_signals.remove_string(sig::ES_SELECTED_LABEL);
    world_signals.clear_flag(sig::UI_ENTITY_EDITOR_OPEN);
    app_state.remove::<SelectionCorners>();
    app_state.remove::<ComponentSnapshot>();
}

fn apply_selection(
    entity: Entity,
    label: &str,
    corners: Option<[[f32; 2]; 4]>,
    world_signals: &mut WorldSignals,
    app_state: &mut AppState,
    commands: &mut Commands,
) {
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
    if let Some(m) = app_state.get::<GroupListMutex>() {
        *m.lock().unwrap() = GroupListCache::default();
    }
    world_signals.clear_integer(sig::ES_SELECTED_ROW);
    world_signals.remove_string(sig::ENTITY_REGISTRY_SELECTED_KEY);
    world_signals.remove_string(sig::GROUPS_SELECTED_GROUP);
    clear_active_selection(world_signals, app_state);
}
