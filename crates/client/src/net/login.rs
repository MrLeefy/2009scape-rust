//! Login handshake protocol for 2009Scape servers.
//!
//! Flow:
//! 1. Client sends opcode 14 (init login)
//! 2. Server responds with 8 bytes (status + server key)
//! 3. Client sends opcode 16/18 (new/reconnect login) + encrypted block
//! 4. Server responds with login result

use rs2_common::buffer::Buffer;
use rs2_common::isaac::IsaacRandom;
use super::transport::Transport;
use anyhow::{Result, Context};
use log::{info, warn};

const LOGIN_OPCODE_INIT: u8 = 14;
const LOGIN_OPCODE_NEW: u8 = 16;
const LOGIN_OPCODE_RECONNECT: u8 = 18;

pub struct LoginResult {
    pub response_code: u8,
    pub player_rights: u8,
    pub flagged: bool,
    pub in_cipher: IsaacRandom,
    pub out_cipher: IsaacRandom,
}

/// Perform the full login handshake.
pub async fn login(
    transport: &mut Transport,
    username: &str,
    password: &str,
    reconnect: bool,
) -> Result<LoginResult> {
    // Step 1: Send init login
    info!("Sending login init (opcode {})...", LOGIN_OPCODE_INIT);
    let mut init_buf = Buffer::new(2);
    init_buf.p1(LOGIN_OPCODE_INIT);
    init_buf.p1(0); // name hash (unused by most servers)
    transport.write(init_buf.written()).await?;

    // Step 2: Read server response
    let response = transport.read_byte().await?;
    info!("Server init response: {}", response);

    if response != 0 {
        anyhow::bail!("Server rejected login init with code: {}", response);
    }

    // Read server session key (8 bytes / i64)
    let mut key_buf = [0u8; 8];
    transport.read_exact(&mut key_buf).await?;
    let server_key = i64::from_be_bytes(key_buf);
    info!("Server session key received");

    // Step 3: Build login block
    let client_key: i64 = rand_i64();

    // ISAAC seed (4 ints from client + server keys)
    let seed = [
        (client_key >> 32) as u32,
        client_key as u32,
        (server_key >> 32) as u32,
        server_key as u32,
    ];

    // Build RSA block
    let mut rsa_block = Buffer::new(256);
    rsa_block.p1(10); // magic number
    rsa_block.p4(seed[0] as i32);
    rsa_block.p4(seed[1] as i32);
    rsa_block.p4(seed[2] as i32);
    rsa_block.p4(seed[3] as i32);
    rsa_block.p4(0); // UID (unused)
    rsa_block.pjstr(username);
    rsa_block.pjstr(password);

    // Build login block
    let opcode = if reconnect { LOGIN_OPCODE_RECONNECT } else { LOGIN_OPCODE_NEW };
    let rsa_len = rsa_block.pos;

    let mut login_buf = Buffer::new(512);
    login_buf.p1(opcode);
    login_buf.p2((rsa_len + 36) as u16); // block size

    login_buf.p4(530); // client revision (2009Scape ~530)
    login_buf.p1(0);   // low memory flag
    
    // Display mode
    login_buf.p1(0); // SD = 0, HD = 1, resizable = 2
    login_buf.p2(765); // screen width
    login_buf.p2(503); // screen height

    // XTEA key placeholder
    for &k in &seed {
        login_buf.p4(k as i32);
    }

    // RSA block (unencrypted on test servers)
    login_buf.p1(rsa_len as u8);
    login_buf.pdata(&rsa_block.data[..rsa_len]);

    transport.write(login_buf.written()).await?;
    info!("Login request sent (opcode {})", opcode);

    // Step 4: Read login response  
    let response_code = transport.read_byte().await?;
    info!("Login response code: {}", response_code);

    match response_code {
        2 => {
            // Success
            let rights = transport.read_byte().await?;
            let flagged = transport.read_byte().await?;
            info!("Login successful! Rights: {}, Flagged: {}", rights, flagged != 0);

            // Build ISAAC ciphers
            let in_cipher = IsaacRandom::new(&seed);
            let mut out_seed = seed;
            for s in &mut out_seed {
                *s = s.wrapping_add(50);
            }
            let out_cipher = IsaacRandom::new(&out_seed);

            Ok(LoginResult {
                response_code,
                player_rights: rights,
                flagged: flagged != 0,
                in_cipher,
                out_cipher,
            })
        }
        _ => {
            let msg = match response_code {
                3 => "Invalid username or password",
                4 => "Account disabled",
                5 => "Already logged in",
                6 => "Game updated",
                7 => "Server full",
                9 => "Too many connections",
                11 => "Bad session ID",
                _ => "Unknown error",
            };
            warn!("Login failed: {} (code {})", msg, response_code);
            anyhow::bail!("Login failed: {} (code {})", msg, response_code);
        }
    }
}

/// Simple pseudo-random i64 for client session key.
fn rand_i64() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (nanos as i64) ^ 0x5DEECE66D
}
