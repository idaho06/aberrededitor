use aberredengine::imgui;
use aberredengine::raylib::camera::Camera2D;
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::input::InputState;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use log::info;
use std::sync::Mutex;

static SHOW_IMGUI_DEMO: Mutex<bool> = Mutex::new(false);

pub fn editor_enter(ctx: &mut GameCtx) {
    info!("editor_enter: entering editor scene");

    ctx.commands.insert_resource(Camera2DRes(Camera2D {
        offset: (0.0, 0.0).into(),
        target: (0.0, 0.0).into(),
        rotation: 0.0,
        zoom: 1.0,
    }));
}

pub fn editor_update(ctx: &mut GameCtx, _dt: f32, _input: &InputState) {
    if ctx.world_signals.has_flag("gui:action:file:save") {
        ctx.world_signals.clear_flag("gui:action:file:save");
        info!("editor_update: save requested");
        // handle save
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
