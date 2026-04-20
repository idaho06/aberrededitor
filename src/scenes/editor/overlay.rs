use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::worldsignals::WorldSignals;

pub(super) fn draw_selection_outline(ui: &imgui::Ui, signals: &WorldSignals) {
    let Some(corners) = signals.get_payload::<[[f32; 2]; 4]>(sig::ES_SELECTION_CORNERS) else {
        return;
    };

    let target_x = signals.get_scalar(sig::CAM_TARGET_X).unwrap_or(0.0);
    let target_y = signals.get_scalar(sig::CAM_TARGET_Y).unwrap_or(0.0);
    let zoom = signals.get_scalar(sig::CAM_ZOOM).unwrap_or(1.0);
    let offset_x = signals.get_scalar(sig::CAM_OFFSET_X).unwrap_or(0.0);
    let offset_y = signals.get_scalar(sig::CAM_OFFSET_Y).unwrap_or(0.0);
    let lb_scale = signals.get_scalar(sig::WIN_SCALE).unwrap_or(1.0);
    let lb_x = signals.get_scalar(sig::WIN_OFFSET_X).unwrap_or(0.0);
    let lb_y = signals.get_scalar(sig::WIN_OFFSET_Y).unwrap_or(0.0);

    let to_screen = |world_x: f32, world_y: f32| -> [f32; 2] {
        let rx = (world_x - target_x) * zoom + offset_x;
        let ry = (world_y - target_y) * zoom + offset_y;
        [rx * lb_scale + lb_x, ry * lb_scale + lb_y]
    };

    let points: Vec<[f32; 2]> = corners
        .iter()
        .map(|&[world_x, world_y]| to_screen(world_x, world_y))
        .collect();

    let color = [1.0_f32, 0.85, 0.0, 1.0];
    let draw_list = ui.get_background_draw_list();
    for index in 0..4 {
        draw_list
            .add_line(points[index], points[(index + 1) % 4], color)
            .thickness(2.0)
            .build();
    }
}
