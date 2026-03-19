//! ISAAC random number generator for RuneScape packet cipher.
//! 
//! Direct translation of rt4/IsaacRandom.java using wrapping arithmetic.

const GOLDEN_RATIO: u32 = 0x9e3779b9;

pub struct IsaacRandom {
    count: usize,
    rsl: [u32; 256],
    mem: [u32; 256],
    a: u32,
    b: u32,
    c: u32,
}

impl IsaacRandom {
    pub fn new(seed: &[u32]) -> Self {
        let mut rng = IsaacRandom {
            count: 0,
            rsl: [0u32; 256],
            mem: [0u32; 256],
            a: 0,
            b: 0,
            c: 0,
        };
        for (i, &s) in seed.iter().enumerate() {
            if i < 256 {
                rng.rsl[i] = s;
            }
        }
        rng.init();
        rng
    }

    fn init(&mut self) {
        let mut a = GOLDEN_RATIO;
        let mut b = GOLDEN_RATIO;
        let mut c = GOLDEN_RATIO;
        let mut d = GOLDEN_RATIO;
        let mut e = GOLDEN_RATIO;
        let mut f = GOLDEN_RATIO;
        let mut g = GOLDEN_RATIO;
        let mut h = GOLDEN_RATIO;

        // Scramble
        for _ in 0..4 {
            a ^= b.wrapping_shl(11); d = d.wrapping_add(a); b = b.wrapping_add(c);
            b ^= c.wrapping_shr(2);  e = e.wrapping_add(b); c = c.wrapping_add(d);
            c ^= d.wrapping_shl(8);  f = f.wrapping_add(c); d = d.wrapping_add(e);
            d ^= e.wrapping_shr(16); g = g.wrapping_add(d); e = e.wrapping_add(f);
            e ^= f.wrapping_shl(10); h = h.wrapping_add(e); f = f.wrapping_add(g);
            f ^= g.wrapping_shr(4);  a = a.wrapping_add(f); g = g.wrapping_add(h);
            g ^= h.wrapping_shl(8);  b = b.wrapping_add(g); h = h.wrapping_add(a);
            h ^= a.wrapping_shr(9);  c = c.wrapping_add(h); a = a.wrapping_add(b);
        }

        // First pass: seed with rsl
        for i in (0..256).step_by(8) {
            a = a.wrapping_add(self.rsl[i]);     b = b.wrapping_add(self.rsl[i + 1]);
            c = c.wrapping_add(self.rsl[i + 2]); d = d.wrapping_add(self.rsl[i + 3]);
            e = e.wrapping_add(self.rsl[i + 4]); f = f.wrapping_add(self.rsl[i + 5]);
            g = g.wrapping_add(self.rsl[i + 6]); h = h.wrapping_add(self.rsl[i + 7]);

            a ^= b.wrapping_shl(11); d = d.wrapping_add(a); b = b.wrapping_add(c);
            b ^= c.wrapping_shr(2);  e = e.wrapping_add(b); c = c.wrapping_add(d);
            c ^= d.wrapping_shl(8);  f = f.wrapping_add(c); d = d.wrapping_add(e);
            d ^= e.wrapping_shr(16); g = g.wrapping_add(d); e = e.wrapping_add(f);
            e ^= f.wrapping_shl(10); h = h.wrapping_add(e); f = f.wrapping_add(g);
            f ^= g.wrapping_shr(4);  a = a.wrapping_add(f); g = g.wrapping_add(h);
            g ^= h.wrapping_shl(8);  b = b.wrapping_add(g); h = h.wrapping_add(a);
            h ^= a.wrapping_shr(9);  c = c.wrapping_add(h); a = a.wrapping_add(b);

            self.mem[i] = a;     self.mem[i + 1] = b;
            self.mem[i + 2] = c; self.mem[i + 3] = d;
            self.mem[i + 4] = e; self.mem[i + 5] = f;
            self.mem[i + 6] = g; self.mem[i + 7] = h;
        }

        // Second pass: seed with mem
        for i in (0..256).step_by(8) {
            a = a.wrapping_add(self.mem[i]);     b = b.wrapping_add(self.mem[i + 1]);
            c = c.wrapping_add(self.mem[i + 2]); d = d.wrapping_add(self.mem[i + 3]);
            e = e.wrapping_add(self.mem[i + 4]); f = f.wrapping_add(self.mem[i + 5]);
            g = g.wrapping_add(self.mem[i + 6]); h = h.wrapping_add(self.mem[i + 7]);

            a ^= b.wrapping_shl(11); d = d.wrapping_add(a); b = b.wrapping_add(c);
            b ^= c.wrapping_shr(2);  e = e.wrapping_add(b); c = c.wrapping_add(d);
            c ^= d.wrapping_shl(8);  f = f.wrapping_add(c); d = d.wrapping_add(e);
            d ^= e.wrapping_shr(16); g = g.wrapping_add(d); e = e.wrapping_add(f);
            e ^= f.wrapping_shl(10); h = h.wrapping_add(e); f = f.wrapping_add(g);
            f ^= g.wrapping_shr(4);  a = a.wrapping_add(f); g = g.wrapping_add(h);
            g ^= h.wrapping_shl(8);  b = b.wrapping_add(g); h = h.wrapping_add(a);
            h ^= a.wrapping_shr(9);  c = c.wrapping_add(h); a = a.wrapping_add(b);

            self.mem[i] = a;     self.mem[i + 1] = b;
            self.mem[i + 2] = c; self.mem[i + 3] = d;
            self.mem[i + 4] = e; self.mem[i + 5] = f;
            self.mem[i + 6] = g; self.mem[i + 7] = h;
        }

        self.isaac();
        self.count = 256;
    }

    /// Get the next pseudo-random key.
    pub fn next_key(&mut self) -> u32 {
        if self.count == 0 {
            self.isaac();
            self.count = 256;
        }
        self.count -= 1;
        self.rsl[self.count]
    }

    fn isaac(&mut self) {
        self.c = self.c.wrapping_add(1);
        self.b = self.b.wrapping_add(self.c);

        for i in 0..256 {
            let x = self.mem[i];
            match i & 3 {
                0 => self.a ^= self.a.wrapping_shl(13),
                1 => self.a ^= self.a.wrapping_shr(6),
                2 => self.a ^= self.a.wrapping_shl(2),
                3 => self.a ^= self.a.wrapping_shr(16),
                _ => unreachable!(),
            }
            self.a = self.a.wrapping_add(self.mem[(i + 128) & 0xFF]);
            let y = self.mem[(x as usize >> 2) & 0xFF]
                .wrapping_add(self.a)
                .wrapping_add(self.b);
            self.mem[i] = y;
            self.b = self.mem[(y as usize >> 10) & 0xFF].wrapping_add(x);
            self.rsl[i] = self.b;
        }
    }
}
