use aberredengine::components::cameratarget::CameraTarget;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::events::switchdebug::SwitchDebugEvent;
use aberredengine::imgui;
use aberredengine::raylib::camera::Camera2D;
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::camerafollowconfig::FollowMode;
use aberredengine::resources::input::InputState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use crate::systems::map_ops::{LoadMapRequested, NewMapRequested, SaveMapRequested};
use crate::systems::tilemap_load::LoadTilemapRequested;
use log::info;

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
    if ctx.world_signals.take_flag("gui:action:file:new_map") {
        ctx.commands.trigger(NewMapRequested);
    }

    if ctx.world_signals.take_flag("gui:action:file:open_map")
        && let Some(path) = rfd::FileDialog::new().add_filter("Map", &["json"]).pick_file()
    {
        let path = path.display().to_string();
        ctx.world_signals.set_string("map:current_path", path.clone());
        ctx.commands.trigger(LoadMapRequested { path });
    }

    if ctx.world_signals.take_flag("gui:action:file:save") {
        if let Some(path) = ctx
            .world_signals
            .get_string("map:current_path")
            .map(|s| s.to_owned())
        {
            ctx.commands.trigger(SaveMapRequested { path });
        } else {
            ctx.world_signals.set_flag("gui:action:file:save_as");
        }
    }

    if ctx.world_signals.take_flag("gui:action:file:save_as")
        && let Some(path) = rfd::FileDialog::new()
            .add_filter("Map", &["json"])
            .save_file()
    {
        let path = path.display().to_string();
        ctx.world_signals.set_string("map:current_path", path.clone());
        ctx.commands.trigger(SaveMapRequested { path });
    }

    if ctx.world_signals.take_flag("gui:action:file:load_tilemap")
        && let Some(path) = rfd::FileDialog::new().pick_folder()
    {
        ctx.commands.trigger(LoadTilemapRequested {
            path: path.display().to_string(),
        });
    }

    if ctx.world_signals.take_flag("gui:action:view:toggle_debug") {
        ctx.commands.trigger(SwitchDebugEvent {});
    }

    let Some(entity) = ctx.world_signals.get_entity("editor:camera").copied() else {
        return;
    };

    if ctx.world_signals.take_flag("gui:action:view:reset_zoom")
        && let Ok(mut ct) = ctx.camera_targets.get_mut(entity)
    {
        ct.zoom = 1.0;
    }

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
            if ui.menu_item("New Map") {
                signals.set_flag("gui:action:file:new_map");
            }
            if ui.menu_item("Open Map...") {
                signals.set_flag("gui:action:file:open_map");
            }
            ui.separator();
            if ui.menu_item("Add Tilemap...") {
                signals.set_flag("gui:action:file:load_tilemap");
            }
            ui.separator();
            if ui.menu_item("Save Map") {
                signals.set_flag("gui:action:file:save");
            }
            if ui.menu_item("Save Map As...") {
                signals.set_flag("gui:action:file:save_as");
            }
        }

        if let Some(_view) = ui.begin_menu("View") {
            if ui.menu_item("Reset Zoom") {
                signals.set_flag("gui:action:view:reset_zoom");
            }
            if ui.menu_item_config("Toggle Debug Mode")
                .shortcut("F11")
                .selected(signals.has_flag("ui:debug_active"))
                .build()
            {
                signals.set_flag("gui:action:view:toggle_debug");
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

