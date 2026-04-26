use crate::editor_types::ComponentKind;
use std::sync::Mutex;

/// Typed owner of transient entity-editor pending values and commit flags.
///
/// Stored in `AppState` as `Mutex<PendingEditState>` so both the GUI callback
/// (which only has `&AppState`) and the ECS update path (which has `&mut AppState`)
/// can access it through the Mutex's interior mutability.
///
/// `Option<T>` encodes dirty state: `None` means "unedited, fall back to snapshot";
/// `Some(v)` means "user has changed this field to v". This eliminates separate dirty flags.
/// Commit booleans signal which component group should be written to ECS this frame.
///
/// Reset with `*self = Self::default()` after each commit or on selection change.
#[derive(Default, Clone)]
pub(crate) struct PendingEditState {
    // MapPosition
    pub pos_x:            Option<f32>,
    pub pos_y:            Option<f32>,
    pub commit_position:  bool,
    // ZIndex
    pub z_index:          Option<f32>,
    pub commit_z:         bool,
    // Group
    pub group:            Option<String>,
    pub commit_group:     bool,
    // Rotation
    pub rotation_deg:     Option<f32>,
    pub commit_rotation:  bool,
    // Scale
    pub scale_x:          Option<f32>,
    pub scale_y:          Option<f32>,
    pub commit_scale:     bool,
    // Sprite
    pub sprite_tex_key:   Option<String>,
    pub sprite_width:     Option<f32>,
    pub sprite_height:    Option<f32>,
    pub sprite_off_x:     Option<f32>,
    pub sprite_off_y:     Option<f32>,
    pub sprite_org_x:     Option<f32>,
    pub sprite_org_y:     Option<f32>,
    pub sprite_flip_h:    Option<bool>,
    pub sprite_flip_v:    Option<bool>,
    pub commit_sprite:    bool,
    // BoxCollider
    pub box_size_x:       Option<f32>,
    pub box_size_y:       Option<f32>,
    pub box_off_x:        Option<f32>,
    pub box_off_y:        Option<f32>,
    pub box_org_x:        Option<f32>,
    pub box_org_y:        Option<f32>,
    pub commit_collider:  bool,
    // Animation
    pub anim_key:         Option<String>,
    pub anim_frame_index: Option<i32>,
    pub anim_elapsed:     Option<f32>,
    pub commit_animation: bool,
    // Remove flags
    pub remove_map_position: bool,
    pub remove_z:            bool,
    pub remove_group:        bool,
    pub remove_sprite:       bool,
    pub remove_collider:     bool,
    pub remove_rotation:     bool,
    pub remove_scale:        bool,
    pub remove_animation:    bool,
    pub remove_ttl:          bool,
    pub remove_timer:        bool,
    pub remove_phase:        bool,
    pub remove_persistent:   bool,
    pub remove_tilemap:        bool,
    pub bake_tilemap:          bool,
    pub select_tilemap_parent: bool,
    // Registration
    pub pending_register_key: Option<String>,
    pub commit_registration:  bool,
    pub remove_registration:  bool,
    // Add component
    pub add_component:       Option<ComponentKind>,
    pub add_combo_selection: usize,
}

impl PendingEditState {
    pub(crate) fn any_commit(&self) -> bool {
        self.commit_position
            || self.commit_z
            || self.commit_group
            || self.commit_rotation
            || self.commit_scale
            || self.commit_sprite
            || self.commit_collider
            || self.commit_animation
            || self.remove_map_position
            || self.remove_z
            || self.remove_group
            || self.remove_sprite
            || self.remove_collider
            || self.remove_rotation
            || self.remove_scale
            || self.remove_animation
            || self.remove_ttl
            || self.remove_timer
            || self.remove_phase
            || self.remove_persistent
            || self.remove_tilemap
            || self.bake_tilemap
            || self.select_tilemap_parent
            || self.commit_registration
            || self.remove_registration
            || self.add_component.is_some()
    }
}

/// Convenience alias used by callers that store this in AppState.
pub(crate) type PendingMutex = Mutex<PendingEditState>;
