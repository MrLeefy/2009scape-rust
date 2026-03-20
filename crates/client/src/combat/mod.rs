//! Combat system — damage calculation, hit splats, XP drops.

use crate::entity::{Entity, EntityKind};
use std::collections::VecDeque;

/// Attack style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttackStyle {
    Accurate,
    Aggressive,
    Defensive,
    Controlled,
    Rapid,
    LongRange,
}

/// Combat type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatType {
    Melee,
    Ranged,
    Magic,
}

/// A hit splat displayed on an entity.
#[derive(Debug, Clone)]
pub struct HitSplat {
    pub damage: u16,
    pub hit_type: HitType,
    pub timer: f32,  // seconds remaining to display
    pub target_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitType {
    Normal,
    Block,  // 0 damage
    Poison,
    Disease,
    Heal,
}

/// XP drop animation.
#[derive(Debug, Clone)]
pub struct XpDrop {
    pub skill_name: String,
    pub amount: f32,
    pub timer: f32,
    pub y_offset: f32,
    pub color: [f32; 4],
}

/// Combat engine.
pub struct CombatSystem {
    pub hit_splats: VecDeque<HitSplat>,
    pub xp_drops: VecDeque<XpDrop>,
    pub attack_style: AttackStyle,
    pub auto_retaliate: bool,
    pub in_combat: bool,
    pub combat_target: Option<u32>,
    pub attack_timer: f32,
    pub attack_speed: f32,  // ticks between attacks
    pub special_energy: f32,
    pub special_active: bool,
}

impl CombatSystem {
    pub fn new() -> Self {
        CombatSystem {
            hit_splats: VecDeque::new(),
            xp_drops: VecDeque::new(),
            attack_style: AttackStyle::Accurate,
            auto_retaliate: true,
            in_combat: false,
            combat_target: None,
            attack_timer: 0.0,
            attack_speed: 2.4,  // 4 game ticks = 2.4 seconds
            special_energy: 100.0,
            special_active: false,
        }
    }

    /// Calculate max hit for melee.
    pub fn max_melee_hit(strength_level: u16, bonus: i16, style_bonus: u8) -> u16 {
        let effective = strength_level as f32 + style_bonus as f32 + 8.0;
        let base = 0.5 + effective * (bonus as f32 + 64.0) / 640.0;
        base as u16
    }

    /// Calculate attack roll.
    pub fn attack_roll(attack_level: u16, bonus: i16, style_bonus: u8) -> u32 {
        let effective = attack_level as u32 + style_bonus as u32 + 8;
        effective * (bonus as u32 + 64)
    }

    /// Calculate defence roll.
    pub fn defence_roll(defence_level: u16, bonus: i16) -> u32 {
        let effective = defence_level as u32 + 9;
        effective * (bonus as u32 + 64)
    }

    /// Calculate hit chance (0.0 to 1.0).
    pub fn hit_chance(attack_roll: u32, defence_roll: u32) -> f32 {
        if attack_roll > defence_roll {
            1.0 - (defence_roll as f32 + 2.0) / (2.0 * (attack_roll as f32 + 1.0))
        } else {
            attack_roll as f32 / (2.0 * (defence_roll as f32 + 1.0))
        }
    }

    /// Process a combat tick.
    pub fn tick(&mut self, dt: f32) {
        // Update attack timer
        if self.in_combat {
            self.attack_timer -= dt;
        }

        // Regenerate special energy (10% per 30 seconds)
        if self.special_energy < 100.0 {
            self.special_energy = (self.special_energy + dt * (10.0 / 30.0)).min(100.0);
        }

        // Decay hit splats
        self.hit_splats.retain(|h| h.timer > 0.0);
        for splat in &mut self.hit_splats {
            splat.timer -= dt;
        }

        // Animate XP drops
        self.xp_drops.retain(|x| x.timer > 0.0);
        for drop in &mut self.xp_drops {
            drop.timer -= dt;
            drop.y_offset += dt * 30.0;
        }
    }

    /// Add a hit splat.
    pub fn add_hit(&mut self, target_id: u32, damage: u16, hit_type: HitType) {
        self.hit_splats.push_back(HitSplat {
            damage,
            hit_type,
            timer: 1.5,
            target_id,
        });
    }

    /// Add an XP drop.
    pub fn add_xp_drop(&mut self, skill: &str, amount: f32, color: [f32; 4]) {
        self.xp_drops.push_back(XpDrop {
            skill_name: skill.to_string(),
            amount,
            timer: 2.0,
            y_offset: 0.0,
            color,
        });
    }

    /// Perform an attack on a target entity.
    pub fn attack(&mut self, attacker: &Entity, target: &mut Entity) -> Option<u16> {
        if self.attack_timer > 0.0 { return None; }

        self.attack_timer = self.attack_speed;
        self.in_combat = true;
        self.combat_target = Some(target.id);

        // Simple damage calc
        let max_hit = Self::max_melee_hit(
            attacker.combat_level * 2 / 3 + 1,
            0,
            match self.attack_style {
                AttackStyle::Aggressive => 3,
                AttackStyle::Controlled => 1,
                _ => 0,
            }
        );

        let att_roll = Self::attack_roll(attacker.combat_level, 0, 0);
        let def_roll = Self::defence_roll(target.combat_level, 0);
        let chance = Self::hit_chance(att_roll as u32, def_roll as u32);

        // Random hit (deterministic seed for now)
        let rng_val = (attacker.anim.frame as f32 * 0.37 + target.id as f32 * 1.3).sin().abs();

        if rng_val < chance {
            let damage = ((rng_val * max_hit as f32) as u16).max(1);
            let actual_damage = damage.min(target.health);
            target.health = target.health.saturating_sub(actual_damage);

            self.add_hit(target.id, actual_damage, HitType::Normal);

            // XP drops
            let xp = actual_damage as f32 * 4.0;
            match self.attack_style {
                AttackStyle::Accurate => self.add_xp_drop("Attack", xp, [0.7, 0.2, 0.2, 1.0]),
                AttackStyle::Aggressive => self.add_xp_drop("Strength", xp, [0.0, 0.6, 0.0, 1.0]),
                AttackStyle::Defensive => self.add_xp_drop("Defence", xp, [0.3, 0.3, 0.9, 1.0]),
                AttackStyle::Controlled => {
                    self.add_xp_drop("Attack", xp / 3.0, [0.7, 0.2, 0.2, 1.0]);
                    self.add_xp_drop("Strength", xp / 3.0, [0.0, 0.6, 0.0, 1.0]);
                    self.add_xp_drop("Defence", xp / 3.0, [0.3, 0.3, 0.9, 1.0]);
                }
                _ => self.add_xp_drop("Ranged", xp, [0.0, 0.5, 0.0, 1.0]),
            }
            self.add_xp_drop("Hitpoints", actual_damage as f32 * 1.33, [0.9, 0.0, 0.0, 1.0]);

            Some(actual_damage)
        } else {
            self.add_hit(target.id, 0, HitType::Block);
            Some(0)
        }
    }
}
