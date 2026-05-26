use super::super::widgets::draw_float_input;
use crate::editor_types::ComponentSnapshot;
use crate::systems::entity_edit::{RemoveSpriteRequested, UpdateSpriteRequested};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::systems::GameCtx;
use log::warn;

#[derive(Default, Clone)]
pub(crate) struct PendingSprite {
    pub tex_key: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub off_x: Option<f32>,
    pub off_y: Option<f32>,
    pub org_x: Option<f32>,
    pub org_y: Option<f32>,
    pub flip_h: Option<bool>,
    pub flip_v: Option<bool>,
    pub commit: bool,
    pub remove: bool,
}

impl PendingSprite {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit || self.remove
    }
}

pub(crate) fn draw_section(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingSprite,
    textures: &TextureStore,
) {
    let Some(ref sprite) = snap.sprite else {
        return;
    };
    ui.separator();
    ui.text("Sprite");
    ui.same_line();
    if ui.button("Del##sprite") {
        p.remove = true;
    }

    let tex_key_current = p.tex_key.as_deref().unwrap_or(&sprite.tex_key);
    let mut texture_keys: Vec<&str> = textures.map.keys().map(|key| key.as_str()).collect();
    texture_keys.sort_unstable();
    if texture_keys.is_empty() {
        ui.text_disabled("No textures loaded.");
    } else {
        let mut current_tex = texture_keys
            .iter()
            .position(|key| *key == tex_key_current)
            .or_else(|| {
                texture_keys
                    .iter()
                    .position(|key| *key == sprite.tex_key.as_str())
            })
            .unwrap_or(0);
        if ui.combo_simple_string("tex_key##sprite", &mut current_tex, &texture_keys) {
            p.tex_key = Some(texture_keys[current_tex].to_owned());
            p.commit = true;
        }
    }

    if let Some(v) = draw_float_input(ui, "width##sprite", p.width.unwrap_or(sprite.width), 1.0) {
        p.width = Some(v);
        p.commit = true;
    }
    if let Some(v) =
        draw_float_input(ui, "height##sprite", p.height.unwrap_or(sprite.height), 1.0)
    {
        p.height = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "offset x##sprite",
        p.off_x.unwrap_or(sprite.offset[0]),
        1.0,
    ) {
        p.off_x = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "offset y##sprite",
        p.off_y.unwrap_or(sprite.offset[1]),
        1.0,
    ) {
        p.off_y = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "origin x##sprite",
        p.org_x.unwrap_or(sprite.origin[0]),
        1.0,
    ) {
        p.org_x = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "origin y##sprite",
        p.org_y.unwrap_or(sprite.origin[1]),
        1.0,
    ) {
        p.org_y = Some(v);
        p.commit = true;
    }

    let mut flip_h = p.flip_h.unwrap_or(sprite.flip_h);
    if ui.checkbox("flip_h##sprite", &mut flip_h) {
        p.flip_h = Some(flip_h);
        p.commit = true;
    }

    let mut flip_v = p.flip_v.unwrap_or(sprite.flip_v);
    if ui.checkbox("flip_v##sprite", &mut flip_v) {
        p.flip_v = Some(flip_v);
        p.commit = true;
    }
}

pub(crate) fn commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingSprite,
) {
    if p.remove {
        ctx.commands.trigger(RemoveSpriteRequested { entity });
    } else if p.commit {
        if let Some(ref sprite) = snap.sprite {
            ctx.commands.trigger(UpdateSpriteRequested {
                entity,
                tex_key: p.tex_key.as_ref().unwrap_or(&sprite.tex_key).clone(),
                width: p.width.unwrap_or(sprite.width),
                height: p.height.unwrap_or(sprite.height),
                offset: [
                    p.off_x.unwrap_or(sprite.offset[0]),
                    p.off_y.unwrap_or(sprite.offset[1]),
                ],
                origin: [
                    p.org_x.unwrap_or(sprite.origin[0]),
                    p.org_y.unwrap_or(sprite.origin[1]),
                ],
                flip_h: p.flip_h.unwrap_or(sprite.flip_h),
                flip_v: p.flip_v.unwrap_or(sprite.flip_v),
            });
        } else {
            warn!(
                "consume_sprite_commit: snapshot missing Sprite for entity {}",
                entity.to_bits()
            );
        }
    }
}
