use std::{borrow::Borrow, collections::{BTreeMap, BTreeSet}};
use crate::sat_solver::*;

pub struct VSIDS {
    pub literal_to_counter : BTreeMap<Literal, u32>,
    pub counter_to_literal : BTreeSet<(u32, Literal)>,
    pub iteration : u32,
}

impl VSIDS {
    pub const fn new() -> Self {
        VSIDS {
            literal_to_counter: BTreeMap::<Literal, u32>::new(),
            counter_to_literal: BTreeSet::<(u32, Literal)>::new(),
            iteration: 1,
        }
    }

    // called when adding parsed or conflict clauses
    pub fn add_clause(&mut self, c: &Clause) {
        for l in c.list_of_literals.iter() {
            *self.literal_to_counter
                .entry(*l)
                .or_insert(0) += self.iteration;
            let counter = self.literal_to_counter[l];
            self.counter_to_literal
                .insert((counter, *l));
            self.counter_to_literal
                .remove(&(counter-1, *l));
        }
    }

    pub fn decide(&mut self) -> Option<Literal> {
        self.iteration += 1;
        let last = self.counter_to_literal.pop_last();
        if last.is_some() {
            self.literal_to_counter.remove(last.unwrap().1.borrow());
        }
        match last {
            Some(last) => Some(last.1),
            None => None,
        }
    }

    pub fn print_sort_by_literals(&self) {
        for c in self.literal_to_counter.iter() {
            let pol = match c.0.polarity {
                Polarity::Off  => '-',
                Polarity::On   => '+',
            };
            println!("{}{} : {}", pol, c.0.variable.index, c.1);
        }
    }

    pub fn print_sort_by_counters(&self) {
        for c in self.counter_to_literal.iter().rev() {
            let pol = match c.1.polarity {
                Polarity::Off  => '-',
                Polarity::On   => '+',
            };
            println!("{}{} : {}", pol, c.1.variable.index, c.0);
        }
    }
}