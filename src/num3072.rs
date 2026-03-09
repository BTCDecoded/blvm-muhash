//! Num3072: 3072-bit number modulo 2^3072 - 1103717.
//!
//! Port of Bitcoin Core src/crypto/muhash.cpp Num3072.

pub const BYTE_SIZE: usize = 384;
pub const LIMBS: usize = 48;
pub const LIMB_SIZE: usize = 64;
pub const SIGNED_LIMBS: usize = 50;
pub const SIGNED_LIMB_SIZE: usize = 62;

const MAX_PRIME_DIFF: u64 = 1103717;
const MODULUS_INVERSE: u64 = 0x70a1421da087d93;
const MAX_SIGNED_LIMB: i64 = (1i64 << SIGNED_LIMB_SIZE) - 1;
const FINAL_LIMB_POSITION: usize = 3072 / SIGNED_LIMB_SIZE;
const FINAL_LIMB_MODULUS_BITS: usize = 3072 % SIGNED_LIMB_SIZE;

#[derive(Clone)]
pub struct Num3072 {
    pub limbs: [u64; LIMBS],
}

fn extract3(c0: &mut u64, c1: &mut u64, c2: &mut u64, n: &mut u64) {
    *n = *c0;
    *c0 = *c1;
    *c1 = *c2;
    *c2 = 0;
}

fn mul(a: u64, b: u64) -> (u64, u64) {
    let t = (a as u128) * (b as u128);
    ((t & 0xffff_ffff_ffff_ffff) as u64, (t >> 64) as u64)
}

fn muladd3(c0: &mut u64, c1: &mut u64, c2: &mut u64, a: u64, b: u64) {
    let t = (a as u128) * (b as u128);
    let th = (t >> 64) as u64;
    let tl = (t & 0xffff_ffff_ffff_ffff) as u64;
    let (new_c0, carry0) = c0.overflowing_add(tl);
    *c0 = new_c0;
    let th = th.wrapping_add(carry0 as u64);
    let (new_c1, carry1) = c1.overflowing_add(th);
    *c1 = new_c1;
    *c2 += carry1 as u64;
}

fn mulnadd3(c0: &mut u64, c1: &mut u64, c2: &mut u64, d0: u64, d1: u64, d2: u64, n: u64) {
    let t = (d0 as u128) * (n as u128) + (*c0 as u128);
    let c0_val = (t & 0xffff_ffff_ffff_ffff) as u64;
    let mut t = t >> 64;
    t += (d1 as u128) * (n as u128) + (*c1 as u128);
    let c1_val = (t & 0xffff_ffff_ffff_ffff) as u64;
    let t = t >> 64;
    let c2_val = (t as u64).wrapping_add(d2.wrapping_mul(n));
    *c0 = c0_val;
    *c1 = c1_val;
    *c2 = c2_val;
}

fn muln2(c0: &mut u64, c1: &mut u64, n: u64) {
    let t = (*c0 as u128) * (n as u128);
    *c0 = (t & 0xffff_ffff_ffff_ffff) as u64;
    let t = (t >> 64) + (*c1 as u128) * (n as u128);
    *c1 = (t & 0xffff_ffff_ffff_ffff) as u64;
}

fn addnextract2(c0: &mut u64, c1: &mut u64, a: u64, n: &mut u64) {
    let (new_c0, carry) = c0.overflowing_add(a);
    *c0 = new_c0;
    let c1_add = if carry {
        let (nc1, c2) = c1.overflowing_add(1);
        *c1 = nc1;
        c2 as u64
    } else {
        0
    };
    *n = *c0;
    *c0 = *c1;
    *c1 = c1_add;
}

impl Num3072 {
    pub fn set_to_one(&mut self) {
        self.limbs[0] = 1;
        for i in 1..LIMBS {
            self.limbs[i] = 0;
        }
    }

    pub fn is_overflow(&self) -> bool {
        if self.limbs[0] <= u64::MAX - MAX_PRIME_DIFF {
            return false;
        }
        for i in 1..LIMBS {
            if self.limbs[i] != u64::MAX {
                return false;
            }
        }
        true
    }

