use std::{borrow::BorrowMut, env};

mod parser;
mod vsids;
mod sat_solver;
use vsids::VSIDS;

use crate::sat_solver::Problem;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        panic!("Usage: cargo run <dimacs file>");
    }

    let mut ps = Vec::<Problem>::new();
    let mut vsids = VSIDS::new();
    // ps.push(sat_solver::get_sample_problem());
    ps.push(parser::parse(&args[1], vsids.borrow_mut()));
    vsids.print_sort_by_counters();
    for i in  0..5 {
        let l = vsids.decide().unwrap();
        let pol = match l.polarity {
            sat_solver::Polarity::Off  => '-',
            sat_solver::Polarity::On   => '+',
        };
        println!("popped {}{}", pol, l.variable.index);
        println!("--------------------------------");
        vsids.print_sort_by_counters();
    }

    for p in ps {
        println!("problem is: {:#?}", p);
        let solution = sat_solver::dpll::dpll(p);
        println!("solution is {:?}", solution);
    }
}
