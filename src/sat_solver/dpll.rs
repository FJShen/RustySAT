use super::*;
use std::{cell::Ref, collections::BTreeSet};
use tailcall::tailcall;
use log::{info, log_enabled, trace};

// If the problem is UNSAT, we will return None
pub fn dpll(mut p: Problem) -> Option<SolutionStack> {
    let mut solution = SolutionStack { stack: vec![] };

    // Baseline DPLL
    // 0. Pre-process the problem
    // 0.1 Identify Unit Clauses and force assign their literals
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

    let ret = force_assignment_for_unit_clauses(&mut p, &mut solution);
    if !ret {return None;}

    while let Some((var, pol)) = get_one_unresolved_var(&p) {
        solution.push_free_choice_first_try(var, pol);
        trace!(target: "dpll", "picking variable {:?}", var);
        trace!(target: "dpll", "solution stack: {:?}", solution);

        mark_variable_assigned(&mut p, var);
        update_literal_info(&mut p, var, pol, UpdateLiteralInfoCause::FreeAssignment);

        // sanity check
        // panic_if_incoherent(&p, &solution);
        if USE_BCP{
            while !boolean_constant_propagation(&mut p, &mut solution) {
                let resolved_all_conflicts = udpate_clause_state_and_resolve_conflict(&mut p, &mut solution);
                if !resolved_all_conflicts {
                    return None;
                } 
            }
        } else {
            let resolved_all_conflicts = udpate_clause_state_and_resolve_conflict(&mut p, &mut solution);
            if !resolved_all_conflicts {
                return None;
            }            
        }


    }

    info!(target: "dpll", "all variables are assigned");

    Some(solution)
}

////////////////////////////////////////////////////////
// Routines for the SAT Algorithm
////////////////////////////////////////////////////////

/// Called once right after reading the problem from file. Aims to identify unit
/// clauses and give them an assignment. 
/// This is function is necessary because the two-watch-variable algorithm used
/// for BCP requires each clause to have at least two variables.  
/// Returns true if no conflict occur during the call
pub fn force_assignment_for_unit_clauses(problem: &mut Problem, solution: &mut SolutionStack) -> bool{
    // Go over all clauses, hunt for those that have only one literal
    let it_literals_to_force = problem.list_of_clauses
        .iter()
        .filter(|rc|rc.borrow().list_of_literals.len() == 1)
        .map(|rc|rc.borrow().list_of_literals[0]);

    let mut _temp_set = BTreeSet::<SolutionStep>::new();
    let mut ret = true;
    for l in it_literals_to_force{
        let v = l.variable;
        let p = l.polarity;

        // update and also check that the literal hasn't been assigned with
        // opposite polarity 
        if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal{
            variable: v,
            polarity: p,
        }){
            if li.status == LiteralState::Unsat {ret = false; break;}
            
            li.status = LiteralState::Sat;
        }

        if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal{
            variable: v,
            polarity: !p,
        }){
            if li.status == LiteralState::Sat {ret = false; break;}
            
            li.status = LiteralState::Unsat;
        }

        // push the assignment to the solution stack
        let step = SolutionStep {
            assignment: Assignment {
                variable: v,
                polarity: p,
            },
            assignment_type: SolutionStepType::ForcedAtInit,
        };
        _temp_set.insert(step);
    };

    // Pop unique assignments from the step, insert into the stack
    _temp_set.iter().for_each(|s|{
        solution.stack.push(*s);
        let v = s.assignment.variable;
        mark_variable_assigned(problem, v);
    });

    match ret{
        true => {
            info!(target: "dpll", "Force-assignment of literals for unit clauses successful");
            info!(target: "dpll", "Solution stack is {:?}", solution);
        }
        false => {
            info!(target: "dpll", "Force-assignment of literals for unit clauses failed");
        }
    };

    return ret;
}

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

pub fn mark_variable_unassigned(problem: &mut Problem, v: Variable) {
    // will panic if v is not in list_of_variables
    let vs = problem.list_of_variables.get_mut(&v).unwrap();
    *vs = VariableState::Unassigned;
}

pub enum UpdateLiteralInfoCause{
    FreeAssignment,
    Backtrack,
    BCPImplication
}

