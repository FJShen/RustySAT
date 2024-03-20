mod sat_solver;
mod sat_structures;

use crate::sat_solver::*;
use crate::sat_structures::*;

fn main() {
    let p = get_sample_problem();
    println!("problem is: {:#?}", p);
    let solution = dpll(p);
    println!("solution is {:?}", solution);
}
