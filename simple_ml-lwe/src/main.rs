use rand::Rng;
use rand::distributions::{Distribution, Uniform};

// Module-LWE in R_q^k, where R_q = Z_q[x]/(x^N + 1).
// k=1 reduces to RLWE; k=2 is Kyber-512 style.
const K: usize = 2;
const BITS_PER_COEFF: usize = 4;
const N: usize = 256;
// Q scales with both BITS_PER_COEFF and K: noise grows as K*N*error_bound^2.
const Q: i64 = 1 << (BITS_PER_COEFF + 13 + K);
const LEVELS: i64 = 1 << BITS_PER_COEFF;
const STEP: i64 = Q / LEVELS;

// Type aliases for clarity:

type Poly    = Vec<i64>;           // (N coefficients in Z_q)
type PolyRef<'a>    = &'a [i64];
type PolyVec = Vec<Poly>;          // (k polynomials)
type PolyVecRef<'a> = &'a [Poly];
type PolyMat = Vec<PolyVec>;       // (k×k polynomials, row-major: A[i][j])
type PolyMatRef<'a> = &'a [PolyVec];

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

fn string_to_coeffs(
    s: &str
) -> Poly {
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

fn coeffs_to_string(
    coeffs: PolyRef
) -> String {
    let bits: Vec<u8> = coeffs.iter()
        .flat_map(|&v| (0..BITS_PER_COEFF).rev().map(move |i| ((v >> i) & 1) as u8))
        .collect();
    bits.chunks(8)
        .map(|chunk| chunk.iter().fold(0u8, |acc, &bit| (acc << 1) | bit) as char)
        .collect()
}

fn small_error(
    rng: &mut impl Rng
) -> i64 {
    Uniform::new_inclusive(-2i64, 2i64).sample(rng)
}

// Polynomial arithmetic in R_q = Z_q[x]/(x^N + 1)
fn poly_add(
    a: PolyRef, 
    b: PolyRef
) -> Poly {
    a.iter().zip(b.iter()).map(|(&x, &y)| (x + y).rem_euclid(Q)).collect()
}

fn poly_sub(
    a: PolyRef, 
    b: PolyRef
) -> Poly {
    a.iter().zip(b.iter()).map(|(&x, &y)| (x - y).rem_euclid(Q)).collect()
}

fn poly_mul(
    a: PolyRef, 
    b: PolyRef
) -> Poly {
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

fn poly_scale(
    a: PolyRef, 
    scalar: i64
) -> Poly {
    a.iter().map(|&x| (x * scalar).rem_euclid(Q)).collect()
}

// Module arithmetic
fn polyvec_dot(
    a: PolyVecRef, 
    b: PolyVecRef
) -> Poly {
    a.iter().zip(b.iter()).fold(vec![0i64; N], |acc, (ai, bi)| {
        poly_add(&acc, &poly_mul(ai, bi))
    })
}

fn polyvec_add(a: PolyVecRef, b: PolyVecRef) -> Vec<Poly> {
    a.iter().zip(b.iter()).map(|(ai, bi)| poly_add(ai, bi)).collect()
}

// Row i of result = dot(A[i], v)
fn polymat_mul_vec(
    a: PolyMatRef, 
    v: PolyVecRef
) -> PolyVec {
    a.iter().map(|row| polyvec_dot(row, v)).collect()
}

fn polymat_transpose(
    a: PolyMatRef
) -> PolyMat {
    (0..K).map(|i| (0..K).map(|j| a[j][i].clone()).collect()).collect()
}

fn keygen() -> (PolyMat, PolyVec, PolyVec) {
    let coeff_range = Uniform::new(0i64, Q);
    let mut rng = rand::thread_rng();

    let a: PolyMat = (0..K)
        .map(|_| (0..K)
            .map(|_| (0..N).map(|_| coeff_range.sample(&mut rng)).collect())
            .collect())
        .collect();

    let s: PolyVec = (0..K)
        .map(|_| (0..N).map(|_| small_error(&mut rng)).collect())
        .collect();

    let e: PolyVec = (0..K)
        .map(|_| (0..N).map(|_| small_error(&mut rng)).collect())
        .collect();

    // t = A*s + e
    let t = polyvec_add(&polymat_mul_vec(&a, &s), &e);

    (a, t, s)
}

fn encrypt(
    a: PolyMatRef,
    t: PolyVecRef,
    message: &[i64],
) -> (PolyVec, Poly) {
    let mut rng = rand::thread_rng();

    let r: PolyVec = (0..K)
        .map(|_| (0..N).map(|_| Uniform::new_inclusive(0i64, 1i64).sample(&mut rng)).collect())
        .collect();
    let e1: PolyVec = (0..K)
        .map(|_| (0..N).map(|_| small_error(&mut rng)).collect())
        .collect();
    let e2: Poly = (0..N).map(|_| small_error(&mut rng)).collect();

    // u = A^T * r + e1
    let u = polyvec_add(&polymat_mul_vec(&polymat_transpose(a), &r), &e1);

    // v = t^T * r + e2 + STEP*m
    let m_scaled = poly_scale(message, STEP);
    let v = poly_add(&poly_add(&polyvec_dot(t, &r), &e2), &m_scaled);

    (u, v)
}

// d = v - s^T*u ≈ STEP*m + noise; round each coefficient to recover m.
fn decrypt(u: PolyVecRef, v: PolyRef, s: PolyVecRef) -> Poly {
    let d = poly_sub(v, &polyvec_dot(s, u));
    d.iter().map(|&x| ((x + STEP / 2) / STEP) % LEVELS).collect()
}
