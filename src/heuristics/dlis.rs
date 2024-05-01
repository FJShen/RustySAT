use crate::heuristics::heuristics::*;
use crate::sat_solver::*;
use core::fmt;
use log::trace;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

pub struct DLIS {
    pub literal_frequency: BTreeMap<Literal, u64>,
    pub frequency_literal_assigned: BTreeSet<(u64, Literal)>,
    pub frequency_literal_unassigned: BTreeSet<(u64, Literal)>,
    use_bcp: bool,
}

impl Debug for DLIS {
    // print unassigned literals ranked by score in descending order
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        _ = writeln!(f, "DLIS: print");
        for (i, l) in self.frequency_literal_unassigned.iter().rev() {
            _ = write!(f, "{} {l:?}", i);
        }
        writeln!(f)
    }
}

impl Heuristics for DLIS {
    // creates a new heuristics struct
    fn new() -> Self {
        DLIS {
            literal_frequency: BTreeMap::<Literal, u64>::new(),
            frequency_literal_assigned: BTreeSet::<(u64, Literal)>::new(),
            frequency_literal_unassigned: BTreeSet::<(u64, Literal)>::new(),
            use_bcp: false,
        }
    }

    // add parsed or conflict clauses
    // literal assignment status will remain unchanged
    fn add_parsed_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            *self.literal_frequency.entry(*l).or_insert(0) += 1;
            let key = (*self.literal_frequency.get_mut(l).unwrap()-1, *l);
            self.frequency_literal_unassigned.remove(&key);
            let key = (*self.literal_frequency.get_mut(l).unwrap(), *l);
            self.frequency_literal_unassigned.insert(key);
        }

        let lits = &c.list_of_literals;
        trace!(target: "DLIS", "DLIS: add clause {lits:?}");
    }

    fn add_conflict_clause(&mut self, _c: &Clause) {
        
    }

    // recommend most frequent literal among unassigned clauses but does not
    // move them into the assigned group
    fn decide(&mut self) -> Option<Literal> {
        if let Some(score_literal) = self.frequency_literal_unassigned.last() {
            let score_literal = score_literal.1;
            trace!(target: "DLIS", "DLIS: decide {score_literal:?}");
            return Some(score_literal);
        }

        return None;
    }

    // move a variable from assigned group to unassigned group
    fn unassign_variable(&mut self, var: Variable) {
        let v0 = Literal {
            variable: var,
            polarity: Polarity::Off,
        };
        let v1 = Literal {
            variable: var,
            polarity: Polarity::On,
        };
        for l in [v0, v1] {
            if let Some(frequency) = self.literal_frequency.get(&l) {
                assert!(self.frequency_literal_assigned.remove(&(*frequency, l)));
                assert!(self.frequency_literal_unassigned.insert((*frequency, l)));
            }
        }
        trace!(target: "DLIS", "DLIS: unassign variable {var:?}");
    }

    // move a variable from assigned group to unassigned group
    fn assign_variable(&mut self, var: Variable) {
        trace!(target: "DLIS", "DLIS: assigning variable {var:?}");
        let v0 = Literal {
            variable: var,
            polarity: Polarity::Off,
        };
        let v1 = Literal {
            variable: var,
            polarity: Polarity::On,
        };
        for l in [v0, v1] {
            if let Some(frequency) = self.literal_frequency.get(&l) {
                let tmp = &self.literal_frequency;
                trace!("{tmp:?}");
                let tmp = &self.frequency_literal_unassigned;
                trace!("{tmp:?}");
                let tmp = &self.frequency_literal_assigned;
                trace!("{tmp:?}");
                assert!(self.frequency_literal_unassigned.remove(&(*frequency, l)));
                assert!(self.frequency_literal_assigned.insert((*frequency, l)));
            }
        }
        trace!(target: "DLIS", "DLIS: assigned variable {var:?}");
    }

    // called when a clause is satisfied
    // decrements frequency of all literals in said clause by one
    fn satisfy_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            let key = (*self.literal_frequency.get_mut(l).unwrap(), *l);
            if self.frequency_literal_unassigned.remove(&key) {
                self.frequency_literal_unassigned.insert((key.0-1, key.1));
            }
            else if self.frequency_literal_assigned.remove(&key) {
                self.frequency_literal_assigned.insert((key.0-1, key.1));
            }
            *self.literal_frequency.get_mut(l).unwrap() -= 1;
        }
    }

    // called when a clause is unsatisfied
    // increments frequency of all literals in said clause by one
    fn unsatisfy_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            let key = (*self.literal_frequency.get_mut(l).unwrap(), *l);
            if self.frequency_literal_unassigned.remove(&key) {
                self.frequency_literal_unassigned.insert((key.0+1, key.1));
            }
            else if self.frequency_literal_assigned.remove(&key) {
                self.frequency_literal_assigned.insert((key.0+1, key.1));
            }
            *self.literal_frequency.get_mut(l).unwrap() += 1;
        }
    }

    fn set_use_bcp(&mut self, _use_bcp: bool) {
        self.use_bcp = _use_bcp;
    }

    fn use_bcp(&self) -> bool {
        self.use_bcp
    }
}
