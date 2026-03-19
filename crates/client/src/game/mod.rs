//! Game state machine and main loop logic.

/// High-level game states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Login,
    Loading,
    InGame,
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
}

impl Game {
    pub fn new() -> Self {
        Game {
            state: GameState::Login,
            username: String::new(),
            password: String::new(),
            server_host: "test.2009scape.org".to_string(),
            server_port: 43594,
            logged_in: false,
            tick: 0,
        }
    }

    /// Process one game tick (600ms in RS).
    pub fn tick(&mut self) {
        self.tick += 1;
        match self.state {
            GameState::Login => {
                // Wait for user to submit credentials via UI
            }
            GameState::Loading => {
                // Loading world data from cache
            }
            GameState::InGame => {
                // Process game logic, NPC updates, etc.
            }
        }
    }
}
