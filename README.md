# 2009Scape Rust Client

A modern Rust/WASM rewrite of the RuneScape RT4 client. Targets both native desktop and browser (PWA) via WebAssembly.

## Features

- **Single wgpu renderer** — one GPU pipeline with SD/HD quality presets (no separate codebases)
- **Native + Browser** — runs on desktop (Vulkan/DX12/Metal) and in-browser via WebAssembly
- **WebSocket Proxy** — bridges browser WebSocket connections to the game server's TCP protocol
- **Full RS2 protocol** — binary buffer with all endian variants, ISAAC cipher, XTEA, CRC32
- **Cache reader** — reads idx/dat2 format game cache files

## Architecture

```
crates/
├── client/         Main game client (wgpu + winit)
│   ├── cache/      RS2 cache file reader
│   ├── game/       Game state machine
│   ├── net/        TCP transport + login protocol
│   └── render/     wgpu renderer + 2D sprite pipeline
├── common/         Shared types (buffer, ISAAC, CRC32)
├── proxy/          WebSocket-to-TCP bridge
└── cache-tool/     CLI cache inspector
```

## Building

```bash
# Build everything
cargo build

# Run the client
cargo run -p rs2-client

# Run the WebSocket proxy
cargo run -p rs2-proxy

# Inspect cache files
cargo run -p rs2-cache-tool -- /path/to/cache
```

## Requirements

- Rust 1.75+ (2021 edition)
- GPU with Vulkan, DX12, or Metal support (or WebGPU for browser)

## Development Phases

| Phase | Status | Description |
|-------|--------|-------------|
| 1. Foundation | ✅ Done | Buffer, ISAAC, cache, networking, wgpu setup |
| 2. Login Screen | ✅ Done | 2D rendering pipeline, login UI, keyboard input |
| 3. World Rendering | 🔲 Next | Tiles, terrain, objects, models |
| 4. Entities | 🔲 | NPCs, players, animations |
| 5. UI System | 🔲 | Inventory, chat, minimap |
| 6. Audio + PWA | 🔲 | Sound effects, music, service worker |

## License

MIT
