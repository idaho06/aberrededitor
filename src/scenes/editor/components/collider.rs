use super::super::widgets::draw_float_input;
use crate::editor_types::ComponentSnapshot;
use crate::systems::entity_edit::{RemoveBoxColliderRequested, UpdateBoxColliderRequested};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::systems::GameCtx;
use log::warn;

#[derive(Default, Clone)]
pub(crate) struct PendingCollider {
    pub size_x: Option<f32>,
    pub size_y: Option<f32>,
    pub off_x: Option<f32>,
    pub off_y: Option<f32>,
    pub org_x: Option<f32>,
    pub org_y: Option<f32>,
    pub commit: bool,
    pub remove: bool,
}

impl PendingCollider {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit || self.remove
    }
}

pub(crate) fn draw_section(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingCollider,
) {
    let Some(ref collider) = snap.box_collider else {
        return;
    };
    ui.separator();
    ui.text("BoxCollider");
    ui.same_line();
    if ui.button("Del##collider") {
        p.remove = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "size x##collider",
        p.size_x.unwrap_or(collider.size[0]),
        1.0,
    ) {
        p.size_x = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "size y##collider",
        p.size_y.unwrap_or(collider.size[1]),
        1.0,
    ) {
        p.size_y = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "offset x##collider",
        p.off_x.unwrap_or(collider.offset[0]),
        1.0,
    ) {
        p.off_x = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "offset y##collider",
        p.off_y.unwrap_or(collider.offset[1]),
        1.0,
    ) {
        p.off_y = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "origin x##collider",
        p.org_x.unwrap_or(collider.origin[0]),
        1.0,
    ) {
        p.org_x = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "origin y##collider",
        p.org_y.unwrap_or(collider.origin[1]),
        1.0,
    ) {
        p.org_y = Some(v);
        p.commit = true;
    }
}

pub(crate) fn commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingCollider,
) {
    if p.remove {
        ctx.commands.trigger(RemoveBoxColliderRequested { entity });
    } else if p.commit {
        if let Some(ref collider) = snap.box_collider {
            ctx.commands.trigger(UpdateBoxColliderRequested {
                entity,
                size: [
                    p.size_x.unwrap_or(collider.size[0]),
                    p.size_y.unwrap_or(collider.size[1]),
                ],
                offset: [
                    p.off_x.unwrap_or(collider.offset[0]),
                    p.off_y.unwrap_or(collider.offset[1]),
                ],
                origin: [
                    p.org_x.unwrap_or(collider.origin[0]),
                    p.org_y.unwrap_or(collider.origin[1]),
                ],
            });
        } else {
            warn!(
                "consume_collider_commit: snapshot missing BoxCollider for entity {}",
                entity.to_bits()
            );
        }
    }
}
