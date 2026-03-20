//! Skills system — XP tables, level calculations, skilling actions.

/// RS XP table — level 1 to 99.
pub const XP_TABLE: [u32; 100] = [
    0,       // level 0 (unused)
    0,       // level 1
    83,      // level 2
    174,     // level 3
    276,     // level 4
    388,     // level 5
    512,     // level 6
    650,     // level 7
    801,     // level 8
    969,     // level 9
    1154,    // level 10
    1358,    // level 11
    1584,    // level 12
    1833,    // level 13
    2107,    // level 14
    2411,    // level 15
    2746,    // level 16
    3115,    // level 17
    3523,    // level 18
    3973,    // level 19
    4470,    // level 20
    5018,    // level 21
    5624,    // level 22
    6291,    // level 23
    7028,    // level 24
    7842,    // level 25
    8740,    // level 26
    9730,    // level 27
    10824,   // level 28
    12031,   // level 29
    13363,   // level 30
    14833,   // level 31
    16456,   // level 32
    18247,   // level 33
    20224,   // level 34
    22406,   // level 35
    24815,   // level 36
    27473,   // level 37
    30408,   // level 38
    33648,   // level 39
    37224,   // level 40
    41171,   // level 41
    45529,   // level 42
    50339,   // level 43
    55649,   // level 44
    61512,   // level 45
    67983,   // level 46
    75127,   // level 47
    83014,   // level 48
    91721,   // level 49
    101333,  // level 50
    111945,  // level 51
    123660,  // level 52
    136594,  // level 53
    150872,  // level 54
    166636,  // level 55
    184040,  // level 56
    203254,  // level 57
    224466,  // level 58
    247886,  // level 59
    273742,  // level 60
    302288,  // level 61
    333804,  // level 62
    368599,  // level 63
    407015,  // level 64
    449428,  // level 65
    496254,  // level 66
    547953,  // level 67
    605032,  // level 68
    668051,  // level 69
    737627,  // level 70
    814445,  // level 71
    899257,  // level 72
    992895,  // level 73
    1096278, // level 74
    1210421, // level 75
    1336443, // level 76
    1475581, // level 77
    1629200, // level 78
    1798808, // level 79
    1986068, // level 80
    2192818, // level 81
    2421087, // level 82
    2673114, // level 83
    2951373, // level 84
    3258594, // level 85
    3597792, // level 86
    3972294, // level 87
    4385776, // level 88
    4842295, // level 89
    5346332, // level 90
    5902831, // level 91
    6517253, // level 92
    7195629, // level 93
    7944614, // level 94
    8771558, // level 95
    9684577, // level 96
    10692629, // level 97
    11805606, // level 98
    13034431, // level 99
];

/// Get level for XP amount.
pub fn level_for_xp(xp: u32) -> u8 {
    for level in (1..=99u8).rev() {
        if xp >= XP_TABLE[level as usize] {
            return level;
        }
    }
    1
}

/// Get XP needed for next level.
pub fn xp_to_next_level(current_xp: u32) -> u32 {
    let current_level = level_for_xp(current_xp);
    if current_level >= 99 { return 0; }
    XP_TABLE[current_level as usize + 1] - current_xp
}

/// Get progress percentage to next level.
pub fn level_progress(current_xp: u32) -> f32 {
    let level = level_for_xp(current_xp);
    if level >= 99 { return 1.0; }
    let current_level_xp = XP_TABLE[level as usize];
    let next_level_xp = XP_TABLE[level as usize + 1];
    let range = next_level_xp - current_level_xp;
    if range == 0 { return 1.0; }
    (current_xp - current_level_xp) as f32 / range as f32
}

/// Skilling action types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkillingAction {
    // Woodcutting
    ChopTree,
    ChopOak,
    ChopWillow,
    ChopMaple,
    ChopYew,
    ChopMagic,
    // Mining
    MineCopper,
    MineTin,
    MineIron,
    MineCoal,
    MineMithril,
    MineAdamant,
    MineRunite,
    // Fishing
    FishShrimps,
    FishTrout,
    FishSalmon,
    FishLobster,
    FishSwordfish,
    FishShark,
    // Cooking
    CookShrimps,
    CookTrout,
    CookLobster,
    CookSwordfish,
    CookShark,
    // Firemaking
    BurnLogs,
    BurnOak,
    BurnWillow,
    BurnMaple,
    BurnYew,
    BurnMagic,
    // Smithing
    SmeltBronze,
    SmeltIron,
    SmeltSteel,
    SmeltMithril,
    SmeltAdamant,
    SmeltRunite,
}

/// Skilling action definition.
#[derive(Debug, Clone)]
pub struct ActionDef {
    pub action: SkillingAction,
    pub skill_name: String,
    pub level_required: u8,
    pub xp_reward: f32,
    pub tick_duration: u8,
    pub product_id: u32,
    pub tool_required: Option<u32>,
    pub success_chance: f32,
}

