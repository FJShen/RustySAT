mod parser;
mod heuristics;
mod sat_solver;
use clap::Parser;

use crate::{heuristics::{ascending::Ascending, heuristics::Heuristics, vsids::VSIDS}, sat_solver::Problem};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input: String,
    heuristics: String,
}

fn test_ascending(input : String) {
    let mut h = Ascending::new();
    let p = parser::parse(&input, &mut h);
    println!("problem is: {:#?}", p);
    let solution = sat_solver::dpll::dpll(p, h);
    println!("solution is {:?}", solution);
}

fn test_vsids(input : String) {
    let mut h = VSIDS::new();
    let p = parser::parse(&input, &mut h);
    println!("problem is: {:#?}", p);
    let solution = sat_solver::dpll::dpll(p, h);
    println!("solution is {:?}", solution);
}

fn main() {
    let args = Args::parse();
    println!("args: {}, {}", args.input, args.heuristics);

    // ps.push(sat_solver::get_sample_problem());
    match args.heuristics.as_str() {
        "vsids" => test_vsids(args.input),
        _       => test_ascending(args.input),
    };
}
