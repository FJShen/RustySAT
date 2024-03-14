mod sat_structures;

use std::cell::RefCell;
use crate::sat_structures::*;

fn main() {
    println!("Hello, world!");
    let p = get_sample_problem();
    println!("{:#?}", p)
}
