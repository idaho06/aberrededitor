//! One-shot setup system: loads shaders and initial assets, inserts all `AppState` caches.
//!
//! Registered via `EngineBuilder::on_setup`. Runs once during the `Setup` game state before
//! the first scene is entered. Responsible for:
//! - Loading shaders (glitch, fade) used by the intro scene.
//! - Loading the intro logo texture.
//! - Inserting all `Mutex<T>` caches into `AppState` so they are available before any system runs.
//! - Inserting `MapData` and `EditorState` Bevy resources.
//! - Advancing the game state to `Playing` to begin the scene loop.
use crate::scenes::editor::map_properties_panel::MapPropertiesMutex;
use crate::scenes::editor::pending_state::PendingMutex;
use crate::scenes::editor::{EditorState, EditorToolMutex, OverlaySettingsMutex};
use crate::systems::animation_store_sync::AnimationStoreMutex;
use crate::systems::entity_selector::{MultiEntitySelectionMutex, RenderableSelectorMutex};
use crate::systems::file_dialogs::AsyncFileDialogMutex;
use crate::systems::group_selector::GroupListMutex;
use crate::systems::render_prefs::RenderPrefsMutex;
use crate::systems::template_selector::TemplateSelectorMutex;
use crate::systems::tilemap_load::PendingLuaSetupLoadMutex;
use aberredengine::bevy_ecs::prelude::{Commands, MessageWriter, ResMut};
use aberredengine::events::render_assets::RenderAssetCmd;
use aberredengine::raylib::prelude::Color;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::gameconfig::GameConfig;
use aberredengine::resources::gamestate::{GameStates, NextGameState};
use aberredengine::resources::mapdata::MapData;
use aberredengine::resources::texturefilter::TextureFilter;
use log::info;

const SHADER_GLITCH_SRC: &str = include_str!("../../assets/shaders/glitch.fs");
const SHADER_FADE_SRC: &str = include_str!("../../assets/shaders/fade.fs");
const TEXTURE_ISOMETRIC_DATA: &[u8] =
    include_bytes!("../../assets/textures/aberred_engine_isometric_alpha.png");

/// Called as a Bevy ECS system during the engine setup stage.
/// Queues shader/texture loads via `RenderAssetCmd` and initialises resource stores.
/// The first scene does not start until `NextGameState` is set to `Playing`.
pub fn load_assets(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    mut next_state: ResMut<NextGameState>,
    mut app_state: ResMut<AppState>,
    mut asset_cmds: MessageWriter<RenderAssetCmd>,
) {
    info!("load_assets: loading editor assets");
    config.background_color = Color::BLACK;

    asset_cmds.write(RenderAssetCmd::ShaderFromMemory {
        id: "glitch".to_string(),
        vs_src: None,
        fs_src: Some(SHADER_GLITCH_SRC.to_string()),
    });
    asset_cmds.write(RenderAssetCmd::ShaderFromMemory {
        id: "fade".to_string(),
        vs_src: None,
        fs_src: Some(SHADER_FADE_SRC.to_string()),
    });
    asset_cmds.write(RenderAssetCmd::TextureFromMemory {
        id: "aberred_engine_isometric_alpha".to_string(),
        ext: ".png".to_string(),
        bytes: TEXTURE_ISOMETRIC_DATA.to_vec(),
        filter: TextureFilter::Nearest,
    });

    commands.insert_resource(MapData::default());
    app_state.insert(RenderableSelectorMutex::default());
    app_state.insert(MultiEntitySelectionMutex::default());
    app_state.insert(AsyncFileDialogMutex::default());
    app_state.insert(GroupListMutex::default());
    app_state.insert(AnimationStoreMutex::default());
    app_state.insert(TemplateSelectorMutex::default());
    app_state.insert(PendingLuaSetupLoadMutex::default());
    app_state.insert(EditorToolMutex::default());
    app_state.insert(OverlaySettingsMutex::default());
    commands.insert_resource(EditorState::default());
    app_state.insert(PendingMutex::default());
    app_state.insert(MapPropertiesMutex::default());
    app_state.insert(RenderPrefsMutex::new(std::sync::Mutex::new(
        config.pixel_snap_camera,
    )));

    next_state.set(GameStates::Playing);
}
