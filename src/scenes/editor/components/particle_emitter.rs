use super::super::widgets::{draw_float_input, draw_int_input};
use crate::editor_types::{ComponentSnapshot, EmitterShapeKind, TtlKind};
use crate::signals as sig;
use crate::systems::entity_edit::{RemoveParticleEmitterRequested, UpdateParticleEmitterRequested};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::resources::worldsignals::WorldSignals;
use aberredengine::systems::GameCtx;
use log::warn;

const WARNING_TEXT_COLOR: [f32; 4] = [1.0, 0.45, 0.3, 1.0];

#[derive(Default, Clone)]
pub(crate) struct PendingParticleEmitter {
    pub template_keys: Option<Vec<String>>,
    pub shape: Option<EmitterShapeKind>,
    pub shape_rect_w: Option<f32>,
    pub shape_rect_h: Option<f32>,
    pub offset_x: Option<f32>,
    pub offset_y: Option<f32>,
    pub particles_per_emission: Option<u32>,
    pub emissions_per_second: Option<f32>,
    pub emissions_remaining: Option<u32>,
    pub arc_min: Option<f32>,
    pub arc_max: Option<f32>,
    pub speed_min: Option<f32>,
    pub speed_max: Option<f32>,
    pub ttl_kind: Option<TtlKind>,
    pub ttl_fixed: Option<f32>,
    pub ttl_min: Option<f32>,
    pub ttl_max: Option<f32>,
    pub commit: bool,
    pub remove: bool,
}

impl PendingParticleEmitter {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit || self.remove
    }
}

