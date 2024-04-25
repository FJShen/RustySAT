use crate::sat_solver::*;

pub trait Heuristics {
    fn new() -> Self;
    fn add_clause(&mut self, c: &Clause);
    fn decide(&mut self) -> Option<Literal>;
    fn unassign_variable(&mut self, var : Variable);
    fn assign_variable(&mut self, var : Variable);
}
