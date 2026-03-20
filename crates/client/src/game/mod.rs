//! Game state machine and main loop logic.

use crate::entity::EntityManager;

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
        if self.state == GameState::InGame {
            self.entities.tick(1.0 / 60.0);
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
