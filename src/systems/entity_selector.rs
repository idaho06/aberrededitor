use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::{Entity, Event, On, Query, Res, ResMut, Resource};
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::globaltransform2d::GlobalTransform2D;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::prelude::Vector2;
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

/// Transient editor resource that holds the real `Entity` handles from the last pick.
///
/// The GUI-facing payload lives in `WorldSignals` as a JSON string and contains only
/// serializable data (entity bits, labels, z-values). This cache keeps the actual
/// `Entity` values, labels, and world-space corner quads so the selection resolve
/// observer can look them up by row index.
#[derive(Resource, Default)]
pub struct EntitySelectorCache {
    pub hits: Vec<Entity>,
    pub labels: Vec<String>,
    /// World-space corners for each hit: TL, TR, BR, BL (clockwise).
    pub corner_sets: Vec<[[f32; 2]; 4]>,
    pub click_pos: (f32, f32),
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
    mut cache: ResMut<EntitySelectorCache>,
    mut world_signals: ResMut<WorldSignals>,
) {
    let click_x = trigger.event().x;
    let click_y = trigger.event().y;
    let click = Vector2 {
        x: click_x,
        y: click_y,
    };

    struct HitEntry {
        entity: Entity,
        label: String,
        zindex: f32,
        corners: [[f32; 2]; 4],
    }

    let mut hits: Vec<HitEntry> = Vec::new();

    for (entity, pos, maybe_collider, maybe_sprite, maybe_rot, maybe_scale, maybe_zindex, maybe_gt, maybe_group) in query.iter() {
        let (resolved_pos, resolved_scale, resolved_rot) =
            resolve_world_transform(*pos, maybe_scale.copied(), maybe_rot.copied(), maybe_gt.copied());

        let hit = if let Some(collider) = maybe_collider {
            // BoxCollider takes priority — axis-aligned, ignores sprite rotation
            collider.contains_point(resolved_pos.pos, click)
        } else if let Some(sprite) = maybe_sprite {
            // Sprite bounds with rotation support
            point_in_sprite(click, &resolved_pos, sprite, resolved_scale.as_ref(), resolved_rot.as_ref())
        } else {
            // No pickable bounds — non-pickable entity
            false
        };

        if hit {
            let zindex = maybe_zindex.map_or(0.0, |z| z.0);
            let group_label = maybe_group.map(|g| format!(" [{}]", g.0)).unwrap_or_default();
            let label = format!("Entity #{}{}", entity.index(), group_label);
            let corners = compute_corners(
                &resolved_pos,
                maybe_collider,
                maybe_sprite,
                resolved_scale.as_ref(),
                resolved_rot.as_ref(),
            );
            hits.push(HitEntry { entity, label, zindex, corners });
        }
    }

    // Sort topmost-first: higher ZIndex = rendered last = visually on top
    hits.sort_by(|a, b| {
        b.zindex
            .partial_cmp(&a.zindex)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.entity.index().cmp(&b.entity.index()))
    });

    // Update cache
    cache.hits = hits.iter().map(|h| h.entity).collect();
    cache.labels = hits.iter().map(|h| h.label.clone()).collect();
    cache.corner_sets = hits.iter().map(|h| h.corners).collect();
    cache.click_pos = (click_x, click_y);

    // Build JSON payload
    let hits_json: Vec<String> = hits
        .iter()
        .map(|h| {
            format!(
                r#"{{"entity_bits":{},"label":{},"zindex":{}}}"#,
                h.entity.to_bits(),
                serde_json::to_string(&h.label).unwrap_or_else(|_| "\"?\"".into()),
                h.zindex
            )
        })
        .collect();

    let payload = format!(
        r#"{{"click":[{},{}],"hits":[{}]}}"#,
        click_x,
        click_y,
        hits_json.join(",")
    );

    world_signals.set_string("gui:entity_selector:payload", payload.as_str());
    world_signals.set_flag("ui:entity_selector:open");

    // Empty click — clear active selection and outline
    if cache.hits.is_empty() {
        world_signals.remove_entity("editor:selected_entity");
        world_signals.remove_string("gui:entity_selector:selection_corners");
    }
}

// ---------------------------------------------------------------------------
// Selection resolve observer
// ---------------------------------------------------------------------------

pub fn select_entity_observer(
    trigger: On<SelectEntityRequested>,
    cache: Res<EntitySelectorCache>,
    mut world_signals: ResMut<WorldSignals>,
) {
    let index = trigger.event().index;
    if let Some(&entity) = cache.hits.get(index) {
        world_signals.set_entity("editor:selected_entity", entity);
        if let Some(label) = cache.labels.get(index) {
            world_signals.set_string("gui:entity_selector:selected_label", label.as_str());
        }
        if let Some(corners) = cache.corner_sets.get(index)
            && let Ok(json) = serde_json::to_string(corners)
        {
            world_signals.set_string("gui:entity_selector:selection_corners", &json);
        }
    } else {
        warn!(
            "select_entity_observer: index {} out of range (cache has {} hits)",
            index,
            cache.hits.len()
        );
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
        let locals: [(f32, f32); 4] = [
            (-ox, -oy),
            (w - ox, -oy),
            (w - ox, h - oy),
            (-ox, h - oy),
        ];
        let mut corners = [[0.0f32; 2]; 4];
        for (i, (lx, ly)) in locals.iter().enumerate() {
            corners[i] = [
                ax + lx * cos_a - ly * sin_a,
                ay + lx * sin_a + ly * cos_a,
            ];
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
// Lifecycle cleanup helper
// ---------------------------------------------------------------------------

/// Clear the WorldSignals keys owned by the entity selector.
/// Use this when only the signals need clearing but the cache is not accessible.
pub fn clear_selector_signals(world_signals: &mut WorldSignals) {
    world_signals.remove_string("gui:entity_selector:payload");
    world_signals.remove_string("gui:entity_selector:selected_label");
    world_signals.remove_string("gui:entity_selector:selection_corners");
    world_signals.remove_entity("editor:selected_entity");
}

/// Clear all entity selector state from WorldSignals and the cache resource.
/// Call on new-map or load-map operations.
pub fn clear_selector_state(world_signals: &mut WorldSignals, cache: &mut EntitySelectorCache) {
    clear_selector_signals(world_signals);
    cache.hits.clear();
    cache.labels.clear();
    cache.corner_sets.clear();
    cache.click_pos = (0.0, 0.0);
}
