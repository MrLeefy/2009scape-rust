//! Mouse input — click-to-walk, right-click menus, drag, hover.

/// Context menu action.
#[derive(Debug, Clone)]
pub enum MenuAction {
    Walk(u16, u16),
    AttackNpc(u32),
    TalkToNpc(u32),
    ExamineNpc(u32),
    PickupItem(u32, u16, u16),
    UseObject(u32),
    InventoryUse(usize),
    InventoryDrop(usize),
    TradePlayer(u32),
    FollowPlayer(u32),
    Cancel,
}

/// Menu option.
#[derive(Debug, Clone)]
pub struct MenuOption {
    pub text: String,
    pub action: MenuAction,
    pub color: [f32; 4],
}

/// Input state tracking.
pub struct InputState {
    pub mouse_x: f32, pub mouse_y: f32,
    pub left_down: bool, pub right_down: bool,
    pub left_clicked: bool, pub right_clicked: bool,
    pub scroll_delta: f32,
    pub is_dragging: bool,
    pub drag_start_x: f32, pub drag_start_y: f32,
    pub menu_open: bool, pub menu_x: f32, pub menu_y: f32,
    pub menu_options: Vec<MenuOption>,
    pub menu_hover: i32,
    pub hover_text: String,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            mouse_x: 0.0, mouse_y: 0.0,
            left_down: false, right_down: false,
            left_clicked: false, right_clicked: false,
            scroll_delta: 0.0,
            is_dragging: false, drag_start_x: 0.0, drag_start_y: 0.0,
            menu_open: false, menu_x: 0.0, menu_y: 0.0,
            menu_options: Vec::new(), menu_hover: -1,
            hover_text: String::new(),
        }
    }

    pub fn on_move(&mut self, x: f32, y: f32) {
        self.mouse_x = x; self.mouse_y = y;
        if self.left_down && !self.is_dragging {
            let dx = x - self.drag_start_x; let dy = y - self.drag_start_y;
            if dx * dx + dy * dy > 25.0 { self.is_dragging = true; }
        }
        if self.menu_open {
            let ry = y - self.menu_y - 20.0;
            self.menu_hover = if ry >= 0.0 && x >= self.menu_x && x <= self.menu_x + 150.0 {
                let idx = (ry / 18.0) as i32;
                if idx < self.menu_options.len() as i32 { idx } else { -1 }
            } else { -1 };
        }
    }

    pub fn on_left_press(&mut self) {
        self.left_down = true; self.left_clicked = true;
        self.drag_start_x = self.mouse_x; self.drag_start_y = self.mouse_y;
        if self.menu_open { self.menu_open = false; }
    }

    pub fn on_left_release(&mut self) {
        self.left_down = false; self.is_dragging = false;
    }

    pub fn on_right_press(&mut self) { self.right_down = true; self.right_clicked = true; }
    pub fn on_right_release(&mut self) { self.right_down = false; }
    pub fn on_scroll(&mut self, delta: f32) { self.scroll_delta += delta; }

    pub fn end_frame(&mut self) {
        self.left_clicked = false; self.right_clicked = false; self.scroll_delta = 0.0;
    }

    pub fn open_menu(&mut self, x: f32, y: f32, opts: Vec<MenuOption>) {
        self.menu_open = true; self.menu_x = x; self.menu_y = y;
        self.menu_options = opts; self.menu_hover = -1;
    }

    pub fn is_over(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        self.mouse_x >= x && self.mouse_x <= x + w && self.mouse_y >= y && self.mouse_y <= y + h
    }
}
