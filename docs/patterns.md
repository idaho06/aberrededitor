# Design Patterns

Seven recurring patterns appear throughout the codebase. Recognising them makes the code
predictable; using them correctly when extending keeps the codebase coherent.

---

## 1. Signal bus (WorldSignals)

**Problem:** The GUI callback and ECS observers cannot share mutable references. They need a
neutral channel to pass flags, values, and entity references between each other.

**Solution:** `WorldSignals` is a typed key-value store held as a Bevy resource. The GUI callback
receives `&mut WorldSignals`; observers receive `ResMut<WorldSignals>`. Both can read and write.

**How to recognise it:** Calls to `signals.set_flag(...)`, `signals.take_flag(...)`,
`signals.get_string(...)`, `ctx.world_signals.get_entity(...)`.

**How to use in new code:**

1. Add a constant to `src/signals.rs`:
   ```rust
   pub const MY_FEATURE_FLAG: &str = "gui:action:myfeature";
   ```
2. In the GUI panel, set the flag when the user clicks a button:
   ```rust
   if ui.button("Do Thing") { signals.set_flag(sig::MY_FEATURE_FLAG); }
   ```
3. In `editor_update()`, consume the flag and trigger an event:
   ```rust
   if ctx.world_signals.take_flag(sig::MY_FEATURE_FLAG) {
       ctx.commands.trigger(MyFeatureRequested { ... });
   }
   ```

**Key rule:** All signal key strings must be constants in `signals.rs`. Never write raw string
literals for keys elsewhere in the codebase.

---

## 2. Observer dispatch

**Problem:** Component mutations should happen in ECS context (with full query access), but they
are initiated from the GUI or another ECS system.

**Solution:** Define a Bevy `Event` struct carrying the mutation parameters. Trigger it with
`commands.trigger(MyEvent { ... })`. Register an observer function with `.add_observer()` in
`main.rs`. The observer runs once per event, after the current command queue is flushed.

**How to recognise it:** `#[derive(Event)]` on a struct, `pub fn foo_observer(trigger: On<FooEvent>, ...)`,
and a `.add_observer(foo_observer)` line in `main.rs`.

**How to use in new code:**

In `src/systems/entity_edit.rs` (or a new file for a new domain):
```rust
#[derive(Event)]
pub struct UpdateMyComponentRequested {
    pub entity: Entity,
    pub value: f32,
}

pub fn update_my_component_observer(
    trigger: On<UpdateMyComponentRequested>,
    mut query: Query<&mut MyComponent>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    if let Ok(mut comp) = query.get_mut(ev.entity) {
        comp.value = ev.value;
        commands.trigger(InspectEntityRequested { entity: ev.entity });
    }
}
```

In `src/main.rs`:
```rust
.add_observer(systems::entity_edit::update_my_component_observer)
```

**Key rule:** Always re-trigger `InspectEntityRequested` at the end of a component-mutation
observer so the GUI snapshot is refreshed immediately.

---

## 3. AppState mutex cache

**Problem:** The GUI callback receives `&AppState` (immutable). It cannot call ECS queries to
fetch entity data, group lists, or store contents.

**Solution:** A per-frame system (or observer) writes data into a `Mutex<T>` stored inside
`AppState`. The GUI acquires the read lock without needing mutation.

**How to recognise it:** `pub type FooMutex = Mutex<FooCache>;` in a systems file; a
`foo_sync_system` that calls `app_state.get::<FooMutex>()` and populates it; GUI code that
calls `app_state.get::<FooMutex>().unwrap().lock().unwrap()`.

**How to use in new code:**

Define the cache type and alias:
```rust
pub struct MyCache { pub items: Vec<String> }
pub type MyCacheMutex = Mutex<MyCache>;
```

Insert it in `load_assets()`:
```rust
app_state.insert(MyCacheMutex::new(MyCache { items: vec![] }));
```

Write a sync system:
```rust
pub fn my_cache_sync_system(my_data: Res<MyData>, app_state: ResMut<AppState>) {
    if my_data.is_changed() {
        if let Some(mutex) = app_state.get::<MyCacheMutex>() {
            let mut cache = mutex.lock().unwrap();
            cache.items = my_data.items.iter().map(|s| s.clone()).collect();
        }
    }
}
```

Register in `main.rs`: `.add_system(systems::my_module::my_cache_sync_system)`

In the GUI callback:
```rust
if let Some(mutex) = app_state.get::<MyCacheMutex>() {
    let cache = mutex.lock().unwrap();
    for item in &cache.items { ui.text(item); }
}
```

---

## 4. PendingEditState dirty encoding

**Problem:** The entity editor shows many fields at once. The user edits one field, then clicks
"Apply". We need to know which fields changed and which to leave at their snapshot value.

**Solution:** `PendingEditState` uses `Option<T>` as a dirty flag: `None` means "unedited, use
the snapshot value"; `Some(v)` means "user changed this to v". Alongside each group of fields is
a `commit_xyz: bool` that the GUI sets when the user triggers a commit action.

**How to recognise it:** Fields like `pub pos_x: Option<f32>`, `pub commit_position: bool` in
`PendingEditState`; code in `commit.rs` that reads `p.pos_x.unwrap_or(snap_x)`.

**How to use in new code** (adding a new editable component field):

In `pending_state.rs`:
```rust
// MyComponent
pub my_value: Option<f32>,
pub commit_my: bool,
```

