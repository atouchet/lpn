extern crate lpn;

use lpn::bkw::*;
use lpn::codes::*;
use lpn::covering_codes::*;
use lpn::oracle::LpnOracle;

fn main() {
    let mut oracle: LpnOracle = LpnOracle::new(25, 1.0 / 16.0);
    oracle.get_samples(800555);
    let code = ConcatenatedCode::new(vec![&HammingCode15_11, &HammingCode7_4, &HammingCode3_1]);
    reduce_sparse_secret(&mut oracle);
    code_reduction(&mut oracle, code);
    let mut secret = oracle.secret.clone();
    secret.truncate(oracle.k as usize);
    let solution = bkw_solve(oracle);

    println!("Found:  {:?}", solution);
    println!("Actual: {:?}", secret);
}
