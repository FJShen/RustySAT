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
    pub fn print_sorted(&self) {
        for c in self.counter_literal_unassigned.iter().rev() {
            let pol = match c.1.polarity {
                Polarity::Off  => '-',
                Polarity::On   => '+',
            };
            println!("{}{} : {}", pol, c.1.variable.index, c.0);
        }
    }
}

impl Heuristics for VSIDS {
    fn new() -> Self {
        VSIDS {
            literal_counter: BTreeMap::<Literal, u32>::new(),
            counter_literal_assigned: BTreeSet::<(u32, Literal)>::new(),
            counter_literal_unassigned: BTreeSet::<(u32, Literal)>::new(),
            iteration: 1,
        }
    }
    
    // called when adding parsed or conflict clauses
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
    }

    fn decide(&mut self) -> Option<Literal> {
        self.iteration += 1;
        let highest_literal = self.counter_literal_unassigned.pop_last();

        if highest_literal == None {
            return None;
        }
        
        let score_literal = highest_literal.unwrap();
        self.counter_literal_assigned.insert(score_literal);
        let compl_literal = Literal {
            variable: score_literal.1.variable,
            polarity: if score_literal.1.polarity == Polarity::Off  {Polarity::On} else {Polarity::Off},
        };
        let compl_literal_counter = self.literal_counter.get(&compl_literal).unwrap().clone();
        self.counter_literal_unassigned.remove(&(compl_literal_counter, compl_literal));
        self.counter_literal_assigned.insert((compl_literal_counter, compl_literal));

        return Some(score_literal.1);
    }

    fn unassign_variable(&mut self, var : Variable) {
        let v0 = Literal {variable: var, polarity: Polarity::Off};
        let v1 = Literal {variable: var, polarity: Polarity::On};
        for l in [v0, v1] {
            let counter = self.literal_counter.get(&l).unwrap().clone();
            self.counter_literal_assigned.remove(&(counter, l));
            self.counter_literal_unassigned.insert((counter, l));
        }
    }
}
