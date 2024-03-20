use super::*;
use tailcall::tailcall;

// If the problem is UNSAT, we will not return anything but throw an exception.
pub fn dpll(mut p: Problem) -> SolutionStack {
    let mut solution = SolutionStack { stack: vec![] };

    // Baseline DPLL
    // 1. Pick a variable to assign
    // 1.1 Pick a variable
    // 1.2 Pick a polarity
    // 2. Update the problem
    // 2.1 Update list of variables: mark one as Assigned
    // 2.2 Update list of literals: mark one literal as Sat, and its complement as
    // Unsat
    // 2.3 Update the state of each Clause associated with the literals touched in
    // the last step
    // 3. Resolve conflicts if any clause is unsatisfiable.
    // 4. Repeat
    // Resolve all variables before we return a solution

    while let Some((var, pol)) = get_one_unresolved_var(&p) {
        solution.push_free_choice_first_try(var, pol);
        println!("dpll picking variable {:?}", var);
        println!("solution stack: {:?}", solution);

        mark_variable_assigned(&mut p, var);
        update_literal_info_and_clauses(&mut p, var, pol);

        // sanity check
        panic_if_incoherent(&p, &solution);

        let resolved_all_conflicts = resolve_conflict(&mut p, &mut solution);
        if !resolved_all_conflicts {
            panic!("UNSAT");
        }
    }

    println!("all variables are assigned");

    solution
}

////////////////////////////////////////////////////////
// Routines for the SAT Algorithm
////////////////////////////////////////////////////////

/// Returns a variable that is unresolved, and a recommendation for which
/// polarity to use. If all variables have been resolved, returns None.  
pub fn get_one_unresolved_var(problem: &Problem) -> Option<(Variable, Polarity)> {
    let tuple_result = problem
        .list_of_variables
        .iter()
        .find(|(v, vs)| **vs == VariableState::Unassigned);

    // for a prototype implementation, alway recommend the "Polarity::On"
    tuple_result.map(|(v, vs)| (*v, Polarity::On))
}

pub fn mark_variable_assigned(problem: &mut Problem, v: Variable) {
    // will panic if v is not in list_of_variables
    let vs = problem.list_of_variables.get_mut(&v).unwrap();
    *vs = VariableState::Assigned;
}

pub fn update_literal_info_and_clauses(problem: &mut Problem, v: Variable, p: Polarity) {
    // for both literals (on and off),
    // - update their state from Unknown to Sat/Unsat
    // - and update their Clauses' status

    // same polarity: becomes Satisfied
    if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
        variable: v,
        polarity: p,
    }) {
        assert!(
            li.status == LiteralState::Unknown,
            "literal must not be Sat/Unsat"
        );
        li.status = LiteralState::Sat;
    }
    // opposite polarity: becomes Unsat
    if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
        variable: v,
        polarity: !p,
    }) {
        assert!(
            li.status == LiteralState::Unknown,
            "literal must not be Sat/Unsat"
        );
        li.status = LiteralState::Unsat;
    }

    // For the SAT literal, it has the potential of changing a clause from
    // Unresolved to Satisfied.
    if let Some(li) = problem.list_of_literal_infos.get(&Literal {
        variable: v,
        polarity: p,
    }) {
        li.list_of_clauses.iter().for_each(|rc| {
            let mut c = (**rc).borrow_mut();
            if c.status == ClauseState::Unresolved {
                c.status = ClauseState::Satisfied;
                println!("Clause {} is satisfied", c.id);
            }
        });
    }

    // For the UNSAT literal, it has the potential of changing a clause from
    // Unresolved to Unsatisfiable.
    if let Some(li) = problem.list_of_literal_infos.get(&Literal {
        variable: v,
        polarity: !p,
    }) {
        li.list_of_clauses.iter().for_each(|rc| {
            let mut c = (**rc).borrow_mut();
            if c.status == ClauseState::Unresolved {
                // are all literals of this clause UNSAT?
                if !c
                    .list_of_literals
                    .iter()
                    .map(|l| problem.list_of_literal_infos[l].status)
                    .any(|ls| ls == LiteralState::Sat || ls == LiteralState::Unknown)
                {
                    c.status = ClauseState::Unsatisfiable;
                    println!("Clause {} is unsatisfiable", c.id);
                }
            }
        });
    }
}

