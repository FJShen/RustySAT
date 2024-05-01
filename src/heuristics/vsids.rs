use crate::heuristics::heuristics::*;
use crate::sat_solver::*;
use core::fmt;
use log::trace;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

pub struct VSIDS {
    pub literal_counter: BTreeMap<Literal, u64>,
    pub counter_literal_assigned: BTreeSet<(u64, Literal)>,
    pub counter_literal_unassigned: BTreeSet<(u64, Literal)>,
    pub iteration: u64,
    use_bcp: bool,
}

impl Debug for VSIDS {
    // print unassigned literals ranked by score in descending order
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        _ = writeln!(f, "VSIDS: print");
        for (i, l) in self.counter_literal_unassigned.iter().rev() {
            _ = write!(f, "{} {l:?}", i);
        }
        writeln!(f)
    }
}

impl Heuristics for VSIDS {
    // creates a new heuristics struct
    fn new() -> Self {
        VSIDS {
            literal_counter: BTreeMap::<Literal, u64>::new(),
            counter_literal_assigned: BTreeSet::<(u64, Literal)>::new(),
            counter_literal_unassigned: BTreeSet::<(u64, Literal)>::new(),
            iteration: 1,
            use_bcp: false,
        }
    }

    // add parsed or conflict clauses and add score of contained literals
    // literal assignment status will remain unchanged
    fn add_parsed_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            let counter_old = self.literal_counter.entry(*l).or_insert(0).clone();
            let counter_new = counter_old + self.iteration;
            *self.literal_counter.get_mut(l).unwrap() = counter_new;
            self.counter_literal_unassigned.remove(&(counter_old, *l));
            assert!(self.counter_literal_unassigned.insert((counter_new, *l)));
        }

        let lits = &c.list_of_literals;
        trace!(target: "vsids", "VSIDS: add clause {lits:?}");
    }

    fn add_conflict_clause(&mut self, c: &Clause) {
        self.add_parsed_clause(c);
    }

    // recommend highest ranked literal but with inverted polarity
    fn decide(&mut self) -> Option<Literal> {
        self.iteration += 1;
        if let Some(score_literal) = self.counter_literal_unassigned.last() {
            let score_literal = score_literal.1;
            trace!(target: "vsids", "VSIDS: decide {score_literal:?}");
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
            if let Some(counter) = self.literal_counter.get(&l) {
                assert!(self.counter_literal_assigned.remove(&(*counter, l)));
                assert!(self.counter_literal_unassigned.insert((*counter, l)));
            }
        }
        trace!(target: "vsids", "VSIDS: unassign variable {var:?}");
    }

    // move a variable from assigned group to unassigned group
    fn assign_variable(&mut self, var: Variable) {
        let v0 = Literal {
            variable: var,
            polarity: Polarity::Off,
        };
        let v1 = Literal {
            variable: var,
            polarity: Polarity::On,
        };
        for l in [v0, v1] {
            if let Some(counter) = self.literal_counter.get(&l) {
                assert!(self.counter_literal_unassigned.remove(&(*counter, l)));
                assert!(self.counter_literal_assigned.insert((*counter, l)));
            }
        }
        trace!(target: "vsids", "VSIDS: assign variable {var:?}");
    }

    fn satisfy_clause(&mut self, c: &Clause) {
        
    }

    fn unsatisfy_clause(&mut self, c: &Clause) {
        
    }

    fn set_use_bcp(&mut self, _use_bcp: bool) {
        self.use_bcp = _use_bcp;
    }

    fn use_bcp(&self) -> bool {
        self.use_bcp
    }
}
