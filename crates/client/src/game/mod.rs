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
    pub active_field: u8, // 0 = username, 1 = password
    pub status_message: String,
    pub login_requested: bool,
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
            active_field: 0,
            status_message: "Enter your username and password".to_string(),
            login_requested: false,
        }
    }

    /// Process one game tick (called every frame for now, 600ms RS ticks later).
    pub fn tick(&mut self) {
        self.tick += 1;
    }

    /// Handle a character being typed.
    pub fn on_char(&mut self, c: char) {
        if self.state != GameState::Login { return; }
        if !c.is_ascii_graphic() && c != ' ' { return; }

        match self.active_field {
            0 => {
                if self.username.len() < 12 {
                    self.username.push(c);
                }
            }
            1 => {
                if self.password.len() < 20 {
                    self.password.push(c);
                }
            }
            _ => {}
        }
    }

    /// Handle backspace.
    pub fn on_backspace(&mut self) {
        if self.state != GameState::Login { return; }
        match self.active_field {
            0 => { self.username.pop(); }
            1 => { self.password.pop(); }
            _ => {}
        }
    }

    /// Handle tab — switch between username and password fields.
    pub fn on_tab(&mut self) {
        if self.state != GameState::Login { return; }
        self.active_field = (self.active_field + 1) % 2;
    }

    /// Handle enter — attempt login.
    pub fn on_enter(&mut self) {
        if self.state != GameState::Login { return; }
        if self.username.is_empty() || self.password.is_empty() {
            self.status_message = "Error: Please enter username and password".to_string();
            return;
        }
        self.login_requested = true;
        self.status_message = "Connecting to server...".to_string();
    }
}
