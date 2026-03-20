//! Game state machine and main loop logic.

use crate::entity::EntityManager;
use crate::combat::{CombatSystem, HitType};
use crate::audio::{AudioEngine, SoundEffect};
use crate::net::protocol::PacketHandler;

/// High-level game states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Login,
    Loading,
    InGame,
}

/// Inventory item.
#[derive(Debug, Clone)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub quantity: u32,
    pub color: [f32; 4],
}

/// Chat message.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub text: String,
    pub color: [f32; 4],
    pub timestamp: u64,
}

/// Skill data.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub level: u8,
    pub xp: u32,
    pub color: [f32; 4],
}

/// Tab IDs for the interface panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterfaceTab {
    Inventory,
    Skills,
    Quests,
    Equipment,
    Prayer,
    Magic,
    Settings,
}

/// Core game state container.
pub struct Game {
    pub state: GameState,
    pub username: String,
    pub password: String,
    pub server_host: String,
    pub server_port: u16,
    pub logged_in: bool,
    pub tick: u64,
    pub active_field: u8,
    pub status_message: String,
    pub login_requested: bool,
    pub entities: EntityManager,
    pub inventory: Vec<Option<Item>>,
    pub chat_messages: Vec<ChatMessage>,
    pub skills: Vec<Skill>,
    pub active_tab: InterfaceTab,
    pub chat_input: String,
    pub chat_active: bool,
    pub combat: CombatSystem,
    pub audio: AudioEngine,
    pub packets: PacketHandler,
    pub run_energy: f32,
    pub world_region: String,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            state: GameState::Login,
            username: String::new(),
            password: String::new(),
            server_host: "test.2009scape.org".to_string(),
            server_port: 43594,
            logged_in: false,
            tick: 0,
            active_field: 0,
            status_message: "Enter your username and password".to_string(),
            login_requested: false,
            entities: EntityManager::new(),
            inventory: vec![None; 28],
            chat_messages: Vec::new(),
            skills: Vec::new(),
            active_tab: InterfaceTab::Inventory,
            chat_input: String::new(),
            chat_active: false,
            combat: CombatSystem::new(),
            audio: AudioEngine::new(),
            packets: PacketHandler::new(),
            run_energy: 100.0,
            world_region: "Lumbridge".to_string(),
        };

        // Initialize inventory with some test items
        game.inventory[0] = Some(Item { id: 995, name: "Coins".into(), quantity: 10000, color: [0.85, 0.72, 0.2, 1.0] });
        game.inventory[1] = Some(Item { id: 1265, name: "Bronze sword".into(), quantity: 1, color: [0.6, 0.4, 0.2, 1.0] });
        game.inventory[2] = Some(Item { id: 1351, name: "Bronze axe".into(), quantity: 1, color: [0.6, 0.4, 0.2, 1.0] });
        game.inventory[3] = Some(Item { id: 590, name: "Tinderbox".into(), quantity: 1, color: [0.5, 0.3, 0.1, 1.0] });
        game.inventory[4] = Some(Item { id: 1925, name: "Bucket".into(), quantity: 1, color: [0.5, 0.5, 0.5, 1.0] });
        game.inventory[5] = Some(Item { id: 2309, name: "Bread".into(), quantity: 5, color: [0.7, 0.55, 0.3, 1.0] });
        game.inventory[6] = Some(Item { id: 380, name: "Lobster".into(), quantity: 10, color: [0.9, 0.3, 0.2, 1.0] });
        game.inventory[7] = Some(Item { id: 1265, name: "Bronze pickaxe".into(), quantity: 1, color: [0.6, 0.4, 0.2, 1.0] });

        // Initialize skills
        let skill_defs = [
            ("Attack", 1, 0, [0.7, 0.2, 0.2, 1.0]),
            ("Strength", 1, 0, [0.0, 0.6, 0.0, 1.0]),
            ("Defence", 1, 0, [0.3, 0.3, 0.9, 1.0]),
            ("Ranged", 1, 0, [0.0, 0.5, 0.0, 1.0]),
            ("Prayer", 1, 0, [0.8, 0.8, 0.3, 1.0]),
            ("Magic", 1, 0, [0.3, 0.3, 0.8, 1.0]),
            ("Hitpoints", 10, 1154, [0.9, 0.0, 0.0, 1.0]),
            ("Agility", 1, 0, [0.2, 0.2, 0.6, 1.0]),
            ("Herblore", 1, 0, [0.0, 0.5, 0.2, 1.0]),
            ("Thieving", 1, 0, [0.4, 0.2, 0.4, 1.0]),
            ("Crafting", 1, 0, [0.6, 0.5, 0.3, 1.0]),
            ("Fletching", 1, 0, [0.0, 0.5, 0.4, 1.0]),
            ("Mining", 1, 0, [0.4, 0.5, 0.5, 1.0]),
            ("Smithing", 1, 0, [0.5, 0.4, 0.3, 1.0]),
            ("Fishing", 1, 0, [0.3, 0.5, 0.7, 1.0]),
            ("Cooking", 1, 0, [0.5, 0.3, 0.5, 1.0]),
            ("Firemaking", 1, 0, [0.8, 0.5, 0.1, 1.0]),
            ("Woodcutting", 1, 0, [0.3, 0.5, 0.2, 1.0]),
            ("Runecrafting", 1, 0, [0.6, 0.6, 0.2, 1.0]),
            ("Slayer", 1, 0, [0.3, 0.2, 0.2, 1.0]),
            ("Farming", 1, 0, [0.2, 0.6, 0.1, 1.0]),
            ("Construction", 1, 0, [0.6, 0.5, 0.4, 1.0]),
            ("Hunter", 1, 0, [0.5, 0.4, 0.2, 1.0]),
            ("Summoning", 1, 0, [0.1, 0.4, 0.5, 1.0]),
            ("Dungeoneering", 1, 0, [0.5, 0.3, 0.1, 1.0]),
        ];
        for (name, level, xp, color) in &skill_defs {
            game.skills.push(Skill {
                name: name.to_string(),
                level: *level,
                xp: *xp,
                color: *color,
            });
        }

        // Initial chat messages
        game.chat_messages.push(ChatMessage {
            sender: "System".into(),
            text: "Welcome to 2009Scape!".into(),
            color: [0.0, 0.8, 0.8, 1.0],
            timestamp: 0,
        });
        game.chat_messages.push(ChatMessage {
            sender: "System".into(),
            text: "Press Enter to open chat. WASD to move.".into(),
            color: [0.7, 0.7, 0.7, 1.0],
            timestamp: 0,
        });

        game
    }

    pub fn tick(&mut self) {
        self.tick += 1;
        let dt = 1.0 / 60.0;
        if self.state == GameState::InGame {
            self.entities.tick(dt);
            self.combat.tick(dt);

            let player_pos = (
                self.entities.local_player.render_x(),
                0.0,
                self.entities.local_player.render_z(),
            );
            self.audio.tick(dt, player_pos.0, player_pos.1, player_pos.2);

            // Process any pending packet updates
            self.process_packet_updates();

            // Demo: auto-attack first nearby NPC every 4 seconds for testing
            if self.tick % 240 == 120 && !self.entities.npcs.is_empty() {
                let attacker = self.entities.local_player.clone();
                let target_pos = (self.entities.npcs[0].x, self.entities.npcs[0].z);
                let dmg = self.combat.attack(&attacker, &mut self.entities.npcs[0]);
                if let Some(d) = dmg {
                    if d > 0 {
                        self.audio.play_sfx(SoundEffect::MeleeHit, target_pos.0, 0.0, target_pos.1);
                    } else {
                        self.audio.play_sfx(SoundEffect::MeleeMiss, target_pos.0, 0.0, target_pos.1);
                    }
                }
                // Respawn NPC if dead (separate borrow scope)
                let npc = &self.entities.npcs[0];
                if npc.health == 0 {
                    let name = npc.name.clone();
                    self.audio.play_sfx(SoundEffect::NpcDeath, target_pos.0, 0.0, target_pos.1);
                    self.entities.npcs[0].health = self.entities.npcs[0].max_health;
                    self.chat_messages.push(ChatMessage {
                        sender: "System".into(),
                        text: format!("{} has been defeated!", name),
                        color: [1.0, 0.3, 0.3, 1.0],
                        timestamp: self.tick,
                    });
                }
            }

            // Set region music on first tick
            if self.tick == 2 {
                self.audio.set_region_music(&self.world_region);
            }
        } else if self.state == GameState::Login {
            self.audio.set_region_music("Login");
        }
    }

    /// Process queued server packet updates into game state.
    fn process_packet_updates(&mut self) {
        // Stats
        for (skill_id, level, xp) in self.packets.stat_updates.drain(..) {
            if let Some(skill) = self.skills.get_mut(skill_id as usize) {
                skill.level = level;
                skill.xp = xp;
            }
        }

        // Chat
        for (sender, text) in self.packets.chat_updates.drain(..) {
            self.chat_messages.push(ChatMessage {
                sender,
                text,
                color: [0.0, 0.8, 0.8, 1.0],
                timestamp: self.tick,
            });
        }

        // Inventory
        for (slot, item_id, quantity) in self.packets.inv_updates.drain(..) {
            let slot = slot as usize;
            if slot < self.inventory.len() {
                if item_id > 0 {
                    self.inventory[slot] = Some(Item {
                        id: item_id,
                        name: format!("Item {}", item_id),
                        quantity,
                        color: [0.5, 0.5, 0.5, 1.0],
                    });
                } else {
                    self.inventory[slot] = None;
                }
            }
        }

        // Sound effects from server
        for (sound_id, _volume, _delay) in self.packets.sound_updates.drain(..) {
            // Map server sound IDs to our SFX enum (simplified)
            let sfx = match sound_id {
                2727 => SoundEffect::MeleeHit,
                2724 => SoundEffect::MeleeMiss,
                2277 => SoundEffect::LevelUp,
                _ => SoundEffect::ButtonClick,
            };
            self.audio.play_ui_sfx(sfx);
        }
    }

    pub fn on_char(&mut self, c: char) {
        if !c.is_ascii_graphic() && c != ' ' { return; }
        match self.state {
            GameState::Login => {
                match self.active_field {
                    0 => { if self.username.len() < 12 { self.username.push(c); } }
                    1 => { if self.password.len() < 20 { self.password.push(c); } }
                    _ => {}
                }
            }
            GameState::InGame if self.chat_active => {
                if self.chat_input.len() < 80 {
                    self.chat_input.push(c);
                }
            }
            _ => {}
        }
    }

    pub fn on_backspace(&mut self) {
        match self.state {
            GameState::Login => {
                match self.active_field {
                    0 => { self.username.pop(); }
                    1 => { self.password.pop(); }
                    _ => {}
                }
            }
            GameState::InGame if self.chat_active => { self.chat_input.pop(); }
            _ => {}
        }
    }

    pub fn on_tab(&mut self) {
        match self.state {
            GameState::Login => { self.active_field = (self.active_field + 1) % 2; }
            GameState::InGame => {
                self.active_tab = match self.active_tab {
                    InterfaceTab::Inventory => InterfaceTab::Skills,
                    InterfaceTab::Skills => InterfaceTab::Quests,
                    InterfaceTab::Quests => InterfaceTab::Equipment,
                    InterfaceTab::Equipment => InterfaceTab::Prayer,
                    InterfaceTab::Prayer => InterfaceTab::Magic,
                    InterfaceTab::Magic => InterfaceTab::Settings,
                    InterfaceTab::Settings => InterfaceTab::Inventory,
                };
                self.audio.play_ui_sfx(SoundEffect::TabSwitch);
            }
            _ => {}
        }
    }

    pub fn on_enter(&mut self) {
        match self.state {
            GameState::Login => {
                if self.username.is_empty() || self.password.is_empty() {
                    self.status_message = "Error: Please enter username and password".to_string();
                    return;
                }
                self.login_requested = true;
                self.status_message = "Connecting to server...".to_string();
            }
            GameState::InGame => {
                if self.chat_active && !self.chat_input.is_empty() {
                    let msg = ChatMessage {
                        sender: self.username.clone(),
                        text: self.chat_input.clone(),
                        color: [0.0, 1.0, 1.0, 1.0],
                        timestamp: self.tick,
                    };
                    self.chat_messages.push(msg);
                    self.chat_input.clear();
                }
                self.chat_active = !self.chat_active;
            }
            _ => {}
        }
    }

    /// Total combat level.
    pub fn combat_level(&self) -> u32 {
        let att = self.skills.first().map(|s| s.level as u32).unwrap_or(1);
        let str = self.skills.get(1).map(|s| s.level as u32).unwrap_or(1);
        let def = self.skills.get(2).map(|s| s.level as u32).unwrap_or(1);
        let hp = self.skills.get(6).map(|s| s.level as u32).unwrap_or(10);
        let prayer = self.skills.get(4).map(|s| s.level as u32).unwrap_or(1);
        ((def + hp + prayer / 2) * 13 / 10 + (att + str) * 13 / 40) / 10 + 1
    }
}
