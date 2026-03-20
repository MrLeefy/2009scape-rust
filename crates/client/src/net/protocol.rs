//! Game protocol — real RS2 rev530 packet definitions.
//!
//! Translated from rt4/ServerProt.java and Protocol.java.
//! All opcodes and packet lengths match the actual 2009Scape server.

use rs2_common::buffer::Buffer;
use rs2_common::isaac::IsaacRandom;
use log::{info, debug, warn};

/// Complete packet length table (256 entries) from Protocol.java.
/// -1 = variable byte (1 byte header), -2 = variable short (2 byte header), 0+ = fixed.
#[rustfmt::skip]
pub static PACKET_LENGTHS: [i16; 256] = [
    -1,  0,  8,  0,  2,  0,  0,  0,  0, 12,  0,  1,  0,  3,  7,  0,  // 0-15
    15,  6,  0,  0,  4,  7, -2, -1,  2,  0,  2,  8,  0,  0,  0,  0,  // 16-31
    -2,  5,  0,  0,  8,  3,  6,  0,  0,  0, -1,  0, -1,  0,  0,  6,  // 32-47
    -2,  0, 12,  0,  0,  0, -1, -2, 10,  0,  0,  0,  3,  0, -1,  0,  // 48-63
     0,  5,  6,  0,  0,  8, -1, -1,  0,  8,  0,  0,  0,  0,  0,  0,  // 64-79
     0, -1,  0,  0,  6,  2,  0,  0,  0,  0,  1,  0,  0,  0,  0,  0,  // 80-95
     0,  5,  0,  0,  0,  0,  5,  0,  0, -2,  0,  0,  0,  0,  0, 12,  // 96-111
     2,  0, -2, -2, 20,  0,  0, 10,  0, 15,  0, -1,  0,  8, -2,  0,  // 112-127
     0,  0,  8,  0, 12,  0,  0,  7,  0,  0,  0,  0,  0, -1, -1,  0,  // 128-143
     4,  5,  0,  0,  0,  6,  0,  0,  0,  0,  8,  9,  0,  0,  0,  2,  // 144-159
    -1,  0, -2,  0,  4, 14,  0,  0,  0, 24,  0, -2,  5,  0,  0,  0,  // 160-175
    10,  0,  0,  4,  0,  0,  0,  0,  0,  0,  0,  6,  0,  0,  0,  2,  // 176-191
     1,  0,  0,  2, -1,  1,  0,  0,  0,  0, 14,  0,  0,  0,  0, 10,  // 192-207
     5,  0,  0,  0,  0,  0, -2,  0,  0,  9,  0,  0,  8,  0,  0,  0,  // 208-223
     0, -2,  6,  0,  0,  0, -2,  0,  3,  0,  1,  7,  0,  0,  0,  0,  // 224-239
     3,  0,  0,  0,  0,  0,  0, -1,  0,  0,  0,  0,  0,  3,  0,  0,  // 240-255
];

// ──────────────── Server Opcodes (from ServerProt.java) ────────────────

/// Map / Region
pub const REBUILD_NORMAL: u8 = 162;
pub const REBUILD_REGION: u8 = 214;

/// Zone updates
pub const UPDATE_ZONE_FULL_FOLLOWS: u8 = 112;
pub const UPDATE_ZONE_PARTIAL_FOLLOWS: u8 = 26;
pub const UPDATE_ZONE_PARTIAL_ENCLOSED: u8 = 230;
pub const LOC_ADD: u8 = 179;
pub const LOC_DEL: u8 = 195;
pub const LOC_ANIM: u8 = 20;
pub const OBJ_ADD: u8 = 135;
pub const OBJ_DEL: u8 = 240;
pub const OBJ_COUNT: u8 = 14;
pub const OBJ_REVEAL: u8 = 33;
pub const SOUND_AREA: u8 = 97;
pub const SPOTANIM_SPECIFIC: u8 = 17;
pub const MAP_PROJANIM: u8 = 104;

/// Entity updates
pub const PLAYER_INFO: u8 = 225;
pub const NPC_INFO: u8 = 32;

/// Var updates
pub const VARBIT_SMALL: u8 = 37;
pub const VARBIT_LARGE: u8 = 84;
pub const VARP_SMALL: u8 = 60;
pub const VARP_LARGE: u8 = 226;
pub const CLIENT_SETVARC_SMALL: u8 = 65;
pub const CLIENT_SETVARC_LARGE: u8 = 69;
pub const RESET_CLIENT_VARCACHE: u8 = 89;

/// Chat
pub const MESSAGE_GAME: u8 = 70;
pub const MESSAGE_PRIVATE: u8 = 0;
pub const MESSAGE_PRIVATE_ECHO: u8 = 71;
pub const MESSAGE_CLANCHANNEL: u8 = 54;

