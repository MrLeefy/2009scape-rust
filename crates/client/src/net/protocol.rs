//! Game protocol — packet definitions and handlers.
//!
//! Implements the RS2 client-server protocol for packet encoding/decoding.


/// Server-to-client packet opcodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerProt {
    // Player updates
    PlayerInfo = 81,
    NpcInfo = 65,
    
    // Map
    RebuildNormal = 73,
    RebuildRegion = 241,
    
    // Interface
    IfOpenTop = 97,
    IfOpenSub = 130,
    IfClose = 219,
    IfSetText = 196,
    IfSetNpcHead = 132,
    IfSetPlayerHead = 205,
    IfSetAnim = 98,
    
    // Items
    UpdateInvFull = 53,
    UpdateInvPartial = 34,
    
    // Stats
    UpdateStat = 134,
    UpdateRunEnergy = 110,
    
    // Chat
    MessageGame = 253,
    MessagePublic = 4,
    MessagePrivate = 197,
    
    // Player state
    UpdateRebootTimer = 114,
    RunClientScript = 35,
    ResetAnims = 1,
    
    // Sound
    SynthSound = 174,
    MidiSong = 74,
    MidiJingle = 105,
    
    // World
    LocAddChange = 151,
    LocDel = 156,
    ObjAdd = 44,
    ObjDel = 157,
    MapAnim = 85,
    
    // Camera
    CamMoveTo = 166,
    CamLookAt = 177,
    CamReset = 107,
}

/// Client-to-server packet opcodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientProt {
    // Movement
    MoveGameClick = 164,
    MoveMinimapClick = 248,
    
    // Player actions
    OpPlayer1 = 128,
    OpPlayer2 = 252,
    OpPlayer3 = 226,
    OpPlayer4 = 213,
    
    // NPC actions
    OpNpc1 = 155,
    OpNpc2 = 17,
    OpNpc3 = 21,
    OpNpc4 = 18,
    OpNpc5 = 236,
    
    // Object actions
    OpLoc1 = 132,
    OpLoc2 = 253,
    OpLoc3 = 70,
    OpLoc4 = 234,
    OpLoc5 = 228,
    
    // Item actions
    OpHeld1 = 122,
    OpHeld2 = 41,
    OpHeld3 = 16,
    OpHeld4 = 75,
    OpHeld5 = 87,
    
    // Chat
    ChatPublic = 4,
    ChatPrivate = 126,
    
    // Interface
    IfButton = 156,
    WindowStatus = 243,
    
    // Misc
    NoTimeout = 108,
    MapBuildComplete = 69,
    ClickAboveObject = 245,
}

/// Decoded player update block.
#[derive(Debug, Clone, Default)]
pub struct PlayerUpdate {
    pub index: u16,
    pub x: u16,
    pub z: u16,
    pub plane: u8,
    pub name: String,
    pub combat_level: u8,
    pub appearance_update: bool,
    pub chat_update: bool,
    pub hit_update: bool,
    pub facing_update: bool,
    pub anim_update: bool,
    // Appearance
    pub gender: u8,
    pub head_icon: i8,
    pub skull_icon: i8,
    // Equipment
    pub equipment: [i16; 12],
    pub colors: [u8; 5],
}

/// Decoded NPC update block.
#[derive(Debug, Clone, Default)]
pub struct NpcUpdate {
    pub index: u16,
    pub npc_id: u16,
    pub x: u16,
    pub z: u16,
    pub facing: u16,
    pub hit_update: bool,
    pub anim_update: bool,
    pub facing_update: bool,
}

/// Packet handler that processes incoming server packets.
pub struct PacketHandler {
    pub pending_rebuilds: Vec<(u16, u16)>,
    pub player_updates: Vec<PlayerUpdate>,
    pub npc_updates: Vec<NpcUpdate>,
    pub stat_updates: Vec<(u8, u8, u32)>, // skill_id, level, xp
    pub chat_updates: Vec<(String, String)>, // sender, message
    pub inv_updates: Vec<(u16, i32, u32)>, // slot, item_id, quantity
    pub sound_updates: Vec<(u16, u8, u8)>, // sound_id, volume, delay
}

