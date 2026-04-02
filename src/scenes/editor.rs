use aberredengine::components::cameratarget::CameraTarget;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::imgui;
use aberredengine::raylib::camera::Camera2D;
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::camerafollowconfig::FollowMode;
use aberredengine::resources::input::InputState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use log::info;
use std::sync::Mutex;

static SHOW_IMGUI_DEMO: Mutex<bool> = Mutex::new(false);

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
    if let Some(_mb) = ui.begin_main_menu_bar() {
        if let Some(_file) = ui.begin_menu("File")
            && ui.menu_item("Save")
        {
            signals.set_flag("gui:action:file:save");
        }

        if let Some(_view) = ui.begin_menu("View") {
            let mut show_demo = SHOW_IMGUI_DEMO
                .lock()
                .expect("SHOW_IMGUI_DEMO mutex poisoned");
            ui.menu_item_config("ImGui Demo")
                .build_with_ref(&mut show_demo);
        }
    }

    let mut show_demo = SHOW_IMGUI_DEMO
        .lock()
        .expect("SHOW_IMGUI_DEMO mutex poisoned");
    if *show_demo {
        ui.show_demo_window(&mut show_demo);
    }
}
