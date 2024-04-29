use crate::heuristics::heuristics::*;
use crate::sat_solver::*;
use core::fmt;
use log::trace;
use std::collections::{BTreeSet, BTreeMap};
use std::fmt::Debug;

pub struct VSIDS {
    pub literal_counter : BTreeMap<Literal, u32>,
    pub counter_literal_assigned : BTreeSet<(u32, Literal)>,
    pub counter_literal_unassigned : BTreeSet<(u32, Literal)>,
    pub iteration : u32,
    use_bcp: bool
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
            literal_counter: BTreeMap::<Literal, u32>::new(),
            counter_literal_assigned: BTreeSet::<(u32, Literal)>::new(),
            counter_literal_unassigned: BTreeSet::<(u32, Literal)>::new(),
            iteration: 1,
            use_bcp: false
        }
    }

    // add parsed or conflict clauses and add score of contained literals
    // literal assignment status will remain unchanged
    fn add_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            let counter_old = self.literal_counter.entry(*l).or_insert(0).clone();
            let counter_new = counter_old + self.iteration;
            *self.literal_counter.get_mut(l).unwrap() = counter_new;
            // update score in assigned set if var assigned
            if self.counter_literal_assigned.remove(&(counter_old, *l)) {
                assert!(self.counter_literal_assigned.insert((counter_new, *l)));
            }
            // add or update score in unassigned set if var unregistered or unassigned
            else {
                self.counter_literal_unassigned.remove(&(counter_old, *l));
                assert!(self.counter_literal_unassigned.insert((counter_new, *l)));
            }
        }

        let lits = &c.list_of_literals;
        trace!(target: "vsids", "VSIDS: add clause {lits:?}");
    }

    // recommend highest ranked literal but with inverted polarity
    fn decide(&mut self) -> Option<Literal> {
        self.iteration += 1;

        if let Some(score_literal) = self.counter_literal_unassigned.pop_last() {
            assert!(self.counter_literal_assigned.insert(score_literal));
            let score_literal = score_literal.1;
            let compl_literal = Literal {
                variable: score_literal.variable,
                polarity: if score_literal.polarity == Polarity::Off {
                    Polarity::On
                } else {
                    Polarity::Off
                },
            };
            if let Some(compl_literal_counter) = self.literal_counter.get(&compl_literal) {
                assert!(self
                    .counter_literal_unassigned
                    .remove(&(*compl_literal_counter, compl_literal)));
                assert!(self
                    .counter_literal_assigned
                    .insert((*compl_literal_counter, compl_literal)));
            }

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

    fn set_use_bcp(&mut self, _use_bcp: bool) {
        self.use_bcp = _use_bcp;
    }

    fn use_bcp(&self) -> bool {
        self.use_bcp
    }
}
