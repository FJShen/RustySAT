use std::collections::BTreeSet;
use crate::sat_solver::*;
use crate::heuristics::heuristics::*;
use log::trace;

pub struct Ascending {
    pub variable_unassigned: BTreeSet<Variable>,
    pub variable_assigned: BTreeSet<Variable>,
}

impl Ascending {
    pub fn print(&self) {
        println!("Assigned variables");
        for v in self.variable_assigned.iter() {
            print!("{} ", v.index);
        }
        println!();
        println!("Unassigned variables");
        for v in self.variable_unassigned.iter() {
            print!("{}", v.index);
        }
        println!();
    }
}

impl Heuristics for Ascending {
    fn new() -> Self {
        Ascending {
            variable_unassigned: BTreeSet::<Variable>::new(),
            variable_assigned: BTreeSet::<Variable>::new(),
        }
    }

    fn add_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            self.variable_unassigned.insert(l.variable);
        }
        trace!(target: "heuristics", "Ascending: add clause {c:?}");
    }

    fn decide(&mut self) -> Option<Literal> {
        let last = self.variable_unassigned.pop_last();
        if let Some(variable) = last {
            self.variable_assigned.insert(variable);
            let l = Literal {variable: variable, polarity: Polarity::On};
            trace!(target: "heuristics", "Ascending: decide {:?}", l);
            return Some(l);
        }

        return None;
    }

    fn unassign_variable(&mut self, var : Variable) {
        self.variable_assigned.remove(&var);
        self.variable_unassigned.insert(var);
        trace!(target: "heuristics", "Ascending: unassign variable {var:?}");
    }

    fn assign_variable(&mut self, var : Variable) {
        self.variable_unassigned.remove(&var);
        self.variable_assigned.insert(var);
        trace!(target: "heuristics", "Ascending: assign variable {var:?}");
    }
}
