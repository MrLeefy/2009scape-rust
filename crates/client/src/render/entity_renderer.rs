//! Entity 3D rendering — draws entities as colored capsules/cubes in the world.

use super::renderer3d::Vertex3D;
use crate::entity::{Entity, EntityKind};

/// Generate vertices for an entity (simplified capsule/box shape).
pub fn generate_entity_mesh(entity: &Entity) -> (Vec<Vertex3D>, Vec<u32>) {
    let x = entity.render_x();
    let z = entity.render_z();
    let y = entity.y;
    let size = entity.size as f32 * 40.0;

    let color = match entity.kind {
        EntityKind::Player => [0.2, 0.4, 0.9, 1.0],  // blue
        EntityKind::Npc => match entity.combat_level {
            0..=5 => [0.3, 0.8, 0.3, 1.0],   // green (low level)
            6..=20 => [0.9, 0.8, 0.2, 1.0],   // yellow (mid level)
            _ => [0.9, 0.2, 0.2, 1.0],         // red (high level)
        },
        EntityKind::Projectile => [1.0, 1.0, 1.0, 1.0],
    };

    let head_color = match entity.kind {
        EntityKind::Player => [0.7, 0.6, 0.5, 1.0],  // skin tone
        _ => color,
    };

    let half = size / 2.0;
    let height = size * 2.5;
    let head_y = y + height;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Body (box)
    let body_verts = [
        // Front face
        ([x - half, y, z + half], [0.0, 0.0, 1.0], color),
        ([x + half, y, z + half], [0.0, 0.0, 1.0], color),
        ([x + half, y + height * 0.7, z + half], [0.0, 0.0, 1.0], color),
        ([x - half, y + height * 0.7, z + half], [0.0, 0.0, 1.0], color),
        // Back
        ([x + half, y, z - half], [0.0, 0.0, -1.0], color),
        ([x - half, y, z - half], [0.0, 0.0, -1.0], color),
        ([x - half, y + height * 0.7, z - half], [0.0, 0.0, -1.0], color),
        ([x + half, y + height * 0.7, z - half], [0.0, 0.0, -1.0], color),
        // Left
        ([x - half, y, z - half], [-1.0, 0.0, 0.0], color),
        ([x - half, y, z + half], [-1.0, 0.0, 0.0], color),
        ([x - half, y + height * 0.7, z + half], [-1.0, 0.0, 0.0], color),
        ([x - half, y + height * 0.7, z - half], [-1.0, 0.0, 0.0], color),
        // Right
        ([x + half, y, z + half], [1.0, 0.0, 0.0], color),
        ([x + half, y, z - half], [1.0, 0.0, 0.0], color),
        ([x + half, y + height * 0.7, z - half], [1.0, 0.0, 0.0], color),
        ([x + half, y + height * 0.7, z + half], [1.0, 0.0, 0.0], color),
        // Top
        ([x - half, y + height * 0.7, z + half], [0.0, 1.0, 0.0], color),
        ([x + half, y + height * 0.7, z + half], [0.0, 1.0, 0.0], color),
        ([x + half, y + height * 0.7, z - half], [0.0, 1.0, 0.0], color),
        ([x - half, y + height * 0.7, z - half], [0.0, 1.0, 0.0], color),
    ];

    let base = vertices.len() as u32;
    for (pos, norm, col) in &body_verts {
        vertices.push(Vertex3D { position: *pos, normal: *norm, color: *col });
    }
    for face in 0..5u32 {
        let b = base + face * 4;
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }

    // Head (smaller box on top)
    let hs = half * 0.6;
    let head_base = y + height * 0.7;
    let head_verts = [
        ([x - hs, head_base, z + hs], [0.0, 0.0, 1.0], head_color),
        ([x + hs, head_base, z + hs], [0.0, 0.0, 1.0], head_color),
        ([x + hs, head_y, z + hs], [0.0, 0.0, 1.0], head_color),
        ([x - hs, head_y, z + hs], [0.0, 0.0, 1.0], head_color),
        ([x + hs, head_base, z - hs], [0.0, 0.0, -1.0], head_color),
        ([x - hs, head_base, z - hs], [0.0, 0.0, -1.0], head_color),
        ([x - hs, head_y, z - hs], [0.0, 0.0, -1.0], head_color),
        ([x + hs, head_y, z - hs], [0.0, 0.0, -1.0], head_color),
        ([x - hs, head_base, z - hs], [-1.0, 0.0, 0.0], head_color),
        ([x - hs, head_base, z + hs], [-1.0, 0.0, 0.0], head_color),
        ([x - hs, head_y, z + hs], [-1.0, 0.0, 0.0], head_color),
        ([x - hs, head_y, z - hs], [-1.0, 0.0, 0.0], head_color),
        ([x + hs, head_base, z + hs], [1.0, 0.0, 0.0], head_color),
        ([x + hs, head_base, z - hs], [1.0, 0.0, 0.0], head_color),
        ([x + hs, head_y, z - hs], [1.0, 0.0, 0.0], head_color),
        ([x + hs, head_y, z + hs], [1.0, 0.0, 0.0], head_color),
        ([x - hs, head_y, z + hs], [0.0, 1.0, 0.0], head_color),
        ([x + hs, head_y, z + hs], [0.0, 1.0, 0.0], head_color),
        ([x + hs, head_y, z - hs], [0.0, 1.0, 0.0], head_color),
        ([x - hs, head_y, z - hs], [0.0, 1.0, 0.0], head_color),
    ];

    let base2 = vertices.len() as u32;
    for (pos, norm, col) in &head_verts {
        vertices.push(Vertex3D { position: *pos, normal: *norm, color: *col });
    }
    for face in 0..5u32 {
        let b = base2 + face * 4;
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }

    (vertices, indices)
}

/// Generate entity health bar vertices (2D billboard above entity).
pub fn health_bar_quads(entity: &Entity) -> Vec<Vertex3D> {
    if entity.health >= entity.max_health { return Vec::new(); }
    
    let x = entity.render_x();
    let z = entity.render_z();
    let y = entity.y + entity.size as f32 * 110.0;
    let bar_w = 40.0;
    let bar_h = 5.0;
    let health_pct = entity.health as f32 / entity.max_health as f32;

    let mut verts = Vec::new();

    // Background (red)
    let red = [0.8, 0.1, 0.1, 1.0];
    verts.push(Vertex3D { position: [x - bar_w, y, z], normal: [0.0, 1.0, 0.0], color: red });
    verts.push(Vertex3D { position: [x + bar_w, y, z], normal: [0.0, 1.0, 0.0], color: red });
    verts.push(Vertex3D { position: [x + bar_w, y + bar_h, z], normal: [0.0, 1.0, 0.0], color: red });
    verts.push(Vertex3D { position: [x - bar_w, y + bar_h, z], normal: [0.0, 1.0, 0.0], color: red });

    // Green fill
    let fill_w = bar_w * 2.0 * health_pct;
    let green = [0.1, 0.8, 0.1, 1.0];
    verts.push(Vertex3D { position: [x - bar_w, y, z + 0.1], normal: [0.0, 1.0, 0.0], color: green });
    verts.push(Vertex3D { position: [x - bar_w + fill_w, y, z + 0.1], normal: [0.0, 1.0, 0.0], color: green });
    verts.push(Vertex3D { position: [x - bar_w + fill_w, y + bar_h, z + 0.1], normal: [0.0, 1.0, 0.0], color: green });
    verts.push(Vertex3D { position: [x - bar_w, y + bar_h, z + 0.1], normal: [0.0, 1.0, 0.0], color: green });

    verts
}