pub(crate) fn draw_section(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingParticleEmitter,
    signals: &WorldSignals,
) {
    let Some(ref pe) = snap.particle_emitter else {
        return;
    };

    ui.separator();
    ui.text("Particle Emitter");
    ui.same_line();
    if ui.small_button("Del##del_pe") {
        p.remove = true;
    }

    // Shape
    let mut shape = p.shape.unwrap_or(pe.shape);
    ui.text("Shape:");
    ui.same_line();
    if ui.radio_button_bool("Point##pe_shape", shape == EmitterShapeKind::Point) {
        p.shape = Some(EmitterShapeKind::Point);
        p.commit = true;
    }
    ui.same_line();
    if ui.radio_button_bool("Rect##pe_shape", shape == EmitterShapeKind::Rect) {
        p.shape = Some(EmitterShapeKind::Rect);
        p.commit = true;
    }
    shape = p.shape.unwrap_or(pe.shape);
    if shape == EmitterShapeKind::Rect {
        if let Some(v) = draw_float_input(
            ui,
            "W##pe_rect_w",
            p.shape_rect_w.unwrap_or(pe.shape_rect_w),
            1.0,
        ) {
            p.shape_rect_w = Some(v);
            p.commit = true;
        }
        if let Some(v) = draw_float_input(
            ui,
            "H##pe_rect_h",
            p.shape_rect_h.unwrap_or(pe.shape_rect_h),
            1.0,
        ) {
            p.shape_rect_h = Some(v);
            p.commit = true;
        }
    }

    // Offset
    if let Some(v) =
        draw_float_input(ui, "Off X##pe_off_x", p.offset_x.unwrap_or(pe.offset[0]), 1.0)
    {
        p.offset_x = Some(v);
        p.commit = true;
    }
    if let Some(v) =
        draw_float_input(ui, "Off Y##pe_off_y", p.offset_y.unwrap_or(pe.offset[1]), 1.0)
    {
        p.offset_y = Some(v);
        p.commit = true;
    }

    // Rate / burst
    if let Some(v) = draw_float_input(
        ui,
        "Rate/s##pe_rate",
        p.emissions_per_second.unwrap_or(pe.emissions_per_second),
        0.5,
    ) {
        p.emissions_per_second = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_int_input(
        ui,
        "Burst##pe_burst",
        p.particles_per_emission
            .unwrap_or(pe.particles_per_emission) as i32,
    ) {
        p.particles_per_emission = Some(v.max(1) as u32);
        p.commit = true;
    }

    // Remaining (number or max)
    let current_remaining = p.emissions_remaining.unwrap_or(pe.emissions_remaining);
    let is_max = current_remaining == u32::MAX;
    ui.text("Remaining:");
    if ui.radio_button_bool("##pe_remaining_num", !is_max) && is_max {
        p.emissions_remaining = Some(100);
        p.commit = true;
    }
    ui.same_line();
    if !is_max
        && let Some(v) = draw_int_input(ui, "Count##pe_remaining", current_remaining as i32)
    {
        p.emissions_remaining = Some(v.max(0) as u32);
        p.commit = true;
    }
    if ui.radio_button_bool("Max (∞)##pe_remaining_max", is_max) && !is_max {
        p.emissions_remaining = Some(u32::MAX);
        p.commit = true;
    }

    // Arc
    if let Some(v) = draw_float_input(
        ui,
        "Arc min°##pe_arc_min",
        p.arc_min.unwrap_or(pe.arc_min_deg),
        1.0,
    ) {
        p.arc_min = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "Arc max°##pe_arc_max",
        p.arc_max.unwrap_or(pe.arc_max_deg),
        1.0,
    ) {
        p.arc_max = Some(v);
        p.commit = true;
    }

    // Speed
    if let Some(v) = draw_float_input(
        ui,
        "Spd min##pe_spd_min",
        p.speed_min.unwrap_or(pe.speed_min),
        1.0,
    ) {
        p.speed_min = Some(v);
        p.commit = true;
    }
    if let Some(v) = draw_float_input(
        ui,
        "Spd max##pe_spd_max",
        p.speed_max.unwrap_or(pe.speed_max),
        1.0,
    ) {
        p.speed_max = Some(v);
        p.commit = true;
    }

    // TTL
    let mut ttl_kind = p.ttl_kind.unwrap_or(pe.ttl_kind);
    ui.text("TTL:");
    ui.same_line();
    if ui.radio_button_bool("None##pe_ttl", ttl_kind == TtlKind::None) {
        p.ttl_kind = Some(TtlKind::None);
        p.commit = true;
    }
    ui.same_line();
    if ui.radio_button_bool("Fixed##pe_ttl", ttl_kind == TtlKind::Fixed) {
        p.ttl_kind = Some(TtlKind::Fixed);
        p.commit = true;
    }
    ui.same_line();
    if ui.radio_button_bool("Range##pe_ttl", ttl_kind == TtlKind::Range) {
        p.ttl_kind = Some(TtlKind::Range);
        p.commit = true;
    }
    ttl_kind = p.ttl_kind.unwrap_or(pe.ttl_kind);
    match ttl_kind {
        TtlKind::Fixed => {
            if let Some(v) = draw_float_input(
                ui,
                "TTL s##pe_ttl_fixed",
                p.ttl_fixed.unwrap_or(pe.ttl_fixed),
                0.1,
            ) {
                p.ttl_fixed = Some(v.max(0.0));
                p.commit = true;
            }
        }
        TtlKind::Range => {
            if let Some(v) = draw_float_input(
                ui,
                "TTL min##pe_ttl_min",
                p.ttl_min.unwrap_or(pe.ttl_min),
                0.1,
            ) {
                p.ttl_min = Some(v.max(0.0));
                p.commit = true;
            }
            if let Some(v) = draw_float_input(
                ui,
                "TTL max##pe_ttl_max",
                p.ttl_max.unwrap_or(pe.ttl_max),
                0.1,
            ) {
                p.ttl_max = Some(v.max(0.0));
                p.commit = true;
            }
        }
        TtlKind::None => {}
    }

    // Template keys
    ui.separator();
    ui.text("Templates:");
    let current_keys = p
        .template_keys
        .clone()
        .unwrap_or_else(|| pe.template_keys.clone());
    let known_entity_keys: Vec<String> = signals
        .entities
        .keys()
        .filter(|key| sig::is_user_entity_key(key))
        .cloned()
        .collect();
    let mut new_keys = current_keys.clone();
    let mut keys_changed = false;
    let mut keys_committed = false;
    let mut remove_idx: Option<usize> = None;
    let mut unresolved_keys: Vec<String> = Vec::new();
    for (i, key) in current_keys.iter().enumerate() {
        let mut buf = key.clone();
        ui.set_next_item_width(-30.0);
        let edited = ui.input_text(&format!("##pe_tkey_{i}"), &mut buf).build();
        let deactivated = ui.is_item_deactivated_after_edit();
        if edited || deactivated {
            new_keys[i] = buf;
            keys_changed = true;
            keys_committed |= deactivated;
        }
        ui.same_line();
        if ui.small_button(format!("-##pe_tkey_rm_{i}")) {
            remove_idx = Some(i);
        }

        let display_key = new_keys[i].trim();
        if display_key.is_empty() {
            ui.text_disabled("Enter a registered entity key.");
        } else if !known_entity_keys.iter().any(|known| known == display_key) {
            unresolved_keys.push(display_key.to_owned());
            ui.text_colored(
                WARNING_TEXT_COLOR,
                format!("Unresolved key: {display_key}"),
            );
        }
    }
    if let Some(idx) = remove_idx {
        new_keys.remove(idx);
        keys_changed = true;
        keys_committed = true;
    }
    if ui.button("Add Template##pe_tkey_add") {
        new_keys.push(String::new());
        keys_changed = true;
    }
    if !unresolved_keys.is_empty() {
        ui.text_colored(
            WARNING_TEXT_COLOR,
            format!(
                "Warning: unresolved template keys will be skipped on apply: {}",
                unresolved_keys.join(", ")
            ),
        );
    }
    if keys_changed {
        p.template_keys = Some(new_keys);
        if keys_committed {
            p.commit = true;
        }
    }
}

pub(crate) fn commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingParticleEmitter,
) {
    if p.remove {
        ctx.commands
            .trigger(RemoveParticleEmitterRequested { entity });
    } else if p.commit {
        if let Some(ref pe) = snap.particle_emitter {
            ctx.commands.trigger(UpdateParticleEmitterRequested {
                entity,
                template_keys: p
                    .template_keys
                    .as_ref()
                    .unwrap_or(&pe.template_keys)
                    .clone(),
                shape: p.shape.unwrap_or(pe.shape),
                shape_rect_w: p.shape_rect_w.unwrap_or(pe.shape_rect_w),
                shape_rect_h: p.shape_rect_h.unwrap_or(pe.shape_rect_h),
                offset: [
                    p.offset_x.unwrap_or(pe.offset[0]),
                    p.offset_y.unwrap_or(pe.offset[1]),
                ],
                particles_per_emission: p
                    .particles_per_emission
                    .unwrap_or(pe.particles_per_emission),
                emissions_per_second: p
                    .emissions_per_second
                    .unwrap_or(pe.emissions_per_second),
                emissions_remaining: p.emissions_remaining.unwrap_or(pe.emissions_remaining),
                arc_min_deg: p.arc_min.unwrap_or(pe.arc_min_deg),
                arc_max_deg: p.arc_max.unwrap_or(pe.arc_max_deg),
                speed_min: p.speed_min.unwrap_or(pe.speed_min),
                speed_max: p.speed_max.unwrap_or(pe.speed_max),
                ttl_kind: p.ttl_kind.unwrap_or(pe.ttl_kind),
                ttl_fixed: p.ttl_fixed.unwrap_or(pe.ttl_fixed),
                ttl_min: p.ttl_min.unwrap_or(pe.ttl_min),
                ttl_max: p.ttl_max.unwrap_or(pe.ttl_max),
            });
        } else {
            warn!(
                "consume_particle_emitter_commit: snapshot missing ParticleEmitter for entity {}",
                entity.to_bits()
            );
        }
    }
}
