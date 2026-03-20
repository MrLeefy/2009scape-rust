//! Cache definition loaders — ObjType, NpcType, LocType.
//!
//! Translated from rt4/ObjType.java, NpcType.java, LocType.java.
//! Reads item/NPC/location definitions from the cache.

use rs2_common::buffer::Buffer;
use super::{Js5Cache, archives};
use std::collections::HashMap;
use anyhow::{Result, bail};
use log::{info, debug, warn};

// ──────────────────────────── ObjType ────────────────────────────

/// An item definition from the cache.
#[derive(Debug, Clone)]
pub struct ObjType {
    pub id: u32,
    pub name: String,
    pub model: u16,
    pub zoom2d: u16,
    pub x_angle_2d: u16,
    pub y_angle_2d: u16,
    pub z_angle_2d: u16,
    pub x_offset_2d: i16,
    pub y_offset_2d: i16,
    pub stackable: bool,
    pub cost: i32,
    pub members: bool,
    pub team: u8,
    pub ops: [Option<String>; 5],       // ground options (right-click)
    pub iops: [Option<String>; 5],      // inventory options
    pub stock_market: bool,
    pub cert_link: i16,
    pub cert_template: i16,
    pub lent_link: i16,
    pub lent_template: i16,
    pub man_wear: i16,
    pub woman_wear: i16,
}

impl ObjType {
    fn new(id: u32) -> Self {
        ObjType {
            id,
            name: "null".into(),
            model: 0,
            zoom2d: 2000,
            x_angle_2d: 0,
            y_angle_2d: 0,
            z_angle_2d: 0,
            x_offset_2d: 0,
            y_offset_2d: 0,
            stackable: false,
            cost: 1,
            members: false,
            team: 0,
            ops: [None, None, Some("Take".into()), None, None],
            iops: [None, None, None, None, Some("Drop".into())],
            stock_market: false,
            cert_link: -1,
            cert_template: -1,
            lent_link: -1,
            lent_template: -1,
            man_wear: -1,
            woman_wear: -1,
        }
    }

    /// Decode an ObjType from buffer data.
    /// Translated from ObjType.decode() in Java.
    fn decode(id: u32, data: &[u8]) -> Result<Self> {
        let mut obj = ObjType::new(id);
        let mut buf = Buffer::wrap(data.to_vec());

        loop {
            let opcode = buf.g1()?;
            if opcode == 0 { break; }

            match opcode {
                1 => { obj.model = buf.g2()?; }
                2 => { obj.name = buf.gjstr()?; }
                4 => { obj.zoom2d = buf.g2()?; }
                5 => { obj.x_angle_2d = buf.g2()?; }
                6 => { obj.y_angle_2d = buf.g2()?; }
                7 => {
                    let v = buf.g2()? as i32;
                    obj.x_offset_2d = if v > 32767 { (v - 65536) as i16 } else { v as i16 };
                }
                8 => {
                    let v = buf.g2()? as i32;
                    obj.y_offset_2d = if v > 32767 { (v - 65536) as i16 } else { v as i16 };
                }
                11 => { obj.stackable = true; }
                12 => { obj.cost = buf.g4()?; }
                16 => { obj.members = true; }
                23 => { obj.man_wear = buf.g2()? as i16; }
                24 => { let _ = buf.g2()?; } // manwear2
                25 => { obj.woman_wear = buf.g2()? as i16; }
                26 => { let _ = buf.g2()?; } // womanwear2
                30..=34 => {
                    let s = buf.gjstr()?;
                    obj.ops[(opcode - 30) as usize] = if s == "Hidden" { None } else { Some(s) };
                }
                35..=39 => {
                    obj.iops[(opcode - 35) as usize] = Some(buf.gjstr()?);
                }
                40 => {
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g2()?; let _ = buf.g2()?; }
                }
                41 => {
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g2()?; let _ = buf.g2()?; }
                }
                42 => {
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g1()?; }
                }
                65 => { obj.stock_market = true; }
                78 | 79 => { let _ = buf.g2()?; } // manwear3/womanwear3
                90 | 91 | 92 | 93 => { let _ = buf.g2()?; } // head models
                95 => { obj.z_angle_2d = buf.g2()?; }
                96 => { let _ = buf.g1()?; } // dummyItem
                97 => { obj.cert_link = buf.g2()? as i16; }
                98 => { obj.cert_template = buf.g2()? as i16; }
                100..=109 => { let _ = buf.g2()?; let _ = buf.g2()?; } // countobj/countco
                110 | 111 | 112 => { let _ = buf.g2()?; } // resize
                113 | 114 => { let _ = buf.g1()?; } // ambient/contrast (g1b)
                115 => { obj.team = buf.g1()?; }
                121 => { obj.lent_link = buf.g2()? as i16; }
                122 => { obj.lent_template = buf.g2()? as i16; }
                125 | 126 => { let _ = buf.g1()?; let _ = buf.g1()?; let _ = buf.g1()?; }
                127 | 128 => { let _ = buf.g1()?; let _ = buf.g2()?; }
                129 | 130 => { let _ = buf.g1()?; let _ = buf.g2()?; }
                249 => {
                    let size = buf.g1()?;
                    for _ in 0..size {
                        let is_string = buf.g1()? == 1;
                        let _ = buf.g3()?;
                        if is_string { let _ = buf.gjstr()?; } else { let _ = buf.g4()?; }
                    }
                }
                _ => { warn!("Unknown ObjType opcode: {}", opcode); break; }
            }
        }
        Ok(obj)
    }
}