/// Get all skilling action definitions.
pub fn get_action_defs() -> Vec<ActionDef> {
    vec![
        ActionDef { action: SkillingAction::ChopTree, skill_name: "Woodcutting".into(), level_required: 1, xp_reward: 25.0, tick_duration: 4, product_id: 1511, tool_required: Some(1351), success_chance: 0.7 },
        ActionDef { action: SkillingAction::ChopOak, skill_name: "Woodcutting".into(), level_required: 15, xp_reward: 37.5, tick_duration: 5, product_id: 1521, tool_required: Some(1351), success_chance: 0.5 },
        ActionDef { action: SkillingAction::ChopWillow, skill_name: "Woodcutting".into(), level_required: 30, xp_reward: 67.5, tick_duration: 5, product_id: 1519, tool_required: Some(1351), success_chance: 0.4 },
        ActionDef { action: SkillingAction::ChopYew, skill_name: "Woodcutting".into(), level_required: 60, xp_reward: 175.0, tick_duration: 8, product_id: 1515, tool_required: Some(1351), success_chance: 0.25 },
        ActionDef { action: SkillingAction::ChopMagic, skill_name: "Woodcutting".into(), level_required: 75, xp_reward: 250.0, tick_duration: 10, product_id: 1513, tool_required: Some(1351), success_chance: 0.15 },
        ActionDef { action: SkillingAction::MineCopper, skill_name: "Mining".into(), level_required: 1, xp_reward: 17.5, tick_duration: 4, product_id: 436, tool_required: Some(1265), success_chance: 0.7 },
        ActionDef { action: SkillingAction::MineTin, skill_name: "Mining".into(), level_required: 1, xp_reward: 17.5, tick_duration: 4, product_id: 438, tool_required: Some(1265), success_chance: 0.7 },
        ActionDef { action: SkillingAction::MineIron, skill_name: "Mining".into(), level_required: 15, xp_reward: 35.0, tick_duration: 5, product_id: 440, tool_required: Some(1265), success_chance: 0.5 },
        ActionDef { action: SkillingAction::MineCoal, skill_name: "Mining".into(), level_required: 30, xp_reward: 50.0, tick_duration: 6, product_id: 453, tool_required: Some(1265), success_chance: 0.4 },
        ActionDef { action: SkillingAction::FishShrimps, skill_name: "Fishing".into(), level_required: 1, xp_reward: 10.0, tick_duration: 4, product_id: 315, tool_required: None, success_chance: 0.8 },
        ActionDef { action: SkillingAction::FishTrout, skill_name: "Fishing".into(), level_required: 20, xp_reward: 50.0, tick_duration: 5, product_id: 335, tool_required: None, success_chance: 0.5 },
        ActionDef { action: SkillingAction::FishLobster, skill_name: "Fishing".into(), level_required: 40, xp_reward: 90.0, tick_duration: 6, product_id: 377, tool_required: None, success_chance: 0.35 },
        ActionDef { action: SkillingAction::FishSwordfish, skill_name: "Fishing".into(), level_required: 50, xp_reward: 100.0, tick_duration: 7, product_id: 371, tool_required: None, success_chance: 0.25 },
        ActionDef { action: SkillingAction::CookShrimps, skill_name: "Cooking".into(), level_required: 1, xp_reward: 30.0, tick_duration: 4, product_id: 315, tool_required: None, success_chance: 0.9 },
        ActionDef { action: SkillingAction::CookLobster, skill_name: "Cooking".into(), level_required: 40, xp_reward: 120.0, tick_duration: 4, product_id: 379, tool_required: None, success_chance: 0.6 },
        ActionDef { action: SkillingAction::BurnLogs, skill_name: "Firemaking".into(), level_required: 1, xp_reward: 40.0, tick_duration: 4, product_id: 0, tool_required: Some(590), success_chance: 0.8 },
        ActionDef { action: SkillingAction::BurnOak, skill_name: "Firemaking".into(), level_required: 15, xp_reward: 60.0, tick_duration: 4, product_id: 0, tool_required: Some(590), success_chance: 0.7 },
        ActionDef { action: SkillingAction::SmeltBronze, skill_name: "Smithing".into(), level_required: 1, xp_reward: 6.25, tick_duration: 4, product_id: 2349, tool_required: None, success_chance: 1.0 },
        ActionDef { action: SkillingAction::SmeltIron, skill_name: "Smithing".into(), level_required: 15, xp_reward: 12.5, tick_duration: 4, product_id: 2351, tool_required: None, success_chance: 0.5 },
    ]
}

/// Calculate combat level from skill levels.
pub fn calculate_combat_level(
    attack: u8, strength: u8, defence: u8, hitpoints: u8,
    prayer: u8, ranged: u8, magic: u8, summoning: u8,
) -> u8 {
    let base = (defence as f64 + hitpoints as f64 + (prayer as f64 / 2.0).floor() + (summoning as f64 / 2.0).floor()) * 0.25;
    let melee = (attack as f64 + strength as f64) * 0.325;
    let range = (ranged as f64 * 1.5).floor() * 0.325;
    let mage = (magic as f64 * 1.5).floor() * 0.325;
    let combat_class = melee.max(range).max(mage);
    (base + combat_class) as u8
}
