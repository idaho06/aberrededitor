# Design Patterns

Eight recurring patterns appear throughout the codebase. Recognising them makes the code
predictable; using them correctly when extending keeps the codebase coherent.

---

## 1. Signal bus (WorldSignals / SignalSnapshot / SignalIntents)

**Problem:** The GUI callback (render thread) and ECS observers (logic thread) cannot share
mutable references across threads. They need a neutral channel to pass flags, values, and entity
references between each other.

**Solution:** `WorldSignals` is the live, logic-side typed key-value store (a Bevy resource);
observers and `editor_update()` read/write it via `ResMut<WorldSignals>` exactly as before. The GUI
callback, on the render thread, never sees `WorldSignals` directly — it receives a one-tick-stale
`&SignalSnapshot` (plain field access, no getters: `signals.flags.contains(k)`,
`signals.scalars.get(k)`) and writes through `&mut SignalIntents` (`intents.set_flag(k)`,
`intents.set_scalar(k, v)`, `intents.clear_flag(k)`), applied to `WorldSignals` at the start of the
next logic tick.

**How to recognise it:** Logic-side: `ctx.world_signals.take_flag(...)`,
`ctx.world_signals.set_string(...)`. GUI-side: `signals.flags.contains(sig::MY_FLAG)`,
`intents.set_flag(sig::MY_FLAG)`.

**How to use in new code:**

1. Add a constant to `src/signals.rs`:

   ```rust
   pub const MY_FEATURE_FLAG: &str = "gui:action:myfeature";
   ```

2. In the GUI panel, queue the flag when the user clicks a button:

   ```rust
   if ui.button("Do Thing") { intents.set_flag(sig::MY_FEATURE_FLAG); }
   ```

3. In `editor_update()` (logic thread, one tick later), consume the flag and trigger an event:

   ```rust
   if ctx.world_signals.take_flag(sig::MY_FEATURE_FLAG) {
       ctx.commands.trigger(MyFeatureRequested { ... });
   }
   ```

**Key rules:** All signal key strings must be constants in `signals.rs`. Never write raw string
literals for keys elsewhere in the codebase. Any panel that sets a flag and expects to see the
effect the *same* frame will observe a one-tick delay now — this is fine for open-flag-gated
panels (the common case) but audit any read-after-write pattern you add.

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

In `src/systems/entity_edit/mod.rs` (with the observer body in the matching concern submodule):

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

**Key rule:** Any observer that changes inspector-visible state should re-trigger
`InspectEntityRequested` (or call the shared helper that does so) before it returns, so the GUI
snapshot stays authoritative.

---

## 3. AppState `Arc<Mutex<T>>` cache

**Problem:** The GUI callback receives `&AppState` — a **clone** taken every frame as part of the
render thread's `DrawableSnapshot` (`AppState::insert` requires `T: Clone`). It cannot call ECS
queries to fetch entity data, group lists, or store contents, and a plain `Mutex<T>` wouldn't
survive being cloned into the snapshot with live data intact.

**Solution:** A per-frame system (or observer) writes data into an `Arc<Mutex<T>>` stored inside
`AppState`. The `Arc` is what makes the type `Clone` — cloning it just bumps the refcount, so the
render-side snapshot's copy and the logic-side original share the same `Mutex`-guarded data. The
GUI acquires the read lock without needing mutation.

**How to recognise it:** `pub type FooMutex = Arc<Mutex<FooCache>>;` in a systems file; a
`foo_sync_system` that calls `app_state.get::<FooMutex>()` and populates it; GUI code that
calls `app_state.get::<FooMutex>().unwrap().lock().unwrap()`.

**How to use in new code:**

Define the cache type and alias:

```rust
pub struct MyCache { pub items: Vec<String> }
pub type MyCacheMutex = std::sync::Arc<std::sync::Mutex<MyCache>>;
```

Insert it in `load_assets()`:

