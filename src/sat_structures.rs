use std::cell::RefCell;
use std::rc::Rc;

pub fn get_sample_problem() -> Problem {
    // f = (a + b + c) (a' + b') (b + c')
    // one example solution: a=1, b=0, c=0
    let v_a = Variable{index: 0};
    let v_b = Variable{index: 1};
    let v_c = Variable{index: 2};

    let mut _list_of_variables = vec![
        (v_a, VariableState::Unassigned),
        (v_b, VariableState::Unassigned),
        (v_c, VariableState::Unassigned),
    ];

    let mut _list_of_clauses = vec![
        Rc::new(RefCell::new(Clause{
            list_of_literals: vec![
                Literal{variable: v_a, polarity: true},
                Literal{variable: v_b, polarity: true},
                Literal{variable: v_c, polarity: true},
            ], 
            status: ClauseState::Unresolved
        })),
        Rc::new(RefCell::new(Clause{
            list_of_literals: vec![
                Literal{variable: v_a, polarity: false},
                Literal{variable: v_b, polarity: false}
            ],
            status: ClauseState::Unresolved
        })),
        Rc::new(RefCell::new(Clause{
            list_of_literals: vec![
                Literal{variable: v_b, polarity: true},
                Literal{variable: v_c, polarity: false}
            ],
            status: ClauseState::Unresolved
        })),
    ];

    // To populate the list for LiteralInfo:
    // Create one LiteralInfo for each literal.
    // Then iterate over the clauses: for each literal in a clause, update its
    // entry. 
    let mut _list_of_literal_infos : Vec<LiteralInfo> = vec![];
    for c in &_list_of_clauses {
        for l in &c.borrow_mut().list_of_literals {
            //println!("i have literal {:?}", l)
            let x = _list_of_literal_infos.iter_mut().find(|x| x.literal.variable.index == l.variable.index && x.literal.polarity == l.polarity);
            match x {
                None => {
                    let i = LiteralInfo{
                        list_of_clauses : vec![Rc::clone(c)],
                        literal : l.clone(),
                        status : LiteralState::Unknown,
                    };
                    _list_of_literal_infos.push(i);
                },
                    
                Some(i) => {
                    (*i).list_of_clauses.push(Rc::clone(c));
                },
            };
        }
    }

    // println!("After the loop, list_of_literal_infos is: {:#?}", _list_of_literal_infos);

    Problem{
        list_of_variables : _list_of_variables,
        list_of_literal_infos : _list_of_literal_infos,
        list_of_clauses : _list_of_clauses,
    }
}


#[derive(Debug,Clone,Copy)]
pub struct Variable{ index: u32 }

#[derive(Debug,PartialEq,Eq)]
pub enum VariableState { Unassigned, Assigned }

#[derive(Debug,Clone)]
pub struct Literal { variable: Variable, polarity: bool }

#[derive(Debug,PartialEq,Eq)]
pub enum LiteralState { Unknown, Unsat, Sat }

#[derive(Debug)]
pub struct Clause { 
    list_of_literals: Vec<Literal>,
    status: ClauseState
}

#[derive(Debug,PartialEq,Eq)]
pub enum ClauseState {Satisfied, Unsatisfiable, Unresolved}

#[derive(Debug)]
pub struct LiteralInfo {
    literal: Literal,
    status: LiteralState,
    list_of_clauses : Vec<Rc<RefCell<Clause>>>,
}

#[derive(Debug)]
pub struct Problem {
    list_of_variables: Vec<(Variable, VariableState)>,
    list_of_literal_infos: Vec<LiteralInfo>,
    list_of_clauses: Vec<Rc<RefCell<Clause>>>
}

pub struct Assignment {
    variable: Variable,
    polarity: bool
}

pub enum SolutionStep {
    // we picked this variable at will
    FreeChoice{
        has_tried_other_polarity: bool,
        assignment: Assignment
    }, 

    // forced due to BCP
    ForcedChoice{assignment: Assignment}, 
}

pub struct SolutionStack {
    // the stack will look like: 
    // (FreeChoice,(ForcedChoice,)*)*
    pub stack: Vec<SolutionStep>
}


impl Problem{
    pub fn get_one_unresolved_var(&self) -> Option<Variable>{
        let tuple_result = self.list_of_variables.iter().find(|x| x.1 == VariableState::Unassigned);
        tuple_result.map(|x| x.0)
    }

    pub fn mark_variable_assigned(&mut self, v: Variable) {
        let v_ref = self.list_of_variables.iter_mut().find(|x| x.0.index == v.index);
        if let Some(x) = v_ref {x.1 = VariableState::Assigned;}
    }
}