// ──────────────────────────── NpcType ────────────────────────────

/// An NPC definition from the cache.
#[derive(Debug, Clone)]
pub struct NpcType {
    pub id: u32,
    pub name: String,
    pub size: u8,
    pub combat_level: i16,
    pub ops: [Option<String>; 5],
    pub visible_on_minimap: bool,
    pub click_area: u16,
    pub head_icon: i16,
}

impl NpcType {
    fn new(id: u32) -> Self {
        NpcType {
            id,
            name: "null".into(),
            size: 1,
            combat_level: -1,
            ops: [None, None, None, None, None],
            visible_on_minimap: true,
            click_area: 128,
            head_icon: -1,
        }
    }

    /// Decode an NpcType from buffer data.
    fn decode(id: u32, data: &[u8]) -> Result<Self> {
        let mut npc = NpcType::new(id);
        let mut buf = Buffer::wrap(data.to_vec());

        loop {
            let opcode = buf.g1()?;
            if opcode == 0 { break; }

            match opcode {
                1 => {
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g2()?; }
                }
                2 => { npc.name = buf.gjstr()?; }
                12 => { npc.size = buf.g1()?; }
                13 => { let _ = buf.g2()?; } // standanim
                14 => { let _ = buf.g2()?; } // walkanim
                17 => {
                    let _ = buf.g2()?; let _ = buf.g2()?;
                    let _ = buf.g2()?; let _ = buf.g2()?;
                }
                30..=34 => {
                    let s = buf.gjstr()?;
                    npc.ops[(opcode - 30) as usize] = if s == "Hidden" { None } else { Some(s) };
                }
                40 => {
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g2()?; let _ = buf.g2()?; }
                }
                41 => {
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g2()?; let _ = buf.g2()?; }
                }
                42 => {
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g1()?; }
                }
                60 => {
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g2()?; }
                }
                93 => { npc.visible_on_minimap = false; }
                95 => { npc.combat_level = buf.g2()? as i16; }
                97 => { npc.click_area = buf.g2()?; }
                98 => { npc.head_icon = buf.g2()? as i16; }
                99 | 100 | 101 | 102 | 103 => { let _ = buf.g2()?; }
                106 | 118 => {
                    let _ = buf.g2()?;
                    let count = buf.g1()?;
                    for _ in 0..count { let _ = buf.g2()?; }
                }
                107 | 109 => { /* boolean flags */ }
                111 | 114 | 119 => { /* boolean flags */ }
                113 => { let _ = buf.g2()?; let _ = buf.g2()?; }
                249 => {
                    let size = buf.g1()?;
                    for _ in 0..size {
                        let is_string = buf.g1()? == 1;
                        let _ = buf.g3()?;
                        if is_string { let _ = buf.gjstr()?; } else { let _ = buf.g4()?; }
                    }
                }
                _ => { debug!("Unknown NpcType opcode: {} for npc {}", opcode, id); break; }
            }
        }
        Ok(npc)
    }
}

