use super::*;
use std::{cell::Ref, collections::BTreeSet};
use tailcall::tailcall;
use log::{trace,info};

// If the problem is UNSAT, we will return None
pub fn dpll(mut p: Problem) -> Option<SolutionStack> {
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
        trace!(target: "dpll", "picking variable {:?}", var);
        trace!(target: "dpll", "solution stack: {:?}", solution);

        mark_variable_assigned(&mut p, var);
        update_literal_info(&mut p, var, pol);

        // sanity check
        // panic_if_incoherent(&p, &solution);

        let resolved_all_conflicts = udpate_clause_state_and_resolve_conflict(&mut p, &mut solution);
        if !resolved_all_conflicts {
            return None;
        }
    }

    info!(target: "dpll", "all variables are assigned");

    Some(solution)
}

////////////////////////////////////////////////////////
// Routines for the SAT Algorithm
////////////////////////////////////////////////////////

/// Returns a variable that is unresolved, and a recommendation for which
/// polarity to use. If all variables have been resolved, returns None.  
pub fn get_one_unresolved_var(problem: &Problem) -> Option<(Variable, Polarity)> {
    // heuristic: pick an unassigned variable that appears in the most amount
    // of clauses.
    let tuple_result: Option<(Variable, usize, usize)> = problem
        .list_of_variables
        .iter()
        .filter(|(v, vs)| **vs == VariableState::Unassigned)
        .map(|(v, vs)| {
            let mut on_count: usize = 0;
            let mut off_count: usize = 0;
            if let Some(li) = problem.list_of_literal_infos.get(
                &Literal { variable: *v, polarity: Polarity::On }){
                on_count = li.list_of_clauses.len();
            }
            if let Some(li) = problem.list_of_literal_infos.get(
                &Literal { variable: *v, polarity: Polarity::Off }){
                off_count = li.list_of_clauses.len();
            }
            (*v, on_count, off_count)
        })
        .max_by_key(|(v, on_count, off_count)| on_count + off_count);

    if let Some((v, on_count, off_count)) = tuple_result{
        if on_count > off_count {
            return Some((v, Polarity::On));
        } else {
            return Some((v, Polarity::Off));
        }  
    } else {return None;}

}

pub fn mark_variable_assigned(problem: &mut Problem, v: Variable) {
    // will panic if v is not in list_of_variables
    let vs = problem.list_of_variables.get_mut(&v).unwrap();
    *vs = VariableState::Assigned;
}