/// Interfaces
pub const IF_OPENTOP: u8 = 155;
pub const IF_OPENSUB: u8 = 145;
pub const IF_CLOSESUB: u8 = 149;
pub const IF_SETTEXT1: u8 = 171;
pub const IF_SETTEXT2: u8 = 48;
pub const IF_SETTEXT3: u8 = 123;
pub const IF_SETHIDE: u8 = 21;
pub const IF_SETANIM: u8 = 36;
pub const IF_SETOBJECT: u8 = 50;
pub const IF_SETNPCHEAD: u8 = 73;
pub const IF_SETPLAYERHEAD: u8 = 66;
pub const IF_SETMODEL: u8 = 130;
pub const IF_SETPOSITION: u8 = 119;
pub const IF_SETANGLE: u8 = 132;
pub const IF_SETCOLOUR: u8 = 2;
pub const IF_SETSCROLLPOS: u8 = 220;
pub const SET_INTERFACE_SETTINGS: u8 = 165;

/// Inventory
pub const UPDATE_INV_CLEAR: u8 = 144;
pub const UPDATE_INV_PARTIAL: u8 = 22;
pub const UPDATE_INV_FULL: u8 = 105;

/// Stats/Skills
pub const UPDATE_STAT: u8 = 38;
pub const UPDATE_RUNENERGY: u8 = 234;
pub const UPDATE_RUNWEIGHT: u8 = 159;

/// Audio
pub const MIDI_SONG: u8 = 4;
pub const MIDI_JINGLE: u8 = 208;
pub const SYNTH_SOUND: u8 = 172;

/// Misc
pub const LOGOUT: u8 = 86;
pub const UPDATE_REBOOT_TIMER: u8 = 85;
pub const HINT_ARROW: u8 = 217;
pub const CAM_RESET: u8 = 24;
pub const CAM_SHAKE: u8 = 27;
pub const CAM_LOOKAT: u8 = 125;
pub const CAM_FORCEANGLE: u8 = 187;
pub const RESET_ANIMS: u8 = 131;
pub const CLEAR_MINIMAP_FLAG: u8 = 153;
pub const SET_MINIMAP_STATE: u8 = 192;
pub const TELEPORT_LOCAL_PLAYER: u8 = 13;
pub const RUN_CS2: u8 = 115;
pub const SET_INTERACTION: u8 = 44;
pub const LAST_LOGIN_INFO: u8 = 164;

// ──────────────── Client Opcodes (from ClientProt.java) ────────────────

pub const CLIENT_CHEAT: u8 = 44;
pub const MOVE_GAMECLICK: u8 = 215;
pub const MOVE_MINIMAPCLICK: u8 = 39;
pub const CLOSE_MODAL: u8 = 184;
pub const NO_TIMEOUT: u8 = 93;
pub const WINDOW_STATUS: u8 = 243;
pub const EVENT_MOUSE_MOVE: u8 = 123;
pub const EVENT_MOUSE_CLICK: u8 = 75;
pub const EVENT_CAMERA_POSITION: u8 = 21;
pub const SOUND_SONGEND: u8 = 137;

// ──────────────── Packet handler ────────────────

/// Processes incoming server packets.
pub struct PacketHandler {
    pub in_cipher: Option<IsaacRandom>,
    pub out_cipher: Option<IsaacRandom>,
    /// Queued game messages.
    pub game_messages: Vec<String>,
    /// Queued stat updates: (skill_id, level, xp).
    pub stat_updates: Vec<(u8, u8, i32)>,
    /// Queued inventory updates: (slot, item_id, amount).
    pub inv_updates: Vec<(u16, u16, i32)>,
    /// Queued sound effects to play.
    pub sound_updates: Vec<u16>,
    /// Current run energy (0-10000).
    pub run_energy: u16,
    /// Current run weight.
    pub run_weight: i16,
    /// Server reboot timer (ticks).
    pub reboot_timer: Option<u16>,
    /// Whether a logout was requested.
    pub should_logout: bool,
    /// Current varps (server variables).
    pub varps: [i32; 2000],
}

impl PacketHandler {
    pub fn new() -> Self {
        PacketHandler {
            in_cipher: None,
            out_cipher: None,
            game_messages: Vec::new(),
            stat_updates: Vec::new(),
            inv_updates: Vec::new(),
            sound_updates: Vec::new(),
            run_energy: 10000,
            run_weight: 0,
            reboot_timer: None,
            should_logout: false,
            varps: [0; 2000],
        }
    }

