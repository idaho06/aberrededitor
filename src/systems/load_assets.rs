use aberredengine::bevy_ecs::prelude::{Commands, NonSendMut, ResMut};
use aberredengine::raylib::prelude::Color;
use aberredengine::resources::gameconfig::GameConfig;
use aberredengine::resources::gamestate::{GameStates, NextGameState};
use aberredengine::resources::shaderstore::ShaderStore;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::tilemapstore::TilemapStore;
use aberredengine::systems::RaylibAccess;
use log::info;

// This functions is called as a bevy_ecs system during the setup stage of the engine.
// It's called asynchronously. We use the RaylibAccess resource to load textures, animations, shaders, etc. and we insert them into our custom store resources for later use in our scenes.
// The first scene does not start until the next game state is set to Playing, so we set that at the end of this function.
pub fn load_assets(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    mut raylib: RaylibAccess,
    mut next_state: ResMut<NextGameState>,
    mut shaders: NonSendMut<ShaderStore>,
) {
    info!("load_assets: loading editor assets");
    config.background_color = Color::BLACK;

    let (rl, th) = (&mut *raylib.rl, &*raylib.th);

    let shader = rl.load_shader(th, None, Some("./assets/shaders/glitch.fs"));
    if shader.is_shader_valid() {
        shaders.add("glitch", shader);
    } else {
        log::warn!("load_assets: glitch shader failed validation");
    }
    let shader = rl.load_shader(th, None, Some("./assets/shaders/fade.fs"));
    if shader.is_shader_valid() {
        shaders.add("fade", shader);
    } else {
        log::warn!("load_assets: fade shader failed validation");
    }

    let mut texture_store = TextureStore::new();
    let texture = rl
        .load_texture(th, "./assets/textures/aberred_engine_isometric_alpha.png")
        .expect("Failed to load texture");
    texture_store.insert("aberred_engine_isometric_alpha", texture);
    commands.insert_resource(texture_store);
    commands.insert_resource(TilemapStore::new());

    next_state.set(GameStates::Playing);
}