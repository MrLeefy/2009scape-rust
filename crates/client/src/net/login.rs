//! Real RS2 login handshake — translated from rt4/LoginManager.java
//!
//! Steps:
//!  1. Connect to game server
//!  2. Send [14, name_hash] → read 1-byte response (must be 0)
//!  3. Read 8-byte server key → build RSA block → send login packet
//!  4. Read reply (2 = success) → read 14 bytes session data
//!  5. Set up ISAAC cipher for both directions

use rs2_common::buffer::Buffer;
use rs2_common::isaac::IsaacRandom;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Result, anyhow, bail};
use log::{info, warn};

/// Revision number the server expects.
const CLIENT_REVISION: i32 = 530;

/// Login reply codes from the Java source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoginReply {
    Success = 2,
    InvalidCredentials = 3,
    Banned = 4,
    AlreadyLoggedIn = 5,
    Updated = 6,
    WorldFull = 7,
    LoginServerOffline = 8,
    TooManyConnections = 9,
    BadSessionId = 10,
    LoginServerRejected = 11,
    MembersWorld = 12,
    CouldNotComplete = 13,
    ServerUpdating = 14,
    Reconnecting = 15,
    TooManyAttempts = 16,
    MembersArea = 17,
    AccountLocked = 18,
    WaitThenRetry = 21,
    DisallowedByScript = 29,
    Unknown = 255,
}

impl From<u8> for LoginReply {
    fn from(v: u8) -> Self {
        match v {
            2 => LoginReply::Success,
            3 => LoginReply::InvalidCredentials,
            4 => LoginReply::Banned,
            5 => LoginReply::AlreadyLoggedIn,
            6 => LoginReply::Updated,
            7 => LoginReply::WorldFull,
            8 => LoginReply::LoginServerOffline,
            9 => LoginReply::TooManyConnections,
            10 => LoginReply::BadSessionId,
            11 => LoginReply::LoginServerRejected,
            12 => LoginReply::MembersWorld,
            13 => LoginReply::CouldNotComplete,
            14 => LoginReply::ServerUpdating,
            15 => LoginReply::Reconnecting,
            16 => LoginReply::TooManyAttempts,
            17 => LoginReply::MembersArea,
            18 => LoginReply::AccountLocked,
            21 => LoginReply::WaitThenRetry,
            29 => LoginReply::DisallowedByScript,
            _ => LoginReply::Unknown,
        }
    }
}

/// Session data returned on successful login.
#[derive(Debug)]
pub struct LoginSession {
    pub stream: TcpStream,
    pub in_cipher: IsaacRandom,
    pub out_cipher: IsaacRandom,
    pub staff_mod_level: u8,
    pub player_member: bool,
    pub player_id: u16,
    pub map_members: bool,
}

/// Encode a username to Base37 (same as JagString.encode37 in Java).
pub fn encode_base37(name: &str) -> i64 {
    let name = name.to_lowercase();
    let mut hash: i64 = 0;
    for c in name.chars().take(12) {
        hash *= 37;
        if c >= 'a' && c <= 'z' {
            hash += (c as i64) - ('a' as i64) + 1;
        } else if c >= '0' && c <= '9' {
            hash += (c as i64) - ('0' as i64) + 27;
        }
    }
    hash
}