    pub fn full_reduce(&mut self) {
        let mut c0 = MAX_PRIME_DIFF;
        let mut c1 = 0u64;
        for i in 0..LIMBS {
            addnextract2(&mut c0, &mut c1, self.limbs[i], &mut self.limbs[i]);
        }
    }

    pub fn multiply(&mut self, a: &Num3072) {
        let mut c0 = 0u64;
        let mut c1 = 0u64;
        let mut c2 = 0u64;
        let mut tmp = [0u64; LIMBS];

        #[allow(clippy::needless_range_loop)]
        for j in 0..LIMBS - 1 {
            let (mut d0, mut d1) = mul(self.limbs[1 + j], a.limbs[LIMBS + j - (1 + j)]);
            let mut d2 = 0u64;
            for i in (2 + j)..LIMBS {
                muladd3(
                    &mut d0,
                    &mut d1,
                    &mut d2,
                    self.limbs[i],
                    a.limbs[LIMBS + j - i],
                );
            }
            mulnadd3(&mut c0, &mut c1, &mut c2, d0, d1, d2, MAX_PRIME_DIFF);
            for i in 0..=j {
                muladd3(&mut c0, &mut c1, &mut c2, self.limbs[i], a.limbs[j - i]);
            }
            extract3(&mut c0, &mut c1, &mut c2, &mut tmp[j]);
        }

        for i in 0..LIMBS {
            muladd3(
                &mut c0,
                &mut c1,
                &mut c2,
                self.limbs[i],
                a.limbs[LIMBS - 1 - i],
            );
        }
        extract3(&mut c0, &mut c1, &mut c2, &mut tmp[LIMBS - 1]);

        muln2(&mut c0, &mut c1, MAX_PRIME_DIFF);
        #[allow(clippy::needless_range_loop)]
        for j in 0..LIMBS {
            addnextract2(&mut c0, &mut c1, tmp[j], &mut self.limbs[j]);
        }

        if self.is_overflow() {
            self.full_reduce();
        }
        if c0 != 0 {
            self.full_reduce();
        }
    }

    pub fn divide(&mut self, a: &Num3072) {
        if self.is_overflow() {
            self.full_reduce();
        }
        let inv = if a.is_overflow() {
            let mut b = a.clone();
            b.full_reduce();
            b.get_inverse()
        } else {
            a.get_inverse()
        };
        self.multiply(&inv);
        if self.is_overflow() {
            self.full_reduce();
        }
    }

    pub fn get_inverse(&self) -> Num3072 {
        let mut d = Num3072Signed::zero();
        let mut e = Num3072Signed::zero();
        let mut f = Num3072Signed::zero();
        let mut g = Num3072Signed::zero();

        e.limbs[0] = 1;
        f.limbs[0] = -(MAX_PRIME_DIFF as i64);
        f.limbs[FINAL_LIMB_POSITION] = 1i64 << FINAL_LIMB_MODULUS_BITS;
        g.load_from_num3072(self);

        let mut len = SIGNED_LIMBS;
        let mut eta: i32 = -1;

        loop {
            let t = compute_divstep_matrix(eta, f.limbs[0], g.limbs[0]);
            eta = t.0;
            update_fg(&mut f, &mut g, &t.1, len);
            update_de(&mut d, &mut e, &t.1);

            let mut cond = 0i64;
            for j in 1..len {
                cond |= g.limbs[j];
            }
            if g.limbs[0] == 0 && cond == 0 {
                break;
            }

            let fn_limb = f.limbs[len - 1];
            let gn_limb = g.limbs[len - 1];
            let cond = ((len as i64 - 2) >> (LIMB_SIZE - 1))
                | (fn_limb ^ (fn_limb >> (LIMB_SIZE - 1)))
                | (gn_limb ^ (gn_limb >> (LIMB_SIZE - 1)));
            if cond == 0 {
                f.limbs[len - 2] |= ((f.limbs[len - 1] as u64) << SIGNED_LIMB_SIZE) as i64;
                g.limbs[len - 2] |= ((g.limbs[len - 1] as u64) << SIGNED_LIMB_SIZE) as i64;
                len -= 1;
            }
        }

        let negate = (f.limbs[len - 1] >> (LIMB_SIZE - 1)) != 0;
        d.normalize(negate);
        d.to_num3072()
    }