/// Returns a list of clauses to be updated (need to recalculate clause state).
pub fn update_literal_info(problem: &mut Problem, v: Variable, p: Polarity) {
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

    // For the SAT literal, it guarantee to make a clause Satisfied.
    if let Some(li) = problem.list_of_literal_infos.get(&Literal {
        variable: v,
        polarity: p,
    }) {
        li.list_of_clauses.iter().for_each(|rc| {
            rc.borrow_mut().status = ClauseState::Satisfied;
        });
    }

    // For the UNSAT literal, it has the potential of changing a clause's state.
    if let Some(li) = problem.list_of_literal_infos.get(&Literal {
        variable: v,
        polarity: !p,
    }) {
        li.list_of_clauses.iter().for_each(|rc| {
            problem.list_of_clauses_to_update.insert(Rc::clone(rc));
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
            assert!(c.recalculate_clause_state(problem) == c.status);
        });
}

/// Returns true if all conflicts (if any) were successfully resolved. Returns false if
/// the problem is UNSAT (i.e., we have tried both the on- and off-assignment for
/// a variable but neither works). Since this is a recursive function, we want to
/// be notified if the compiler cannot apply tail-recursion optimization.
#[tailcall]
pub fn udpate_clause_state_and_resolve_conflict(
    problem: &mut Problem, 
    solution_stack: &mut SolutionStack
) -> bool {
    // do we even have an unsatiafiable clause?
    while let Some(rc) = problem.list_of_clauses_to_update.pop_first() {
        let mut c = rc.borrow_mut();
        trace!(target: "backtrack", "Examining clause {}", c.id);

        // we want to see if this clause becomes satisfied or
        // unsatisfiable
        let new_status = c.recalculate_clause_state(problem);

        if new_status != c.status {
            c.status = new_status;
            let s = match c.status {
                ClauseState::Satisfied => "satisfied",
                ClauseState::Unsatisfiable => "unsatisfiable",
                ClauseState::Unresolved => "unresolved",
            };
            trace!(target: "backtrack", "Clause {} becomes {}", c.id, s);
        }

        if c.status == ClauseState::Unsatisfiable {
            // One unsat clause is enough, we have to keep backtracking
            // Carry-on the remaining list of clauses to update to the
            // following recursive call. Also put rc back. 
            drop(c);
            problem.list_of_clauses_to_update.insert(rc);
            break;
        }
    }

    if problem.list_of_clauses_to_update.len() == 0 {
        trace!(target: "backtrack", "All conflicts resolved.");
        return true;
    }

    // We do have a conflict. Backtrack!
    // Find the last variable that we have not tried both polarities
    trace!(target: "backtrack", "Trying to resolve conflict.");
    let f_step_can_try_other_polarity = |step: &SolutionStep| -> bool {
        matches!(step.assignment_type, SolutionStepType::FreeChoiceFirstTry)
    };
    let op_back_track_target = solution_stack
        .stack
        .iter()
        .rfind(|step| f_step_can_try_other_polarity(step));

    if op_back_track_target.is_none() {
        trace!(target: "backtrack", "cannot find a solution");
        return false;
    } else {
        // 1. Un-mark the literals and variables touched any step that need to
        //    be dropped
        // 2. Drop those steps from the solution_stack
        // 3. Flip the first step that we can flip, mark its literal/variable
        // 4. Update (only) the clauses that are affected by steps 1 and 3

        // Updated in step 1, used in step 2
        let mut steps_to_drop: usize = 0;

        // Updated in step 1 and 3, used in step 4
        // Both Rc and RefCell are compared by the values they contain, so
        // different instances of Rc that point to the same RefCell<Clause> end
        // up being "equal", so we avoid redundancy in the set.

        solution_stack
            .stack
            .iter()
            .rev() // younger steps are at the tail
            .take_while(|step| !f_step_can_try_other_polarity(step))
            .for_each(|step| {
                steps_to_drop += 1;

                // un-assign this variable
                let var = step.assignment.variable;
                trace!(target: "backtrack", "Dropping variable {:?}", var);

                // Update the list_of_variables
                // May panic in the unlikely event var does not exist in
                // list_of_variables
                let vs_ref: &mut VariableState = problem.list_of_variables.get_mut(&var).unwrap();
                *vs_ref = VariableState::Unassigned;

                // update the list_of_literal_infos
                if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
                    variable: var,
                    polarity: Polarity::On,
                }) {
                    assert!(li.status != LiteralState::Unknown);
                    li.status = LiteralState::Unknown;
                    li.list_of_clauses.iter().for_each(|rc| {
                        let _r = problem.list_of_clauses_to_update.insert(Rc::clone(rc));
                        trace!(
                            target: "backtrack", 
                            "Trying to add clause {} to set, was {} already there",
                            (*rc).borrow().id,
                            if _r { "not" } else { "" }
                        );
                    });
                }
                if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
                    variable: var,
                    polarity: Polarity::Off,
                }) {
                    assert!(li.status != LiteralState::Unknown);
                    li.status = LiteralState::Unknown;
                    li.list_of_clauses.iter().for_each(|rc| {
                        let _r = problem.list_of_clauses_to_update.insert(Rc::clone(rc));
                        trace!(
                            target: "backtrack", 
                            "Trying to add clause {} to set, was {}already there",
                            (*rc).borrow().id,
                            if _r { "not " } else { "" }
                        );
                    });
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
        trace!(target: "backtrack", "Flipping variable {:?}", last_step.assignment.variable);

        last_step.assignment.polarity = !last_step.assignment.polarity;
        last_step.assignment_type = SolutionStepType::FreeChoiceSecondTry;

        let var = last_step.assignment.variable;
        let new_pol = last_step.assignment.polarity;
        if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
            variable: var,
            polarity: new_pol,
        }) {
            assert!(li.status != LiteralState::Unknown);
            li.status = LiteralState::Sat;
            li.list_of_clauses.iter().for_each(|rc| {
                let _r = problem.list_of_clauses_to_update.insert(Rc::clone(rc));
                trace!(
                    target: "backtrack", 
                    "Trying to add clause {} to set, was {}already there",
                    (*rc).borrow().id,
                    if _r { "not " } else { "" }
                );
            });
        }
        if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
            variable: var,
            polarity: !new_pol,
        }) {
            assert!(li.status != LiteralState::Unknown);
            li.status = LiteralState::Unsat;
            li.list_of_clauses.iter().for_each(|rc| {
                let _r = problem.list_of_clauses_to_update.insert(Rc::clone(rc));
                trace!(
                    target: "backtrack", 
                    "Trying to add clause {} to set, was {}already there",
                    (*rc).borrow().id,
                    if _r { "not " } else { "" }
                );
            });
        }

        trace!(target: "backtrack", "solution stack: {:?}", solution_stack);
        // panic_if_incoherent(problem, solution_stack);

        // recursively call into this function to resolve any new conflicts
        return udpate_clause_state_and_resolve_conflict(problem, solution_stack);
    }
}
