mod heuristics;
mod parser;
mod profiler;
mod sat_solver;
use clap::Parser;
use log::{info, trace};

use crate::heuristics::{ascending::Ascending, heuristics::Heuristics, vsids::VSIDS};
use crate::profiler::SolverProfiler;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input: String,

    #[arg(default_value_t = String::from("vsids"), long)]
    heuristics: String,

    #[arg(long)]
    no_bcp: bool,
}

fn test_ascending(input: String, use_bcp: bool) {
    let mut h = Ascending::new();
    let mut prof = SolverProfiler::new();
    h.set_use_bcp(use_bcp);
    let p = parser::parse(&input, &mut h);
    trace!(target: "solver", "problem is: {:#?}", p);
    prof.reset_start_time();
    let solution = sat_solver::dpll::dpll(p, h, &mut prof);
    prof.calc_duration_till_now();
    info!(target: "solver", "solution is {:?}", solution);
    info!(target: "profiler", "Profiling results: {}", prof);
}

fn test_vsids(input: String, use_bcp: bool) {
    let mut h = VSIDS::new();
    let mut prof = SolverProfiler::new();
    h.set_use_bcp(use_bcp);
    let p = parser::parse(&input, &mut h);
    trace!(target: "solver", "problem is: {:#?}", p);
    prof.reset_start_time();
    let solution = sat_solver::dpll::dpll(p, h, &mut prof);
    prof.calc_duration_till_now();
    info!(target: "solver", "solution is {:?}", solution);
    info!(target: "profiler", "Profiling results: {}", prof);
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    info!(target: "solver", "{:?}", args);

    let use_bcp = !args.no_bcp;

    // ps.push(sat_solver::get_sample_problem());
    match args.heuristics.as_str() {
        "vsids" => test_vsids(args.input, use_bcp),
        _ => test_ascending(args.input, use_bcp),
    };
}