```rust
app_state.insert(MyCacheMutex::new(std::sync::Mutex::new(MyCache { items: vec![] })));
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

**Key rule:** GUI (render thread) and observers/systems (logic thread) now genuinely contend on
these mutexes. Keep lock scopes short — never hold a lock across a whole panel draw — and never
lock two different `AppState` mutexes in inconsistent orders across call sites.

---

## 4. PendingEditState dirty encoding

**Problem:** The entity editor shows many fields at once. The user edits one field, then clicks
"Apply". We need to know which fields changed and which to leave at their snapshot value.

**Solution:** `PendingEditState` is now a thin aggregate of per-component pending sub-structs.
Each `Pending*` type lives next to its UI/commit logic in `src/scenes/editor/components/`.
Inside those sub-structs, `Option<T>` still means "unchanged vs edited", but the aggregate no
longer stores flat `pos_x` / `commit_*` fields itself. Instead, `PendingEditState::any_commit()`
delegates to `is_dirty()` on each sub-struct plus a handful of entity-level action flags.

**How to recognise it:** `PendingEditState` contains fields like `transform: PendingTransform`
and `sprite: PendingSprite` in `pending_state.rs`; each component module defines its own
`PendingMyComponent`; `commit.rs` delegates with calls like
`components::transform::commit(ctx, entity, &snapshot, &p.transform)`.

**How to use in new code** (adding a new editable component):

In `src/scenes/editor/components/my_component.rs`:

```rust
#[derive(Default, Clone)]
pub(crate) struct PendingMyComponent {
    pub value: Option<f32>,
    pub remove: bool,
}

impl PendingMyComponent {
    pub(crate) fn is_dirty(&self) -> bool {
        self.value.is_some() || self.remove
    }
}
```

In `pending_state.rs`, add the sub-struct to the aggregate and wire `any_commit()`:

```rust
pub my_component: PendingMyComponent,
```

```rust
|| self.my_component.is_dirty()
```

In `entity_editor_panel.rs`, pass the sub-struct to the component-local draw helper:

```rust
components::my_component::draw_section(ui, &snap, &mut p.my_component);
```

In `commit.rs`, delegate to the component-local commit helper:

```rust
components::my_component::commit(ctx, entity, &snapshot, &p.my_component);
```

**Key rule:** Reset with `*self = Self::default()` after every commit and on selection change.
In practice this happens through `clear_entity_editor_pending()` after commit and from the
selection-change system when the inspected entity changes. Stale pending state will otherwise
leak across selections or overwrite values the user did not intend to change.

---

## 5. Selector caches and multi-selection state

**Problem:** The editor has several selection entry points (click, rectangle, group, registry),
but the GUI panels need one stable place to read the latest hit list and any bulk-edit buffers.

**Solution:** Normalize every selection source through the observers in
`src/systems/entity_selector.rs`. Store single-selection results in `RenderableSelectorMutex` and
multi-selection results in `MultiEntitySelectionMutex`. Both caches carry a `SelectorSource` so
the GUI can explain where the result set came from, and the multi-selection cache embeds a
`MultiEntityBulkEditState` for move and Z-index modal buffers.

**How to recognise it:** `PickEntitiesAtPointRequested`, `PickEntitiesInRectRequested`,
`SelectGroupRequested`, or `SelectRegisteredEntityRequested` events feeding
`RenderableSelectorCache` / `MultiEntitySelectionCache` in `entity_selector.rs`.

**How to use in new code:**

1. Add a new selection event only if an existing source cannot represent it.
2. Resolve the event into entities inside `entity_selector.rs`, not in the GUI callback.
3. Populate the aligned cache fields (`hits`, `labels`, `corner_sets`, and source metadata)
    under the matching mutex.
4. If the selection can lead to bulk actions, extend `MultiEntityBulkEditState` rather than
    inventing a second temporary store.
5. Trigger `InspectEntityRequested` only for flows that truly collapse to a single selected
    entity.

**Key rule:** Keep the GUI read-only with respect to selection results. Panels may queue a new
selection intent, but they should not mutate the selector caches directly.

---

## 6. ComponentSnapshot serialization

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

**Key rules:**

- `ComponentSnapshot` stores `entity_bits: u64` instead of `Entity` because `Entity` cannot cross
    the `AppState` boundary safely. Reconstruct with `Entity::from_bits(snapshot.entity_bits)`.
- Keep the `entity_inspect_observer` query tuple and snapshot-population code in lockstep with
    the fields you add. This is the only place the GUI's read model is assembled.

---

## 7. MapEntity marker

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

## 8. Async dialog bridge

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
