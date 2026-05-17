use rand::Rng;
use rand::distributions::{Distribution, Uniform};

// Ring R_q = Z_q[x]/(x^N + 1)
const BITS_PER_COEFF: usize = 4;
const N: usize = 256;
// Q scales with BITS_PER_COEFF so that Q/LEVELS >> noise per coefficient.
// Noise is O(N * error_bound^2) ~ 1024, so Q/LEVELS must exceed that by a wide margin.
const Q: i64 = 1 << (BITS_PER_COEFF + 14);
const LEVELS: i64 = 1 << BITS_PER_COEFF;
const STEP: i64 = Q / LEVELS;

type Poly = Vec<i64>; // (N coefficients in Z_q)
type PolyRef<'a> = &'a [i64];

fn main() {
    let text = "Hello, world!";
    println!("Message: {}", text);
    let m: Poly = string_to_coeffs(text);
    let (a, t, s) = keygen();
    let (u, v) = encrypt(&a, &t, &m);
    let decoded_m = decrypt(&u, &v, &s);

    let diff: i64 = m.iter().zip(decoded_m.iter()).map(|(a, b)| (a - b).abs()).sum();
    println!("Coefficients wrong: {}", diff);
    assert_eq!(m, decoded_m);

    let decoded_str = coeffs_to_string(&decoded_m);
    println!("Decoded: {}", decoded_str);
}

// Pack bits of string into polynomial coefficients, BITS_PER_COEFF bits each.
fn string_to_coeffs(s: &str) -> Poly {
    let bits: Vec<u8> = s.bytes()
        .flat_map(|b| (0..8usize).rev().map(move |i| (b >> i) & 1))
        .collect();
    let mut coeffs = vec![0i64; N];
    for (i, chunk) in bits.chunks(BITS_PER_COEFF).enumerate() {
        if i >= N { break; }
        coeffs[i] = chunk.iter().fold(0i64, |acc, &bit| (acc << 1) | bit as i64);
    }
    coeffs
}

// Unpack BITS_PER_COEFF bits from each coefficient back into a string.
fn coeffs_to_string(coeffs: PolyRef) -> String {
    let bits: Vec<u8> = coeffs.iter()
        .flat_map(|&v| (0..BITS_PER_COEFF).rev().map(move |i| ((v >> i) & 1) as u8))
        .collect();
    bits.chunks(8)
        .map(|chunk| chunk.iter().fold(0u8, |acc, &bit| (acc << 1) | bit) as char)
        .collect()
}

fn small_error(rng: &mut impl Rng) -> i64 {
    Uniform::new_inclusive(-2i64, 2i64).sample(rng)
}

fn poly_add(a: PolyRef, b: PolyRef) -> Poly {
    a.iter().zip(b.iter()).map(|(&x, &y)| (x + y).rem_euclid(Q)).collect()
}

fn poly_sub(a: PolyRef, b: PolyRef) -> Poly {
    a.iter().zip(b.iter()).map(|(&x, &y)| (x - y).rem_euclid(Q)).collect()
}

// Multiply in R_q = Z_q[x]/(x^N + 1): x^N = -1
fn poly_mul(a: PolyRef, b: PolyRef) -> Poly {
    let mut result = vec![0i64; N];
    for i in 0..N {
        for j in 0..N {
            let deg = i + j;
            if deg < N {
                result[deg] += a[i] * b[j];
            } else {
                result[deg - N] -= a[i] * b[j];
            }
        }
    }
    result.iter().map(|&x| x.rem_euclid(Q)).collect()
}

fn poly_scale(a: PolyRef, scalar: i64) -> Poly {
    a.iter().map(|&x| (x * scalar).rem_euclid(Q)).collect()
}

fn keygen() -> (Poly, Poly, Poly) {
    let coeff_range = Uniform::new(0i64, Q);
    let mut rng = rand::thread_rng();

    let a: Poly = (0..N).map(|_| coeff_range.sample(&mut rng)).collect();
    let s: Poly = (0..N).map(|_| small_error(&mut rng)).collect();
    let e: Poly = (0..N).map(|_| small_error(&mut rng)).collect();
    let t = poly_add(&poly_mul(&a, &s), &e);

    (a, t, s)
}

fn encrypt(a: PolyRef, t: PolyRef, message: PolyRef) -> (Poly, Poly) {
    let mut rng = rand::thread_rng();

    let r: Poly = (0..N).map(|_| Uniform::new_inclusive(0i64, 1i64).sample(&mut rng)).collect();
    let e1: Poly = (0..N).map(|_| small_error(&mut rng)).collect();
    let e2: Poly = (0..N).map(|_| small_error(&mut rng)).collect();

    // u = a*r + e1
    let u = poly_add(&poly_mul(a, &r), &e1);

    // v = t*r + e2 + STEP*m
    // Each coefficient value in [0, LEVELS) maps to a multiple of STEP in [0, Q).
    let m_scaled = poly_scale(message, STEP);
    let v = poly_add(&poly_add(&poly_mul(t, &r), &e2), &m_scaled);

    (u, v)
}

fn decrypt(u: PolyRef, v: PolyRef, s: PolyRef) -> Poly {
    // d = v - s*u ≈ STEP*m + small noise
    let d = poly_sub(v, &poly_mul(s, u));
    d.iter().map(|&x| {
        // Round to nearest multiple of STEP, then recover the coefficient value.
        ((x + STEP / 2) / STEP) % LEVELS
    }).collect()
}
