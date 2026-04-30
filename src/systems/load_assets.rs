use crate::scenes::editor::EditorState;
use crate::scenes::editor::pending_state::{PendingEditState, PendingMutex};
use crate::systems::entity_selector::{RenderableSelectorCache, RenderableSelectorMutex};
use crate::systems::group_selector::{GroupListCache, GroupListMutex};
use crate::systems::template_selector::{TemplateSelectorCache, TemplateSelectorMutex};
use crate::systems::tilemap_load::{PendingLuaSetupLoadMutex, PendingLuaSetupLoadState};
use aberredengine::bevy_ecs::prelude::{Commands, NonSendMut, ResMut};
use aberredengine::raylib::prelude::Color;
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::gameconfig::GameConfig;
use aberredengine::resources::gamestate::{GameStates, NextGameState};
use aberredengine::resources::mapdata::MapData;
use aberredengine::resources::shaderstore::ShaderStore;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::systems::RaylibAccess;
use log::info;

const SHADER_GLITCH: &str = "./assets/shaders/glitch.fs";
const SHADER_FADE: &str = "./assets/shaders/fade.fs";
const TEXTURE_ISOMETRIC: &str = "./assets/textures/aberred_engine_isometric_alpha.png";

/// Called as a Bevy ECS system during the engine setup stage.
/// Loads shaders, textures, and initialises resource stores.
/// The first scene does not start until `NextGameState` is set to `Playing`.
pub fn load_assets(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    mut raylib: RaylibAccess,
    mut next_state: ResMut<NextGameState>,
    mut shaders: NonSendMut<ShaderStore>,
    mut app_state: ResMut<AppState>,
) {
    info!("load_assets: loading editor assets");
    config.background_color = Color::BLACK;

    let (rl, th) = (&mut *raylib.rl, &*raylib.th);

    let mut load_shader = |name: &str, path: &str| {
        let shader = rl.load_shader(th, None, Some(path));
        if shader.is_shader_valid() {
            shaders.add(name, shader);
        } else {
            log::warn!("load_assets: {} shader failed validation", name);
        }
    };
    load_shader("glitch", SHADER_GLITCH);
    load_shader("fade", SHADER_FADE);

    let mut texture_store = TextureStore::new();
    let texture = rl
        .load_texture(th, TEXTURE_ISOMETRIC)
        .expect("Failed to load texture");
    texture_store.insert("aberred_engine_isometric_alpha", texture);
    commands.insert_resource(texture_store);
    commands.insert_resource(MapData::default());
    app_state.insert(RenderableSelectorMutex::new(
        RenderableSelectorCache::default(),
    ));
    app_state.insert(GroupListMutex::new(GroupListCache::default()));
    app_state.insert(TemplateSelectorMutex::new(TemplateSelectorCache::default()));
    app_state.insert(PendingLuaSetupLoadMutex::new(
        PendingLuaSetupLoadState::default(),
    ));
    commands.insert_resource(EditorState::default());
    app_state.insert(PendingMutex::new(PendingEditState::default()));

    next_state.set(GameStates::Playing);
}
