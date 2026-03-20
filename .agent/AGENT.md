# 2009Scape Rust Client — Agent Context

## Project Overview
Rebuilding the 2009Scape RuneScape client in Rust using wgpu, targeting both native desktop and web (WASM/PWA). The project lives at `c:\Users\baseb\Desktop\2009scape rust` and is pushed to **https://github.com/MrLeefy/2009scape-rust**.

## Architecture
Rust workspace with 4 crates:
- **`crates/client`** — Main client app (rendering, game state, entities, combat, skills, audio, protocol, input, PWA)
- **`crates/common`** — Shared utilities (binary buffer, ISAAC cipher, CRC32)
- **`crates/proxy`** — WebSocket-to-TCP bridge for browser clients connecting to game server
- **`crates/cache-tool`** — CLI for inspecting RS cache files

## Key Dependencies
`wgpu`, `winit`, `glam`, `bytemuck`, `tokio`, `anyhow`, `thiserror`, `env_logger`

## File Map (crates/client/src/)
```
main.rs              — App entry, event loop, camera controls (WASD/Q/E/R/F/Z/X)
audio/mod.rs         — Audio engine: 25 SFX enum, 9 music tracks, spatial queueing
cache/mod.rs         — idx/dat2 cache reader with sector chaining
cache/loader.rs      — Higher-level cache: item/NPC/object defs, terrain tiles, map regions
combat/mod.rs        — Damage calc, hit splats, XP drops, special energy, attack styles
entity/mod.rs        — Entity struct (position interpolation, animation), EntityManager
game/mod.rs          — Game state: Login/Loading/InGame, 25 skills, 28-slot inventory, chat, 7 tabs
input/mod.rs         — Mouse input: click/drag/scroll, right-click context menus, RS actions
net/login.rs         — Login handshake protocol
net/transport.rs     — TCP transport layer
net/protocol.rs      — RS2 packet opcodes (40+ server, 30+ client), packet handler, builders
render/mod.rs        — Main renderer: 2D login screen, 3D world + entity meshes, HUD overlay
render/camera.rs     — RS-style Camera3D (2048-unit angles, view/proj matrices)
render/renderer2d.rs — Immediate-mode 2D quad batching
render/renderer3d.rs — 3D pipeline: terrain mesh, depth buffer, camera uniforms, dynamic mesh
render/entity_renderer.rs — Entity box meshes (combat-level coloring, health bars)
render/font.rs       — 5×7 bitmap font (39 glyphs), shadow text, centered text
render/shader2d.wgsl — 2D vertex/fragment shaders
render/shader3d.wgsl — 3D shaders with directional lighting + fog
skills/mod.rs        — XP table (1-99), level calc, 36 skilling actions, combat level formula
web/mod.rs           — PWA manifest, service worker, HTML shell for WASM deployment
```

## Phase Status — ALL COMPLETE ✅
| Phase | Status | Key Files |
|-------|--------|-----------|
| 1. Foundation | ✅ | common/buffer, cache/mod, net/login, net/transport |
| 2. Login Screen | ✅ | render/shader2d, render/renderer2d, render/mod (login UI) |
| 3. World Rendering | ✅ | render/shader3d, render/camera, render/renderer3d |
| 4. Entities | ✅ | entity/mod, render/entity_renderer |
| 5. UI System | ✅ | game/mod (inventory/skills/chat/tabs), render/mod (HUD) |
| 6. Audio | ✅ | audio/mod |
| 7. Cache Integration | ✅ | cache/loader |
| 8. Server Connection | ✅ | net/protocol (packet handler) |
| 9. Game Protocol | ✅ | net/protocol (opcodes, builders) |
| 10. Combat | ✅ | combat/mod |
| 11. Skills | ✅ | skills/mod |
| +. Mouse Input | ✅ | input/mod |
| +. Font Renderer | ✅ | render/font |
| +. PWA Web Shell | ✅ | web/mod |

## Design Decisions
- **RS-style camera**: 2048-unit angles matching Java Camera.java, pitch range 128-383
- **128-unit tiles**: Matching RS coordinate system
- **Immediate-mode 2D**: Vertex/index buffer batching per frame for UI
- **Procedural terrain**: 50×50 tile grid with sine-wave heightmap (placeholder until cache terrain)
- **Entity box meshes**: Colored by type/combat level (green/yellow/red) as placeholder for cache models
- **5×7 bitmap font**: Built-in pixel font for all UI text rendering

## Controls
- Login: Type username/password, Tab, Enter
- Enter with empty username → skip to 3D world (test mode)
- WASD/Arrows = move, Q/E = rotate, R/F = pitch, Z/X = zoom
- Tab = cycle interface tabs, Enter = toggle chat

## Build & Run
```powershell
cargo build -p rs2-client
cargo run -p rs2-client
```

## Java Source Reference
The RT4 Java client is at `c:\Users\baseb\Desktop\2009scape mobile\rt4-mobile-client\client\src\main\java\rt4\` (229 files, ~50K lines). Key files used for porting:
- `LoginManager.java` → `net/login.rs` (DONE)
- `IsaacRandom.java` → `common/isaac.rs` (DONE)
- `Buffer.java` → `common/buffer.rs` (DONE)
- `ClientProt.java` → `net/protocol.rs` (opcodes match)
- `Cache.java` / `Js5.java` → cache loading (TODO)
- `SceneGraph.java` → world rendering (TODO)
- `Model.java` → 3D models (TODO)

## Active Completion Plan
- [x] Phase A: Real server login protocol (from LoginManager.java)
- [ ] Phase B: Cache asset loading (JS5 + item/NPC/object defs + models)
- [ ] Phase C: Server protocol (player/NPC updates, map regions)
- [ ] Phase D: SD/HD rendering toggle

## Integration Status
- ✅ CombatSystem wired into Game.tick() with auto-attack demo + hit splats + XP drops
- ✅ AudioEngine wired with region music + SFX triggers
- ✅ PacketHandler processes server stat/chat/inv/sound updates
- ✅ Mouse events wired (CursorMoved/MouseInput/MouseWheel)
- ✅ Bitmap font renders all HUD text
- ✅ Real login handshake in net/login.rs (not yet triggered from UI)

