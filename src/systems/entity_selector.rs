use super::entity_inspector::InspectEntityRequested;
use crate::editor_types::{ComponentSnapshot, SelectionCorners};
use crate::signals as sig;
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Commands, Entity, Event, On, Query, ResMut};
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::globaltransform2d::GlobalTransform2D;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
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

// ---------------------------------------------------------------------------
// Cache resource
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct RenderableSelectorCache {
    pub hits: Vec<Entity>,
    pub labels: Vec<String>,
    pub z_indices: Vec<f32>,
    /// World-space corners for each hit: TL, TR, BR, BL (clockwise).
    pub corner_sets: Vec<[[f32; 2]; 4]>,
    /// `None` means no pick has happened yet; `Some` holds the click position.
    pub click_pos: Option<(f32, f32)>,
}

pub type RenderableSelectorMutex = std::sync::Mutex<RenderableSelectorCache>;

// ---------------------------------------------------------------------------
// Pick observer — internal types
// ---------------------------------------------------------------------------

struct PickResult {
    entity: Entity,
    label: String,
    zindex: f32,
    corners: [[f32; 2]; 4],
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
            // Sprite bounds with rotation support
            point_in_sprite(
                click,
                &resolved_pos,
                sprite,
                resolved_scale.as_ref(),
                resolved_rot.as_ref(),
            )
        } else {
            // No pickable bounds — non-pickable entity
            false
        };

        if hit {
            let zindex = maybe_zindex.map_or(0.0, |z| z.0);
            let group_label = maybe_group
                .map(|g| format!(" [{}]", g.0))
                .unwrap_or_default();
            let label = format!("Entity #{}{}", entity.index(), group_label);
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
                corners,
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
        cache.hits = hits.iter().map(|h| h.entity).collect();
        cache.labels = hits.iter().map(|h| h.label.clone()).collect();
        cache.z_indices = hits.iter().map(|h| h.zindex).collect();
        cache.corner_sets = hits.iter().map(|h| h.corners).collect();
        cache.click_pos = Some((click_x, click_y));
        let top = cache.hits.first().map(|&e| (e, cache.labels[0].clone(), cache.corner_sets[0]));
        (cache.hits.is_empty(), top)
    };

    world_signals.set_flag(sig::UI_ENTITY_SELECTOR_OPEN);

    // Empty click — clear active selection and outline; otherwise auto-select topmost
    if is_empty {
        world_signals.remove_entity(sig::ES_SELECTED_ENTITY);
        world_signals.remove_string(sig::ES_SELECTED_LABEL);
        world_signals.clear_flag(sig::UI_ENTITY_EDITOR_OPEN);
        app_state.remove::<SelectionCorners>();
        app_state.remove::<ComponentSnapshot>();
    } else if let Some((top, top_label, top_corners)) = top_hit {
        world_signals.set_entity(sig::ES_SELECTED_ENTITY, top);
        world_signals.set_string(sig::ES_SELECTED_LABEL, &top_label);
        app_state.remove::<ComponentSnapshot>();
        app_state.insert(SelectionCorners(top_corners));
        commands.trigger(InspectEntityRequested { entity: top });
    }
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
            let corners = cache.corner_sets.get(index).copied();
            (entity, label, corners)
        }).ok_or(cache.hits.len())
    };
    match hit {
        Ok((entity, label, corners)) => {
            world_signals.set_entity(sig::ES_SELECTED_ENTITY, entity);
            if let Some(label) = label {
                world_signals.set_string(sig::ES_SELECTED_LABEL, label.as_str());
            }
            if let Some(corners) = corners {
                app_state.insert(SelectionCorners(corners));
            }
            commands.trigger(InspectEntityRequested { entity });
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

// ---------------------------------------------------------------------------
// Lifecycle cleanup helpers
// ---------------------------------------------------------------------------

/// Clear WorldSignals keys, AppState entries, and the selector cache.
/// Call on new-map or load-map operations.
pub fn clear_selector_state(world_signals: &mut WorldSignals, app_state: &mut AppState) {
    if let Some(m) = app_state.get::<RenderableSelectorMutex>() {
        *m.lock().unwrap() = RenderableSelectorCache::default();
    }
    app_state.remove::<SelectionCorners>();
    app_state.remove::<ComponentSnapshot>();
    world_signals.remove_string(sig::ES_SELECTED_LABEL);
    world_signals.remove_entity(sig::ES_SELECTED_ENTITY);
    world_signals.clear_flag(sig::UI_ENTITY_EDITOR_OPEN);
}
