use std::cell::RefCell;

pub fn get_sample_problem() -> Problem {
    // f = (a + b + c) (a' + b') (b + c')
    // one solution: a=1, b=0, c=0
    let v_a = Variable{index: 0};
    let v_b = Variable{index: 1};
    let v_c = Variable{index: 2};

    let mut _list_of_variables = vec![
        (v_a, VariableState::Unassigned),
        (v_b, VariableState::Unassigned),
        (v_c, VariableState::Unassigned),
    ];

    let mut _list_of_clauses = vec![
        RefCell::new(Clause{
            list_of_literals: vec![
                Literal{variable: v_a, polarity: true},
                Literal{variable: v_b, polarity: true},
                Literal{variable: v_c, polarity: true},
            ], 
            status: ClauseState::Unresolved
        }),
        RefCell::new(Clause{
            list_of_literals: vec![
                Literal{variable: v_a, polarity: false},
                Literal{variable: v_b, polarity: false}
            ],
            status: ClauseState::Unresolved
        }),
        RefCell::new(Clause{
            list_of_literals: vec![
                Literal{variable: v_b, polarity: true},
                Literal{variable: v_c, polarity: false}
            ],
            status: ClauseState::Unresolved
        }),
    ];

    // to populate the list for LiteralInfo:
    // Iterate over the clauses.
    let mut _list_of_literal_infos = vec![];
    for c in &_list_of_clauses {
        println!("{:?}", c);
    }

    Problem{
        list_of_variables : _list_of_variables,
        list_of_literal_infos : _list_of_literal_infos,
        list_of_clauses : _list_of_clauses,
    }
}


#[derive(Debug,Clone,Copy)]
pub struct Variable{ index: u32 }

#[derive(Debug)]
pub enum VariableState { Unassigned, Assigned }

#[derive(Debug)]
pub struct Literal { variable: Variable, polarity: bool }

#[derive(Debug)]
pub enum LiteralState { Unknown, Unsat, Sat }

#[derive(Debug)]
pub struct Clause { 
    list_of_literals: Vec<Literal>,
    status: ClauseState
}

#[derive(Debug)]
pub enum ClauseState {Satisfied, Unsatisfiable, Unresolved}

#[derive(Debug)]
pub struct LiteralInfo {
    list_of_clauses : Vec<RefCell<Clause>>,
    literal: Literal,
    status: LiteralState
}

#[derive(Debug)]
pub struct Problem {
    list_of_variables: Vec<(Variable, VariableState)>,
    list_of_literal_infos: Vec<LiteralInfo>,
    list_of_clauses: Vec<RefCell<Clause>>
}

pub struct Assignment {
    variable: Variable,
    polarity: bool
}

pub enum SolutionStep {
    // we picked this variable at will
    FreeChoice{assignment: Assignment}, 

    // forced due to BCP
    ForcedChoice{assignment: Assignment}, 
}

pub struct SolutionStack {
    // the stack will look like: 
    // (FreeChoice,(ForcedChoice,)*)*
    stack: Vec<SolutionStep>
}
