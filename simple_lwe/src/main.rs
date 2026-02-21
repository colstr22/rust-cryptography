use std::io::Bytes;

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
    // println!("Polynomial Form (len {})", m.len());

    let (a, t, s) = keygen();
    // println!("A:(dim {}*{})\ns:(len {})\nt:(len {})", a.nrows(), a.ncols(), s.len(), t.len());

    let (u, v) = encrypt(&a, &t, &m);
    // println!("v:({})\nu:({}x{})", v.len(), u.nrows(), u.ncols());

    let decoded_m = decrypt(&u, &v, &s);
    // println!("Decoded Polynomial {}", decoded_m);

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
        bytes[byte_index] ^= (byte as u8) <<7-(i % 8);
    }
    bytes.iter().map(|x : &u8| *x as char).collect::<String>()
}

fn small_error(rng: &mut impl Rng) -> i64 {
    Uniform::new_inclusive(-2i64, 2i64).sample(rng)
}

fn keygen() -> (DMatrix<i64>, DVector<i64>, DVector<i64>) {
    let coeff_range = Uniform::new(0i64, Q);
    let mut rng = rand::thread_rng();

    // Public Matrix
    let a: DMatrix<i64> = DMatrix::from_fn(N, N, |_, _| coeff_range.sample(&mut rng));
    // Secret Vector
    let s: DVector<i64> = DVector::from_fn(N, |_, _| small_error(&mut rng));
    // Original Perturbation
    let e: DVector<i64> = DVector::from_fn(N, |_, _| small_error(&mut rng));
    // Public Vector
    let t: DVector<i64> = DVector::from_fn(N, |row, _| {
        let dot: i64 = (0..N).map(|k| a[(row, k)] * s[k]).sum();
        dot + e[row]  // NO rem_euclid here
    });

    (a, t, s)
}

fn encrypt(a: &DMatrix<i64>, t: &DVector<i64>, message: &DVector<i64>) -> (DMatrix<i64>, DVector<i64>) {
    let mut rng = rand::thread_rng();
    let m_len = message.len();

    let r: DMatrix<i64> = DMatrix::from_fn(N, m_len, |_, _| {
        Uniform::new_inclusive(0i64, 1i64).sample(&mut rng)
    });
    let e1: DMatrix<i64> = DMatrix::from_fn(N, m_len, |_, _| small_error(&mut rng));
    let e2: DVector<i64> = DVector::from_fn(m_len, |_, _| small_error(&mut rng));

    // u = A^T * r + e1 mod Q
    let u: DMatrix<i64> = DMatrix::from_fn(N, m_len, |row, col| {
        let dot: i64 = (0..N).map(|k| a[(k, row)] * r[(k, col)]).sum(); // a[(k,row)] = A^T[(row,k)]
        (dot + e1[(row, col)]).rem_euclid(Q)
    });

    // v = t·r + e2 + (Q/2)*m mod Q
    let v: DVector<i64> = DVector::from_fn(m_len, |i, _| {
        let r_col = r.column(i);
        let dot: i64 = t.iter().zip(r_col.iter()).map(|(&ti, &ri)| ti * ri).sum();
        (dot + e2[i] + (Q / 2) * message[i]).rem_euclid(Q)
    });

    (u, v)
}

fn decrypt(u: &DMatrix<i64>, v: &DVector<i64>, s: &DVector<i64>) -> DVector<i64> {
    // Compute Errorless solution, find distance (including wraparound) 
    // Maybe try centered-residue representation for branchless?

    let m_len = v.len();
    DVector::from_fn(m_len, |i, _| {
        let u_col = u.column(i);
        let dot: i64 = s.iter().zip(u_col.iter()).map(|(&si, &ui)| si * ui).sum();
        let d = (v[i] - dot).rem_euclid(Q);
        
        // Distance to 0, accounting for wraparound in [0, Q)
        let dist_to_zero = d.min(Q - d);
        let dist_to_half = (d - Q / 2).abs();
        
        if dist_to_half < dist_to_zero { 1 } else { 0 }
    })
}