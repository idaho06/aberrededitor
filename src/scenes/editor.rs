use std::sync::Arc;

use aberredengine::bevy_ecs::prelude::{Commands, ResMut};
use aberredengine::components::cameratarget::CameraTarget;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::zindex::ZIndex;
use aberredengine::imgui;
use aberredengine::raylib::camera::Camera2D;
use aberredengine::raylib::prelude::Vector2;
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::camerafollowconfig::FollowMode;
use aberredengine::resources::input::InputState;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::tilemapstore::{Tilemap, TilemapStore};
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use aberredengine::systems::RaylibAccess;
use log::{error, info};

pub fn editor_enter(ctx: &mut GameCtx) {
    info!("editor_enter: entering editor scene");

    let rw = ctx.config.render_width as f32;
    let rh = ctx.config.render_height as f32;

    ctx.commands.insert_resource(Camera2DRes(Camera2D {
        offset: (rw / 2.0, rh / 2.0).into(),
        target: (0.0, 0.0).into(),
        rotation: 0.0,
        zoom: 1.0,
    }));

    let entity = ctx
        .commands
        .spawn((MapPosition::new(0.0, 0.0), CameraTarget::new(0)))
        .id();
    ctx.world_signals.set_entity("editor:camera", entity);

    ctx.camera_follow.enabled = true;
    ctx.camera_follow.mode = FollowMode::Instant;
    ctx.camera_follow.zoom_lerp_speed = 10.0;
}

pub fn editor_update(ctx: &mut GameCtx, dt: f32, input: &InputState) {
    if ctx.world_signals.has_flag("gui:action:file:save") {
        ctx.world_signals.clear_flag("gui:action:file:save");
        info!("editor_update: save requested");
        // handle save
    }

    if ctx.world_signals.has_flag("gui:action:file:load_tilemap") {
        ctx.world_signals.clear_flag("gui:action:file:load_tilemap");
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Tilesetter JSON", &["txt", "json"])
            .pick_file()
        {
            ctx.world_signals.set_string(
                "gui:pending:load_tilemap_path",
                path.display().to_string(),
            );
        }
    }

    let Some(entity) = ctx.world_signals.get_entity("editor:camera").copied() else {
        return;
    };

    // Pan: WASD + arrow keys move the camera target entity
    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;
    if input.maindirection_left.active || input.secondarydirection_left.active {
        dx -= 1.0;
    }
    if input.maindirection_right.active || input.secondarydirection_right.active {
        dx += 1.0;
    }
    if input.maindirection_up.active || input.secondarydirection_up.active {
        dy -= 1.0;
    }
    if input.maindirection_down.active || input.secondarydirection_down.active {
        dy += 1.0;
    }
    if dx != 0.0 || dy != 0.0 {
        let pan_speed = 300.0_f32; // pixels/sec at zoom 1.0
        let zoom = ctx
            .camera_targets
            .get(entity)
            .map(|ct| ct.zoom)
            .unwrap_or(1.0);
        let speed = pan_speed * dt / zoom;
        if let Ok(mut pos) = ctx.positions.get_mut(entity) {
            pos.translate(dx * speed, dy * speed);
        }
    }

    // Zoom: scroll wheel scales CameraTarget.zoom multiplicatively
    if input.scroll_y.abs() > 0.0
        && let Ok(mut ct) = ctx.camera_targets.get_mut(entity)
    {
        let factor = 1.1_f32.powf(input.scroll_y);
        ct.zoom = (ct.zoom * factor).clamp(0.1, 10.0);
    }
}

pub fn editor_gui(ui: &imgui::Ui, signals: &mut WorldSignals) {
    let mut open_about = false;

    if let Some(_mb) = ui.begin_main_menu_bar() {
        if let Some(_file) = ui.begin_menu("File") {
            if ui.menu_item("Load Tilesetter map...") {
                signals.set_flag("gui:action:file:load_tilemap");
            }
            if ui.menu_item("Save") {
                signals.set_flag("gui:action:file:save");
            }
        }

        if let Some(_help) = ui.begin_menu("Help")
            && ui.menu_item("About")
        {
            open_about = true;
        }
    }

    if open_about {
        ui.open_popup("About");
    }

    ui.modal_popup_config("About")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            ui.text(format!(
                "Aberred Map Editor version {}",
                env!("CARGO_PKG_VERSION")
            ));
            ui.text("By Idaho06 from AkinoSoft!");
            ui.text("(c) 2026");
            ui.separator();
            if ui.button("OK") {
                ui.close_current_popup();
            }
        });
}

pub fn tilemap_load_system(
    mut commands: Commands,
    mut raylib: RaylibAccess,
    mut world_signals: ResMut<WorldSignals>,
    mut texture_store: ResMut<TextureStore>,
    mut tilemap_store: ResMut<TilemapStore>,
) {
    let Some(json_path) = world_signals.remove_string("gui:pending:load_tilemap_path") else {
        return;
    };

    let path = std::path::Path::new(&json_path);
    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tilemap".to_string());
    let png_path = path.with_extension("png");
    let png_path_str = png_path.display().to_string();

    // Parse JSON
    let json_string = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            error!("tilemap_load_system: failed to read '{}': {}", json_path, e);
            return;
        }
    };
    let tilemap: Tilemap = match serde_json::from_str(&json_string) {
        Ok(t) => t,
        Err(e) => {
            error!("tilemap_load_system: failed to parse tilemap JSON: {}", e);
            return;
        }
    };

    // Load tileset texture (must be a sibling .png with the same stem)
    let (rl, th) = (&mut *raylib.rl, &*raylib.th);
    let texture = match rl.load_texture(th, &png_path_str) {
        Ok(t) => t,
        Err(_) => {
            error!(
                "tilemap_load_system: tileset texture not found at '{}' — skipping load",
                png_path_str
            );
            return;
        }
    };

    let tex_width = texture.width;
    texture_store.insert(&id, texture);
    tilemap_store.insert(&id, tilemap);

    // Spawn tile entities
    let tilemap = tilemap_store.get(&id).expect("just inserted");
    let tex_key: Arc<str> = Arc::from(id.as_str());
    let tile_size = tilemap.tile_size as f32;
    let tiles_per_row = ((tex_width as f32 / tile_size).floor() as u32).max(1);
    let layer_count = tilemap.layers.len() as f32;

    for (layer_index, layer) in tilemap.layers.iter().enumerate() {
        let z = -(layer_count - layer_index as f32);
        for pos in &layer.positions {
            let col = pos.id % tiles_per_row;
            let row = pos.id / tiles_per_row;
            commands.spawn((
                Group::new("tiles"),
                MapPosition::new(pos.x as f32 * tile_size, pos.y as f32 * tile_size),
                ZIndex(z),
                Sprite {
                    tex_key: tex_key.clone(),
                    width: tile_size,
                    height: tile_size,
                    offset: Vector2 {
                        x: col as f32 * tile_size,
                        y: row as f32 * tile_size,
                    },
                    origin: Vector2 { x: 0.0, y: 0.0 },
                    flip_h: false,
                    flip_v: false,
                },
            ));
        }
    }

    info!(
        "tilemap_load_system: loaded tilemap '{}' from '{}'",
        id, json_path
    );
}
