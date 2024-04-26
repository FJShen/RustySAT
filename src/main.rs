mod parser;
mod heuristics;
mod sat_solver;
use std::{borrow::{Borrow, BorrowMut}, collections::BTreeSet};

use clap::Parser;
use log::{trace,info};
use sat_solver::*;

use crate::heuristics::{ascending::Ascending, heuristics::Heuristics, vsids::VSIDS};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input: String,
    heuristics: String,
}

fn test_ascending(input : String) -> (Problem, Option<SolutionStack>) {
    let mut h = Ascending::new();
    let mut p = parser::parse(&input, &mut h);
    trace!(target: "solver", "problem is: {:#?}", p);
    let s = sat_solver::dpll::dpll(p.borrow_mut(), h);
    (p, s)
}

fn test_vsids(input : String)  -> (Problem, Option<SolutionStack>)  {
    let mut h = VSIDS::new();
    let mut p = parser::parse(&input, &mut h);
    trace!(target: "solver", "problem is: {:#?}", p);
    let s = sat_solver::dpll::dpll(p.borrow_mut(), h);
    (p, s)
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
    info!(target: "solver", "args: {}, {}", args.input, args.heuristics);

    // ps.push(sat_solver::get_sample_problem());
    let (p, s) = match args.heuristics.as_str() {
        "vsids" => test_vsids(args.input),
        _       => test_ascending(args.input),
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
