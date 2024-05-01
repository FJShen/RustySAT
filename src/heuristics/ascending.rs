use crate::heuristics::heuristics::*;
use crate::sat_solver::*;
use core::fmt;
use log::trace;
use std::collections::BTreeSet;
use std::fmt::Debug;

pub struct Ascending {
    pub variable_unassigned: BTreeSet<Variable>,
    pub variable_assigned: BTreeSet<Variable>,
    use_bcp: bool,
}

impl Debug for Ascending {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        _ = writeln!(f, "Assigned variables");
        for v in self.variable_assigned.iter() {
            _ = write!(f, "{} ", v.index);
        }
        _ = writeln!(f, "\nUnassigned variables");
        for v in self.variable_unassigned.iter() {
            _ = write!(f, "{}", v.index);
        }
        writeln!(f, "")
    }
}

impl Heuristics for Ascending {
    fn new() -> Self {
        Ascending {
            variable_unassigned: BTreeSet::<Variable>::new(),
            variable_assigned: BTreeSet::<Variable>::new(),
            use_bcp: false,
        }
    }

    fn add_parsed_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            self.variable_unassigned.insert(l.variable);
        }
        trace!(target: "heuristics", "Ascending: add clause {c:?}");
    }

    fn add_conflict_clause(&mut self, _c: &Clause) {
        
    }

    fn decide(&mut self) -> Option<Literal> {
        let last = self.variable_unassigned.last();
        if let Some(variable) = last {
            let l = Literal {
                variable: *variable,
                polarity: Polarity::On,
            };
            trace!(target: "heuristics", "Ascending: decide {:?}", l);
            return Some(l);
        }

        return None;
    }

    fn unassign_variable(&mut self, var: Variable) {
        self.variable_assigned.remove(&var);
        self.variable_unassigned.insert(var);
        trace!(target: "heuristics", "Ascending: unassign variable {var:?}");
    }

    fn assign_variable(&mut self, var: Variable) {
        assert!(self.variable_unassigned.remove(&var));
        assert!(self.variable_assigned.insert(var));
        trace!(target: "heuristics", "Ascending: assign variable {var:?}");
    }

    fn satisfy_clause(&mut self, _c: &Clause) {
        
    }

    fn unsatisfy_clause(&mut self, _c: &Clause) {
        
    }

    fn set_use_bcp(&mut self, _use_bcp: bool) {
        self.use_bcp = _use_bcp;
    }

    fn use_bcp(&self) -> bool {
        self.use_bcp
    }
}
