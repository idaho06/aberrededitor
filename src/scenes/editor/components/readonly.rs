use crate::editor_types::ComponentSnapshot;
use crate::systems::entity_edit::{
    RemovePersistentRequested, RemovePhaseRequested, RemoveTimerRequested, RemoveTtlRequested,
};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::systems::GameCtx;

#[derive(Default, Clone)]
pub(crate) struct PendingReadonlyRemovals {
    pub remove_ttl: bool,
    pub remove_timer: bool,
    pub remove_phase: bool,
    pub remove_persistent: bool,
}

impl PendingReadonlyRemovals {
    pub(crate) fn is_dirty(&self) -> bool {
        self.remove_ttl || self.remove_timer || self.remove_phase || self.remove_persistent
    }
}

pub(crate) fn draw_section(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingReadonlyRemovals,
) {
    if let Some(ref ttl) = snap.ttl {
        ui.separator();
        ui.text("Ttl");
        ui.same_line();
        if ui.button("Del##ttl") {
            p.remove_ttl = true;
        }
        ui.group(|| ui.text_disabled(format!("  remaining: {:.3}", ttl.remaining)));
    }

    if let Some(ref timer) = snap.timer {
        ui.separator();
        ui.text("Timer");
        ui.same_line();
        if ui.button("Del##timer") {
            p.remove_timer = true;
        }
        ui.group(|| {
            ui.text_disabled(format!("  duration: {:.3}", timer.duration));
            ui.text_disabled(format!("  elapsed: {:.3}", timer.elapsed));
        });
    }

    if let Some(ref phase) = snap.phase {
        ui.separator();
        ui.text("Phase");
        ui.same_line();
        if ui.button("Del##phase") {
            p.remove_phase = true;
        }
        ui.group(|| {
            ui.text_disabled(format!("  current: {}", phase.current));
            ui.text_disabled(format!(
                "  previous: {}",
                phase.previous.as_deref().unwrap_or("(none)")
            ));
            ui.text_disabled(format!(
                "  next: {}",
                phase.next.as_deref().unwrap_or("(none)")
            ));
            ui.text_disabled(format!(
                "  time_in_phase: {:.3}",
                phase.time_in_phase
            ));
            if phase.phase_names.is_empty() {
                ui.text_disabled("  phase_names: (none)");
            } else {
                ui.text_disabled(format!(
                    "  phase_names: {}",
                    phase.phase_names.join(", ")
                ));
            }
        });
    }

    if snap.persistent {
        ui.separator();
        ui.text("Persistent");
        ui.same_line();
        if ui.button("Del##persistent") {
            p.remove_persistent = true;
        }
    }
}

pub(crate) fn commit(ctx: &mut GameCtx, entity: Entity, p: &PendingReadonlyRemovals) {
    if p.remove_ttl {
        ctx.commands.trigger(RemoveTtlRequested { entity });
    }
    if p.remove_timer {
        ctx.commands.trigger(RemoveTimerRequested { entity });
    }
    if p.remove_phase {
        ctx.commands.trigger(RemovePhaseRequested { entity });
    }
    if p.remove_persistent {
        ctx.commands.trigger(RemovePersistentRequested { entity });
    }
}
