use bitset_matrix::BitMatrix;

mod lin_alg;
pub use crate::lin_alg::linear_algebra;

fn main() {
    let message : Box<BitMatrix> = generate_message(128);
    let (t, t_inv) = linear_algebra::generate_invertible_mat(8);
    // 
    // let signature = generate_signature(message, pk);
    // let 
    // let verify = linear_algebra::mat_mult(&pk, &)
}

fn generate_keys(var_num : usize, eq_num : usize) -> (Box<BitMatrix>, Box<BitMatrix>, Box<BitMatrix>) {
    let mut f = Box::new(BitMatrix::new(eq_num, var_num)); //
    let (t, t_inv) = linear_algebra::generate_invertible_mat(var_num);
    let pk = Box::new(BitMatrix::new(eq_num, var_num)); // [n] -> [m]

    (pk, f, t)
}

fn generate_message(len : usize) -> Box<BitMatrix> {
    let random_bool_vec: Vec<bool> = (0..len)
        .map(|_| rand::random::<bool>())
        .collect();
    let bits = Box::new(BitMatrix::from_vec(vec!(random_bool_vec,)));
    bits
}

fn generate_signature(message : Box<BitMatrix>, len : usize) -> Box<BitMatrix> {
    let sig = BitMatrix::new(len, 1);
    // Fix Vinegar Variables
    // Collapse Matrix
    // Invert resulting matrix
    // Find Preimage 
    Box::new(sig)
}

fn verify_signature(message : Box<BitMatrix>, sig : Box<BitMatrix>, pk : Box<BitMatrix>) -> bool {
    message.eq(&linear_algebra::mat_mult(&sig, &pk))
}