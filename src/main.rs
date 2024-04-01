mod sat_solver;

fn main() {
    let p = sat_solver::get_sample_problem();
    println!("problem is: {:#?}", p);
    let solution = sat_solver::dpll::dpll(p).unwrap();
    println!("solution is {:?}", solution);
}
