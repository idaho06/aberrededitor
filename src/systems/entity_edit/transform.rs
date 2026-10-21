use super::{
    AdjustMultiSelectionZRequested, MoveMultiSelectionRequested, RemoveRotationRequested,
    RemoveScaleRequested, RemoveWorldPositionRequested, RemoveZIndexRequested,
    UpdateRotationRequested, UpdateScaleRequested, UpdateWorldPositionRequested,
    UpdateZIndexRequested,
};
use crate::systems::entity_selector::MultiEntitySelectionMutex;
use aberredengine::prelude::{
    AppState, Commands, On, Query, Res, Rotation, Scale, Vec2, WorldPosition, ZIndex,
};
use log::debug;

component_edit_observer!(
    update_world_position_observer,
    UpdateWorldPositionRequested,
    WorldPosition,
    "WorldPosition",
    |world_position, event, entity| {
        world_position.pos = Vec2::new(event.x, event.y);
        debug!(
            "update_world_position_observer: updated entity {} -> ({:.3}, {:.3})",
            entity.to_bits(),
            event.x,
            event.y
        );
    }
);

component_edit_observer!(
    update_z_index_observer,
    UpdateZIndexRequested,
    ZIndex,
    "ZIndex",
    |z_index, event, entity| {
        z_index.0 = event.z_index;
        debug!(
            "update_z_index_observer: updated entity {} -> {:.3}",
            entity.to_bits(),
            event.z_index
        );
    }
);

component_edit_observer!(
    update_rotation_observer,
    UpdateRotationRequested,
    Rotation,
    "Rotation",
    |rotation, event, entity| {
        rotation.degrees = event.rotation_deg;
        debug!(
            "update_rotation_observer: updated entity {} -> {:.3} deg",
            entity.to_bits(),
            event.rotation_deg
        );
    }
);

component_edit_observer!(
    update_scale_observer,
    UpdateScaleRequested,
    Scale,
    "Scale",
    |scale, event, entity| {
        scale.scale = Vec2::new(event.x, event.y);
        debug!(
            "update_scale_observer: updated entity {} -> ({:.3}, {:.3})",
            entity.to_bits(),
            event.x,
            event.y
        );
    }
);

component_remove_observer!(
    remove_world_position_observer,
    RemoveWorldPositionRequested,
    WorldPosition,
    "WorldPosition"
);
component_remove_observer!(
    remove_z_index_observer,
    RemoveZIndexRequested,
    ZIndex,
    "ZIndex"
);
component_remove_observer!(
    remove_rotation_observer,
    RemoveRotationRequested,
    Rotation,
    "Rotation"
);
component_remove_observer!(remove_scale_observer, RemoveScaleRequested, Scale, "Scale");

pub fn move_multi_selection_observer(
    trigger: On<MoveMultiSelectionRequested>,
    mut positions: Query<&mut WorldPosition>,
    app_state: Res<AppState>,
) {
    let Some(mutex) = app_state.get::<MultiEntitySelectionMutex>() else {
        return;
    };
    let Ok(cache) = mutex.lock() else {
        return;
    };
    let event = trigger.event();

    for &entity in &cache.hits {
        let Ok(mut position) = positions.get_mut(entity) else {
            super::warn_missing_component("move_multi_selection_observer", entity, "WorldPosition");
            continue;
        };
        position.pos.x += event.dx;
        position.pos.y += event.dy;
        debug!(
            "move_multi_selection_observer: moved entity {} by ({:.3}, {:.3})",
            entity.to_bits(),
            event.dx,
            event.dy
        );
    }
}

pub fn adjust_multi_selection_z_observer(
    trigger: On<AdjustMultiSelectionZRequested>,
    mut z_indices: Query<&mut ZIndex>,
    mut commands: Commands,
    app_state: Res<AppState>,
) {
    let Some(mutex) = app_state.get::<MultiEntitySelectionMutex>() else {
        return;
    };
    let Ok(cache) = mutex.lock() else {
        return;
    };
    let event = trigger.event();

    for &entity in &cache.hits {
        if let Ok(mut z_index) = z_indices.get_mut(entity) {
            z_index.0 += event.delta;
        } else if let Ok(mut ec) = commands.get_entity(entity) {
            ec.insert(ZIndex(event.delta));
        } else {
            continue;
        }
        debug!(
            "adjust_multi_selection_z_observer: adjusted entity {} by {:.3}",
            entity.to_bits(),
            event.delta
        );
    }
}
