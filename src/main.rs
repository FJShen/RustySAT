mod heuristics;
mod parser;
mod profiler;
mod sat_solver;
use core::panic;
use std::collections::BTreeSet;

use clap::Parser;
use log::{trace,info};
use sat_solver::*;
use crate::heuristics::{ascending::Ascending, heuristics::Heuristics, dlis::DLIS, vsids::VSIDS};
use crate::profiler::SolverProfiler;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(index=1)]
    input: String,

    #[arg(long, default_value_t = String::from("vsids"))]
    heuristics: String,

    #[arg(long)]
    no_bcp: bool,

    #[arg(long)]
    check: bool,

    #[arg(long)]
    satisfiable: bool,
}

fn test(input : &String, mut h: impl Heuristics, use_bcp: bool) -> (Problem, Option<SolutionStack>) {
    let mut prof = SolverProfiler::new();
    h.set_use_bcp(use_bcp);
    let mut problem = parser::parse(input, &mut h);
    trace!(target: "solver", "problem is: {:#?}", problem);
    prof.reset_start_time();
    let solution = sat_solver::dpll::dpll(&mut problem, &mut h, &mut prof);
    prof.calc_duration_till_now();
    info!(target: "solver", "solution is {:?}", solution);
    info!(target: "profiler", "Profiling results: {}", prof);
    (problem, solution)
}

fn verify(p : &Problem, s: &SolutionStack) -> bool {
    let mut clauses_unsatisfied = BTreeSet::<u32>::new();
    for c in p.list_of_clauses.iter() {
        clauses_unsatisfied.insert((**c).borrow().id);
    }
    for step in s.stack.iter() {
        let literal = Literal{
            variable: step.assignment.variable,
            polarity: step.assignment.polarity,
        };
        if let Some(clauses_appeared) = p.list_of_literal_infos.get(&literal) {
            let cs = &(**clauses_appeared).borrow_mut().list_of_clauses;
            for clause in cs.iter() {
                clauses_unsatisfied.remove(&(**clause).borrow().id);
            }
        }
    }

    clauses_unsatisfied.is_empty()
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    info!(target: "solver", "{:?}", args);
    let use_bcp = !args.no_bcp;

    let (p, s) = match args.heuristics.as_str() {
        "ascending" => test(&args.input, Ascending::new(), use_bcp),
        "dlis"      => test(&args.input, DLIS::new(), use_bcp),
        "vsids"     => test(&args.input, VSIDS::new(), use_bcp),
        _           => panic!("Unrecognised heuristics specified"),
    };

    if args.check {
        assert!(args.satisfiable == s.is_some());
    }
    if let Some(sol) = &s  {
        info!("solution is {:?}", sol);
        assert!(verify(&p, &sol));
        println!("RESULT: SAT");
        print!("ASSIGNMENT:");
        let mut sorted = BTreeSet::new();
        for s in sol.stack.iter() {
            sorted.insert((s.assignment.variable.index, s.assignment.polarity));
        }
        for (var, pol) in sorted {
            print!(" {}={}", var, if pol == Polarity::Off {0} else {1});
        }
        println!();
    }
    else {
        println!("RESULT: UNSAT");
    }
}
