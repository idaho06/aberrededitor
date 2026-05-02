//! Marker component that identifies entities owned by the current map.
//!
//! All entities spawned from a `.map` file or created by the editor are tagged with
//! [`MapEntity`]. This scopes queries to user-placed content and excludes internal editor
//! entities (camera, shader nodes, intro sprites). It also acts as a filter during
//! serialisation — only `MapEntity` entities are saved by `save_map_observer`.
use aberredengine::bevy_ecs;
use aberredengine::bevy_ecs::prelude::Component;

/// Zero-size marker for entities that belong to the current map.
///
/// Spawn with `commands.spawn((MapEntity, ...))`. Query with `With<MapEntity>` to exclude
/// internal editor entities from map-level operations.
#[derive(Component, Debug)]
pub struct MapEntity;
