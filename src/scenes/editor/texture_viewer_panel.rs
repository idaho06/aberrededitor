use crate::signals as sig;
use aberredengine::imgui;
use aberredengine::resources::fontstore::FontStore;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;

const CHECKER_TILE_SIZE: f32 = 16.0;
const CHECKER_LIGHT: [f32; 4] = [0.40, 0.40, 0.40, 1.0];
const CHECKER_DARK: [f32; 4] = [0.24, 0.24, 0.24, 1.0];

struct ResolvedViewerTexture {
    texture_id: imgui::TextureId,
    width: i32,
    height: i32,
    path: String,
}

pub(super) fn open_texture_viewer(signals: &mut WorldSignals, kind: &str, key: &str) {
    signals.set_string(sig::TEXTURE_VIEWER_SOURCE_KIND, kind);
    signals.set_string(sig::TEXTURE_VIEWER_SOURCE_KEY, key);
    signals.set_flag(sig::UI_TEXTURE_VIEWER_OPEN);
}

pub(super) fn draw_texture_viewer(
    ui: &imgui::Ui,
    signals: &mut WorldSignals,
    textures: &TextureStore,
    fonts: &FontStore,
) {
    if !signals.has_flag(sig::UI_TEXTURE_VIEWER_OPEN) {
        return;
    }

    let mut window_open = true;
    let source_kind = signals
        .get_string(sig::TEXTURE_VIEWER_SOURCE_KIND)
        .map(String::as_str);
    let source_key = signals
        .get_string(sig::TEXTURE_VIEWER_SOURCE_KEY)
        .map(String::as_str)
        .unwrap_or("");

    ui.window("Texture Viewer")
        .size([720.0, 540.0], imgui::Condition::FirstUseEver)
        .opened(&mut window_open)
        .build(|| {
            if source_key.is_empty() {
                ui.text_disabled("Click a texture or font preview to inspect it here.");
                return;
            }

            let Some(view) = resolve_viewer_texture(source_kind, source_key, textures, fonts) else {
                ui.text_disabled("The selected preview is no longer available.");
                ui.text_disabled(format!("Key: {source_key}"));
                return;
            };

            ui.text(format!("{} x {} px", view.width, view.height));
            ui.text_disabled(view.path.as_str());
            ui.separator();

            ui.child_window("##texture_viewer_scroll")
                .size([0.0, 0.0])
                .flags(imgui::WindowFlags::HORIZONTAL_SCROLLBAR)
                .build(|| {
                    draw_checkerboard(ui, view.width as f32, view.height as f32);
                    imgui::Image::new(view.texture_id, [view.width as f32, view.height as f32])
                        .build(ui);
                });
        });

    if !window_open {
        signals.take_flag(sig::UI_TEXTURE_VIEWER_OPEN);
    }
}

fn draw_checkerboard(ui: &imgui::Ui, width: f32, height: f32) {
    let origin = ui.cursor_screen_pos();
    let draw_list = ui.get_window_draw_list();

    let rows = (height / CHECKER_TILE_SIZE).ceil() as i32;
    let cols = (width / CHECKER_TILE_SIZE).ceil() as i32;

    for row in 0..rows {
        for col in 0..cols {
            let min_x = origin[0] + col as f32 * CHECKER_TILE_SIZE;
            let min_y = origin[1] + row as f32 * CHECKER_TILE_SIZE;
            let max_x = (min_x + CHECKER_TILE_SIZE).min(origin[0] + width);
            let max_y = (min_y + CHECKER_TILE_SIZE).min(origin[1] + height);
            let color = if (row + col) % 2 == 0 {
                CHECKER_LIGHT
            } else {
                CHECKER_DARK
            };

            draw_list
                .add_rect([min_x, min_y], [max_x, max_y], color)
                .filled(true)
                .build();
        }
    }
}

fn resolve_viewer_texture(
    source_kind: Option<&str>,
    source_key: &str,
    textures: &TextureStore,
    fonts: &FontStore,
) -> Option<ResolvedViewerTexture> {
    match source_kind {
        Some(sig::TEXTURE_VIEWER_SOURCE_TEXTURE) => {
            let texture = textures.map.get(source_key)?;
            let tex = texture.as_ref();
            Some(ResolvedViewerTexture {
                texture_id: imgui::TextureId::from(tex as *const _ as usize),
                width: tex.width,
                height: tex.height,
                path: textures
                    .paths
                    .get(source_key)
                    .cloned()
                    .unwrap_or_else(|| "(path unavailable)".to_owned()),
            })
        }
        Some(sig::TEXTURE_VIEWER_SOURCE_FONT) => {
            let font = fonts.get(source_key)?;
            let meta = fonts.meta.get(source_key)?;
            Some(ResolvedViewerTexture {
                texture_id: imgui::TextureId::from(&font.texture as *const _ as usize),
                width: font.texture.width,
                height: font.texture.height,
                path: meta.path.clone(),
            })
        }
        Some(sig::TEXTURE_VIEWER_SOURCE_ANIMATION) => {
            let texture = textures.map.get(source_key)?;
            let tex = texture.as_ref();
            Some(ResolvedViewerTexture {
                texture_id: imgui::TextureId::from(tex as *const _ as usize),
                width: tex.width,
                height: tex.height,
                path: textures
                    .paths
                    .get(source_key)
                    .cloned()
                    .unwrap_or_else(|| "(path unavailable)".to_owned()),
            })
        }
        _ => None,
    }
}