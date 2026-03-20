//! Entity system — NPCs, players, and projectiles.

/// Entity types in the game world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityKind {
    Player,
    Npc,
    Projectile,
}

/// Animation state for an entity.
#[derive(Debug, Clone)]
pub struct AnimState {
    pub anim_id: i32,
    pub frame: u32,
    pub frame_timer: f32,
    pub idle_anim: i32,
    pub walk_anim: i32,
    pub run_anim: i32,
}

impl Default for AnimState {
    fn default() -> Self {
        AnimState {
            anim_id: -1,
            frame: 0,
            frame_timer: 0.0,
            idle_anim: 808,   // RS human idle
            walk_anim: 819,   // RS human walk
            run_anim: 824,    // RS human run
        }
    }
}

/// A single entity in the game world.
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u32,
    pub kind: EntityKind,
    pub name: String,
    pub combat_level: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub target_x: f32,
    pub target_z: f32,
    pub yaw: f32,          // facing direction (0-2047 RS units)
    pub move_speed: f32,
    pub health: u16,
    pub max_health: u16,
    pub anim: AnimState,
    pub model_id: i32,
    pub size: u8,           // tile size (1 = normal, 2+ = large NPCs)
    pub visible: bool,
    pub interacting: i32,   // index of entity being interacted with (-1 = none)
    // Interpolation
    pub prev_x: f32,
    pub prev_z: f32,
    pub lerp_t: f32,
}

impl Entity {
    pub fn new_player(id: u32, name: &str, x: f32, z: f32) -> Self {
        Entity {
            id,
            kind: EntityKind::Player,
            name: name.to_string(),
            combat_level: 3,
            x, y: 0.0, z,
            target_x: x, target_z: z,
            yaw: 0.0,
            move_speed: 4.0,
            health: 10, max_health: 10,
            anim: AnimState::default(),
            model_id: -1,
            size: 1,
            visible: true,
            interacting: -1,
            prev_x: x, prev_z: z,
            lerp_t: 1.0,
        }
    }

    pub fn new_npc(id: u32, name: &str, npc_id: i32, x: f32, z: f32, combat_level: u16) -> Self {
        Entity {
            id,
            kind: EntityKind::Npc,
            name: name.to_string(),
            combat_level,
            x, y: 0.0, z,
            target_x: x, target_z: z,
            yaw: 0.0,
            move_speed: 2.0,
            health: combat_level * 5 + 10,
            max_health: combat_level * 5 + 10,
            anim: AnimState::default(),
            model_id: npc_id,
            size: 1,
            visible: true,
            interacting: -1,
            prev_x: x, prev_z: z,
            lerp_t: 1.0,
        }
    }

    /// Update entity position with interpolation.
    pub fn tick(&mut self, dt: f32) {
        self.prev_x = self.render_x();
        self.prev_z = self.render_z();

        let dx = self.target_x - self.x;
        let dz = self.target_z - self.z;
        let dist = (dx * dx + dz * dz).sqrt();

        if dist > 1.0 {
            let speed = self.move_speed * 128.0 * dt;
            if dist <= speed {
                self.x = self.target_x;
                self.z = self.target_z;
            } else {
                self.x += (dx / dist) * speed;
                self.z += (dz / dist) * speed;
            }
            // Face movement direction
            self.yaw = (-(dx).atan2(dz) / std::f32::consts::TAU * 2048.0 + 2048.0) % 2048.0;
            self.lerp_t = 0.0;
        }

        // Smooth interpolation
        self.lerp_t = (self.lerp_t + dt * 10.0).min(1.0);

        // Animation tick
        self.anim.frame_timer += dt;
        if self.anim.frame_timer >= 0.1 {
            self.anim.frame_timer = 0.0;
            self.anim.frame += 1;
        }
    }

    /// Interpolated render X position.
    pub fn render_x(&self) -> f32 {
        self.prev_x + (self.x - self.prev_x) * self.lerp_t
    }

    /// Interpolated render Z position.
    pub fn render_z(&self) -> f32 {
        self.prev_z + (self.z - self.prev_z) * self.lerp_t
    }

    pub fn is_moving(&self) -> bool {
        let dx = self.target_x - self.x;
        let dz = self.target_z - self.z;
        (dx * dx + dz * dz) > 1.0
    }
}

/// Manages all entities in a scene region.
pub struct EntityManager {
    pub local_player: Entity,
    pub players: Vec<Entity>,
    pub npcs: Vec<Entity>,
}

impl EntityManager {
    pub fn new() -> Self {
        let mut mgr = EntityManager {
            local_player: Entity::new_player(0, "You", 3200.0, 3200.0),
            players: Vec::new(),
            npcs: Vec::new(),
        };

        // Spawn some test NPCs
        mgr.npcs.push(Entity::new_npc(1, "Man", 1, 3328.0, 3200.0, 2));
        mgr.npcs.push(Entity::new_npc(2, "Woman", 2, 3200.0, 3456.0, 2));
        mgr.npcs.push(Entity::new_npc(3, "Guard", 3, 3072.0, 3072.0, 21));
        mgr.npcs.push(Entity::new_npc(4, "Goblin", 4, 3520.0, 3520.0, 5));
        mgr.npcs.push(Entity::new_npc(5, "Cow", 5, 2944.0, 3328.0, 2));
        mgr.npcs.push(Entity::new_npc(6, "Chicken", 6, 3456.0, 2944.0, 1));
        mgr.npcs.push(Entity::new_npc(7, "Rat", 7, 3136.0, 3520.0, 1));
        mgr.npcs.push(Entity::new_npc(8, "Imp", 8, 3392.0, 3136.0, 2));

        // Spawn some test players
        mgr.players.push(Entity::new_player(100, "Zezima", 3264.0, 3264.0));
        mgr.players.push(Entity::new_player(101, "Durial321", 3328.0, 3328.0));

        mgr
    }

    /// Tick all entities.
    pub fn tick(&mut self, dt: f32) {
        self.local_player.tick(dt);
        for p in &mut self.players { p.tick(dt); }
        for n in &mut self.npcs {
            // NPCs wander randomly
            if !n.is_moving() && (n.anim.frame % 40) == 0 {
                let angle = (n.id as f32 * 1.7 + n.anim.frame as f32 * 0.3).sin();
                n.target_x = n.x + angle.cos() * 256.0;
                n.target_z = n.z + angle.sin() * 256.0;
                // Keep in bounds
                n.target_x = n.target_x.clamp(128.0, 6272.0);
                n.target_z = n.target_z.clamp(128.0, 6272.0);
            }
            n.tick(dt);
        }
    }

    /// Get all visible entities for rendering.
    pub fn all_entities(&self) -> Vec<&Entity> {
        let mut all: Vec<&Entity> = Vec::new();
        all.push(&self.local_player);
        all.extend(self.players.iter().filter(|e| e.visible));
        all.extend(self.npcs.iter().filter(|e| e.visible));
        all
    }
}