// ──────────────────────────── DefinitionLoader ────────────────────

/// Loads and caches definitions from the JS5 cache.
pub struct DefinitionLoader {
    pub items: HashMap<u32, ObjType>,
    pub npcs: HashMap<u32, NpcType>,
}

impl DefinitionLoader {
    pub fn new() -> Self {
        DefinitionLoader {
            items: HashMap::new(),
            npcs: HashMap::new(),
        }
    }

    /// Load all item definitions from the cache.
    pub fn load_items(&mut self, cache: &Js5Cache) -> Result<usize> {
        let idx = cache.parsed_indices.get(&archives::CONFIG_OBJ)
            .ok_or_else(|| anyhow::anyhow!("CONFIG_OBJ index (16) not found in cache"))?;

        let mut count = 0;
        for &gid in &idx.group_ids {
            match cache.read_file(archives::CONFIG_OBJ, gid as u32) {
                Ok(data) => {
                    match ObjType::decode(gid as u32, &data) {
                        Ok(obj) => {
                            self.items.insert(gid as u32, obj);
                            count += 1;
                        }
                        Err(e) => debug!("Failed to decode item {}: {}", gid, e),
                    }
                }
                Err(e) => debug!("Failed to read item {}: {}", gid, e),
            }
        }

        info!("Loaded {} item definitions", count);
        Ok(count)
    }

    /// Load all NPC definitions from the cache.
    pub fn load_npcs(&mut self, cache: &Js5Cache) -> Result<usize> {
        let idx = cache.parsed_indices.get(&archives::CONFIG_NPC)
            .ok_or_else(|| anyhow::anyhow!("CONFIG_NPC index (18) not found in cache"))?;

        let mut count = 0;
        for &gid in &idx.group_ids {
            match cache.read_file(archives::CONFIG_NPC, gid as u32) {
                Ok(data) => {
                    match NpcType::decode(gid as u32, &data) {
                        Ok(npc) => {
                            self.npcs.insert(gid as u32, npc);
                            count += 1;
                        }
                        Err(e) => debug!("Failed to decode NPC {}: {}", gid, e),
                    }
                }
                Err(e) => debug!("Failed to read NPC {}: {}", gid, e),
            }
        }

        info!("Loaded {} NPC definitions", count);
        Ok(count)
    }

    /// Get an item by ID.
    pub fn get_item(&self, id: u32) -> Option<&ObjType> {
        self.items.get(&id)
    }

    /// Get an NPC by ID.
    pub fn get_npc(&self, id: u32) -> Option<&NpcType> {
        self.npcs.get(&id)
    }

    /// Search items by name (case-insensitive partial match).
    pub fn search_items(&self, query: &str) -> Vec<&ObjType> {
        let q = query.to_lowercase();
        self.items.values()
            .filter(|item| item.name.to_lowercase().contains(&q))
            .collect()
    }

    /// Search NPCs by name.
    pub fn search_npcs(&self, query: &str) -> Vec<&NpcType> {
        let q = query.to_lowercase();
        self.npcs.values()
            .filter(|npc| npc.name.to_lowercase().contains(&q))
            .collect()
    }
}