    pub fn to_bytes(&self, out: &mut [u8; BYTE_SIZE]) {
        for i in 0..LIMBS {
            out[i * 8..(i + 1) * 8].copy_from_slice(&self.limbs[i].to_le_bytes());
        }
    }

    pub fn from_bytes(data: &[u8; BYTE_SIZE]) -> Self {
        let mut limbs = [0u64; LIMBS];
        for i in 0..LIMBS {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&data[i * 8..(i + 1) * 8]);
            limbs[i] = u64::from_le_bytes(buf);
        }
        Num3072 { limbs }
    }
}

impl Default for Num3072 {
    fn default() -> Self {
        let mut n = Num3072 { limbs: [0; LIMBS] };
        n.set_to_one();
        n
    }
}

#[derive(Clone)]
struct Num3072Signed {
    limbs: [i64; SIGNED_LIMBS],
}

const NEGINV256: [u8; 128] = [
    0xFF, 0x55, 0x33, 0x49, 0xC7, 0x5D, 0x3B, 0x11, 0x0F, 0xE5, 0xC3, 0x59, 0xD7, 0xED, 0xCB, 0x21,
    0x1F, 0x75, 0x53, 0x69, 0xE7, 0x7D, 0x5B, 0x31, 0x2F, 0x05, 0xE3, 0x79, 0xF7, 0x0D, 0xEB, 0x41,
    0x3F, 0x95, 0x73, 0x89, 0x07, 0x9D, 0x7B, 0x51, 0x4F, 0x25, 0x03, 0x99, 0x17, 0x2D, 0x0B, 0x61,
    0x5F, 0xB5, 0x93, 0xA9, 0x27, 0xBD, 0x9B, 0x71, 0x6F, 0x45, 0x23, 0xB9, 0x37, 0x4D, 0x2B, 0x81,
    0x7F, 0xD5, 0xB3, 0xC9, 0x47, 0xDD, 0xBB, 0x91, 0x8F, 0x65, 0x43, 0xD9, 0x57, 0x6D, 0x4B, 0xA1,
    0x9F, 0xF5, 0xD3, 0xE9, 0x67, 0xFD, 0xDB, 0xB1, 0xAF, 0x85, 0x63, 0xF9, 0x77, 0x8D, 0x6B, 0xC1,
    0xBF, 0x15, 0xF3, 0x09, 0x87, 0x1D, 0xFB, 0xD1, 0xCF, 0xA5, 0x83, 0x19, 0x97, 0xAD, 0x8B, 0xE1,
    0xDF, 0x35, 0x13, 0x29, 0xA7, 0x3D, 0x1B, 0xF1, 0xEF, 0xC5, 0xA3, 0x39, 0xB7, 0xCD, 0xAB, 0x01,
];

#[derive(Clone, Copy)]
struct SignedMatrix {
    u: i64,
    v: i64,
    q: i64,
    r: i64,
}

fn countr_zero(v: u64) -> u32 {
    if v == 0 {
        64
    } else {
        v.trailing_zeros()
    }
}

