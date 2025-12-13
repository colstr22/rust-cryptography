use std::ops::Mul;

use num_primes::Generator;
use crypto_bigint::{NonZero, U4096, Random, rand_core::OsRng};

const BITS : usize = 2048;
const RADIX : u32 = 16;

fn main() {
    // Generate Primes in Big Endian
    let prime_1_bytes : &str = &Box::new(Generator::new_prime(BITS).to_str_radix(RADIX));
    let prime_2_bytes : &str = &Box::new(Generator::new_prime(BITS).to_str_radix(RADIX));

    // Convert to U4096
    let prime_1 : U4096 = U4096::from_str_radix_vartime(prime_1_bytes, RADIX).unwrap();
    let prime_2 : U4096 = U4096::from_str_radix_vartime(prime_2_bytes, RADIX).unwrap();

    println!("Primes Selected:\n{}\n{}", prime_1, prime_2);

    // Determine modulus
    let n : NonZero<U4096> = NonZero::new(prime_1.mul(prime_2)).unwrap();

    println!("Modulus Computed:\n{}", n);


    let (public_key, private_key) : (U4096, U4096) = key_generation(prime_1, prime_2, n);
    
    println!("Keys Computed:\n{}\n{}", public_key, private_key);

    let message : U4096 = U4096::random(&mut OsRng).shr_vartime(4);
    println!("Message (Original): {}", message);

    let cipher_text: U4096 = encrypt(message, public_key, n);
    let decrypted_text: U4096 = decrypt(cipher_text, private_key, n);
    
    println!("Message (Decrypted): {}", decrypted_text);

    assert_eq!(message, decrypted_text);
}

fn key_generation(prime_1 : U4096, prime_2 : U4096, n : NonZero<U4096>) -> (U4096, U4096) {
    let phi_n : U4096 = (prime_1 - U4096::ONE).mul_mod(&(prime_2 - U4096::ONE), &n);

    // By Fermat's Little Theorem, x^phi(n) = 1 mod n
    let a: U4096 = U4096::from_u32(19_u32);
    let b: U4096 = U4096::from_u32(7_u32);

    let public_key = mod_pow(a, b, n);
    let private_key = mod_pow(a, phi_n-b, n);

    assert_eq!(public_key.mul_mod(&private_key, &n), U4096::ONE);

    (public_key, private_key)
}

fn encrypt(m:U4096, pk: U4096, modulus: NonZero<U4096>) -> U4096 {
    m.mul_mod(&pk, &modulus)
}

fn decrypt(c:U4096, sk: U4096, modulus: NonZero<U4096>) -> U4096 {
    c.mul_mod(&sk, &modulus)
}

fn mod_pow(mut base: U4096, mut exp: U4096, modulus: NonZero<U4096>) -> U4096 {
    if modulus == NonZero::new(U4096::ONE).unwrap() { return U4096::ZERO }
    let mut result: U4096 = U4096::ONE;
    base = base % modulus;
    while exp > U4096::ZERO {
        if exp.bitand(&U4096::ONE) == U4096::ONE {
            result = result.mul_mod(&base, &modulus);
        }
        exp = exp >> 1;
        base = base.mul_mod(&base, &modulus);
    }
    result
}