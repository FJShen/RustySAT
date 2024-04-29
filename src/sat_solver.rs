use core::fmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::BTreeSet;
use std::rc::Rc;

// impl of DPLL algorithm
pub mod dpll;

// impl of data structure methods
mod sat_structures;
// pub use sat_structures::get_sample_problem;

////////////////////////////////////////////////////////
// Data structures for the SAT Problem
////////////////////////////////////////////////////////
pub static NULL_VARIABLE: Variable = Variable { index: 0 };
pub static NULL_LITERAL: Literal = Literal {
    variable: NULL_VARIABLE,
    polarity: Polarity::Off,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variable {
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableState {
    Unassigned,
    Assigned,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Literal {
    pub variable: Variable,
    pub polarity: Polarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Clause {
    pub id: u32,
    // pub status: ClauseState,
    pub list_of_literals: Vec<Literal>,
    pub list_of_literal_infos: Vec<Rc<RefCell<LiteralInfo>>>,
    pub watch_literals: [Literal; 2],
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum ClauseState {
    Satisfied,
    Unsatisfiable,
    Unresolved,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiteralInfo {
    pub status: LiteralState,
    pub list_of_clauses: Vec<Rc<RefCell<Clause>>>,
}

pub enum BCPSubstituteWatchLiteralResult {
    FoundSubstitute,
    ClauseIsSAT,
    UnitClauseUnsat,
    ForcedAssignment { l: Literal },
}

#[derive(Debug)]
pub struct Problem {
    // The benefit of using BTreeMap instead of a HashMap: when debug-printing
    // the contents of the former, entries are sorted in a human-friendly way.
    pub list_of_variables: HashMap<Variable, VariableState>,
    pub list_of_literal_infos: HashMap<Literal, Rc<RefCell<LiteralInfo>>>,
    pub list_of_clauses: Vec<Rc<RefCell<Clause>>>,

    // This container contains (reference to) clauses that need to have their
    // ClauseState checked. As an optimization, we do not calculate the
    // ClauseState immediately after a literal is assigned/unassigned/flipped.
    //
    // A clause can be added to this container when we (1) assign a new variable
    // or (2) backtrack a past assignment.
    //
    // One invariant holds: any Clause that "might" be in Unsatisfiable state
    // must be in this container. I.e., if a clause is not in this container, it
    // is certainly not Unsatisfiable.
    // Corollary: This list must be empty when the solver declares SAT.
    pub list_of_clauses_to_check: BTreeSet<Rc<RefCell<Clause>>>,
}

////////////////////////////////////////////////////////
// Data structures for the SAT Solution
////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Assignment {
    pub variable: Variable,
    pub polarity: Polarity,
}

// we have custom impl of Debug
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SolutionStep {
    assignment: Assignment,
    assignment_type: SolutionStepType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SolutionStepType {
    // we picked this variable at will and we haven't flipped its complement
    FreeChoiceFirstTry,
    // we have flipped this assignment's polarity during a backtrack
    FreeChoiceSecondTry,
    // forced due to BCP
    ForcedAtBCP,
    // forced due to it belonging to a unit clause
    ForcedAtInit,
}

#[derive(Debug)]
pub struct SolutionStack {
    // the stack will look like:
    // (FreeChoice,(ForcedChoice,)*)*
    pub stack: Vec<SolutionStep>,
}
