fn main() {
    let (prime_1, prime_2) : (u64, u64) = get_primes();
    let n : u64 = prime_1 * prime_2;
    println!("n={}*{}={}", prime_1, prime_2, n);
    let (public_key, private_key) : (u64, u64) = key_generation(prime_1, prime_2);
    println!("Keys: {}*{}={}mod{}", public_key, private_key, public_key*private_key%n, n);

    let message: u64 = 16;
    println!("Message: {}", message);
    let cipher_text: u64 = encrypt(message, public_key, n);
    println!("Ciphertext: {}", cipher_text);
    let decrypted_text: u64 = decrypt(cipher_text, private_key, n);
    println!("Message was: {}", decrypted_text);
}

fn get_primes() -> (u64, u64) {
    (13_u64, 23_u64)
}

fn key_generation(prime_1 : u64, prime_2 : u64) -> (u64, u64) {
    let n : u64  = prime_1 * prime_2;
    let phi_n : u64 = (prime_1 - 1) * (prime_2 - 1);
    // By Fermat's Little Theorem, x^phi(n) = 1 mod n
    let a: u64 = 19;
    let b: u64 = 7;
    (mod_pow(a, b, n), mod_pow(a, phi_n-b, n))
}

fn encrypt(m:u64, pk: u64, modulus: u64) -> u64 {
    (m*pk) % modulus
}

fn decrypt(c:u64, sk: u64, modulus: u64) -> u64 {
    (c*sk) % modulus
}

// Using Square and Multiply
fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 { return 0 }
    let mut result = 1;
    base = base % modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % modulus;
        }
        exp = exp >> 1;
        base = base * base % modulus
    }
    result
}