/// Sanity check solely for debug purpose. Does there exist incoherence in the
/// representation? If so, panic!
pub fn panic_if_incoherent(problem: &Problem, solution_stack: &SolutionStack) {
    // does the Problem's variable states match with the current Solution?
    solution_stack.stack.iter().for_each(|step| {
        let a = step.assignment;
        let sol_v = a.variable;
        // the variable state must be Assigned
        if problem.list_of_variables[&sol_v] != VariableState::Assigned {
            panic!(
                "variable {:?} is on solution stack, but variable state in problem is not assigned",
                sol_v
            );
        }
    });

    problem
        .list_of_variables
        .iter()
        .filter(|(_, vs)| **vs == VariableState::Unassigned)
        .for_each(|(v, vs)| {
            if solution_stack
                .stack
                .iter()
                .any(|step| step.assignment.variable == *v)
            {
                panic!(
                    "variable {:?} is unassigned, but it appears on solution stack",
                    (v, vs)
                );
            }
        });

    // does the state of a literal match with the state of variable?
    problem.list_of_variables.iter().for_each(|(v, vs)| {
        if let Some(li) = problem.list_of_literal_infos.get(&Literal {
            variable: *v,
            polarity: Polarity::On,
        }) {
            if li.status == LiteralState::Unknown && *vs == VariableState::Unassigned {
            } else if li.status == LiteralState::Sat && *vs == VariableState::Assigned {
            } else if li.status == LiteralState::Unsat && *vs == VariableState::Assigned {
            } else {
                panic!(
                    "LiteralInfo {:?} is incoherent with variable {:?}",
                    li,
                    (v, vs)
                );
            }
        }
        if let Some(li) = problem.list_of_literal_infos.get(&Literal {
            variable: *v,
            polarity: Polarity::Off,
        }) {
            if li.status == LiteralState::Unknown && *vs == VariableState::Unassigned {
            } else if li.status == LiteralState::Sat && *vs == VariableState::Assigned {
            } else if li.status == LiteralState::Unsat && *vs == VariableState::Assigned {
            } else {
                panic!(
                    "LiteralInfo {:?} is incoherent with variable {:?}",
                    li,
                    (v, vs)
                );
            }
        }
    });

    // does the state of a clause match with the state of its literals?
    problem
        .list_of_clauses
        .iter()
        .map(|rc| rc.borrow())
        .for_each(|c| {
            let literal_states: Vec<LiteralState> = c
                .list_of_literals
                .iter()
                .map(|l| problem.list_of_literal_infos[l].status)
                .collect();
            // exist one SAT => clause should be SAT
            if literal_states.iter().any(|s| *s == LiteralState::Sat) {
                assert!(c.status == ClauseState::Satisfied);
            }
            // else if exist one UNKNOWN => clause should be UNRESOLVED
            else if literal_states.iter().any(|s| *s == LiteralState::Unknown) {
                assert!(c.status == ClauseState::Unresolved);
            }
            // otherwise => clause should be UNSAT
            else {
                assert!(c.status == ClauseState::Unsatisfiable);
            }
        });
}

