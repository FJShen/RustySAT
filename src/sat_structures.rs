use tailcall::tailcall;
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Not;

// id for clauses
use global_counter::primitive::exact::CounterU32;

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
            id: CLAUSE_COUNTER.inc(),
            list_of_literals: vec![
                Literal{variable: v_a, polarity: Polarity::On},
                Literal{variable: v_b, polarity: Polarity::On},
                Literal{variable: v_c, polarity: Polarity::On},
            ], 
            status: ClauseState::Unresolved
        })),
        Rc::new(RefCell::new(Clause{
            id: CLAUSE_COUNTER.inc(),
            list_of_literals: vec![
                Literal{variable: v_a, polarity: Polarity::Off},
                Literal{variable: v_b, polarity: Polarity::Off}
            ],
            status: ClauseState::Unresolved
        })),
        Rc::new(RefCell::new(Clause{
            id: CLAUSE_COUNTER.inc(),
            list_of_literals: vec![
                Literal{variable: v_b, polarity: Polarity::On},
                Literal{variable: v_c, polarity: Polarity::Off}
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
        for l in &(**c).borrow().list_of_literals {
            //println!("i have literal {:?}", l)
            let x = _list_of_literal_infos.iter_mut().find(|x| x.literal.variable == l.variable && x.literal.polarity == l.polarity);
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


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Variable{ index: u32 }

#[derive(Debug,PartialEq,Eq)]
pub enum VariableState { Unassigned, Assigned }

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Literal { variable: Variable, polarity: Polarity }

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LiteralState { Unknown, Unsat, Sat }

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum Polarity {Off, On}

impl Not for Polarity {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Polarity::Off => Polarity::On,
            Polarity::On => Polarity::Off,
        }
    }
}

static CLAUSE_COUNTER: CounterU32 = CounterU32::new(0);

#[derive(Debug)]
pub struct Clause { 
    id: u32,
    status: ClauseState,
    list_of_literals: Vec<Literal>,
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


#[derive(Debug,Clone,Copy)]
pub struct Assignment {
    variable: Variable,
    polarity: Polarity
}

#[derive(Debug)]
pub enum SolutionStep {
    // we picked this variable at will
    FreeChoice{
        has_tried_other_polarity: bool,
        assignment: Assignment
    }, 

    // forced due to BCP
    ForcedChoice{assignment: Assignment}, 
}

impl SolutionStep{
    pub fn get_assignment(&self)->Assignment{
        match self {
            Self::FreeChoice { assignment, .. } => assignment.clone(),
            Self::ForcedChoice { assignment } => assignment.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SolutionStack {
    // the stack will look like: 
    // (FreeChoice,(ForcedChoice,)*)*
    pub stack: Vec<SolutionStep>
}

impl SolutionStack{
    pub fn push_free_choice(&mut self, var: Variable, pol: Polarity){
        let step = SolutionStep::FreeChoice { 
            has_tried_other_polarity: false, 
            assignment: Assignment{variable: var, polarity: pol}
        };
        self.stack.push(step);
    }
}

impl Problem{
    /// Returns a variable that is unresolved, and a recommendation for which
    /// polarity to use.
    pub fn get_one_unresolved_var(&self) -> Option<(Variable, Polarity)>{
        let tuple_result = self.list_of_variables.iter().find(|x| x.1 == VariableState::Unassigned);
        
        // for a prototype implementation, alway recommend the "Polarity::On"
        tuple_result.map(|x| (x.0, Polarity::On))
    }

    pub fn mark_variable_assigned(&mut self, v: Variable) {
        let v_ref = self.list_of_variables.iter_mut().find(|x| x.0.index == v.index);
        if let Some(x) = v_ref {x.1 = VariableState::Assigned;}
    }

    pub fn update_literal_info_and_clauses(&mut self, v: Variable, p: Polarity) {
        println!("updating variable {:?} with polarity {:?}", v, p);
        // for both literals (on and off), 
        // - update their state from Unknown to Sat/Unsat
        // - and update their Clauses
        self.list_of_literal_infos
            .iter_mut()
            .filter(|li| li.literal.variable == v)
            .for_each(|li|{
                match li.literal.polarity == p {
                    true => {
                        // this literal is satisfied.
                        assert!(li.status == LiteralState::Unknown, "literal must not be Sat/Unsat");
                        li.status = LiteralState::Sat;
                    },
                    false => {
                        // this literal is unsatisfied.
                        assert!(li.status == LiteralState::Unknown, "literal must not be Sat/Unsat");
                        li.status = LiteralState::Unsat;
                    },
                };
            });

        self.list_of_literal_infos
            .iter()
            .filter(|li| li.literal.variable == v)
            .for_each(|li| {
                li.list_of_clauses.iter().for_each(|rc|{
                    let mut c = (**rc).borrow_mut();
                    // we want to see if this clause becomes satisfied or
                    // unsatisfiable 
                    let clause_literal_states : Vec<LiteralState>
                        = c.list_of_literals.iter().map(|l| {
                            // query the list of literals for their states
                            self.list_of_literal_infos.iter().find(|li| *l == li.literal).map(|li| li.status).unwrap()
                        }).collect();

                    //println!("list of literal states are {:?}", clause_literal_states);

                    // does the clause have at least one Sat? Or is it all
                    // Unsats?
                    if let Some(_) = clause_literal_states.iter().find(|s| **s==LiteralState::Sat) {
                        c.status = ClauseState::Satisfied;
                        println!("Clause {} is satisfied", c.id);
                    } else if let None = clause_literal_states.iter().find(|s| **s==LiteralState::Unknown){
                        println!("Clause {} is unsatisfied", c.id);
                        c.status = ClauseState::Unsatisfiable;
                    }
                })
            });
    }

    // For debug purpose. Does there exist incoherence in the
    // representation? If so, panic!
    pub fn panic_if_incoherent(& self,  solution_stack: &SolutionStack){
        // does the Problem's variable states match with the current Solution?
        solution_stack.stack.iter().for_each(|step| {
            let a = step.get_assignment();
            let sol_v = a.variable;
            // the variable state must be Assigned
            if let None = self.list_of_variables.iter()
                .find(|(v,vs)|sol_v==*v && *vs==VariableState::Assigned){
                    panic!("variable {:?} is on solution stack, but variable state in problem is not assigned", sol_v);
                }
        });

        self.list_of_variables.iter().filter(|(_,vs)|*vs==VariableState::Unassigned)
            .for_each(|(v,vs)|{
                if let Some(_) = solution_stack.stack.iter().find(|step| step.get_assignment().variable==*v){
                    panic!("variable {:?} is unassigned, but it appears on solution stack", (v,vs));
                }
            });

        // does the state of a literal match with the state of variable?
        self.list_of_variables.iter().for_each(|(v, vs)|{
            self.list_of_literal_infos.iter()
                .filter(|li| li.literal.variable == *v)
                .for_each(|li| {
                    if li.status == LiteralState::Unknown && *vs == VariableState::Unassigned{}
                    else if li.status == LiteralState::Sat && *vs == VariableState::Assigned {}
                    else if li.status == LiteralState::Unsat && *vs == VariableState::Assigned {}
                    else {panic!("LiteralInfo {:?} is incoherent with variable {:?}", li, (v,vs));}
                })
        });

        // does the state of a clause match with the state of its literals?
        self.list_of_clauses.iter().map(|rc|rc.borrow()).for_each(|c|{
            let literal_states: Vec<LiteralState> = c.list_of_literals.iter().map(|l|{
                self.list_of_literal_infos.iter().find(|li| *l == li.literal).map(|li| li.status).unwrap()
            }).collect();
            // exist one SAT => clause should be SAT
            if let Some(_) = literal_states.iter().find(|s| **s==LiteralState::Sat) {assert!(c.status==ClauseState::Satisfied);}
            // else if exist one UNKNOWN => clause should be UNRESOLVED
            else if let Some(_) = literal_states.iter().find(|s| **s==LiteralState::Unknown) {assert!(c.status==ClauseState::Unresolved);}
            // otherwise => clause should be UNSAT
            else {assert!(c.status==ClauseState::Unsatisfiable);}
        });
    }
}

// Returns true if all conflicts (if any) were successfully resolved. Returns false if
// the problem is UNSAT (i.e., we have tried both the on- and off-assignment for
// a variable but neither works). Since this is a recursive function, we want to
// be notified if the compiler cannot apply tail-recursion optimization. 
#[tailcall]
pub fn resolve_conflict(problem: &mut Problem, solution_stack: &mut SolutionStack) -> bool {
    // do we even have an unsatiafiable clause? 
    if let None = problem.list_of_clauses.iter()
        .map(|rc|rc.borrow())
        .find(|c|c.status==ClauseState::Unsatisfiable){
            println!("no conflicts in the current solution stack");
            return true;
        };

    // We do have a conflict. Backtrack!
    // Find the last variable that we have not tried both polarities
    let f_step_can_try_other_polarity = |step: &SolutionStep| -> bool{
        if let SolutionStep::FreeChoice { has_tried_other_polarity, .. } = step{
            !has_tried_other_polarity
        } else {false}
    };
    let op_back_track_target = solution_stack.stack.iter().rfind(|step|f_step_can_try_other_polarity(step));

    if op_back_track_target.is_none() {
        println!("cannot find a solution");
        return false;
    } else {
        let mut steps_to_drop: usize = 0;
        solution_stack.stack.iter().rev()
            .take_while(|step|!f_step_can_try_other_polarity(step))
            .for_each(|step|{
                steps_to_drop += 1;

                // un-assign this variable
                let var = match step {
                    SolutionStep::FreeChoice { assignment, .. } => {assignment.variable},
                    SolutionStep::ForcedChoice { assignment } => {assignment.variable},
                };
                println!("Dropping variable {:?}", var);

                // update the list_of_variables
                problem.list_of_variables.iter_mut()
                    .filter(|(v,_)|*v==var)
                    .for_each(|(_,vs)|*vs = VariableState::Unassigned);

                // update the list_of_literal_infos
                problem.list_of_literal_infos.iter_mut()
                    .filter(|li| li.literal.variable == var)
                    .for_each(|li| {
                        assert!(li.status != LiteralState::Unknown);
                        li.status = LiteralState::Unknown;
                    });
            });

        // drop that amount of elements
        let stack_depth = solution_stack.stack.len();
        assert!(stack_depth > steps_to_drop);
        solution_stack.stack.truncate(stack_depth - steps_to_drop);
        
        // reverse the polarity of the last element in the current solution
        // stack, and update list_of_variables and list_of_literal_infos
        let last_step = solution_stack.stack.last_mut().unwrap();
        if let SolutionStep::FreeChoice { has_tried_other_polarity, assignment } = last_step {
            println!("Reversing polarity of assignment {:?}", assignment);

            assignment.polarity = !assignment.polarity;
            *has_tried_other_polarity = true;
            
            problem.list_of_literal_infos.iter_mut()
                .filter(|li| li.literal.variable == assignment.variable)
                .for_each(|li| {
                    assert!(li.status != LiteralState::Unknown);
                    if li.status == LiteralState::Sat{
                        li.status = LiteralState::Unsat;
                    } else {
                        li.status = LiteralState::Sat;
                    }
                });
        } else {panic!("last_step must not be forced choice");}
        
        // update the clause states
        problem.list_of_clauses.iter()
            .for_each(|rc|{
                let mut c = (**rc).borrow_mut();
                // we want to see if this clause becomes satisfied or
                // unsatisfiable 
                let clause_literal_states : Vec<LiteralState>
                    = c.list_of_literals.iter().map(|l| {
                        // query the list of literals for their states
                        problem.list_of_literal_infos.iter().find(|li| *l == li.literal).map(|li| li.status).unwrap()
                    }).collect();

                // does the clause have at least one Sat? Or is it all
                // Unsats?
                if let Some(_) = clause_literal_states.iter().find(|s| **s==LiteralState::Sat) {
                    c.status = ClauseState::Satisfied;
                    println!("Clause {} is satisfied", c.id);
                } else if let None = clause_literal_states.iter().find(|s| **s==LiteralState::Unknown){
                    println!("Clause {} is unsatisfied", c.id);
                    c.status = ClauseState::Unsatisfiable;
                } else {
                    println!("Clause {} is unresolved", c.id);
                    c.status = ClauseState::Unresolved;
                }
            });
        problem.panic_if_incoherent(&solution_stack);

        // recursively call into this function to resolve any new conflicts
        return resolve_conflict(problem, solution_stack);
    }
}