use std::collections::{BTreeMap, BTreeSet};
use crate::sat_solver::*;
use crate::heuristics::heuristics::*;

pub struct VSIDS {
    pub literal_counter : BTreeMap<Literal, u32>,
    pub counter_literal_assigned : BTreeSet<(u32, Literal)>,
    pub counter_literal_unassigned : BTreeSet<(u32, Literal)>,
    pub iteration : u32,
}

impl VSIDS {
    // print unassigned literals ranked by score in descending order
    pub fn print_sorted(&self) {
        println!("VSIDS: print sorted");
        for (i, l) in self.counter_literal_unassigned.iter().rev() {
            println!("{} {l:?}", i)
        }
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
        }
    }
    
    // add parsed or conflict clauses and bump score of contained literals
    fn add_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            *self.literal_counter
                .entry(*l)
                .or_insert(0) += self.iteration;
            let counter = self.literal_counter[l];
            self.counter_literal_unassigned
                .insert((counter, *l));
            self.counter_literal_unassigned
                .remove(&(counter-self.iteration, *l));
        }

        let lits = &c.list_of_literals;
        trace!(target: "vsids", "VSIDS: add clause {lits:?}");
    }

    // recommend highest ranked literal but with inverted polarity
    fn decide(&mut self) -> Option<Literal> {
        self.iteration += 1;
        let highest_literal = self.counter_literal_unassigned.pop_last();

        if highest_literal == None {
            return None;
        }
        
        let score_literal = highest_literal.unwrap();
        self.counter_literal_assigned.insert(score_literal);
        let score_literal = score_literal.1;
        let compl_literal = Literal {
            variable: score_literal.variable,
            polarity: if score_literal.polarity == Polarity::Off  {Polarity::On} else {Polarity::Off},
        };
        let compl_literal_counter = self.literal_counter.get(&compl_literal).unwrap().clone();
        self.counter_literal_unassigned.remove(&(compl_literal_counter, compl_literal));
        self.counter_literal_assigned.insert((compl_literal_counter, compl_literal));

        trace!(target: "vsids", "VSIDS: decide {score_literal:?}");
        return Some(score_literal);
    }

    // move a variable from assigned group to unassigned group
    fn unassign_variable(&mut self, var : Variable) {
        let v0 = Literal {variable: var, polarity: Polarity::Off};
        let v1 = Literal {variable: var, polarity: Polarity::On};
        for l in [v0, v1] {
            let counter = self.literal_counter.get(&l).unwrap().clone();
            self.counter_literal_assigned.remove(&(counter, l));
            self.counter_literal_unassigned.insert((counter, l));
        }
        trace!(target: "vsids", "VSIDS: unassign variable {var:?}");
    }
}
