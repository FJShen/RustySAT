use std::env;

mod parser;
mod sat_solver;
use crate::sat_solver::Problem;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        panic!("Usage: cargo run <dimacs file>");
    }

    let mut ps = Vec::<Problem>::new();
    // ps.push(sat_solver::get_sample_problem());
    ps.push(parser::parse(&args[1]));

    for p in ps {
        println!("problem is: {:#?}", p);
        let solution = sat_solver::dpll::dpll(p);
        if let Some(ss) = solution{
            println!("solution is {:?}", ss)
        }
    }
}
