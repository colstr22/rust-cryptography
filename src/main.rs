use std::ops::Mul;

use num_primes::Generator;
use crypto_bigint::{NonZero, U256};

const BITS : usize = 256;
const RADIX : u32 = 16;

fn main() {
    // Generate Primes in Big Endian
    let prime_1_bytes : &str = &Box::new(Generator::new_prime(128).to_str_radix(RADIX));
    let prime_2_bytes : &str = &Box::new(Generator::new_prime(128).to_str_radix(RADIX));

    // Convert to U256
    let prime_1 : U256 = U256::from_str_radix_vartime(prime_1_bytes, RADIX).unwrap();
    let prime_2 : U256 = U256::from_str_radix_vartime(prime_2_bytes, RADIX).unwrap();

    println!("Primes Selected:\n{}\n{}", prime_1, prime_2);

    // Determine modulus
    let n : NonZero<U256> = NonZero::new(prime_1.mul(prime_2)).unwrap();

    println!("Modulus Computed:\n{}", n);


    let (public_key, private_key) : (U256, U256) = key_generation(prime_1, prime_2, n);
    
    println!("Keys Computed:\n{}\n{}", public_key, private_key);

    let message : U256 = U256::from(16_u64);
    println!("Message (Original): {}", message);

    let cipher_text: U256 = encrypt(message, public_key, n);
    let decrypted_text: U256 = decrypt(cipher_text, private_key, n);
    
    println!("Message (Decrypted): {}", decrypted_text);
}

fn key_generation(prime_1 : U256, prime_2 : U256, n : NonZero<U256>) -> (U256, U256) {
    let phi_n : U256 = (prime_1 - U256::ONE).mul_mod(&(prime_2 - U256::ONE), &n);

    // By Fermat's Little Theorem, x^phi(n) = 1 mod n
    let a: U256 = U256::from_u32(19_u32);
    let b: U256 = U256::from_u32(7_u32);

    let public_key = mod_pow(a, b, n);
    let private_key = mod_pow(a, phi_n-b, n);

    assert_eq!(public_key.mul_mod(&private_key, &n), U256::ONE);

    (public_key, private_key)
}

fn encrypt(m:U256, pk: U256, modulus: NonZero<U256>) -> U256 {
    m.mul_mod(&pk, &modulus)
}

fn decrypt(c:U256, sk: U256, modulus: NonZero<U256>) -> U256 {
    c.mul_mod(&sk, &modulus)
}

fn mod_pow(mut base: U256, mut exp: U256, modulus: NonZero<U256>) -> U256 {
    if modulus == NonZero::new(U256::ONE).unwrap() { return U256::ZERO }
    let mut result: U256 = U256::ONE;
    base = base % modulus;
    while exp > U256::ZERO {
        if exp.bitand(&U256::ONE) == U256::ONE {
            result = result.mul_mod(&base, &modulus);
        }
        exp = exp >> 1;
        base = base.mul_mod(&base, &modulus);
    }
    result
}