    /// Process a single server packet.
    pub fn process_packet(&mut self, opcode: u8, data: &[u8]) {
        let mut buf = Buffer::wrap(data.to_vec());

        match opcode {
            UPDATE_STAT => {
                // 6 bytes: g4(xp) + g1(boosted_level) + g1(skill_id)
                if let (Ok(xp), Ok(level), Ok(skill_id)) = (buf.g4(), buf.g1(), buf.g1()) {
                    debug!("UPDATE_STAT: skill={} level={} xp={}", skill_id, level, xp);
                    self.stat_updates.push((skill_id, level, xp));
                }
            }
            UPDATE_RUNENERGY => {
                if let Ok(energy) = buf.g1() {
                    self.run_energy = (energy as u16) * 100; // 0-100 → 0-10000
                    debug!("UPDATE_RUNENERGY: {}", energy);
                }
            }
            UPDATE_RUNWEIGHT => {
                if let Ok(weight) = buf.g2b() {
                    self.run_weight = weight;
                    debug!("UPDATE_RUNWEIGHT: {}", weight);
                }
            }
            MESSAGE_GAME => {
                if let Ok(msg) = buf.gjstr() {
                    info!("MESSAGE_GAME: {}", msg);
                    self.game_messages.push(msg);
                }
            }
            SYNTH_SOUND => {
                if let (Ok(sound_id), Ok(_loops), Ok(_delay)) = (buf.g2(), buf.g1(), buf.g2()) {
                    debug!("SYNTH_SOUND: id={}", sound_id);
                    self.sound_updates.push(sound_id);
                }
            }
            MIDI_SONG => {
                if let Ok(song_id) = buf.g2() {
                    info!("MIDI_SONG: id={}", song_id);
                }
            }
            LOGOUT => {
                info!("LOGOUT received from server");
                self.should_logout = true;
            }
            UPDATE_REBOOT_TIMER => {
                if let Ok(timer) = buf.g2() {
                    info!("UPDATE_REBOOT_TIMER: {} ticks ({} minutes)", timer, timer / 100);
                    self.reboot_timer = Some(timer);
                }
            }
            VARP_SMALL => {
                if let (Ok(varp_id), Ok(value)) = (buf.g2(), buf.g1()) {
                    let idx = varp_id as usize;
                    if idx < self.varps.len() {
                        self.varps[idx] = value as i32;
                        debug!("VARP_SMALL: id={} val={}", varp_id, value);
                    }
                }
            }
            VARP_LARGE => {
                if let (Ok(varp_id), Ok(value)) = (buf.g2(), buf.g4()) {
                    let idx = varp_id as usize;
                    if idx < self.varps.len() {
                        self.varps[idx] = value;
                        debug!("VARP_LARGE: id={} val={}", varp_id, value);
                    }
                }
            }
            IF_OPENTOP => {
                if let Ok(top_id) = buf.g2() {
                    info!("IF_OPENTOP: id={}", top_id);
                }
            }
            IF_OPENSUB => {
                // 5 bytes: component + interface + type
                debug!("IF_OPENSUB packet");
            }
            IF_CLOSESUB => {
                debug!("IF_CLOSESUB packet");
            }
            UPDATE_INV_FULL => {
                // Variable-length inventory update
                debug!("UPDATE_INV_FULL packet ({} bytes)", data.len());
            }
            UPDATE_INV_PARTIAL => {
                debug!("UPDATE_INV_PARTIAL packet");
            }
            REBUILD_NORMAL => {
                info!("REBUILD_NORMAL packet ({} bytes) — map region change", data.len());
            }
            PLAYER_INFO => {
                debug!("PLAYER_INFO packet ({} bytes)", data.len());
            }
            NPC_INFO => {
                debug!("NPC_INFO packet ({} bytes)", data.len());
            }
            CLEAR_MINIMAP_FLAG => {
                debug!("CLEAR_MINIMAP_FLAG");
            }
            RESET_ANIMS => {
                debug!("RESET_ANIMS");
            }
            CAM_RESET => {
                debug!("CAM_RESET");
            }
            HINT_ARROW => {
                debug!("HINT_ARROW packet");
            }
            LAST_LOGIN_INFO => {
                debug!("LAST_LOGIN_INFO packet");
            }
            RUN_CS2 => {
                debug!("RUN_CS2 clientscript");
            }
            _ => {
                debug!("Unhandled server opcode: {} ({} bytes)", opcode, data.len());
            }
        }
    }

    /// Drain all queued game messages.
    pub fn drain_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.game_messages)
    }

    /// Drain all queued stat updates.
    pub fn drain_stats(&mut self) -> Vec<(u8, u8, i32)> {
        std::mem::take(&mut self.stat_updates)
    }

    /// Drain all queued inventory updates.
    pub fn drain_inv_updates(&mut self) -> Vec<(u16, u16, i32)> {
        std::mem::take(&mut self.inv_updates)
    }

    /// Drain all queued sound updates.
    pub fn drain_sounds(&mut self) -> Vec<u16> {
        std::mem::take(&mut self.sound_updates)
    }
}

/// Build a no-timeout (keepalive) packet.
pub fn build_no_timeout(cipher: &mut IsaacRandom) -> Vec<u8> {
    let opcode = NO_TIMEOUT.wrapping_add(cipher.next_key() as u8);
    vec![opcode]
}

/// Build a window status packet.
pub fn build_window_status(cipher: &mut IsaacRandom, mode: u8, width: u16, height: u16) -> Vec<u8> {
    let opcode = WINDOW_STATUS.wrapping_add(cipher.next_key() as u8);
    let mut buf = Buffer::new(6);
    buf.p1(opcode);
    buf.p1(mode);
    buf.p2(width);
    buf.p2(height);
    buf.p1(0); // anti-aliasing
    buf.data[..buf.pos].to_vec()
}
