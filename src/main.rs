use std::env;
use log::trace;

mod parser;
mod sat_solver;
use crate::sat_solver::Problem;

fn main() {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        panic!("Usage: cargo run <dimacs file>");
    }

    let mut ps = Vec::<Problem>::new();
    // ps.push(sat_solver::get_sample_problem());
    ps.push(parser::parse(&args[1]));

    for p in ps {
        trace!("problem is: {:#?}", p);
        let result = sat_solver::dpll::dpll(p);
        if let Some(solution) = result{
            println!("SAT: solution is {:?}", solution);
        } else {
            println!("UNSAT");
        }
    }
}
