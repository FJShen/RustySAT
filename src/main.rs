mod heuristics;
mod parser;
mod profiler;
mod sat_solver;
use std::{borrow::BorrowMut, collections::BTreeSet};

use clap::Parser;
use log::{info, trace};
use crate::heuristics::{ascending::Ascending, heuristics::Heuristics, vsids::VSIDS};
use crate::profiler::SolverProfiler;
use sat_solver::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input: String,

    #[arg(default_value_t = String::from("vsids"), long)]
    heuristics: String,

    #[arg(long)]
    no_bcp: bool,
}

fn test_ascending(input: String, use_bcp: bool) -> (Problem, Option<SolutionStack>) {
    let mut h = Ascending::new();
    let mut prof = SolverProfiler::new();
    h.set_use_bcp(use_bcp);
    let mut p = parser::parse(&input, &mut h);
    trace!(target: "solver", "problem is: {:#?}", p);
    prof.reset_start_time();
    let solution = sat_solver::dpll::dpll(&mut p, h, &mut prof);
    prof.calc_duration_till_now();
    info!(target: "solver", "solution is {:?}", solution);
    info!(target: "profiler", "Profiling results: {}", prof);
    (p, solution)
}

fn test_vsids(input: String, use_bcp: bool) -> (Problem, Option<SolutionStack>) {
    let mut h = VSIDS::new();
    let mut prof = SolverProfiler::new();
    h.set_use_bcp(use_bcp);
    let mut p = parser::parse(&input, &mut h);
    trace!(target: "solver", "problem is: {:#?}", p);
    prof.reset_start_time();
    let solution = sat_solver::dpll::dpll(&mut p, h, &mut prof);
    prof.calc_duration_till_now();
    info!(target: "solver", "solution is {:?}", solution);
    info!(target: "profiler", "Profiling results: {}", prof);
    (p, solution)
}

fn verify_solution(p : &Problem, s: &SolutionStack) -> bool {
    let mut clauses_unsatisfied = BTreeSet::<u32>::new();
    for c in p.list_of_clauses.iter() {
        clauses_unsatisfied.insert((**c).borrow().id);
    }
    for step in s.stack.iter() {
        let literal = Literal{
            variable: step.assignment.variable,
            polarity: step.assignment.polarity,
        };
        let clauses_appeared = &(**p.list_of_literal_infos.get(&literal).unwrap()).borrow_mut().list_of_clauses;
        for clause in clauses_appeared.iter() {
            clauses_unsatisfied.remove(&(**clause).borrow().id);
        }
    }

    clauses_unsatisfied.is_empty()
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    info!(target: "solver", "{:?}", args);

    let use_bcp = !args.no_bcp;

    // ps.push(sat_solver::get_sample_problem());
    let (p, s) = match args.heuristics.as_str() {
        "vsids" => test_vsids(args.input, use_bcp),
        _       => test_ascending(args.input, use_bcp),
    };

    if let Some(s) = &s  {
        if verify_solution(&p, &s) {
            info!(target: "verifier", "solution is correct");
        }
        else {
            info!(target: "verifier", "solution is incorrect");
        }
    }
    info!(target: "solver", "solution is {:?}", s);
}