/// Returns true if all conflicts (if any) were successfully resolved. Returns false if
/// the problem is UNSAT (i.e., we have tried both the on- and off-assignment for
/// a variable but neither works). Since this is a recursive function, we want to
/// be notified if the compiler cannot apply tail-recursion optimization.
#[tailcall]
pub fn resolve_conflict(problem: &mut Problem, solution_stack: &mut SolutionStack) -> bool {
    // do we even have an unsatiafiable clause?
    if !problem
        .list_of_clauses
        .iter()
        .map(|rc| rc.borrow())
        .any(|c| c.status == ClauseState::Unsatisfiable)
    {
        // println!("no conflicts in the current solution stack");
        return true;
    };

    // We do have a conflict. Backtrack!
    // Find the last variable that we have not tried both polarities
    println!("Trying to resolve conflict.");
    let f_step_can_try_other_polarity = |step: &SolutionStep| -> bool {
        matches!(step.assignment_type, SolutionStepType::FreeChoiceFirstTry)
    };
    let op_back_track_target = solution_stack
        .stack
        .iter()
        .rfind(|step| f_step_can_try_other_polarity(step));

    if op_back_track_target.is_none() {
        println!("cannot find a solution");
        return false;
    } else {
        let mut steps_to_drop: usize = 0;
        solution_stack
            .stack
            .iter()
            .rev() // younger steps are at the tail
            .take_while(|step| !f_step_can_try_other_polarity(step))
            .for_each(|step| {
                steps_to_drop += 1;

                // un-assign this variable
                let var = step.assignment.variable;
                println!("Dropping variable {:?}", var);

                // update the list_of_variables
                *problem.list_of_variables.get_mut(&var).unwrap() = VariableState::Unassigned;

                // update the list_of_literal_infos
                if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
                    variable: var,
                    polarity: Polarity::On,
                }) {
                    assert!(li.status != LiteralState::Unknown);
                    li.status = LiteralState::Unknown;
                }
                if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
                    variable: var,
                    polarity: Polarity::Off,
                }) {
                    assert!(li.status != LiteralState::Unknown);
                    li.status = LiteralState::Unknown;
                }
            });

        // drop that amount of elements
        let stack_depth = solution_stack.stack.len();
        assert!(stack_depth > steps_to_drop);
        solution_stack.stack.truncate(stack_depth - steps_to_drop);

        // Reverse the polarity of the last element in the current solution
        // stack, and update list_of_literal_infos. list_of_variables need not
        // be modified.
        let last_step = solution_stack.stack.last_mut().unwrap();
        assert!(last_step.assignment_type == SolutionStepType::FreeChoiceFirstTry);
        println!("Flipping variable {:?}", last_step.assignment.variable);

        last_step.assignment.polarity = !last_step.assignment.polarity;
        last_step.assignment_type = SolutionStepType::FreeChoiceSecondTry;

        let var = &last_step.assignment.variable;
        let new_pol = &last_step.assignment.polarity;
        if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
            variable: *var,
            polarity: *new_pol,
        }) {
            assert!(li.status != LiteralState::Unknown);
            li.status = LiteralState::Sat;
        }
        if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
            variable: *var,
            polarity: !*new_pol,
        }) {
            assert!(li.status != LiteralState::Unknown);
            li.status = LiteralState::Unsat;
        }

        // update the clause states
        problem.list_of_clauses.iter().for_each(|rc| {
            let mut c = (**rc).borrow_mut();
            // we want to see if this clause becomes satisfied or
            // unsatisfiable
            let clause_literal_states: Vec<LiteralState> = c
                .list_of_literals
                .iter()
                .map(|l| {
                    // query the list of literals for their states
                    problem.list_of_literal_infos[l].status
                })
                .collect();

            // does the clause have at least one Sat? Or is it all
            // Unsats?
            let mut new_status = ClauseState::Satisfied;
            if clause_literal_states
                .iter()
                .any(|s| *s == LiteralState::Sat)
            {
                new_status = ClauseState::Satisfied;
            } else if !clause_literal_states
                .iter()
                .any(|s| *s == LiteralState::Unknown)
            {
                new_status = ClauseState::Unsatisfiable;
            } else {
                new_status = ClauseState::Unresolved;
            }

            if new_status != c.status {
                c.status = new_status;
                let s = match c.status {
                    ClauseState::Satisfied => "satisfied",
                    ClauseState::Unsatisfiable => "unsatisfiable",
                    ClauseState::Unresolved => "unresolved",
                };
                println!("Clause {} becomes {}", c.id, s);
            }
        });
        println!("solution stack: {:?}", solution_stack);
        panic_if_incoherent(problem, solution_stack);

        // recursively call into this function to resolve any new conflicts
        resolve_conflict(problem, solution_stack)
    }
}
