# Recipe: Add a new entity component

Full walkthrough for making a new engine component type visible and editable in the editor.
Follow the steps in order; each builds on the previous.

## 1. Add a ComponentKind variant

In `src/editor_types.rs`, add a variant to the `ComponentKind` enum:

```rust
pub enum ComponentKind {
    // ... existing ...
    MyComponent,
}
```

This is the "add component" dropdown entry. The editor uses this enum when the user picks
"Add Component" from the entity inspector.

## 2. Add a snapshot struct

In `src/editor_types.rs`, add a plain-Rust snapshot type that carries the component's data
across the ECS/GUI boundary:

```rust
#[derive(Clone)]
pub struct MyComponentSnapshot {
    pub value: f32,
    pub label: String,
}
```

Keep it `Clone` — `AppState` stores it by value and the commit path clones it.

## 3. Add an optional field to ComponentSnapshot

In `src/editor_types.rs`, add it to `ComponentSnapshot`:

```rust
pub struct ComponentSnapshot {
    // ... existing fields ...
    pub my_component: Option<MyComponentSnapshot>,
}
```

## 4. Populate the snapshot in the inspector

In `src/systems/entity_inspector.rs`, extend the big `Query` tuple to include your component,
and populate the new field:

```rust
// Add to the query
Option<&MyComponent>,

// Populate
my_component: my_comp.map(|c| MyComponentSnapshot {
    value: c.value,
    label: c.label.clone(),
}),
```

## 5. Create src/scenes/editor/components/my_component.rs

This is the main new file — it co-locates pending state, UI, and commit logic:

```rust
use crate::editor_types::ComponentSnapshot;
use crate::systems::entity_edit::{RemoveMyComponentRequested, UpdateMyComponentRequested};
use aberredengine::bevy_ecs::prelude::Entity;
use aberredengine::imgui;
use aberredengine::systems::GameCtx;
use log::warn;
use super::super::widgets::draw_float_input;

#[derive(Default, Clone)]
pub(crate) struct PendingMyComponent {
    pub value: Option<f32>,
    pub label: Option<String>,
    pub commit: bool,
    pub remove: bool,
}

impl PendingMyComponent {
    pub(crate) fn is_dirty(&self) -> bool {
        self.commit || self.remove
    }
}

pub(crate) fn draw_section(
    ui: &imgui::Ui,
    snap: &ComponentSnapshot,
    p: &mut PendingMyComponent,
) {
    let Some(ref my_snap) = snap.my_component else { return; };
    ui.separator();
    ui.text("MyComponent");
    ui.same_line();
    if ui.button("Del##my") { p.remove = true; }

    if let Some(v) = draw_float_input(ui, "Value##my", p.value.unwrap_or(my_snap.value), 1.0) {
        p.value = Some(v);
        p.commit = true;
    }
}

pub(crate) fn commit(
    ctx: &mut GameCtx,
    entity: Entity,
    snap: &ComponentSnapshot,
    p: &PendingMyComponent,
) {
    if p.remove {
        ctx.commands.trigger(RemoveMyComponentRequested { entity });
    } else if p.commit {
        if let Some(ref my_snap) = snap.my_component {
            ctx.commands.trigger(UpdateMyComponentRequested {
                entity,
                value: p.value.unwrap_or(my_snap.value),
                label: p.label.clone().unwrap_or_else(|| my_snap.label.clone()),
            });
        } else {
            warn!("consume_my_component_commit: snapshot missing for entity {}", entity.to_bits());
        }
    }
}
```

## 6. Add Update and Remove events in entity_edit/mod.rs

In `src/systems/entity_edit/mod.rs`:

```rust
#[derive(Event)]
pub struct UpdateMyComponentRequested {
    pub entity: Entity,
    pub value: f32,
    pub label: String,
}

#[derive(Event)]
pub struct RemoveMyComponentRequested {
    pub entity: Entity,
}
```

## 7. Add observer handlers in the appropriate entity_edit submodule

In `src/systems/entity_edit/visual.rs` (or whichever concern fits), add:

```rust
use super::{RemoveMyComponentRequested, UpdateMyComponentRequested};

component_edit_observer!(
    update_my_component_observer,
    UpdateMyComponentRequested,
    MyComponent,
    "MyComponent",
    |comp, event, entity| {
        comp.value = event.value;
        comp.label = event.label.clone();
        debug!("update_my_component_observer: updated entity {}", entity.to_bits());
    }
);

component_remove_observer!(
    remove_my_component_observer,
    RemoveMyComponentRequested,
    MyComponent,
    "MyComponent"
);
```

Always call `super::refresh_inspector` at the end of `component_edit_observer!` (the macro
does this automatically).

## 8. Register observers in entity_edit/mod.rs register()

In the `register(builder)` function:

```rust
.add_observer(visual::update_my_component_observer)
.add_observer(visual::remove_my_component_observer)
```

## 9. Handle AddComponentRequested in lifecycle.rs

In `src/systems/entity_edit/lifecycle.rs`, in the `add_component_observer` match:

```rust
ComponentKind::MyComponent => {
    commands.entity(entity).insert(MyComponent { value: 0.0, label: String::new() });
}
```

## 10. Wire into PendingEditState aggregate

In `src/scenes/editor/pending_state.rs`, add the sub-struct field:

```rust
pub my_component: PendingMyComponent,
```

The `any_commit()` method calls `is_dirty()` on each sub-struct — add:

```rust
|| self.my_component.is_dirty()
```

## 11. Wire into the components/ registry

In `src/scenes/editor/components/mod.rs`:
```rust
pub(super) mod my_component;
```

In `src/scenes/editor/entity_editor_panel.rs` (inside the scroll area):
```rust
components::my_component::draw_section(ui, &snap, &mut p.my_component);
```

In `src/scenes/editor/commit.rs` (in `consume_entity_editor_commits`):
```rust
components::my_component::commit(ctx, entity, &snapshot, &p.my_component);
```

## 12. Add map serialization

In `src/systems/map_ops.rs`:

**save_map_observer:** Extract the component from the entity and write it to `EntityDef`:
```rust
my_component: my_comp.map(|c| MyComponentEntry { value: c.value, label: c.label.clone() }),
```
`MyComponentEntry` must be defined in the engine's `MapData`/`EntityDef` types.

**load_map_observer:** After spawning the entity, insert the component if the def has it:
```rust
if let Some(ref my_entry) = entity_def.my_component {
    entity_commands.insert(MyComponent { value: my_entry.value, label: my_entry.label.clone() });
}
```

## Shortcut: string-typed components

If the component stores only a single `String` value (like `LuaSetup`), the snapshot can be
`Option<String>` instead of a dedicated snapshot struct:

```rust
// In ComponentSnapshot:
pub lua_setup: Option<String>,
```

The pending state is similarly just `Option<String>`, and the inspector widget is a plain
`input_text`. See `LuaSetup` as the reference implementation.

## Verification

- `cargo check` passes
- Run the editor, select an entity, click "Add Component" → "MyComponent" appears in the list
- After adding, the inspector shows the MyComponent section
- Edit a value → component updates in ECS
- Save and reload the map → component persists