/// Perform the full RS2 login handshake.
///
/// Translated from LoginManager.loop() steps 1-9.
pub async fn login(host: &str, port: u16, username: &str, password: &str) -> Result<LoginSession> {
    info!("Connecting to {}:{}...", host, port);

    // ─── Step 1: Connect ───
    let mut stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
    info!("TCP connected");

    // ─── Step 2: Send [14, name_hash] ───
    // From LoginManager.java step 2:
    //   outboundBuffer.p1(14);
    //   outboundBuffer.p1((int)(name37 >> 16 & 0x1F));
    let name37 = encode_base37(username);
    let name_hash = ((name37 >> 16) & 0x1F) as u8;

    let mut init_buf = Buffer::new(2);
    init_buf.p1(14);
    init_buf.p1(name_hash);
    stream.write_all(init_buf.written()).await?;
    info!("Sent login init (opcode=14, namehash={})", name_hash);

    // Read 1-byte response (must be 0 for success)
    let response = stream.read_u8().await?;
    if response != 0 {
        bail!("Login init failed: server returned {} (expected 0)", response);
    }
    info!("Server accepted init (response=0)");

    // ─── Step 3: Read 8-byte server key ───
    let mut key_bytes = [0u8; 8];
    stream.read_exact(&mut key_bytes).await?;
    let mut key_buf = Buffer::wrap(key_bytes.to_vec());
    let server_key = key_buf.g8()?;
    info!("Received server key: {}", server_key);

    // Generate ISAAC seed (4 ints)
    let client_key0 = (rand_u32() as u32) % 100_000_000;
    let client_key1 = (rand_u32() as u32) % 100_000_000;
    let server_key_hi = (server_key >> 32) as u32;
    let server_key_lo = server_key as u32;
    let isaac_seed = [client_key0, client_key1, server_key_hi as u32, server_key_lo as u32];

    // Build the RSA block (inner login payload)
    // From LoginManager.java step 3:
    //   outboundBuffer.p1(10);  // RSA block opcode
    //   outboundBuffer.p4(key[0]); outboundBuffer.p4(key[1]);
    //   outboundBuffer.p4(key[2]); outboundBuffer.p4(key[3]);
    //   outboundBuffer.p8(Player.usernameInput.encode37());
    //   outboundBuffer.pjstr(Player.password);
    //   outboundBuffer.rsaenc(...)  <-- RSA encryption
    let mut rsa_buf = Buffer::new(256);
    rsa_buf.p1(10); // RSA opcode
    rsa_buf.p4(client_key0 as i32);
    rsa_buf.p4(client_key1 as i32);
    rsa_buf.p4(server_key_hi as i32);
    rsa_buf.p4(server_key_lo as i32);
    rsa_buf.p8(name37);
    rsa_buf.pjstr(password);
    // NOTE: RSA encryption skipped — 2009Scape test server doesn't enforce RSA
    warn!("RSA encryption skipped (test server mode)");

    let rsa_data_len = rsa_buf.pos;

    // Build the outer login packet
    // From LoginManager.java step 3:
    //   buffer.p1(16);  // new login opcode (18 = reconnect)
    //   buffer.p2(rsa_len + header + 159); // total size
    //   buffer.p4(530);  // revision
    //   ... window/display info ...
    //   ... 28 archive checksums ...
    //   ... RSA block data ...
    let settings_str = "";
    let settings_len = settings_str.len() + 1; // +1 for null terminator
    let total_size = rsa_data_len + settings_len + 159;

    let mut login_buf = Buffer::new(total_size + 3);
    login_buf.p1(16); // login opcode (16 = new login, 18 = reconnect)
    login_buf.p2(total_size as u16);
    login_buf.p4(CLIENT_REVISION); // revision 530
    login_buf.p1(0); // anInt39 (-1 → use 0)
    login_buf.p1(0); // advertSuppressed
    login_buf.p1(1); // constant 1
    login_buf.p1(0); // window mode (0 = SD fixed)
    login_buf.p2(765); // canvas width
    login_buf.p2(503); // canvas height
    login_buf.p1(0); // anti-aliasing mode

    // writeUid — 24 bytes of UID (send zeros)
    for _ in 0..24 {
        login_buf.p1(0);
    }

    login_buf.pjstr(settings_str); // settings string
    login_buf.p4(0); // affiliate
    login_buf.p4(0); // preferences int
    login_buf.p2(0); // verify id

    // 28 archive checksums (send 0 for all — server will send cache data)
    for _ in 0..28 {
        login_buf.p4(0);
    }

    // Append the RSA block data
    login_buf.pdata(&rsa_buf.data[..rsa_data_len]);

    stream.write_all(login_buf.written()).await?;
    info!("Sent login packet ({} bytes, revision {})", login_buf.pos, CLIENT_REVISION);

    // Set up ISAAC ciphers
    let out_cipher = IsaacRandom::new(&isaac_seed);
    let mut in_seed = isaac_seed;
    for i in 0..4 {
        in_seed[i] = in_seed[i].wrapping_add(50);
    }
    let in_cipher = IsaacRandom::new(&in_seed);

    // ─── Step 4: Read login reply ───
    let reply_byte = stream.read_u8().await?;
    let reply = LoginReply::from(reply_byte);
    info!("Login reply: {:?} ({})", reply, reply_byte);

    match reply {
        LoginReply::Success => {
            // Step 8: Read 14 bytes of session data
            // From LoginManager.java:
            //   staffModLevel = g1(); blackmarks = g1();
            //   playerUnderage = g1() == 1; parentalChatConsent = g1() == 1;
            //   parentalAdvertConsent = g1() == 1; mapQuickChat = g1() == 1;
            //   mouseRecorderEnabled = g1() == 1; selfId = g2();
            //   playerMember = g1() == 1; mapMembers = g1() == 1;
            let mut session_bytes = [0u8; 14];
            stream.read_exact(&mut session_bytes).await?;
            let mut sb = Buffer::wrap(session_bytes.to_vec());

            let staff_mod_level = sb.g1()?;
            let _blackmarks = sb.g1()?;
            let _player_underage = sb.g1()? == 1;
            let _parental_chat = sb.g1()? == 1;
            let _parental_advert = sb.g1()? == 1;
            let _map_quickchat = sb.g1()? == 1;
            let _mouse_recorder = sb.g1()? == 1;
            let player_id = sb.g2()?;
            let player_member = sb.g1()? == 1;
            let map_members = sb.g1()? == 1;

            info!("Login successful! player_id={}, staff={}, member={}", player_id, staff_mod_level, player_member);

            // Step 9: Read first opcode + 2-byte length (the initial map rebuild packet)
            // From LoginManager.java:
            //   opcode = g1isaac(); length = g2();
            // We'll read this from the stream but let the packet handler process it
            let first_opcode = stream.read_u8().await?;
            let len_hi = stream.read_u8().await?;
            let len_lo = stream.read_u8().await?;
            let first_packet_len = ((len_hi as u16) << 8) | (len_lo as u16);
            info!("First server packet: opcode={} (ISAAC-encoded), len={}", first_opcode, first_packet_len);

            // Read the full first packet data
            if first_packet_len > 0 && first_packet_len < 5000 {
                let mut packet_data = vec![0u8; first_packet_len as usize];
                stream.read_exact(&mut packet_data).await?;
                info!("Read first packet data ({} bytes)", packet_data.len());
            }

            Ok(LoginSession {
                stream,
                in_cipher,
                out_cipher,
                staff_mod_level,
                player_member,
                player_id,
                map_members,
            })
        }
        LoginReply::WaitThenRetry => {
            let delay = stream.read_u8().await?;
            bail!("Server said wait {} minutes before retrying", (delay + 3) * 60);
        }
        _ => {
            bail!("Login failed: {:?} (code {})", reply, reply_byte);
        }
    }
}

/// Simple deterministic random (not crypto-secure, just for ISAAC seed).
fn rand_u32() -> u32 {
    use std::time::SystemTime;
    let t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (t as u32) ^ 0xDEAD_BEEF
}
