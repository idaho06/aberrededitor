use crate::editor_types::ComponentSnapshot;
use crate::systems::entity_edit::{RemoveTintRequested, UpdateTintRequested};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::systems::GameCtx;
use log::warn;

#[derive(Default, Clone)]
pub(crate) struct PendingTint {
    pub color: Option<[f32; 4]>,
    pub commit: bool,
    pub remove: bool,
}

impl PendingTint {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit || self.remove
    }
}

pub(crate) fn draw_section(ui: &imgui::Ui, snap: &ComponentSnapshot, p: &mut PendingTint) {
    let Some(ref tint_snap) = snap.tint else {
        return;
    };
    ui.separator();
    ui.text("Tint");
    ui.same_line();
    if ui.button("Del##tint") {
        p.remove = true;
    }
    let mut color = p.color.unwrap_or(tint_snap.color_normalized());
    if ui.color_edit4("##tint_color", &mut color) {
        p.color = Some(color);
    }
    if ui.is_item_deactivated_after_edit() {
        p.commit = true;
    }
}

pub(crate) fn commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingTint,
) {
    if p.remove {
        ctx.commands.trigger(RemoveTintRequested { entity });
    } else if p.commit {
        if let Some(ref tint) = snap.tint {
            let [r, g, b, a] = p.color.unwrap_or(tint.color_normalized());
            ctx.commands.trigger(UpdateTintRequested {
                entity,
                r: (r * 255.0).round() as u8,
                g: (g * 255.0).round() as u8,
                b: (b * 255.0).round() as u8,
                a: (a * 255.0).round() as u8,
            });
        } else {
            warn!(
                "consume_tint_commit: snapshot missing Tint for entity {}",
                entity.to_bits()
            );
        }
    }
}