fn compute_divstep_matrix(eta: i32, f: i64, g: i64) -> (i32, SignedMatrix) {
    let mut u: u64 = 1;
    let mut v: u64 = 0;
    let mut q: u64 = 0;
    let mut r: u64 = 1;
    let mut f = f as u64;
    let mut g = g as u64;
    let mut eta = eta;
    let mut i = SIGNED_LIMB_SIZE as i32;

    loop {
        let zeros = (countr_zero(g | (u64::MAX << i))).min(i as u32) as i32;
        if zeros > 0 {
            g >>= zeros;
            u <<= zeros;
            v <<= zeros;
            eta -= zeros;
            i -= zeros;
        }
        if i == 0 {
            break;
        }
        if eta < 0 {
            eta = -eta;
            (f, g) = (g, f.wrapping_neg());
            (u, q) = (q, u.wrapping_neg());
            (v, r) = (r, v.wrapping_neg());
        }
        let limit = (eta + 1).min(i);
        let m = (u64::MAX >> (LIMB_SIZE - limit as usize)) & 255;
        let w = (g.wrapping_mul(NEGINV256[(f >> 1) as usize & 127] as u64)) & m;
        g = g.wrapping_add(f.wrapping_mul(w));
        q = q.wrapping_add(u.wrapping_mul(w));
        r = r.wrapping_add(v.wrapping_mul(w));
    }

    (
        eta,
        SignedMatrix {
            u: u as i64,
            v: v as i64,
            q: q as i64,
            r: r as i64,
        },
    )
}

impl Num3072Signed {
    fn zero() -> Self {
        Num3072Signed {
            limbs: [0i64; SIGNED_LIMBS],
        }
    }

    fn load_from_num3072(&mut self, in_val: &Num3072) {
        let mut c: i128 = 0;
        let mut b = 0i32;
        let mut outpos = 0usize;
        for i in 0..LIMBS {
            c += (in_val.limbs[i] as i128) << b;
            b += LIMB_SIZE as i32;
            while b >= SIGNED_LIMB_SIZE as i32 {
                self.limbs[outpos] = (c & (MAX_SIGNED_LIMB as i128)) as i64;
                c >>= SIGNED_LIMB_SIZE;
                b -= SIGNED_LIMB_SIZE as i32;
                outpos += 1;
            }
        }
        self.limbs[SIGNED_LIMBS - 1] = c as i64;
    }

    fn to_num3072(&self) -> Num3072 {
        let mut out = Num3072 { limbs: [0; LIMBS] };
        let mut c: u128 = 0;
        let mut b = 0i32;
        let mut outpos = 0usize;
        for i in 0..SIGNED_LIMBS {
            c += (self.limbs[i] as u64 as u128) << b;
            b += SIGNED_LIMB_SIZE as i32;
            if b >= LIMB_SIZE as i32 {
                out.limbs[outpos] = (c & 0xffff_ffff_ffff_ffff) as u64;
                c >>= LIMB_SIZE;
                b -= LIMB_SIZE as i32;
                outpos += 1;
            }
        }
        out
    }

    fn normalize(&mut self, negate: bool) {
        let cond_add = self.limbs[SIGNED_LIMBS - 1] >> (LIMB_SIZE - 1);
        self.limbs[0] += (-(MAX_PRIME_DIFF as i64)) & cond_add;
        self.limbs[FINAL_LIMB_POSITION] += (1i64 << FINAL_LIMB_MODULUS_BITS) & cond_add;
        let cond_negate = if negate { -1i64 } else { 0i64 };
        for i in 0..SIGNED_LIMBS {
            self.limbs[i] = (self.limbs[i] ^ cond_negate) - cond_negate;
        }
        for i in 0..SIGNED_LIMBS - 1 {
            self.limbs[i + 1] += self.limbs[i] >> SIGNED_LIMB_SIZE;
            self.limbs[i] &= MAX_SIGNED_LIMB;
        }
        let cond_add = self.limbs[SIGNED_LIMBS - 1] >> (LIMB_SIZE - 1);
        self.limbs[0] += (-(MAX_PRIME_DIFF as i64)) & cond_add;
        self.limbs[FINAL_LIMB_POSITION] += (1i64 << FINAL_LIMB_MODULUS_BITS) & cond_add;
        for i in 0..SIGNED_LIMBS - 1 {
            self.limbs[i + 1] += self.limbs[i] >> SIGNED_LIMB_SIZE;
            self.limbs[i] &= MAX_SIGNED_LIMB;
        }
    }
}

