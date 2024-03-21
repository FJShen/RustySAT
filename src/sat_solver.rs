use core::fmt;
use global_counter::primitive::exact::CounterU32;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

// impl of DPLL algorithm
pub mod dpll;

// impl of data structure methods
mod sat_structures;
pub use sat_structures::get_sample_problem;

////////////////////////////////////////////////////////
// Data structures for the SAT Problem
////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variable {
    index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableState {
    Unassigned,
    Assigned,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Literal {
    variable: Variable,
    polarity: Polarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralState {
    Unknown,
    Unsat,
    Sat,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum Polarity {
    Off,
    On,
}

static CLAUSE_COUNTER: CounterU32 = CounterU32::new(0);

#[derive(Debug,PartialEq, Eq, PartialOrd, Ord)]
pub struct Clause {
    id: u32,
    status: ClauseState,
    list_of_literals: Vec<Literal>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum ClauseState {
    Satisfied,
    Unsatisfiable,
    Unresolved,
}

#[derive(Debug)]
pub struct LiteralInfo {
    status: LiteralState,
    list_of_clauses: Vec<Rc<RefCell<Clause>>>,
}

#[derive(Debug)]
pub struct Problem {
    // The benefit of using BTreeMap instead of a BTreeMap: when debug-printing
    // the contents of the former, entries are sorted in a human-friendly way.
    list_of_variables: BTreeMap<Variable, VariableState>,
    list_of_literal_infos: BTreeMap<Literal, LiteralInfo>,
    list_of_clauses: Vec<Rc<RefCell<Clause>>>,
}

////////////////////////////////////////////////////////
// Data structures for the SAT Solution
////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct Assignment {
    variable: Variable,
    polarity: Polarity,
}

// we have custom impl of Debug
#[derive(Clone, Copy)]
pub struct SolutionStep {
    assignment: Assignment,
    assignment_type: SolutionStepType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolutionStepType {
    // we picked this variable at will and we haven't flipped its complement
    FreeChoiceFirstTry,
    // we have flipped this assignment's polarity during a backtrack
    FreeChoiceSecondTry,
    // forced due to BCP
    ForcedChoice,
}

#[derive(Debug)]
pub struct SolutionStack {
    // the stack will look like:
    // (FreeChoice,(ForcedChoice,)*)*
    pub stack: Vec<SolutionStep>,
}
