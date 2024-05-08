use crate::sat_solver::*;

pub trait Heuristics {
    fn new() -> Self;
    fn add_parsed_clause(&mut self, c: &Clause);
    fn add_conflict_clause(&mut self, c: &Clause);
    fn decide(&mut self) -> Option<Literal>;
    fn assign_variable(&mut self, var: Variable);
    fn unassign_variable(&mut self, var: Variable);
    fn satisfy_clause(&mut self, c: &Clause);
    fn unsatisfy_clause(&mut self, c: &Clause);
    fn set_use_bcp(&mut self, _use_bcp: bool);
    fn use_bcp(&self) -> bool {
        /* default impl */
        false
    }
}
