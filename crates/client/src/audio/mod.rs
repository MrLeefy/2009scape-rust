//! Audio system — sound effects and music playback.
//!
//! Phase 6: Basic audio pipeline with spatial sound support.
//! Uses web-audio on WASM, rodio/cpal on native.

use std::collections::HashMap;

/// Sound effect categories matching RS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    // UI
    ButtonClick,
    TabSwitch,
    ItemPickup,
    ItemDrop,
    // Combat
    MeleeHit,
    MeleeMiss,
    RangedShot,
    MagicCast,
    PlayerDeath,
    NpcDeath,
    EatFood,
    DrinkPotion,
    // Skills
    TreeChop,
    OreHit,
    FishCatch,
    CookItem,
    FireLight,
    SmithAnvil,
    FletchCut,
    CraftItem,
    // Environment
    DoorOpen,
    DoorClose,
    LevelUp,
    QuestComplete,
    TeleportCast,
}

/// Music track identifiers.
#[derive(Debug, Clone)]
pub struct MusicTrack {
    pub id: u32,
    pub name: String,
    pub region: String,
    pub duration_secs: f32,
}

/// Audio engine state.
pub struct AudioEngine {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
    pub current_track: Option<MusicTrack>,
    pub track_position: f32,
    queued_sfx: Vec<(SoundEffect, f32, f32, f32)>, // effect, x, y, z
    known_tracks: Vec<MusicTrack>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let tracks = vec![
            MusicTrack { id: 0, name: "Scape Main".into(), region: "Login".into(), duration_secs: 180.0 },
            MusicTrack { id: 1, name: "Newbie Melody".into(), region: "Lumbridge".into(), duration_secs: 120.0 },
            MusicTrack { id: 2, name: "Harmony".into(), region: "Lumbridge".into(), duration_secs: 150.0 },
            MusicTrack { id: 3, name: "Adventure".into(), region: "Varrock".into(), duration_secs: 200.0 },
            MusicTrack { id: 4, name: "Autumn Voyage".into(), region: "Falador".into(), duration_secs: 165.0 },
            MusicTrack { id: 5, name: "Spirit".into(), region: "Wilderness".into(), duration_secs: 180.0 },
            MusicTrack { id: 6, name: "Medieval".into(), region: "Draynor".into(), duration_secs: 140.0 },
            MusicTrack { id: 7, name: "Flute Salad".into(), region: "Falador Park".into(), duration_secs: 130.0 },
            MusicTrack { id: 8, name: "Sea Shanty 2".into(), region: "Port Sarim".into(), duration_secs: 155.0 },
        ];

        AudioEngine {
            master_volume: 0.8,
            music_volume: 0.5,
            sfx_volume: 0.7,
            music_enabled: true,
            sfx_enabled: true,
            current_track: None,
            track_position: 0.0,
            queued_sfx: Vec::new(),
            known_tracks: tracks,
        }
    }

    /// Queue a sound effect at a world position.
    pub fn play_sfx(&mut self, effect: SoundEffect, x: f32, y: f32, z: f32) {
        if !self.sfx_enabled { return; }
        self.queued_sfx.push((effect, x, y, z));
    }

    /// Play a sound effect without position (UI sounds).
    pub fn play_ui_sfx(&mut self, effect: SoundEffect) {
        self.play_sfx(effect, 0.0, 0.0, 0.0);
    }

    /// Set the music track for a region.
    pub fn set_region_music(&mut self, region: &str) {
        if !self.music_enabled { return; }
        if let Some(track) = self.known_tracks.iter().find(|t| t.region == region) {
            if self.current_track.as_ref().map(|t| t.id) != Some(track.id) {
                self.current_track = Some(track.clone());
                self.track_position = 0.0;
            }
        }
    }

    /// Tick the audio engine (process queued effects, advance music).
    pub fn tick(&mut self, dt: f32, _listener_x: f32, _listener_y: f32, _listener_z: f32) {
        // Advance music
        if let Some(track) = &self.current_track {
            self.track_position += dt;
            if self.track_position >= track.duration_secs {
                self.track_position = 0.0; // loop
            }
        }

        // Process queued SFX (in a real impl, this would submit to audio backend)
        // For now we just drain the queue
        self.queued_sfx.clear();
    }

    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_volume = vol.clamp(0.0, 1.0);
    }
}
