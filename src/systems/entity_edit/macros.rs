/// Generates an observer function that mutates a single ECS component and refreshes the inspector.
///
/// # Usage
/// ```
/// component_edit_observer!(
///     fn_name, EventType, ComponentType, "ComponentName",
///     |component_var, event_var, entity_var| { /* mutation body */ }
/// );
/// ```
macro_rules! component_edit_observer {
    (
        $fn_name:ident,
        $event:ty,
        $component:ty,
        $component_name:literal,
        |$component_var:ident, $event_var:ident, $entity_var:ident| $body:block
    ) => {
        pub fn $fn_name(
            trigger: aberredengine::bevy_ecs::prelude::On<$event>,
            mut query: aberredengine::bevy_ecs::prelude::Query<&mut $component>,
            mut commands: aberredengine::bevy_ecs::prelude::Commands,
        ) {
            let $event_var = trigger.event();
            let $entity_var = $event_var.entity;
            let Ok(mut $component_var) = query.get_mut($entity_var) else {
                super::warn_missing_component(stringify!($fn_name), $entity_var, $component_name);
                return;
            };
            $body
            super::refresh_inspector(&mut commands, $entity_var);
        }
    };
}

/// Generates an observer function that removes a single ECS component and refreshes the inspector.
///
/// # Usage
/// ```
/// component_remove_observer!(fn_name, EventType, ComponentType, "ComponentName");
/// ```
macro_rules! component_remove_observer {
    ($fn_name:ident, $event:ty, $component:ty, $component_name:literal) => {
        pub fn $fn_name(
            trigger: aberredengine::bevy_ecs::prelude::On<$event>,
            mut commands: aberredengine::bevy_ecs::prelude::Commands,
        ) {
            let entity = trigger.event().entity;
            commands.entity(entity).remove::<$component>();
            log::debug!(
                concat!(
                    stringify!($fn_name),
                    ": removed ",
                    $component_name,
                    " from entity {}"
                ),
                entity.to_bits()
            );
            super::refresh_inspector(&mut commands, entity);
        }
    };
}
