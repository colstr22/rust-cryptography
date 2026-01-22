pub mod linear_algebra {
    use std::cmp;
    use bitset_matrix::BitMatrix;
    
    pub fn split_word(word : u64, cols: usize) -> Vec<u8> {
        let mut oct_arr : Vec<u8> = Vec::new();
        match word.to_le_bytes().get(0) {
            Some(num) => {
                for i in 0..cmp::min(cols, 8) {
                    oct_arr.push(*num>>(i)&1);
                }
            }
            None => {}
        }
        oct_arr
    }

    pub fn split_words(words : &[u64], cols : usize) -> Vec<u8> {
        let mut oct_arr : Vec<u8> = Vec::new();
        for word in words {
            oct_arr.extend(split_word(*word, cols));
        }
        oct_arr
    }
    
    pub fn print_mat(mat : &BitMatrix) {
        println!("{:?}", mat);
        for n in 0..mat.rows() {
            println!("{:?}", split_words(mat.row_words(n), mat.cols()));
        }
    }
    
    pub fn mat_mult(mat_1 : &BitMatrix, mat_2 : &BitMatrix) -> Box<BitMatrix> {
        let mut product = Box::new(BitMatrix::new(mat_1.cols(), mat_2.rows()));
        for n in 0..mat_1.rows() {
            let row : Vec<u8> = split_words(mat_1.row_words(n), mat_1.cols());
            for m in 0..mat_2.cols() {
                let col : Vec<u8> =  mat_2.column(m).into_iter().map(|i| i as u8).collect();
                
                let mut sum : u8 = 0;
                for i in 0..row.len() {
                    sum = (sum + row[i] * col[i]) % 2;
                }
                product.set(n, m, sum != 0);
            }
        }
        product
    }
    
    pub fn augment_matrix(mat : BitMatrix, n : usize) -> Box<BitMatrix> {
        let mut aug_mat : Box<BitMatrix> = Box::new(BitMatrix::new(n, n + n));
        for i in 0..n {
            let row = split_words(mat.row_words(i), n);
            for (j, word) in row.iter().enumerate() {
                aug_mat.set(i, j, *word != 0);
            }
            aug_mat.set(i, n + i, true);
        }
        aug_mat
    }
    
    pub fn invert_mat(mat : BitMatrix, size : usize) -> Option<BitMatrix> {
        let mut aug_mat : Box<BitMatrix> = augment_matrix(mat, size);
        
        // Reduce to Upper Triangular
        for n in 0..size { // n x n matrix has n pivots in RREF
            if aug_mat.get(n, n) == false {
                for i in n+1..size {
                    if aug_mat.get(i, n) == true {
                        aug_mat.row_xor_assign(n, i);
                        break;
                    }
                }

                if aug_mat.get(n, n) == false {
                    return None;
                }
            }
            // Zero out rest of column 
            for m in n+1..aug_mat.rows() {
                if aug_mat.get(m, n) {
                    aug_mat.row_xor_assign(m, n);
                }
    
            }
        }

        // To RREF
        for n in 1..aug_mat.rows() {
            for m in 0..n {
                if aug_mat.get(m, n) == true {
                    aug_mat.row_xor_assign(m, n);
                }
            }
        }

        let mut inv_arr : Vec<Vec<bool>> = Vec::new();
        for n in 0..size {
            let mut row : Vec<bool> = Vec::new();
            for m in 0..size {
                row.push(aug_mat.get(n, size + m));
            }
            inv_arr.push(row);
        }
            
        Some(BitMatrix::from_vec(inv_arr))
    }
    
    pub fn generate_invertible_mat(n : usize) -> (Box<BitMatrix>, Box<BitMatrix>) {
        let mut mat : BitMatrix = BitMatrix::from_vec((0..n).map(|_| (0..n).map(|_| rand::random::<bool>()).collect()).collect());
        loop {
            let inv : Option<BitMatrix> = invert_mat(mat.clone(), n);
            match inv {
                Option::Some(b) => {
                    return (Box::new(mat), Box::new(b));
                }
                Option::None => {
                    mat = BitMatrix::from_vec((0..n).map(|_| (0..n).map(|_| rand::random::<bool>()).collect()).collect());
                }
            }
        }
    }
}