In `entity_editor_panel.rs`, update the pending field when the widget changes:
```rust
if ui.input_float("Value", &mut display_value).build() {
    pending.my_value = Some(display_value);
}
if ui.button("Apply") { pending.commit_my = true; }
```

In `commit.rs`, handle the commit:
```rust
if p.commit_my {
    consume_my_commit(ctx, entity, &snapshot, &p);
}

fn consume_my_commit(ctx: &mut GameCtx, entity: Entity, snapshot: &ComponentSnapshot, p: &PendingEditState) {
    let snap_val = snapshot.my_value.unwrap_or(0.0);
    ctx.commands.trigger(UpdateMyComponentRequested {
        entity,
        value: p.my_value.unwrap_or(snap_val),
    });
}
```

Also add `|| self.commit_my` to `any_commit()`.

**Key rule:** Reset with `*self = Self::default()` after every commit and on selection change.
Stale pending state will overwrite values the user did not intend to change.

---

## 5. ComponentSnapshot serialization

**Problem:** The entity editor needs consistent access to all of an entity's component data across
multiple ImGui frames, but ECS queries cannot run inside the GUI callback.

**Solution:** `entity_inspect_observer` runs once in response to `InspectEntityRequested`. It
reads all relevant components in a single ECS query and stores a plain-Rust `ComponentSnapshot`
struct into `AppState`. The GUI callback reads this snapshot each frame.

**How to recognise it:** `ComponentSnapshot` in `src/editor_types.rs`; `app_state.insert(snapshot)`
in `entity_inspector.rs`; `app_state.get::<ComponentSnapshot>()` in GUI panels.

**How to use when adding a new component to the inspector:**

1. Add a snapshot field to `ComponentSnapshot` in `editor_types.rs`:
   ```rust
   pub my_component: Option<MySnapshot>,
   ```
2. Add the snapshot struct if needed:
   ```rust
   #[derive(Clone)]
   pub struct MySnapshot { pub value: f32 }
   ```
3. In `entity_inspect_observer`, populate it:
   ```rust
   my_component: my_comp.map(|c| MySnapshot { value: c.value }),
   ```
4. In the entity editor panel, read it:
   ```rust
   if let Some(ref my_snap) = snapshot.my_component { ... }
   ```

**Key rule:** `ComponentSnapshot` stores `entity_bits: u64` instead of `Entity` because `Entity`
cannot cross the `AppState` boundary safely. Reconstruct with `Entity::from_bits(snapshot.entity_bits)`.

---

## 6. MapEntity marker

**Problem:** The ECS world contains both user-placed map entities and internal editor entities
(camera, shader nodes, intro screen sprites). Queries for "all entities" would catch internal ones.

**Solution:** Every entity that belongs to the map is tagged with `MapEntity` (a zero-size marker
`Component`). All editor queries that should only touch map entities include `With<MapEntity>`.

**How to recognise it:** `#[derive(Component)] pub struct MapEntity;` in
`src/components/map_entity.rs`; `Query<Entity, With<MapEntity>>` in `map_ops.rs`.

**How to use in new code:**

When spawning a user-placed entity:
```rust
commands.spawn((MapEntity, MapPosition::new(x, y), ...));
```

When querying only map entities:
```rust
fn my_observer(query: Query<&MyComponent, With<MapEntity>>) { ... }
```

When saving: only `MapEntity` entities are serialized by `save_map_observer`. The marker doubles
as a filter for serialization.

---

## 7. Async dialog bridge

**Problem:** Native file dialogs are initiated from user actions, but opening them directly in
`editor_update()` blocks the frame loop. The GUI callback also cannot own them because it should
stay read-mostly and signal-driven.

**Solution:** Route dialog opening through a small bridge module that stores one in-flight dialog
receiver in `AppState`, awaits the dialog off the frame loop, and then re-emits the completion as
the same ECS events the old synchronous flow used.

**How to recognise it:** `AsyncFileDialogRequest`, `request_async_dialog()`,
`AsyncFileDialogMutex`, and `poll_async_dialogs()` in `src/systems/file_dialogs.rs`.

**How to use in new code:**

1. Collect all non-path parameters before opening the dialog.
    ```rust
    let key = ctx
         .world_signals
         .get_string(sig::TEX_ADD_KEY_BUF)
         .map(|s| s.to_owned())
         .unwrap_or_default();
    ```
2. In `editor_update()`, enqueue a dialog request instead of opening `rfd::FileDialog` inline.
    ```rust
    if !key.is_empty() {
        request_async_dialog(&ctx.app_state, AsyncFileDialogRequest::AddTexture { key });
    }
    ```
    `request_async_dialog` returns `()`. If another dialog is already in flight it silently ignores the call (logs at debug level).
3. In `src/systems/file_dialogs.rs`, add a request variant and a matching result variant if the
    existing ones do not fit.
4. Extend `build_dialog_task()` to create the correct `rfd::AsyncFileDialog` future.
5. Extend `poll_async_dialogs()` to normalize the path with `to_relative()` and trigger the
    downstream event that already owns the real mutation.

**Key rules:**

- Keep dialogs as orchestration only. Do not load assets or mutate stores inside the bridge.
- Convert absolute paths to relative paths in the completion path before triggering observers.
- Treat cancel as a no-op.
- Assume only one native dialog may be open at a time unless the bridge design changes.