fn update_de(d: &mut Num3072Signed, e: &mut Num3072Signed, t: &SignedMatrix) {
    let u = t.u;
    let v = t.v;
    let q = t.q;
    let r = t.r;
    let sd = d.limbs[SIGNED_LIMBS - 1] >> (LIMB_SIZE - 1);
    let se = e.limbs[SIGNED_LIMBS - 1] >> (LIMB_SIZE - 1);
    let mut md = (u & sd) + (v & se);
    let mut me = (q & sd) + (r & se);
    let di = d.limbs[0];
    let ei = e.limbs[0];
    let mut cd = (u as i128) * (di as i128) + (v as i128) * (ei as i128);
    let mut ce = (q as i128) * (di as i128) + (r as i128) * (ei as i128);
    md = md.wrapping_sub(
        (MODULUS_INVERSE as i64)
            .wrapping_mul(cd as i64)
            .wrapping_add(md)
            & MAX_SIGNED_LIMB,
    );
    me = me.wrapping_sub(
        (MODULUS_INVERSE as i64)
            .wrapping_mul(ce as i64)
            .wrapping_add(me)
            & MAX_SIGNED_LIMB,
    );
    cd -= (1103717i128) * (md as i128);
    ce -= (1103717i128) * (me as i128);
    cd >>= SIGNED_LIMB_SIZE;
    ce >>= SIGNED_LIMB_SIZE;
    for i in 1..SIGNED_LIMBS - 1 {
        let di = d.limbs[i];
        let ei = e.limbs[i];
        cd += (u as i128) * (di as i128) + (v as i128) * (ei as i128);
        ce += (q as i128) * (di as i128) + (r as i128) * (ei as i128);
        d.limbs[i - 1] = (cd as i64) & MAX_SIGNED_LIMB;
        e.limbs[i - 1] = (ce as i64) & MAX_SIGNED_LIMB;
        cd >>= SIGNED_LIMB_SIZE;
        ce >>= SIGNED_LIMB_SIZE;
    }
    let di = d.limbs[SIGNED_LIMBS - 1];
    let ei = e.limbs[SIGNED_LIMBS - 1];
    cd += (u as i128) * (di as i128) + (v as i128) * (ei as i128);
    ce += (q as i128) * (di as i128) + (r as i128) * (ei as i128);
    cd += (md as i128) << FINAL_LIMB_MODULUS_BITS;
    ce += (me as i128) << FINAL_LIMB_MODULUS_BITS;
    d.limbs[SIGNED_LIMBS - 2] = (cd as i64) & MAX_SIGNED_LIMB;
    e.limbs[SIGNED_LIMBS - 2] = (ce as i64) & MAX_SIGNED_LIMB;
    cd >>= SIGNED_LIMB_SIZE;
    ce >>= SIGNED_LIMB_SIZE;
    d.limbs[SIGNED_LIMBS - 1] = cd as i64;
    e.limbs[SIGNED_LIMBS - 1] = ce as i64;
}

fn update_fg(f: &mut Num3072Signed, g: &mut Num3072Signed, t: &SignedMatrix, len: usize) {
    let u = t.u;
    let v = t.v;
    let q = t.q;
    let r = t.r;
    let mut cf = (u as i128) * (f.limbs[0] as i128) + (v as i128) * (g.limbs[0] as i128);
    let mut cg = (q as i128) * (f.limbs[0] as i128) + (r as i128) * (g.limbs[0] as i128);
    cf >>= SIGNED_LIMB_SIZE;
    cg >>= SIGNED_LIMB_SIZE;
    for i in 1..len {
        cf += (u as i128) * (f.limbs[i] as i128) + (v as i128) * (g.limbs[i] as i128);
        cg += (q as i128) * (f.limbs[i] as i128) + (r as i128) * (g.limbs[i] as i128);
        f.limbs[i - 1] = (cf as i64) & MAX_SIGNED_LIMB;
        g.limbs[i - 1] = (cg as i64) & MAX_SIGNED_LIMB;
        cf >>= SIGNED_LIMB_SIZE;
        cg >>= SIGNED_LIMB_SIZE;
    }
    f.limbs[len - 1] = cf as i64;
    g.limbs[len - 1] = cg as i64;
}
