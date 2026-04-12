use aberredengine::components::cameratarget::CameraTarget;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::events::input::InputAction;
use aberredengine::events::switchdebug::SwitchDebugEvent;
use aberredengine::imgui;
use aberredengine::raylib::camera::Camera2D;
use aberredengine::raylib::ffi::{KeyboardKey, MouseButton};
use aberredengine::resources::camera2d::Camera2DRes;
use aberredengine::resources::camerafollowconfig::FollowMode;
use aberredengine::resources::input::InputState;
use aberredengine::resources::input_bindings::InputBinding;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use crate::systems::entity_selector::{clear_selector_signals, PickEntitiesAtPointRequested, SelectEntityRequested};
use crate::systems::map_ops::{
    AddTextureRequested, LoadMapRequested, NewMapRequested, PreviewMapDataRequested,
    RemoveTextureRequested, RenameTextureKeyRequested, SaveMapRequested,
};
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

    // Rebind Action1 to mouse-left only so Space doesn't trigger entity picking
    ctx.input_bindings.rebind(
        InputAction::Action1,
        InputBinding::MouseButton(MouseButton::MOUSE_BUTTON_LEFT),
    );
}

pub fn editor_exit(ctx: &mut GameCtx) {
    info!("editor_exit: leaving editor scene");

    // Restore default Action1 bindings (Space + MouseLeft)
    ctx.input_bindings
        .rebind(InputAction::Action1, InputBinding::Keyboard(KeyboardKey::KEY_SPACE));
    ctx.input_bindings
        .add_binding(InputAction::Action1, InputBinding::MouseButton(MouseButton::MOUSE_BUTTON_LEFT));

    clear_selector_signals(&mut ctx.world_signals);
    ctx.world_signals.clear_flag("imgui:wants_mouse");
}

