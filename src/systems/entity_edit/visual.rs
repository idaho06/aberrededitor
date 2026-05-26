use super::{
    RemoveAnimationRequested, RemoveBoxColliderRequested, RemoveDynamicTextRequested,
    RemoveSpriteRequested, RemoveTintRequested, UpdateAnimationRequested,
    UpdateBoxColliderRequested, UpdateDynamicTextRequested, UpdateSpriteRequested,
    UpdateTintRequested,
};
use crate::components::serialized_lua_setup::SerializedLuaSetup;
use aberredengine::components::animation::Animation;
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::dynamictext::DynamicText;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::tint::Tint;
use aberredengine::raylib::prelude::{Color, Vector2};
use log::debug;
use std::sync::Arc;

component_edit_observer!(
    update_sprite_observer,
    UpdateSpriteRequested,
    Sprite,
    "Sprite",
    |sprite, event, entity| {
        sprite.tex_key = Arc::from(event.tex_key.as_str());
        sprite.width = event.width;
        sprite.height = event.height;
        sprite.offset = Vector2::new(event.offset[0], event.offset[1]);
        sprite.origin = Vector2::new(event.origin[0], event.origin[1]);
        sprite.flip_h = event.flip_h;
        sprite.flip_v = event.flip_v;
        debug!(
            "update_sprite_observer: updated entity {} sprite '{}'",
            entity.to_bits(),
            event.tex_key
        );
    }
);

component_remove_observer!(
    remove_sprite_observer,
    RemoveSpriteRequested,
    Sprite,
    "Sprite"
);

component_edit_observer!(
    update_box_collider_observer,
    UpdateBoxColliderRequested,
    BoxCollider,
    "BoxCollider",
    |collider, event, entity| {
        collider.size = Vector2::new(event.size[0], event.size[1]);
        collider.offset = Vector2::new(event.offset[0], event.offset[1]);
        collider.origin = Vector2::new(event.origin[0], event.origin[1]);
        debug!(
            "update_box_collider_observer: updated entity {} collider",
            entity.to_bits()
        );
    }
);

component_remove_observer!(
    remove_box_collider_observer,
    RemoveBoxColliderRequested,
    BoxCollider,
    "BoxCollider"
);

component_edit_observer!(
    update_tint_observer,
    UpdateTintRequested,
    Tint,
    "Tint",
    |tint, event, entity| {
        tint.color = Color::new(event.r, event.g, event.b, event.a);
        debug!(
            "update_tint_observer: updated entity {} tint -> ({}, {}, {}, {})",
            entity.to_bits(),
            event.r,
            event.g,
            event.b,
            event.a
        );
    }
);

component_remove_observer!(
    remove_tint_observer,
    RemoveTintRequested,
    Tint,
    "Tint"
);

component_edit_observer!(
    update_dynamic_text_observer,
    UpdateDynamicTextRequested,
    DynamicText,
    "DynamicText",
    |dt, event, entity| {
        let text: Arc<str> = Arc::from(event.text.as_str());
        let color = Color::new(event.r, event.g, event.b, event.a);
        dt.text = Arc::clone(&text);
        dt.initial_text = text;
        dt.font = Arc::from(event.font_key.as_str());
        dt.font_size = event.font_size;
        dt.color = color;
        dt.initial_color = color;
        debug!(
            "update_dynamic_text_observer: updated entity {} text='{}' font='{}'",
            entity.to_bits(),
            event.text,
            event.font_key
        );
    }
);

component_remove_observer!(
    remove_dynamic_text_observer,
    RemoveDynamicTextRequested,
    DynamicText,
    "DynamicText"
);

component_edit_observer!(
    update_animation_observer,
    UpdateAnimationRequested,
    Animation,
    "Animation",
    |animation, event, entity| {
        animation.animation_key = event.animation_key.clone();
        animation.frame_index = 0;
        animation.elapsed_time = 0.0;
        debug!(
            "update_animation_observer: updated entity {} animation '{}'",
            entity.to_bits(),
            event.animation_key,
        );
    }
);

component_remove_observer!(
    remove_animation_observer,
    RemoveAnimationRequested,
    Animation,
    "Animation"
);

// LuaSetup lives in visual because it's serialized data (not truly a separate concern)
use super::{RemoveLuaSetupRequested, UpdateLuaSetupRequested};

component_edit_observer!(
    update_lua_setup_observer,
    UpdateLuaSetupRequested,
    SerializedLuaSetup,
    "LuaSetup",
    |lua_setup, event, entity| {
        lua_setup.callback = event.callback.clone();
        debug!(
            "update_lua_setup_observer: updated entity {} callback '{}'",
            entity.to_bits(),
            event.callback
        );
    }
);

component_remove_observer!(
    remove_lua_setup_observer,
    RemoveLuaSetupRequested,
    SerializedLuaSetup,
    "LuaSetup"
);
