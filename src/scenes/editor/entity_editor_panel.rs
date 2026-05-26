//! Per-entity component inspector and edit panel.
//!
//! `draw_entity_editor` is the thin orchestrator: it handles entry/guards, the entity-level
//! actions (clone/delete, registration, tilemap, add-component), then delegates each component
//! section to `components::<name>::draw_section`. Commit logic lives in `commit.rs`.
//!
//! `draw_entity_delete_modal` renders the entity deletion confirmation popup.
use super::components;
use super::pending_state::PendingMutex;
use super::widgets::draw_text_buffer_input;
use crate::editor_types::{ComponentKind, ComponentSnapshot};
use crate::signals as sig;
use crate::systems::animation_store_sync::AnimationStoreMutex;
use aberredengine::imgui;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::fontstore::FontStore;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_entity_editor(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    textures: &TextureStore,
    fonts: &FontStore,
    app_state: &AppState,
) -> bool {
    if !signals.has_flag(sig::UI_ENTITY_EDITOR_OPEN) {
        return false;
    }

    let Some(mutex) = app_state.get::<PendingMutex>() else {
        return false;
    };

    let mut open_delete_popup = false;
    let mut window_open = true;
    ui.window("Entity Inspector")
        .size([380.0, 420.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            let Some(snap) = app_state.get::<ComponentSnapshot>().cloned() else {
                ui.text_disabled("No entity selected.");
                return;
            };

            ui.text(format!("Entity #{}", snap.entity_bits & 0xFFFF_FFFF));
            ui.separator();

            if ui.button("Clone Entity") {
                mutex.lock().unwrap().clone_entity = true;
            }
            ui.same_line();
            if ui.button("Delete Entity") {
                open_delete_popup = true;
            }
            ui.separator();

            let mut p = mutex.lock().unwrap();

            // Named slot in WorldSignals.entities (not a real ECS component)
            let current_key = snap.world_signal_keys.first().cloned();
            if p.pending_register_key.is_none() {
                if let Some(ref key) = current_key {
                    ui.text_disabled(format!("  Key: {}", key));
                    ui.same_line();
                    if ui.button("Edit##reg") {
                        p.pending_register_key = Some(key.clone());
                    }
                    ui.same_line();
                    if ui.button("Del##reg") {
                        p.remove_registration = true;
                    }
                } else {
                    ui.text_disabled("  Not registered");
                    ui.same_line();
                    if ui.button("+##reg") {
                        p.pending_register_key =
                            Some(format!("entity_{:08x}", snap.entity_bits & 0xFFFF_FFFF));
                    }
                }
            } else {
                let fallback = current_key.as_deref().unwrap_or("");
                {
                    let ps = &mut *p;
                    draw_text_buffer_input(
                        ui,
                        "##reg_key",
                        &mut ps.pending_register_key,
                        &mut ps.commit_registration,
                        fallback,
                    );
                }
                ui.same_line();
                if ui.button("OK##reg") {
                    p.commit_registration = true;
                }
                ui.same_line();
                if ui.button("X##reg") {
                    p.pending_register_key = None;
                }
            }
            ui.separator();

            // If this entity is a tile child of a TileMap root, show a navigation button.
            if snap.tilemap_parent.is_some() {
                if ui.button("Select parent entity (TileMap root)") {
                    p.select_tilemap_parent = true;
                }
                ui.text_disabled("Changes to the components of");
                ui.text_disabled("this entity will not be saved");
                ui.separator();
            }

            let addable: Vec<(&str, ComponentKind)> = [
                (
                    snap.map_position.is_none(),
                    "MapPosition",
                    ComponentKind::MapPosition,
                ),
                (snap.z_index.is_none(), "ZIndex", ComponentKind::ZIndex),
                (snap.group.is_none(), "Group", ComponentKind::Group),
                (
                    snap.rotation_deg.is_none(),
                    "Rotation",
                    ComponentKind::Rotation,
                ),
                (snap.scale.is_none(), "Scale", ComponentKind::Scale),
                (snap.sprite.is_none(), "Sprite", ComponentKind::Sprite),
                (
                    snap.box_collider.is_none(),
                    "BoxCollider",
                    ComponentKind::BoxCollider,
                ),
                (
                    snap.animation.is_none(),
                    "Animation",
                    ComponentKind::Animation,
                ),
                (snap.ttl.is_none(), "Ttl", ComponentKind::Ttl),
                (!snap.persistent, "Persistent", ComponentKind::Persistent),
                (snap.tint.is_none(), "Tint", ComponentKind::Tint),
                (
                    snap.lua_setup.is_none(),
                    "LuaSetup",
                    ComponentKind::LuaSetup,
                ),
                (
                    snap.dynamic_text.is_none(),
                    "DynamicText",
                    ComponentKind::DynamicText,
                ),
                (
                    snap.particle_emitter.is_none(),
                    "Particle Emitter",
                    ComponentKind::ParticleEmitter,
                ),
            ]
            .into_iter()
            .filter_map(|(absent, label, kind)| absent.then_some((label, kind)))
            .collect();

            if addable.is_empty() {
                ui.text_disabled("All components present");
            } else {
                p.add_combo_selection = p.add_combo_selection.min(addable.len() - 1);
                let mut sel = p.add_combo_selection;
                ui.set_next_item_width(-60.0);
                ui.combo_simple_string(
                    "##add_combo",
                    &mut sel,
                    &addable.iter().map(|(l, _)| *l).collect::<Vec<_>>(),
                );
                p.add_combo_selection = sel;
                ui.same_line();
                if ui.button("Add##add_component") {
                    p.add_component = Some(addable[sel].1);
                }
            }
            ui.separator();

            let anim_store = app_state.get::<AnimationStoreMutex>();

            ui.child_window("##components_scroll")
                .size([0.0, 0.0])
                .build(|| {
                    components::transform::draw_map_position(ui, &snap, &mut p.transform);
                    components::transform::draw_z_index(ui, &snap, &mut p.transform);
                    components::transform::draw_group(ui, &snap, &mut p.transform);
                    components::sprite::draw_section(ui, &snap, &mut p.sprite, textures);
                    components::collider::draw_section(ui, &snap, &mut p.collider);
                    components::transform::draw_rotation(ui, &snap, &mut p.transform);
                    components::transform::draw_scale(ui, &snap, &mut p.transform);
                    components::animation::draw_section(
                        ui,
                        &snap,
                        &mut p.animation,
                        anim_store,
                    );
                    components::readonly::draw_section(ui, &snap, &mut p.readonly_removals);

                    if let Some(ref path) = snap.tilemap_path {
                        ui.separator();
                        ui.text("TileMap");
                        ui.same_line();
                        if ui.button("Bake##tilemap") {
                            p.bake_tilemap = true;
                        }
                        ui.same_line();
                        if ui.button("Del##tilemap") {
                            p.remove_tilemap = true;
                        }
                        ui.group(|| ui.text_disabled(format!("  path: {}", path)));
                    }

                    components::tint::draw_section(ui, &snap, &mut p.tint);
                    components::lua_setup::draw_section(ui, &snap, &mut p.lua_setup);
                    components::dynamic_text::draw_section(ui, &snap, &mut p.dynamic_text, fonts);
                    components::particle_emitter::draw_section(
                        ui,
                        &snap,
                        &mut p.particle_emitter,
                        signals,
                    );
                });
        });

    if !window_open {
        signals.take_flag(sig::UI_ENTITY_EDITOR_OPEN);
    }
    open_delete_popup
}

pub(super) fn draw_entity_delete_modal(ui: &imgui::Ui, app_state: &AppState) {
    let entity_id = app_state
        .get::<ComponentSnapshot>()
        .map(|s| s.entity_bits & 0xFFFF_FFFF)
        .unwrap_or(0);

    ui.modal_popup_config("Delete Entity##entity_editor")
        .always_auto_resize(true)
        .resizable(false)
        .movable(false)
        .build(|| {
            ui.text(format!(
                "Are you sure you want to despawn entity #{}?",
                entity_id
            ));
            ui.spacing();
            ui.separator();
            if ui.button("Yes##delete_yes") {
                if let Some(mutex) = app_state.get::<PendingMutex>() {
                    let mut p = mutex.lock().unwrap();
                    p.remove_entity = true;
                }
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("No##delete_no") {
                ui.close_current_popup();
            }
        });
}
