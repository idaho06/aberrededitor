// Shared payload types for cross-module communication between ECS systems and GUI panels.

// ---------------------------------------------------------------------------
// Component kind selector (for Add-component feature)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ComponentKind {
    MapPosition,
    ZIndex,
    Group,
    Rotation,
    Scale,
    Sprite,
    BoxCollider,
    Animation,
    Ttl,
    Persistent,
}

// ---------------------------------------------------------------------------
// Entity selector payloads
// ---------------------------------------------------------------------------

/// World-space quad corners for the active entity selection outline: TL → TR → BR → BL.
#[derive(Debug, Clone, Copy)]
pub struct SelectionCorners(pub [[f32; 2]; 4]);

// ---------------------------------------------------------------------------
// Entity inspector payloads
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SpriteSnapshot {
    pub tex_key: String,
    pub width: f32,
    pub height: f32,
    pub offset: [f32; 2],
    pub origin: [f32; 2],
    pub flip_h: bool,
    pub flip_v: bool,
}

#[derive(Clone)]
pub struct ColliderSnapshot {
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub origin: [f32; 2],
}

#[derive(Clone)]
pub struct AnimationSnapshot {
    pub animation_key: String,
    pub frame_index: usize,
    pub elapsed_time: f32,
}

#[derive(Clone)]
pub struct TtlSnapshot {
    pub remaining: f32,
}

#[derive(Clone)]
pub struct TimerSnapshot {
    pub duration: f32,
    pub elapsed: f32,
}

#[derive(Clone)]
pub struct PhaseSnapshot {
    pub current: String,
    pub previous: Option<String>,
    pub next: Option<String>,
    pub time_in_phase: f32,
    pub phase_names: Vec<String>,
}

#[derive(Clone)]
pub struct ComponentSnapshot {
    pub entity_bits: u64,
    /// WorldSignals entity keys whose value matches this entity.
    pub world_signal_keys: Vec<String>,
    pub map_position: Option<[f32; 2]>,
    pub z_index: Option<f32>,
    pub group: Option<String>,
    pub sprite: Option<SpriteSnapshot>,
    pub box_collider: Option<ColliderSnapshot>,
    pub rotation_deg: Option<f32>,
    pub scale: Option<[f32; 2]>,
    pub animation: Option<AnimationSnapshot>,
    pub ttl: Option<TtlSnapshot>,
    pub timer: Option<TimerSnapshot>,
    pub phase: Option<PhaseSnapshot>,
    pub persistent: bool,
    pub tilemap_path: Option<String>,
    /// `u64` bits because `Entity` cannot cross the `AppState` boundary.
    pub tilemap_parent: Option<u64>,
}
