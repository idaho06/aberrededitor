use super::super::widgets::draw_text_buffer_input;
use crate::editor_types::ComponentSnapshot;
use crate::systems::entity_edit::{RemoveLuaSetupRequested, UpdateLuaSetupRequested};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::systems::GameCtx;
use log::warn;

#[derive(Default, Clone)]
pub(crate) struct PendingLuaSetup {
    pub callback: Option<String>,
    pub commit: bool,
    pub remove: bool,
}

impl PendingLuaSetup {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit || self.remove
    }
}

pub(crate) fn draw_section(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingLuaSetup,
) {
    let Some(ref callback) = snap.lua_setup else {
        return;
    };
    ui.separator();
    ui.text("LuaSetup");
    ui.same_line();
    if ui.button("Del##luasetup") {
        p.remove = true;
    }
    let mut committed = false;
    draw_text_buffer_input(
        ui,
        "callback##luasetup",
        &mut p.callback,
        &mut committed,
        callback.as_str(),
    );
    if committed {
        p.commit = true;
    }
}

pub(crate) fn commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingLuaSetup,
) {
    if p.remove {
        ctx.commands.trigger(RemoveLuaSetupRequested { entity });
    } else if p.commit {
        if let Some(ref callback) = snap.lua_setup {
            ctx.commands.trigger(UpdateLuaSetupRequested {
                entity,
                callback: p.callback.as_ref().unwrap_or(callback).clone(),
            });
        } else {
            warn!(
                "consume_lua_setup_commit: snapshot missing LuaSetup for entity {}",
                entity.to_bits()
            );
        }
    }
}
