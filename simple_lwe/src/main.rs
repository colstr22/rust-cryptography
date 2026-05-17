use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::Rng;
use rand::distributions::{Distribution, Uniform};

const N: usize = 256;
const Q: i64 = 8192;

fn main() {
    let text = "Hello, world!";
    println!("Message: {}", text);
    let m: DVector<i64> = string_to_poly_coeffs(text, N);
    let (a, t, s) = keygen();

    // Encrypt each bit separately
    let ciphertexts: Vec<(DVector<i64>, i64)> = m.iter()
        .map(|&bit| encrypt_bit(&a, &t, bit))
        .collect();

    // Decrypt each bit separately
    let decoded_bits: Vec<i64> = ciphertexts.iter()
        .map(|(u, v)| decrypt_bit(u, *v, &s))
        .collect();

    let decoded_m = DVector::from_vec(decoded_bits);
    let diff: i64 = m.iter().zip(decoded_m.iter()).map(|(a, b)| (a - b).abs()).sum();

    println!("Bits wrong: {}", diff);
    assert_eq!(m, decoded_m);

    let decoded_str = poly_coeffs_to_string(decoded_m);
    println!("Decoded: {}", decoded_str);
}

fn string_to_poly_coeffs(s: &str, n_degree: usize) -> DVector<i64> {
    let bytes = s.as_bytes();
    let mut coeffs = vec![0i64; n_degree];
    for (i, &byte) in bytes.iter().enumerate() {
        if i * 8 >= n_degree { break; }
        for bit in 0..8 {
            if i * 8 + bit < n_degree {
                coeffs[i * 8 + bit] = ((byte >> (7 - bit)) & 1) as i64;
            }
        }
    }
    DVector::from(coeffs)
}

fn poly_coeffs_to_string(poly: DVector<i64>) -> String {
    let byte_array_length = poly.len() / 8 + 1;
    let mut bytes : Vec<u8> = vec![0; byte_array_length];
    for (i , &byte ) in poly.iter().enumerate() {
        let byte_index : usize = ((i as f64) / (8 as f64)).floor() as usize ;
        bytes[byte_index] ^= (byte as u8) << 7-(i % 8);
    }
    bytes.iter().map(|x : &u8| *x as char).collect::<String>()
}

fn small_vector(rng: &mut impl Rng) -> i64 {
    Uniform::new_inclusive(-2i64, 2i64).sample(rng)
}

fn keygen() -> (DMatrix<i64>, DVector<i64>, DVector<i64>) {
    let coeff_range = Uniform::new(0i64, Q);
    let mut rng = rand::thread_rng();

    // Public Matrix
    let a: DMatrix<i64> = DMatrix::from_fn(N, N, |_, _| coeff_range.sample(&mut rng));
    // Secret Vector
    let s: DVector<i64> = DVector::from_fn(N, |_, _| small_vector(&mut rng));
    // Original Perturbation
    let e: DVector<i64> = DVector::from_fn(N, |_, _| small_vector(&mut rng));
    // Public Vector
    let t: DVector<i64> = DVector::from_fn(N, |row, _| {
        let dot: i64 = (0..N).map(|k| a[(row, k)] * s[k]).sum();
        dot + e[row]
    });

    (a, t, s)
}

fn encrypt_bit(a: &DMatrix<i64>, t: &DVector<i64>, bit: i64) -> (DVector<i64>, i64) {
    // bit is 0 or 1, mapped to 0 or Q/2
    let mut rng = rand::thread_rng();

    let r: DVector<i64> = DVector::from_fn(N, |_, _| {
        Uniform::new_inclusive(-1i64, 1i64).sample(&mut rng)
    });
    let e1: DVector<i64> = DVector::from_fn(N, |_, _| small_vector(&mut rng));
    let e2: i64 = small_vector(&mut rng);

    // u = A^T * r + e1 mod Q
    let u: DVector<i64> = DVector::from_fn(N, |row, _| {
        let dot: i64 = (0..N).map(|k| a[(k, row)] * r[k]).sum();
        (dot + e1[row]).rem_euclid(Q)
    });

    // v = t·r + e2 + (Q/2)*bit mod Q
    let dot: i64 = t.iter().zip(r.iter()).map(|(&ti, &ri)| ti * ri).sum();
    let v: i64 = (dot + e2 + (Q / 2) * bit).rem_euclid(Q);

    (u, v)
}

fn decrypt_bit(u: &DVector<i64>, v: i64, s: &DVector<i64>) -> i64 {
    let dot: i64 = u.iter().zip(s.iter()).map(|(&ui, &si)| ui * si).sum();
    let d = (v - dot).rem_euclid(Q);
    if (d - Q / 2).abs() < Q / 4 { 1 } else { 0 }
}