/// Updates LiteralInfo for the affected literals; if this assignment has the
/// possibility of making a Clause UNSAT, add the Clause to
/// list_of_clauses_to_check .
pub fn update_literal_info(problem: &mut Problem, v: Variable, p: Polarity, cause: UpdateLiteralInfoCause) {
    // for both literals (on and off),
    // - update their state from Unknown to Sat/Unsat
    // - and update their Clauses' status

    // same polarity: becomes Satisfied
    let same_pol_literal = Literal {
        variable: v,
        polarity: p,
    };
    if let Some(li) = problem.list_of_literal_infos.get_mut(&same_pol_literal) {
        match cause {
            UpdateLiteralInfoCause::FreeAssignment |
            UpdateLiteralInfoCause::BCPImplication => {
                assert!(li.status == LiteralState::Unknown, "literal must not be Sat/Unsat");
            },
            UpdateLiteralInfoCause::Backtrack => {
                assert!(li.status == LiteralState::Unsat, "literal must not be Unknown/Sat");
            },
        }
        li.status = LiteralState::Sat;
    }
    
    // opposite polarity: becomes Unsat
    let opposite_pol_literal = Literal {
        variable: v,
        polarity: !p,
    };
    if let Some(li) = problem.list_of_literal_infos.get_mut(&opposite_pol_literal) {
        match cause {
            UpdateLiteralInfoCause::FreeAssignment |
            UpdateLiteralInfoCause::BCPImplication => {
                assert!(li.status == LiteralState::Unknown, "literal must not be Sat/Unsat");
            },
            UpdateLiteralInfoCause::Backtrack => {
                assert!(li.status == LiteralState::Sat, "literal must not be Unknown/Unsat");
            },
        }
        li.status = LiteralState::Unsat;
        
        // For the UNSAT literal, it has the potential of changing a clause's
        // state. 
        li.list_of_clauses.iter().for_each(|rc| {
            if USE_BCP {
                if rc.borrow().hits_watch_literals(opposite_pol_literal) {
                    problem.list_of_clauses_to_check.insert(Rc::clone(rc)); 
                }
            } else {
                problem.list_of_clauses_to_check.insert(Rc::clone(rc));  
            }
        });
    }
}

