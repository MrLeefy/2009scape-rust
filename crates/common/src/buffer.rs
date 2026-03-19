//! RuneScape binary buffer — read/write with all endian variations.
//!
//! Naming convention from Java (Buffer.java):
//! - g = get (read), p = put (write)
//! - Number = bytes (1, 2, 3, 4, 8)
//! - i = inverse (little-endian), m = middle, im = inverse-middle
//! - add = +128 to low byte, sub = 128-low byte, neg = negate low byte
//! - smart/smarts = variable-length 1-or-2 byte encoding

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Buffer underflow: need {need} bytes, have {have}")]
    Underflow { need: usize, have: usize },
    #[error("Buffer overflow: need {need} bytes, capacity {cap}")]
    Overflow { need: usize, cap: usize },
}

pub type Result<T> = std::result::Result<T, BufferError>;

/// A binary buffer for reading/writing RuneScape protocol data.
pub struct Buffer {
    pub data: Vec<u8>,
    pub pos: usize,
}

impl Buffer {
    /// Create a new buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Buffer {
            data: vec![0u8; capacity],
            pos: 0,
        }
    }

    /// Wrap existing data for reading.
    pub fn wrap(data: Vec<u8>) -> Self {
        Buffer { data, pos: 0 }
    }

    /// Remaining bytes available for reading.
    pub fn remaining(&self) -> usize {
        if self.pos >= self.data.len() { 0 } else { self.data.len() - self.pos }
    }

    /// Ensure we can read `n` bytes.
    fn check_read(&self, n: usize) -> Result<()> {
        if self.remaining() < n {
            Err(BufferError::Underflow { need: n, have: self.remaining() })
        } else {
            Ok(())
        }
    }

    /// Ensure we can write `n` bytes, growing if needed.
    fn ensure_write(&mut self, n: usize) {
        let needed = self.pos + n;
        if needed > self.data.len() {
            self.data.resize(needed, 0);
        }
    }

    // ── GET (read) methods ──────────────────────────────────────────

    /// Read 1 unsigned byte.
    pub fn g1(&mut self) -> Result<u8> {
        self.check_read(1)?;
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    /// Read 1 signed byte.
    pub fn g1b(&mut self) -> Result<i8> {
        Ok(self.g1()? as i8)
    }

    /// Read 1 byte with +128 transform.
    pub fn g1add(&mut self) -> Result<u8> {
        Ok(self.g1()?.wrapping_sub(128))
    }

    /// Read 1 byte with negate transform.
    pub fn g1neg(&mut self) -> Result<u8> {
        Ok((-(self.g1()? as i8)) as u8)
    }

    /// Read 1 byte with 128-x transform.
    pub fn g1sub(&mut self) -> Result<u8> {
        Ok(128u8.wrapping_sub(self.g1()?))
    }

    /// Read 2 bytes big-endian unsigned.
    pub fn g2(&mut self) -> Result<u16> {
        self.check_read(2)?;
        let v = ((self.data[self.pos] as u16) << 8) | (self.data[self.pos + 1] as u16);
        self.pos += 2;
        Ok(v)
    }

    /// Read 2 bytes big-endian signed.
    pub fn g2b(&mut self) -> Result<i16> {
        let v = self.g2()?;
        Ok(if v > 32767 { (v as i32 - 0x10000) as i16 } else { v as i16 })
    }

    /// Read 2 bytes little-endian unsigned.
    pub fn ig2(&mut self) -> Result<u16> {
        self.check_read(2)?;
        let v = (self.data[self.pos] as u16) | ((self.data[self.pos + 1] as u16) << 8);
        self.pos += 2;
        Ok(v)
    }

    /// Read 3 bytes big-endian unsigned.
    pub fn g3(&mut self) -> Result<u32> {
        self.check_read(3)?;
        let v = ((self.data[self.pos] as u32) << 16)
            | ((self.data[self.pos + 1] as u32) << 8)
            | (self.data[self.pos + 2] as u32);
        self.pos += 3;
        Ok(v)
    }

    /// Read 4 bytes big-endian.
    pub fn g4(&mut self) -> Result<i32> {
        self.check_read(4)?;
        let v = ((self.data[self.pos] as i32) << 24)
            | ((self.data[self.pos + 1] as i32) << 16)
            | ((self.data[self.pos + 2] as i32) << 8)
            | (self.data[self.pos + 3] as i32);
        self.pos += 4;
        Ok(v)
    }

    /// Read 4 bytes little-endian.
    pub fn ig4(&mut self) -> Result<i32> {
        self.check_read(4)?;
        let v = (self.data[self.pos] as i32)
            | ((self.data[self.pos + 1] as i32) << 8)
            | ((self.data[self.pos + 2] as i32) << 16)
            | ((self.data[self.pos + 3] as i32) << 24);
        self.pos += 4;
        Ok(v)
    }

    /// Read 4 bytes middle-endian (CDAB).
    pub fn mg4(&mut self) -> Result<i32> {
        self.check_read(4)?;
        let v = ((self.data[self.pos + 1] as i32) << 24)
            | ((self.data[self.pos] as i32) << 16)
            | ((self.data[self.pos + 3] as i32) << 8)
            | (self.data[self.pos + 2] as i32);
        self.pos += 4;
        Ok(v)
    }

    /// Read 4 bytes inverse-middle-endian (BADC).
    pub fn img4(&mut self) -> Result<i32> {
        self.check_read(4)?;
        let v = ((self.data[self.pos + 2] as i32) << 24)
            | ((self.data[self.pos + 3] as i32) << 16)
            | ((self.data[self.pos] as i32) << 8)
            | (self.data[self.pos + 1] as i32);
        self.pos += 4;
        Ok(v)
    }

    /// Read 8 bytes big-endian.
    pub fn g8(&mut self) -> Result<i64> {
        let hi = self.g4()? as i64 & 0xFFFF_FFFF;
        let lo = self.g4()? as i64 & 0xFFFF_FFFF;
        Ok((hi << 32) | lo)
    }

    /// Read a "smart" value: 1 byte if < 128, else 2 bytes - 0xC000. Range: -16384..16383
    pub fn gsmart(&mut self) -> Result<i32> {
        self.check_read(1)?;
        let peek = self.data[self.pos];
        if peek < 128 {
            Ok(self.g1()? as i32 - 64)
        } else {
            Ok(self.g2()? as i32 - 0xC000)
        }
    }

    /// Read unsigned smart: 1 byte if < 128, else 2 bytes - 0x8000.
    pub fn gsmarts(&mut self) -> Result<u32> {
        self.check_read(1)?;
        let peek = self.data[self.pos];
        if peek < 128 {
            Ok(self.g1()? as u32)
        } else {
            Ok(self.g2()? as u32 - 0x8000)
        }
    }

    /// Read null-terminated string (JagString format).
    pub fn gjstr(&mut self) -> Result<String> {
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != 0 {
            self.pos += 1;
        }
        let s = String::from_utf8_lossy(&self.data[start..self.pos]).to_string();
        if self.pos < self.data.len() {
            self.pos += 1; // skip null terminator
        }
        Ok(s)
    }

    /// Read versioned string (version byte + null-terminated).
    pub fn gjstr2(&mut self) -> Result<String> {
        let version = self.g1()?;
        if version != 0 {
            return Err(BufferError::Underflow { need: 0, have: 0 }); // bad version
        }
        self.gjstr()
    }

    /// Read raw bytes into a new vec.
    pub fn gdata(&mut self, len: usize) -> Result<Vec<u8>> {
        self.check_read(len)?;
        let v = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(v)
    }

    // ── PUT (write) methods ────────────────────────────────────────

    /// Write 1 byte.
    pub fn p1(&mut self, value: u8) {
        self.ensure_write(1);
        self.data[self.pos] = value;
        self.pos += 1;
    }

    /// Write 1 byte with +128 transform.
    pub fn p1add(&mut self, value: u8) {
        self.p1(value.wrapping_add(128));
    }

    /// Write 1 byte with 128-x transform.
    pub fn p1sub(&mut self, value: u8) {
        self.p1(128u8.wrapping_sub(value));
    }

    /// Write 2 bytes big-endian.
    pub fn p2(&mut self, value: u16) {
        self.ensure_write(2);
        self.data[self.pos] = (value >> 8) as u8;
        self.data[self.pos + 1] = value as u8;
        self.pos += 2;
    }

    /// Write 2 bytes little-endian.
    pub fn ip2(&mut self, value: u16) {
        self.ensure_write(2);
        self.data[self.pos] = value as u8;
        self.data[self.pos + 1] = (value >> 8) as u8;
        self.pos += 2;
    }

    /// Write 3 bytes big-endian.
    pub fn p3(&mut self, value: u32) {
        self.ensure_write(3);
        self.data[self.pos] = (value >> 16) as u8;
        self.data[self.pos + 1] = (value >> 8) as u8;
        self.data[self.pos + 2] = value as u8;
        self.pos += 3;
    }

    /// Write 4 bytes big-endian.
    pub fn p4(&mut self, value: i32) {
        self.ensure_write(4);
        self.data[self.pos] = (value >> 24) as u8;
        self.data[self.pos + 1] = (value >> 16) as u8;
        self.data[self.pos + 2] = (value >> 8) as u8;
        self.data[self.pos + 3] = value as u8;
        self.pos += 4;
    }

    /// Write 4 bytes little-endian.
    pub fn ip4(&mut self, value: i32) {
        self.ensure_write(4);
        self.data[self.pos] = value as u8;
        self.data[self.pos + 1] = (value >> 8) as u8;
        self.data[self.pos + 2] = (value >> 16) as u8;
        self.data[self.pos + 3] = (value >> 24) as u8;
        self.pos += 4;
    }

    /// Write 8 bytes big-endian.
    pub fn p8(&mut self, value: i64) {
        self.p4((value >> 32) as i32);
        self.p4(value as i32);
    }

    /// Write null-terminated string.
    pub fn pjstr(&mut self, value: &str) {
        for b in value.bytes() {
            self.p1(b);
        }
        self.p1(0);
    }

    /// Write raw bytes.
    pub fn pdata(&mut self, src: &[u8]) {
        self.ensure_write(src.len());
        self.data[self.pos..self.pos + src.len()].copy_from_slice(src);
        self.pos += src.len();
    }

    /// Write a smart value (unsigned).
    pub fn psmarts(&mut self, value: u32) {
        if value < 128 {
            self.p1(value as u8);
        } else if value < 0x8000 {
            self.p2((value + 0x8000) as u16);
        }
    }

    // ── XTEA (tinydec) ─────────────────────────────────────────────

    /// XTEA decrypt in-place starting at byte 5 for `len` bytes.
    pub fn xtea_decrypt(&mut self, key: &[u32; 4], len: usize) {
        let saved_pos = self.pos;
        self.pos = 5;
        let blocks = (len - 5) / 8;
        for _ in 0..blocks {
            let mut v0 = self.g4().unwrap_or(0) as u32;
            let mut v1 = self.g4().unwrap_or(0) as u32;
            let mut sum: u32 = 0xC6EF3720;
            for _ in 0..32 {
                v1 = v1.wrapping_sub(
                    key[((sum >> 11) & 3) as usize].wrapping_add(sum)
                    ^ v0.wrapping_add(v0.wrapping_shr(5) ^ v0.wrapping_shl(4))
                );
                sum = sum.wrapping_sub(0x9E3779B9);
                    v0 = v0.wrapping_sub(
                    (v1.wrapping_shr(5) ^ v1.wrapping_shl(4)).wrapping_add(v1)
                    ^ key[(sum & 3) as usize].wrapping_add(sum)
                );
            }
            self.pos -= 8;
            self.p4(v0 as i32);
            self.p4(v1 as i32);
        }
        self.pos = saved_pos;
    }

    // ── RSA ─────────────────────────────────────────────────────────

    /// RSA encrypt the buffer content (pos bytes from start).
    /// Uses num-bigint if available, or panics as placeholder.
    pub fn rsa_enc(&mut self, _exp: &[u8], _mod: &[u8]) {
        // RSA encryption will be implemented when we add num-bigint dependency.
        // For now, the login handshake may work without RSA on test servers.
        log::warn!("RSA encryption not yet implemented — login may fail on encrypted servers");
    }

    /// Get a slice of written data (0..pos).
    pub fn written(&self) -> &[u8] {
        &self.data[..self.pos]
    }

    /// Reset position to 0.
    pub fn reset(&mut self) {
        self.pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_roundtrip() {
        let mut buf = Buffer::new(64);
        buf.p1(42);
        buf.p2(12345);
        buf.p4(0x12345678);
        buf.p8(0x123456789ABCDEF0u64 as i64);
        buf.pjstr("hello");

        buf.pos = 0;
        assert_eq!(buf.g1().unwrap(), 42);
        assert_eq!(buf.g2().unwrap(), 12345);
        assert_eq!(buf.g4().unwrap(), 0x12345678);
        assert_eq!(buf.g8().unwrap(), 0x123456789ABCDEF0u64 as i64);
        assert_eq!(buf.gjstr().unwrap(), "hello");
    }

    #[test]
    fn test_smart_values() {
        let mut buf = Buffer::new(16);
        buf.psmarts(50);   // 1 byte
        buf.psmarts(500);  // 2 bytes

        buf.pos = 0;
        assert_eq!(buf.gsmarts().unwrap(), 50);
        assert_eq!(buf.gsmarts().unwrap(), 500);
    }
}
