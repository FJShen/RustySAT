mod sat_structures;
mod sat_solver;

use std::cell::RefCell;
use crate::sat_structures::*;
use crate::sat_solver::*;

fn main() {
    println!("Hello, world!");
    let p = get_sample_problem();
    println!("problem: {:#?}", p);
    dpll(p);
}