impl PacketHandler {
    pub fn new() -> Self {
        PacketHandler {
            pending_rebuilds: Vec::new(),
            player_updates: Vec::new(),
            npc_updates: Vec::new(),
            stat_updates: Vec::new(),
            chat_updates: Vec::new(),
            inv_updates: Vec::new(),
            sound_updates: Vec::new(),
        }
    }

    /// Process a raw server packet.
    pub fn handle_packet(&mut self, opcode: u8, data: &[u8]) {
        match opcode {
            134 => self.handle_update_stat(data),
            253 => self.handle_message_game(data),
            53 => self.handle_update_inv(data),
            110 => self.handle_run_energy(data),
            174 => self.handle_synth_sound(data),
            _ => {
                // Unknown packet — log for debugging
                log::debug!("Unhandled packet opcode: {} (len={})", opcode, data.len());
            }
        }
    }

    fn handle_update_stat(&mut self, data: &[u8]) {
        if data.len() < 6 { return; }
        let skill_id = data[0];
        let xp = ((data[1] as u32) << 24) | ((data[2] as u32) << 16) |
                 ((data[3] as u32) << 8) | (data[4] as u32);
        let level = data[5];
        self.stat_updates.push((skill_id, level, xp));
    }

    fn handle_message_game(&mut self, data: &[u8]) {
        if let Ok(text) = String::from_utf8(data.to_vec()) {
            self.chat_updates.push(("System".to_string(), text));
        }
    }

    fn handle_update_inv(&mut self, data: &[u8]) {
        // Simplified — real protocol is more complex
        let mut offset = 0;
        while offset + 4 <= data.len() {
            let slot = ((data[offset] as u16) << 8) | (data[offset + 1] as u16);
            let item_id = ((data[offset + 2] as i32) << 8) | (data[offset + 3] as i32);
            let quantity = if offset + 8 <= data.len() {
                ((data[offset + 4] as u32) << 24) | ((data[offset + 5] as u32) << 16) |
                ((data[offset + 6] as u32) << 8) | (data[offset + 7] as u32)
            } else { 1 };
            self.inv_updates.push((slot, item_id, quantity));
            offset += 8;
        }
    }

    fn handle_run_energy(&mut self, _data: &[u8]) {
        // Run energy update
    }

    fn handle_synth_sound(&mut self, data: &[u8]) {
        if data.len() >= 4 {
            let sound_id = ((data[0] as u16) << 8) | (data[1] as u16);
            let volume = data[2];
            let delay = data[3];
            self.sound_updates.push((sound_id, volume, delay));
        }
    }

    /// Drain all updates (called by game loop after processing).
    pub fn drain(&mut self) {
        self.pending_rebuilds.clear();
        self.player_updates.clear();
        self.npc_updates.clear();
        self.stat_updates.clear();
        self.chat_updates.clear();
        self.inv_updates.clear();
        self.sound_updates.clear();
    }
}

/// Build a client packet.
pub fn build_packet(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(3 + payload.len());
    packet.push(opcode);
    if payload.len() >= 128 {
        packet.push((payload.len() >> 8) as u8 | 0x80);
        packet.push(payload.len() as u8);
    } else {
        packet.push(payload.len() as u8);
    }
    packet.extend_from_slice(payload);
    packet
}

/// Build a walk/movement packet.
pub fn build_walk_packet(dest_x: u16, dest_z: u16, run: bool) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.push((dest_x >> 8) as u8);
    payload.push(dest_x as u8);
    payload.push((dest_z >> 8) as u8);
    payload.push(dest_z as u8);
    payload.push(if run { 1 } else { 0 });
    build_packet(ClientProt::MoveGameClick as u8, &payload)
}

/// Build a chat message packet.
pub fn build_chat_packet(message: &str) -> Vec<u8> {
    let bytes = message.as_bytes();
    let mut payload = Vec::with_capacity(bytes.len() + 2);
    payload.push(0); // chat type
    payload.push(bytes.len() as u8);
    payload.extend_from_slice(bytes);
    build_packet(ClientProt::ChatPublic as u8, &payload)
}

/// Build a no-timeout keepalive packet.
pub fn build_keepalive() -> Vec<u8> {
    build_packet(ClientProt::NoTimeout as u8, &[])
}
