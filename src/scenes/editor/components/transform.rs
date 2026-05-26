use super::super::widgets::{draw_drag_float_input, draw_float_input, draw_text_buffer_input};
use crate::editor_types::ComponentSnapshot;
use crate::systems::entity_edit::{
    RemoveGroupRequested, RemoveMapPositionRequested, RemoveRotationRequested,
    RemoveScaleRequested, RemoveZIndexRequested, UpdateGroupRequested,
    UpdateMapPositionRequested, UpdateRotationRequested, UpdateScaleRequested,
    UpdateZIndexRequested,
};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::systems::GameCtx;
use log::warn;

#[derive(Default, Clone)]
pub(crate) struct PendingTransform {
    // MapPosition
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
    pub commit_position: bool,
    pub remove_map_position: bool,
    // ZIndex
    pub z_index: Option<f32>,
    pub commit_z: bool,
    pub remove_z: bool,
    // Group
    pub group: Option<String>,
    pub commit_group: bool,
    pub remove_group: bool,
    // Rotation
    pub rotation_deg: Option<f32>,
    pub commit_rotation: bool,
    pub remove_rotation: bool,
    // Scale
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub commit_scale: bool,
    pub remove_scale: bool,
}

impl PendingTransform {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit_position
            || self.remove_map_position
            || self.commit_z
            || self.remove_z
            || self.commit_group
            || self.remove_group
            || self.commit_rotation
            || self.remove_rotation
            || self.commit_scale
            || self.remove_scale
    }
}

pub(crate) fn draw_map_position(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingTransform,
) {
    let Some([pos_x, pos_y]) = snap.map_position else {
        return;
    };
    ui.text("MapPosition");
    ui.same_line();
    if ui.button("Del##map_position") {
        p.remove_map_position = true;
    }
    if let Some(v) =
        draw_drag_float_input(ui, "x##map_position", p.pos_x.unwrap_or(pos_x), 1.0, 0.1)
    {
        p.pos_x = Some(v);
        p.commit_position = true;
    }
    if let Some(v) =
        draw_drag_float_input(ui, "y##map_position", p.pos_y.unwrap_or(pos_y), 1.0, 0.1)
    {
        p.pos_y = Some(v);
        p.commit_position = true;
    }
}

pub(crate) fn draw_z_index(ui: &imgui::Ui, snap: &ComponentSnapshot, p: &mut PendingTransform) {
    let Some(z) = snap.z_index else {
        return;
    };
    ui.separator();
    ui.text("ZIndex");
    ui.same_line();
    if ui.button("Del##zindex") {
        p.remove_z = true;
    }
    if let Some(v) = draw_float_input(ui, "value##zindex", p.z_index.unwrap_or(z), 1.0) {
        p.z_index = Some(v);
        p.commit_z = true;
    }
}

pub(crate) fn draw_group(ui: &imgui::Ui, snap: &ComponentSnapshot, p: &mut PendingTransform) {
    let Some(ref group) = snap.group else {
        return;
    };
    ui.separator();
    ui.text("Group");
    ui.same_line();
    if ui.button("Del##group") {
        p.remove_group = true;
    }
    ui.set_next_item_width(-1.0);
    let mut committed = false;
    draw_text_buffer_input(ui, "name##group", &mut p.group, &mut committed, group);
    if committed {
        p.commit_group = true;
    }
}

pub(crate) fn draw_rotation(ui: &imgui::Ui, snap: &ComponentSnapshot, p: &mut PendingTransform) {
    let Some(rotation_deg) = snap.rotation_deg else {
        return;
    };
    ui.separator();
    ui.text("Rotation");
    ui.same_line();
    if ui.button("Del##rotation") {
        p.remove_rotation = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "degrees##rotation",
        p.rotation_deg.unwrap_or(rotation_deg),
        1.0,
    ) {
        p.rotation_deg = Some(v);
        p.commit_rotation = true;
    }
}

pub(crate) fn draw_scale(ui: &imgui::Ui, snap: &ComponentSnapshot, p: &mut PendingTransform) {
    let Some([scale_x, scale_y]) = snap.scale else {
        return;
    };
    ui.separator();
    ui.text("Scale");
    ui.same_line();
    if ui.button("Del##scale") {
        p.remove_scale = true;
    }
    if let Some(v) = draw_float_input(ui, "x##scale", p.scale_x.unwrap_or(scale_x), 1.0) {
        p.scale_x = Some(v);
        p.commit_scale = true;
    }
    if let Some(v) = draw_float_input(ui, "y##scale", p.scale_y.unwrap_or(scale_y), 1.0) {
        p.scale_y = Some(v);
        p.commit_scale = true;
    }
}

pub(crate) fn commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingTransform,
) {
    if p.remove_map_position {
        ctx.commands.trigger(RemoveMapPositionRequested { entity });
    } else if p.commit_position {
        if let Some([snap_x, snap_y]) = snap.map_position {
            ctx.commands.trigger(UpdateMapPositionRequested {
                entity,
                x: p.pos_x.unwrap_or(snap_x),
                y: p.pos_y.unwrap_or(snap_y),
            });
        } else {
            warn!(
                "consume_position_commit: snapshot missing MapPosition for entity {}",
                entity.to_bits()
            );
        }
    }

    if p.remove_z {
        ctx.commands.trigger(RemoveZIndexRequested { entity });
    } else if p.commit_z {
        if let Some(z_index) = snap.z_index {
            ctx.commands.trigger(UpdateZIndexRequested {
                entity,
                z_index: p.z_index.unwrap_or(z_index),
            });
        } else {
            warn!(
                "consume_z_commit: snapshot missing ZIndex for entity {}",
                entity.to_bits()
            );
        }
    }

    if p.remove_group {
        ctx.commands.trigger(RemoveGroupRequested { entity });
    } else if p.commit_group {
        if let Some(ref group) = snap.group {
            ctx.commands.trigger(UpdateGroupRequested {
                entity,
                group: p.group.as_ref().unwrap_or(group).clone(),
            });
        } else {
            warn!(
                "consume_group_commit: snapshot missing Group for entity {}",
                entity.to_bits()
            );
        }
    }

    if p.remove_rotation {
        ctx.commands.trigger(RemoveRotationRequested { entity });
    } else if p.commit_rotation {
        if let Some(rotation_deg) = snap.rotation_deg {
            ctx.commands.trigger(UpdateRotationRequested {
                entity,
                rotation_deg: p.rotation_deg.unwrap_or(rotation_deg),
            });
        } else {
            warn!(
                "consume_rotation_commit: snapshot missing Rotation for entity {}",
                entity.to_bits()
            );
        }
    }

    if p.remove_scale {
        ctx.commands.trigger(RemoveScaleRequested { entity });
    } else if p.commit_scale {
        if let Some([scale_x, scale_y]) = snap.scale {
            ctx.commands.trigger(UpdateScaleRequested {
                entity,
                x: p.scale_x.unwrap_or(scale_x),
                y: p.scale_y.unwrap_or(scale_y),
            });
        } else {
            warn!(
                "consume_scale_commit: snapshot missing Scale for entity {}",
                entity.to_bits()
            );
        }
    }
}
