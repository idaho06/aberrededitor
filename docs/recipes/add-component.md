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

## 5. Add Update and Remove events

In `src/systems/entity_edit.rs`:

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

## 6. Add observer handlers

Still in `src/systems/entity_edit.rs`, use the macros for simple cases:

```rust
// For the update observer (if you need access to the full component):
pub fn update_my_component_observer(
    trigger: On<UpdateMyComponentRequested>,
    mut query: Query<&mut MyComponent>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    if let Ok(mut comp) = query.get_mut(ev.entity) {
        comp.value = ev.value;
        comp.label = ev.label.clone();
    }
    commands.trigger(InspectEntityRequested { entity: ev.entity });
}

// For the remove observer, use the macro:
component_remove_observer!(
    remove_my_component_observer,
    RemoveMyComponentRequested,
    MyComponent,
    "MyComponent"
);
```

Always call `refresh_inspector` (or trigger `InspectEntityRequested` directly) at the end of
mutation observers so the GUI snapshot refreshes.

## 7. Register observers in main.rs

In `src/main.rs`:

```rust
.add_observer(systems::entity_edit::update_my_component_observer)
.add_observer(systems::entity_edit::remove_my_component_observer)
```

## 8. Handle AddComponentRequested

In the `add_component_observer` match in `src/systems/entity_edit.rs`:

```rust
ComponentKind::MyComponent => {
    commands.entity(entity).insert(MyComponent { value: 0.0, label: String::new() });
}
```

## 9. Add pending fields

In `src/scenes/editor/pending_state.rs`:

```rust
// MyComponent
pub my_value: Option<f32>,
pub my_label: Option<String>,
pub commit_my: bool,
pub remove_my: bool,
```

Add `|| self.commit_my || self.remove_my` to `any_commit()`.

## 10. Wire the commit in commit.rs

In `src/scenes/editor/commit.rs`, inside `consume_entity_editor_commits`:

```rust
if p.remove_my {
    ctx.commands.trigger(RemoveMyComponentRequested { entity });
} else if p.commit_my {
    consume_my_commit(ctx, entity, &snapshot, &p);
}
```

Add the helper:

```rust
fn consume_my_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot, p: &PendingEditState) {
    let Some(ref snap) = snapshot.my_component else { return; };
    ctx.commands.trigger(UpdateMyComponentRequested {
        entity,
        value: p.my_value.unwrap_or(snap.value),
        label: p.my_label.clone().unwrap_or_else(|| snap.label.clone()),
    });
}
```

## 11. Add the GUI widget block

In `src/scenes/editor/entity_editor_panel.rs`, inside the per-component section pattern:

```rust
if let Some(ref my_snap) = snapshot.my_component {
    ui.separator();
    ui.text("MyComponent");
    ui.same_line();
    if ui.small_button("X##remove_my") {
        pending.remove_my = true;
    }

    let mut value = pending.my_value.unwrap_or(my_snap.value);
    if draw_float_input(ui, "Value##my", &mut value, 1.0) {
        pending.my_value = Some(value);
        pending.commit_my = true;
    }
}
```

For the "Add Component" combo, `ComponentKind::MyComponent` will appear automatically once you
added it to the enum in step 1 (assuming the combo list is derived from `ComponentKind`).
Check `entity_editor_panel.rs` for how existing variants are listed.

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

If the component stores only a single `String` value (like `LuaSetup` or `DynamicText` for its
script/text content), the snapshot can be `Option<String>` instead of a dedicated snapshot struct:

```rust
// In ComponentSnapshot:
pub lua_setup: Option<String>,
```

The pending state is similarly just `Option<String>`, and the inspector widget is a plain
`input_text` that writes back into the pending field on change. See `LuaSetup` in
`entity_inspector.rs`, `entity_editor_panel.rs`, `pending_state.rs`, and `entity_edit.rs` as
the reference implementation for this simpler pattern.

## Verification

- `cargo check` passes
- Run the editor, select an entity, click "Add Component" → "MyComponent" appears in the list
- After adding, the inspector shows the MyComponent section
- Edit a value and click Apply → component updates in ECS
- Save and reload the map → component persists
