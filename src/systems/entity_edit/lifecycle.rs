use super::{
    AddComponentRequested, CloneEntityRequested, CreateBlankEntityRequested,
    CreateColliderEntityRequested, RemoveEntityRequested,
};
use crate::components::map_entity::MapEntity;
use crate::components::serialized_lua_setup::SerializedLuaSetup;
use crate::editor_types::ComponentKind;
use crate::systems::entity_selector::{apply_selection, clear_selector_state};
use crate::systems::utils::entity_label;
use aberredengine::bevy_ecs::prelude::{Commands, NonSend, On, Query, Res, ResMut};
use aberredengine::components::animation::Animation;
use aberredengine::components::boxcollider::BoxCollider;
use aberredengine::components::dynamictext::DynamicText;
use aberredengine::components::group::Group;
use aberredengine::components::mapposition::MapPosition;
use aberredengine::components::particleemitter::ParticleEmitter;
use aberredengine::components::persistent::Persistent;
use aberredengine::components::rotation::Rotation;
use aberredengine::components::scale::Scale;
use aberredengine::components::sprite::Sprite;
use aberredengine::components::tint::Tint;
use aberredengine::components::ttl::Ttl;
use aberredengine::components::zindex::ZIndex;
use aberredengine::raylib::prelude::{Color, Vector2};
use aberredengine::resources::appstate::AppState;
use aberredengine::resources::fontstore::FontStore;
use aberredengine::resources::texturestore::TextureStore;
use aberredengine::resources::worldsignals::WorldSignals;
use log::debug;
use std::sync::Arc;

pub fn create_blank_entity_observer(
    trigger: On<CreateBlankEntityRequested>,
    mut commands: Commands,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let event = trigger.event();
    let entity = commands
        .spawn((MapEntity, MapPosition::new(event.x, event.y)))
        .id();
    apply_selection(
        entity,
        &entity_label(entity, None, None),
        None,
        &mut world_signals,
        &mut app_state,
        &mut commands,
    );
    debug!(
        "create_blank_entity_observer: spawned entity {} at ({:.3}, {:.3})",
        entity.to_bits(),
        event.x,
        event.y
    );
}

pub fn create_collider_entity_observer(
    trigger: On<CreateColliderEntityRequested>,
    mut commands: Commands,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let event = trigger.event();
    let entity = commands
        .spawn((
            MapEntity,
            MapPosition::new(event.x, event.y),
            BoxCollider::new(event.width, event.height),
        ))
        .id();
    apply_selection(
        entity,
        &entity_label(entity, None, None),
        None,
        &mut world_signals,
        &mut app_state,
        &mut commands,
    );
    debug!(
        "create_collider_entity_observer: spawned {:?} at ({:.1},{:.1}) size {:.1}x{:.1}",
        entity, event.x, event.y, event.width, event.height,
    );
}

pub fn clone_entity_observer(
    trigger: On<CloneEntityRequested>,
    mut commands: Commands,
    source_query: Query<(Option<&Group>, Option<&Persistent>)>,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let event = trigger.event();
    let mut src = commands.entity(event.entity);
    let mut ec = src.clone_and_spawn();
    ec.insert(MapPosition::new(event.x, event.y));
    let cloned = ec.id();
    let (group, persistent) = source_query.get(event.entity).unwrap_or((None, None));
    apply_selection(
        cloned,
        &entity_label(cloned, group, persistent),
        None,
        &mut world_signals,
        &mut app_state,
        &mut commands,
    );
    debug!(
        "clone_entity_observer: cloned entity {} → {} at ({:.3}, {:.3})",
        event.entity.to_bits(),
        cloned.to_bits(),
        event.x,
        event.y
    );
}

pub fn remove_entity_observer(
    trigger: On<RemoveEntityRequested>,
    mut commands: Commands,
    mut world_signals: ResMut<WorldSignals>,
    mut app_state: ResMut<AppState>,
) {
    let entity = trigger.event().entity;
    super::remove_entity_registrations(&mut world_signals, entity);
    commands.entity(entity).despawn();
    clear_selector_state(&mut world_signals, &mut app_state);
}

pub fn add_component_observer(
    trigger: On<AddComponentRequested>,
    textures: Res<TextureStore>,
    fonts: NonSend<FontStore>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = event.entity;
    let mut ec = commands.entity(entity);
    match event.kind {
        ComponentKind::MapPosition => {
            ec.insert(MapPosition::new(0.0, 0.0));
        }
        ComponentKind::ZIndex => {
            ec.insert(ZIndex(0.0));
        }
        ComponentKind::Group => {
            ec.insert(Group::new(""));
        }
        ComponentKind::Rotation => {
            ec.insert(Rotation::default());
        }
        ComponentKind::Scale => {
            ec.insert(Scale::default());
        }
        ComponentKind::Sprite => {
            let tex_key: Arc<str> =
                Arc::from(textures.map.keys().min().map(|k| k.as_str()).unwrap_or(""));
            ec.insert(Sprite {
                tex_key,
                width: 32.0,
                height: 32.0,
                offset: Vector2::zero(),
                origin: Vector2::zero(),
                flip_h: false,
                flip_v: false,
            });
        }
        ComponentKind::BoxCollider => {
            ec.insert(BoxCollider::new(32.0, 32.0));
        }
        ComponentKind::Animation => {
            ec.insert(Animation::new(""));
        }
        ComponentKind::Ttl => {
            ec.insert(Ttl::new(5.0));
        }
        ComponentKind::Persistent => {
            ec.insert(Persistent);
        }
        ComponentKind::Tint => {
            ec.insert(Tint::default());
        }
        ComponentKind::LuaSetup => {
            ec.insert(SerializedLuaSetup::new(""));
        }
        ComponentKind::DynamicText => {
            let font_key = fonts
                .meta
                .keys()
                .min()
                .map(|k| k.as_str())
                .unwrap_or("")
                .to_owned();
            ec.insert(DynamicText::new("", font_key, 16.0, Color::WHITE));
        }
        ComponentKind::ParticleEmitter => {
            ec.insert(ParticleEmitter::default());
        }
    }
    debug!(
        "add_component_observer: added {:?} to entity {}",
        event.kind,
        entity.to_bits()
    );
    super::refresh_inspector(&mut commands, entity);
}