/// Called after assigning a free variable, or performing a backtrack. 
/// Returns true if no more implications can be made; returns false if a
/// variable is implied to be both On and Off. 
//#[tailcall]  
pub fn boolean_constant_propagation(
    problem: &mut Problem,
    solution: &mut SolutionStack
) -> bool {
    let mut implied_assignments = BTreeMap::<Variable, Polarity>::new();

    while problem.list_of_clauses_to_check.len() > 0 || implied_assignments.len() > 0 {

        // Examine each clause, we either find a substitute variable to watch, or
        // are forced to assign the other watch variable.
        while let Some(rc) = problem.list_of_clauses_to_check.pop_first() {
            let mut c = rc.borrow_mut();
            trace!(target: "bcp", "Examining clause {}", c.id);

            let substitution_result = c.try_substitute_watch_literal(problem);
            if let BCPSubstituteWatchLiteralResult::ForcedAssignment{l} = substitution_result {
                match implied_assignments.get(&l.variable) {
                    Some(p) => {
                        if *p != l.polarity {
                            // conflict!
                            trace!(target: "bcp", "Variable {:?} implied to be both polarities", l.variable);
                            return false;
                        } else {
                            trace!(target: "bcp", "Variable {:?} implied to be {:?}", l.variable, l.polarity);
                        }
                    } 
                    None => {
                        implied_assignments.insert(l.variable, l.polarity);
                        trace!(target: "bcp", "Variable {:?} implied to be {:?}", l.variable, l.polarity);
                    }
                }
            }
        }

        // At this point, we have finished examining all clauses affected by a
        // literal assignment, but we end up with a list of more implied assignments.   
        // We try those implied assignments one at a time. 
        if let Some((v, p)) = implied_assignments.pop_first() {
            solution.push_step(v, p, SolutionStepType::ForcedAtBCP);
            trace!(target: "bcp", "picking variable {:?}", v);
            trace!(target: "bcp", "solution stack: {:?}", solution);

            mark_variable_assigned(problem, v);
            update_literal_info(problem, v, p, UpdateLiteralInfoCause::BCPImplication); // adds clauses to list_of_clauses_to_check
        }
    }


    return true;
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
    if !USE_BCP{
        // do we even have an unsatiafiable clause?
        let mut found_unsat = false;
        while let Some(rc) = problem.list_of_clauses_to_check.pop_first() {
            let mut c = rc.borrow_mut();
            trace!(target: "backtrack", "Examining clause {}", c.id);

            // we want to see if this clause becomes satisfied or
            // unsatisfiable
            let new_status = c.recalculate_clause_state(problem);

            let s = match new_status {
                ClauseState::Satisfied => "satisfied",
                ClauseState::Unsatisfiable => "unsatisfiable",
                ClauseState::Unresolved => "unresolved",
            };
            trace!(target: "backtrack", "Clause {} becomes {}", c.id, s);

            if new_status == ClauseState::Unsatisfiable {
                // One unsat clause is enough, we have to keep backtracking
                found_unsat = true;
                break;
            }
        }

        if !found_unsat {
            trace!(target: "backtrack", "All conflicts resolved.");
            return true;
        }        
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
        // 1. Un-mark the literals and variables touched by any step that need to
        //    be dropped, and add the affected clauses to list_of_clauses_to_check
        // 2. Drop those steps from the solution_stack
        // 3. Flip the first step that we can flip, mark its literal/variable,
        //    and add affected clauses to list_of_clauses_to_check

        // Updated in step 1, used in step 2
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
                trace!(target: "backtrack", "Dropping variable {:?}", var);

                // Update the list_of_variables
                // May panic in the unlikely event var does not exist in
                // list_of_variables
                mark_variable_unassigned(problem, var);

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

        // There may be leftover clauses, but we have backtracked, which means
        // the very assignment that caused any Clause to be added to this list
        // have been invalidated, so it's okay to just clear the worklist. 
        problem.list_of_clauses_to_check.clear();

        // Reverse the polarity of the last element in the current solution
        // stack, and update list_of_literal_infos and list_of_clauses_to_check.
        // However, list_of_variables need not be modified.
        let last_step = solution_stack.stack.last_mut().unwrap();
        assert!(last_step.assignment_type == SolutionStepType::FreeChoiceFirstTry);
        trace!(target: "backtrack", "Flipping variable {:?}", last_step.assignment.variable);

        last_step.assignment.polarity = !last_step.assignment.polarity;
        last_step.assignment_type = SolutionStepType::FreeChoiceSecondTry;

        let var = last_step.assignment.variable;
        let new_pol = last_step.assignment.polarity;
        // if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
        //     variable: var,
        //     polarity: new_pol,
        // }) {
        //     assert!(li.status == LiteralState::Unsat);
        //     li.status = LiteralState::Sat;
        //     if log_enabled!(target: "backtrack", log::Level::Trace){
        //         li.list_of_clauses.iter().for_each(|rc| {
        //             trace!(
        //                 target: "backtrack", 
        //                 "Clause {} becomes satisfied",
        //                 (*rc).borrow().id
        //             );                
        //         });                
        //     }
        // }
        
        // if let Some(li) = problem.list_of_literal_infos.get_mut(&Literal {
        //     variable: var,
        //     polarity: !new_pol,
        // }) {
        //     assert!(li.status == LiteralState::Sat);
        //     li.status = LiteralState::Unsat;
        //     li.list_of_clauses.iter().for_each(|rc| {
        //         let _r = problem.list_of_clauses_to_check.insert(Rc::clone(rc));
        //         trace!(
        //             target: "backtrack", 
        //             "Trying to add clause {} to set, was {}already there",
        //             (*rc).borrow().id,
        //             if _r { "not " } else { "" }
        //         );
        //     });
        // }

        update_literal_info(problem, var, new_pol, UpdateLiteralInfoCause::Backtrack);

        trace!(target: "backtrack", "solution stack: {:?}", solution_stack);
        // panic_if_incoherent(problem, solution_stack);

        if USE_BCP {
            return true;
        }

        // recursively call into this function to resolve any new conflicts
        return udpate_clause_state_and_resolve_conflict(problem, solution_stack);
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
            // assert!(c.recalculate_clause_state(problem) == c.status);
        });
}