pub fn editor_update(ctx: &mut GameCtx, dt: f32, input: &InputState) {
    // Entity picking — left mouse click (Action1 rebound to mouse-only in editor_enter).
    // Suppressed when ImGui captured the mouse last frame to prevent clicks on UI widgets
    // from triggering world picks.
    if input.action_1.just_pressed && !ctx.world_signals.has_flag("imgui:wants_mouse") {
        ctx.commands.trigger(PickEntitiesAtPointRequested {
            x: input.mouse_world_x,
            y: input.mouse_world_y,
        });
    }

    // Resolve entity selection from GUI row click
    if let Some(row) = ctx.world_signals.clear_integer("gui:entity_selector:selected_row") {
        ctx.commands.trigger(SelectEntityRequested { index: row as usize });
    }

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

    if ctx.world_signals.take_flag("gui:action:texture:rename") {
        let old_key = ctx
            .world_signals
            .get_string("gui:texture_editor:rename_src")
            .map(|s| s.to_owned());
        let new_key = ctx
            .world_signals
            .get_string("gui:texture_editor:rename_buf")
            .map(|s| s.to_owned());
        if let (Some(old_key), Some(new_key)) = (old_key, new_key) {
            ctx.commands
                .trigger(RenameTextureKeyRequested { old_key, new_key });
        }
    }

    if ctx.world_signals.take_flag("gui:action:texture:remove")
        && let Some(key) = ctx
            .world_signals
            .get_string("gui:texture_editor:remove_key")
            .map(|s| s.to_owned())
    {
        ctx.commands.trigger(RemoveTextureRequested { key });
    }

    if ctx.world_signals.take_flag("gui:action:texture:add_browse") {
        let key = ctx
            .world_signals
            .get_string("gui:texture_editor:add_key_buf")
            .map(|s| s.to_owned())
            .unwrap_or_default();
        if !key.is_empty()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
                .pick_file()
        {
            ctx.commands.trigger(AddTextureRequested {
                key,
                path: path.display().to_string(),
            });
        }
    }

    if ctx.world_signals.take_flag("gui:action:view:toggle_debug") {
        ctx.commands.trigger(SwitchDebugEvent {});
    }

    if ctx.world_signals.take_flag("gui:action:view:preview_mapdata") {
        ctx.commands.trigger(PreviewMapDataRequested);
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

pub fn editor_gui(ui: &imgui::Ui, signals: &mut WorldSignals, textures: &TextureStore) {
    // Publish ImGui mouse-capture state so editor_update can suppress world picks next frame.
    if ui.io().want_capture_mouse {
        signals.set_flag("imgui:wants_mouse");
    } else {
        signals.clear_flag("imgui:wants_mouse");
    }

    let mut open_about = false;
    let mut open_rename_popup = false;
    let mut open_remove_popup = false;

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
            ui.separator();
            if ui.menu_item_config("Texture Store")
                .selected(signals.has_flag("ui:texture_editor:open"))
                .build()
            {
                signals.toggle_flag("ui:texture_editor:open");
            }
            if ui.menu_item_config("Entity Selector")
                .selected(signals.has_flag("ui:entity_selector:open"))
                .build()
            {
                signals.toggle_flag("ui:entity_selector:open");
            }
            let preview_open = signals.has_flag("gui:view:preview_mapdata_open");
            if ui.menu_item_config("Preview Map Data")
                .selected(preview_open)
                .build()
            {
                if preview_open {
                    signals.take_flag("gui:view:preview_mapdata_open");
                } else {
                    signals.set_flag("gui:action:view:preview_mapdata");
                }
            }
        }

        if let Some(_help) = ui.begin_menu("Help")
            && ui.menu_item("About")
        {
            open_about = true;
        }
    }

    // ---- Texture Store window ----

    if signals.has_flag("ui:texture_editor:open") {
        let mut window_open = true;
        ui.window("Texture Store")
            .size([460.0, 520.0], imgui::Condition::FirstUseEver)
            .opened(&mut window_open)
            .build(|| {
                // Scrollable list — leaves ~45px for the Add row at the bottom
                ui.child_window("##texture_list")
                    .size([0.0, -65.0])
                    .build(|| {
                        let mut sorted_keys: Vec<&String> = textures.map.keys().collect();
                        sorted_keys.sort();

                        if sorted_keys.is_empty() {
                            ui.text_disabled("No textures loaded.");
                        }

                        for key in &sorted_keys {
                            let Some(texture) = textures.map.get(key.as_str()) else {
                                continue;
                            };

                            let tex = texture.as_ref();
                            let tex_id =
                                imgui::TextureId::from(tex as *const _ as usize);

                            // Thumbnail on the left
                            imgui::Image::new(tex_id, [96.0f32, 96.0]).build(ui);
                            ui.same_line();

                            // Key label + action buttons in a vertical group.
                            // push_id scopes the row so button labels need no ## suffix.
                            let _id = ui.push_id(key.as_str());
                            ui.group(|| {
                                ui.text(key.as_str());
                                ui.text_disabled(format!("{}×{}", tex.width, tex.height));
                                if let Some(path) = textures.paths.get(key.as_str()) {
                                    let filename = std::path::Path::new(path)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or(path.as_str());
                                    ui.text_disabled(filename);
                                    if ui.is_item_hovered() {
                                        ui.tooltip_text(path.as_str());
                                    }
                                }
                                if ui.small_button("Rename") {
                                    signals.set_string(
                                        "gui:texture_editor:rename_src",
                                        key.as_str(),
                                    );
                                    signals.set_string(
                                        "gui:texture_editor:rename_buf",
                                        key.as_str(),
                                    );
                                    open_rename_popup = true;
                                }
                                ui.same_line();
                                if ui.small_button("Remove") {
                                    signals.set_string(
                                        "gui:texture_editor:remove_key",
                                        key.as_str(),
                                    );
                                    open_remove_popup = true;
                                }
                            });
                        }
                    });

                // Add-texture row
                ui.separator();
                ui.text("Add texture");
                ui.text("Key:");
                ui.same_line();
                let mut add_key = signals
                    .get_string("gui:texture_editor:add_key_buf")
                    .cloned()
                    .unwrap_or_default();
                ui.set_next_item_width(ui.content_region_avail()[0] - 85.0);
                if ui.input_text("##add_key", &mut add_key).build() {
                    signals.set_string("gui:texture_editor:add_key_buf", add_key.as_str());
                }
                ui.same_line();
                if ui.button("Browse...##add") {
                    signals.set_flag("gui:action:texture:add_browse");
                }
            });

        if !window_open {
            signals.take_flag("ui:texture_editor:open");
        }
    }

    // ---- Map Data Preview window ----

    if signals.has_flag("gui:view:preview_mapdata_open") {
        let mut window_open = true;
        ui.window("Map Data Preview")
            .size([600.0, 500.0], imgui::Condition::FirstUseEver)
            .opened(&mut window_open)
            .build(|| {
                if ui.button("Refresh") {
                    signals.set_flag("gui:action:view:preview_mapdata");
                }
                ui.separator();
                let mut json = signals
                    .get_string("gui:mapdata_preview_json")
                    .cloned()
                    .unwrap_or_default();
                ui.input_text_multiline("##mapdata_json", &mut json, [-1.0, -1.0])
                    .read_only(true)
                    .build();
            });
        if !window_open {
            signals.take_flag("gui:view:preview_mapdata_open");
        }
    }

    // ---- Entity Selector window ----

    if signals.has_flag("ui:entity_selector:open") {
        let mut window_open = true;
        ui.window("Entity Selector")
            .size([320.0, 400.0], imgui::Condition::FirstUseEver)
            .opened(&mut window_open)
            .build(|| {
                let payload_str = signals
                    .get_string("gui:entity_selector:payload")
                    .cloned();

                match payload_str
                    .as_deref()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                {
                    None => {
                        ui.text_disabled("Left-click in the scene to pick entities.");
                    }
                    Some(payload) => {
                        // Click position header
                        if let (Some(cx), Some(cy)) = (
                            payload["click"][0].as_f64(),
                            payload["click"][1].as_f64(),
                        ) {
                            ui.text_disabled(format!("Click: ({:.1}, {:.1})", cx, cy));
                        }
                        ui.separator();

                        // Hit list
                        if let Some(hits) = payload["hits"].as_array() {
                            if hits.is_empty() {
                                ui.text_disabled("No entities at click position.");
                            } else {
                                for (i, hit) in hits.iter().enumerate() {
                                    let label = hit["label"].as_str().unwrap_or("?");
                                    let zindex = hit["zindex"].as_f64().unwrap_or(0.0);
                                    let row_text = format!("{} (z={:.1})", label, zindex);
                                    let _id = ui.push_id_usize(i);
                                    if ui.selectable_config(&row_text).build() {
                                        signals.set_integer(
                                            "gui:entity_selector:selected_row",
                                            i as i32,
                                        );
                                    }
                                }
                            }
                        }

                        // Active selection footer
                        ui.separator();
                        if let Some(label) = signals
                            .get_string("gui:entity_selector:selected_label")
                            .cloned()
                        {
                            ui.text(format!("Selected: {}", label));
                        } else {
                            ui.text_disabled("No entity selected.");
                        }
                    }
                }
            });
        if !window_open {
            signals.take_flag("ui:entity_selector:open");
        }
    }

    // ---- Popup triggers (must come after window content, same frame) ----

    if open_rename_popup {
        ui.open_popup("Rename Key##texture_editor");
    }
    if open_remove_popup {
        ui.open_popup("Remove Texture##texture_editor");
    }
    if open_about {
        ui.open_popup("About");
    }

    // ---- Modal definitions ----

    ui.modal_popup_config("Rename Key##texture_editor")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let src = signals
                .get_string("gui:texture_editor:rename_src")
                .cloned()
                .unwrap_or_default();
            ui.text(format!("Old key: {src}"));
            ui.spacing();

            let mut buf = signals
                .get_string("gui:texture_editor:rename_buf")
                .cloned()
                .unwrap_or_default();
            if ui.input_text("New key##rename_input", &mut buf).build() {
                signals.set_string("gui:texture_editor:rename_buf", buf.as_str());
            }

            ui.spacing();
            ui.separator();
            if ui.button("OK##rename_ok") {
                signals.set_flag("gui:action:texture:rename");
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel##rename_cancel") {
                ui.close_current_popup();
            }
        });

    ui.modal_popup_config("Remove Texture##texture_editor")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            let key = signals
                .get_string("gui:texture_editor:remove_key")
                .cloned()
                .unwrap_or_default();
            ui.text(format!("Remove \"{key}\"?"));
            ui.text_disabled("This also unloads the texture from GPU memory.");
            ui.spacing();
            ui.separator();
            if ui.button("Yes##remove_yes") {
                signals.set_flag("gui:action:texture:remove");
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("No##remove_no") {
                ui.close_current_popup();
            }
        });

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

    // ---- Selection outline (ImGui background draw list) ----

    if let Some(corners_str) = signals.get_string("gui:entity_selector:selection_corners").cloned()
        && let Ok(corners) = serde_json::from_str::<Vec<[f32; 2]>>(&corners_str)
        && corners.len() == 4
    {
        let target_x = signals.get_scalar("editor:cam:target_x").unwrap_or(0.0);
        let target_y = signals.get_scalar("editor:cam:target_y").unwrap_or(0.0);
        let zoom     = signals.get_scalar("editor:cam:zoom").unwrap_or(1.0);
        let offset_x = signals.get_scalar("editor:cam:offset_x").unwrap_or(0.0);
        let offset_y = signals.get_scalar("editor:cam:offset_y").unwrap_or(0.0);
        let lb_scale = signals.get_scalar("editor:win:scale").unwrap_or(1.0);
        let lb_x     = signals.get_scalar("editor:win:offset_x").unwrap_or(0.0);
        let lb_y     = signals.get_scalar("editor:win:offset_y").unwrap_or(0.0);

        let to_screen = |wx: f32, wy: f32| -> [f32; 2] {
            let rx = (wx - target_x) * zoom + offset_x;
            let ry = (wy - target_y) * zoom + offset_y;
            [rx * lb_scale + lb_x, ry * lb_scale + lb_y]
        };

        let pts: Vec<[f32; 2]> = corners
            .iter()
            .map(|&[wx, wy]| to_screen(wx, wy))
            .collect();

        let color = [1.0_f32, 0.85, 0.0, 1.0]; // gold
        let dl = ui.get_background_draw_list();
        for i in 0..4 {
            dl.add_line(pts[i], pts[(i + 1) % 4], color)
                .thickness(2.0)
                .build();
        }
    }
}

