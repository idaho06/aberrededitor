# Recipe: Add a new asset store

How to add an asset store to the editor — a named collection of assets (textures, fonts,
animations) that the user can create/edit/delete and that persists in the map file.

Use the animation store (`AnimationStore` + animation panel) as the reference implementation.
The steps below mirror that pattern.

## 1. Define the store type

If the store lives in the engine, it's already a Bevy resource. If you need an editor-only store,
define it as a Bevy resource in the appropriate location.

The editor never queries the engine's store directly from the GUI callback — it needs a mirror.

## 2. Define the AppState mirror type

In a new file `src/systems/my_store_sync.rs`:

```rust
use aberredengine::resources::appstate::AppState;
use aberredengine::bevy_ecs::change_detection::DetectChanges;
use aberredengine::bevy_ecs::prelude::{Res, ResMut};
use std::sync::Mutex;

#[derive(Default, Clone)]
pub struct MyStoreEntry {
    pub key: String,
    pub value: f32,
    // ... whatever fields are relevant for the GUI
}

#[derive(Default)]
pub struct MyStoreCache {
    pub entries: Vec<MyStoreEntry>,
}

pub type MyStoreMutex = Mutex<MyStoreCache>;

pub fn my_store_sync_system(store: Res<MyStore>, app_state: ResMut<AppState>) {
    if !store.is_changed() {
        return;
    }
    let Some(mutex) = app_state.get::<MyStoreMutex>() else { return; };
    let mut cache = mutex.lock().unwrap();
    cache.entries = store.items.iter().map(|(k, v)| MyStoreEntry {
        key: k.clone(),
        value: v.value,
    }).collect();
    cache.entries.sort_by(|a, b| a.key.cmp(&b.key));
}
```

Import `DetectChanges` from `aberredengine::bevy_ecs::change_detection::DetectChanges` to call
`.is_changed()` on the resource.

## 3. Insert the mutex in load_assets

In `src/systems/load_assets.rs`:

```rust
use crate::systems::my_store_sync::MyStoreMutex;

// Inside load_assets():
app_state.insert(MyStoreMutex::new(MyStoreCache::default()));
```

## 4. Register the sync system

In `src/main.rs`:

```rust
.add_system(systems::my_store_sync::my_store_sync_system)
```

## 5. Define CRUD events

In `src/systems/map_ops.rs` (or a new file):

```rust
#[derive(Event)]
pub struct AddMyAssetRequested { pub key: String }

#[derive(Event)]
pub struct UpdateMyAssetRequested { pub key: String, pub value: f32 }

#[derive(Event)]
pub struct RenameMyAssetKeyRequested { pub old_key: String, pub new_key: String }

#[derive(Event)]
pub struct RemoveMyAssetRequested { pub key: String }
```

## 6. Write the CRUD observers

```rust
pub fn add_my_asset_observer(
    trigger: On<AddMyAssetRequested>,
    mut store: ResMut<MyStore>,
) {
    let key = &trigger.event().key;
    if store.items.contains_key(key.as_str()) { return; }
    store.items.insert(key.clone(), MyAsset::default());
    info!("add_my_asset_observer: added '{}'", key);
}

pub fn remove_my_asset_observer(
    trigger: On<RemoveMyAssetRequested>,
    mut store: ResMut<MyStore>,
) {
    store.items.remove(trigger.event().key.as_str());
}

// etc. for update and rename
```

## 7. Register the observers

In `src/main.rs`:

```rust
.add_observer(systems::map_ops::add_my_asset_observer)
.add_observer(systems::map_ops::update_my_asset_observer)
.add_observer(systems::map_ops::rename_my_asset_key_observer)
.add_observer(systems::map_ops::remove_my_asset_observer)
```

## 8. Add signal constants

In `src/signals.rs`:

```rust
/// Flag: my asset store window is open.
pub const UI_MY_STORE_OPEN: &str = "ui:my_store:open";
pub const MY_STORE_ADD_KEY_BUF: &str = "gui:my_store:add_key_buf";
pub const MY_STORE_RENAME_SRC: &str = "gui:my_store:rename_src";
pub const MY_STORE_RENAME_BUF: &str = "gui:my_store:rename_buf";
pub const MY_STORE_REMOVE_KEY: &str = "gui:my_store:remove_key";
pub const ACTION_MY_STORE_ADD: &str = "gui:action:my_store:add";
pub const ACTION_MY_STORE_RENAME: &str = "gui:action:my_store:rename";
pub const ACTION_MY_STORE_REMOVE: &str = "gui:action:my_store:remove";
pub const ACTION_MY_STORE_UPDATE: &str = "gui:action:my_store:update";
```

## 9. Create the GUI panel

Create `src/scenes/editor/my_store_panel.rs`. Model it closely on `animation_panel.rs` or
`texture_panel.rs` — the pattern is the same:

- Check `signals.has_flag(sig::UI_MY_STORE_OPEN)` at the top; return early if false
- Show a list of entries from `app_state.get::<MyStoreMutex>()`
- "Add" button → set `ACTION_MY_STORE_ADD` flag
- Edit fields → set `ACTION_MY_STORE_UPDATE` flag
- "Rename" / "Remove" buttons → return booleans to open modals (same as texture_panel pattern)
- Modal popups in a separate `draw_my_store_modals` function

## 10. Wire menu item + action handling

Menu item in `menu.rs` → set `UI_MY_STORE_OPEN` flag.

In `editor_update` / `handle_*_actions`, consume action flags and trigger the CRUD events.
Follow `handle_animation_actions` in `update.rs` as the template.

## 11. Wire map serialization

In `save_map_observer` (`map_ops.rs`): iterate the store and populate `map_data.my_assets`.

In `load_map_observer`: iterate `map_data.my_assets` and insert into the store. Also clear the
store in `new_map_observer` alongside the other stores.

## Verification

- `cargo check` passes
- Run the editor, open View → My Store → the window appears
- Add an asset → it appears in the list immediately (sync system picks up the change)
- Save the map, reload it → assets persist
- New map → store is cleared
