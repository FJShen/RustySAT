use crate::{heuristics::heuristics::Heuristics, profiler::SolverProfiler};

use super::*;
use log::{info, trace};
use std::collections::BTreeMap;
use tailcall::tailcall;

// If the problem is UNSAT, we will return None
pub fn dpll(
    mut p: &mut Problem,
    h: &mut impl Heuristics,
    prof: &mut SolverProfiler,
) -> Option<SolutionStack> {
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

    let ret = force_assignment_for_unit_clauses(&mut p, &mut solution, h, prof);
    if !ret {
        return None;
    }
    trace!(target: "dpll", "solution stack: {:?}", solution);

    while let Some(Literal {
        variable: var,
        polarity: pol,
    }) = h.decide()
    {
        solution.push_free_choice_first_try(var, pol);
        trace!(target: "dpll", "Assigning variable {:?}", var);
        trace!(target: "dpll", "solution stack: {:?}", solution);

        h.assign_variable(var);
        mark_variable_assigned(&mut p, var);
        update_literal_info(&mut p, var, pol, UpdateLiteralInfoCause::FreeAssignment, h);
        prof.bump_free_decisions();

        // sanity check
        // panic_if_incoherent(&p, &solution);
        if h.use_bcp() {
            while !boolean_constraint_propagation(&mut p, &mut solution, h, prof) {
                let resolved_all_conflicts =
                    update_clause_state_and_resolve_conflict(&mut p, &mut solution, h, prof);
                if !resolved_all_conflicts {
                    return None;
                }
            }
            trace!(target: "bcp", "No more implications");
        } else {
            let resolved_all_conflicts =
                update_clause_state_and_resolve_conflict(&mut p, &mut solution, h, prof);
            if !resolved_all_conflicts {
                return None;
            }
            trace!(target: "dpll", "All conflicts cleared.")
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
/// Returns true if no conflict occur during the call.
///
/// Overview
/// 1. Scan all clauses to hunt for unit clauses
/// 1.1 Take note of variables and polarities to force-assign
/// 1.2 Make sure no variable is forced to be both On and Off
/// 2. Assign one variable at a time, performing BCP and conflict resolution
///    along the way
/// 2.1 This step is much like how freely-assigned variables are handled.  
pub fn force_assignment_for_unit_clauses(
    problem: &mut Problem,
    solution: &mut SolutionStack,
    heuristics: &mut impl Heuristics,
    prof: &mut SolverProfiler,
) -> bool {
    // Go over all clauses, hunt for those that have only one literal
    let it_literals_to_force = problem
        .list_of_clauses
        .iter()
        .filter(|rc| rc.borrow().list_of_literals.len() == 1)
        .map(|rc| rc.borrow().list_of_literals[0]);

    let mut _temp_assignment_map = BTreeMap::<Variable, Polarity>::new();
    let mut ret = true;
    for l in it_literals_to_force {
        let this_v = l.variable;
        let this_p = l.polarity;

        match _temp_assignment_map.get(&this_v) {
            Some(p) => {
                if *p != this_p {
                    // conflict!
                    trace!(target: "unit_clause", "Variable {:?} appeared with both polarities in various unit clauses", this_v);
                    ret = false;
                    break;
                }
            }
            None => {
                _temp_assignment_map.insert(this_v, this_p);
                trace!(target: "unit_clause", "Variable {:?} implied to be {:?}", this_v, this_p);
            }
        }
    }

    if !ret {
        return false;
    }

    while let Some((ass_v, ass_p)) = _temp_assignment_map.pop_first() {
        // it's possible a variable has already been implied during the BCP
        // phase
        if problem.list_of_variables[&ass_v] == VariableState::Assigned {
            trace!(target:"unit_clause", "Variable {:?} was already assigned", ass_v);
            trace!(target: "unit_clause", "solution stack: {:?}", solution);
            continue;
        }

        solution.push_step(ass_v, ass_p, SolutionStepType::ForcedAtInit);

        trace!(target: "unit_clause", "Assigning variable {:?}", ass_v);
        trace!(target: "unit_clause", "solution stack: {:?}", solution);

        heuristics.assign_variable(ass_v);
        mark_variable_assigned(problem, ass_v);
        update_literal_info(
            problem,
            ass_v,
            ass_p,
            UpdateLiteralInfoCause::UnitClauseImplication,
            heuristics,
        );
        prof.bump_implied_decisions();

        if heuristics.use_bcp() {
            while !boolean_constraint_propagation(problem, solution, heuristics, prof) {
                let resolved_all_conflicts =
                    update_clause_state_and_resolve_conflict(problem, solution, heuristics, prof);
                if !resolved_all_conflicts {
                    return false;
                }
            }
            trace!(target: "bcp", "No more implications");
        } else {
            let resolved_all_conflicts =
                update_clause_state_and_resolve_conflict(problem, solution, heuristics, prof);
            if !resolved_all_conflicts {
                return false;
            }
            trace!(target: "dpll", "All conflicts cleared.")
        }
    }

    return true;
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

pub enum UpdateLiteralInfoCause {
    FreeAssignment,
    Backtrack,
    BCPImplication,
    UnitClauseImplication,
}

/// Updates LiteralInfo for the affected literals; if this assignment has the
/// possibility of making a Clause UNSAT, add the Clause to
/// list_of_clauses_to_check .
pub fn update_literal_info(
    problem: &mut Problem,
    v: Variable,
    p: Polarity,
    cause: UpdateLiteralInfoCause,
    heuristics: &mut impl Heuristics,
) {
    // for both literals (on and off),
    // - update their state from Unknown to Sat/Unsat
    // - and update their Clauses' status

    // same polarity: becomes Satisfied
    let same_pol_literal = Literal {
        variable: v,
        polarity: p,
    };
    if let Some(li) = problem.list_of_literal_infos.get(&same_pol_literal) {
        let status_ref = &mut li.borrow_mut().status;
        match cause {
            UpdateLiteralInfoCause::FreeAssignment
            | UpdateLiteralInfoCause::BCPImplication
            | UpdateLiteralInfoCause::UnitClauseImplication => {
                assert!(
                    *status_ref == LiteralState::Unknown,
                    "literal must not be Sat/Unsat"
                );
            }
            UpdateLiteralInfoCause::Backtrack => {
                assert!(
                    *status_ref == LiteralState::Unsat,
                    "literal must not be Unknown/Sat"
                );
            }
        }
        *status_ref = LiteralState::Sat;
    }

    // opposite polarity: becomes Unsat
    let opposite_pol_literal = Literal {
        variable: v,
        polarity: !p,
    };
    if let Some(li) = problem.list_of_literal_infos.get(&opposite_pol_literal) {
        let mut li_mut_borrow = li.borrow_mut();
        let status_ref = &mut li_mut_borrow.status;
        match cause {
            UpdateLiteralInfoCause::FreeAssignment
            | UpdateLiteralInfoCause::BCPImplication
            | UpdateLiteralInfoCause::UnitClauseImplication => {
                assert!(
                    *status_ref == LiteralState::Unknown,
                    "literal must not be Sat/Unsat"
                );
            }
            UpdateLiteralInfoCause::Backtrack => {
                assert!(
                    *status_ref == LiteralState::Sat,
                    "literal must not be Unknown/Unsat"
                );
            }
        }
        *status_ref = LiteralState::Unsat;

        // For the UNSAT literal, it has the potential of changing a clause's
        // state.
        li_mut_borrow.list_of_clauses.iter().for_each(|rc| {
            if heuristics.use_bcp() {
                if rc.borrow().hits_watch_literals(opposite_pol_literal) {
                    problem.list_of_clauses_to_check.insert(Rc::clone(rc));
                }
            } else {
                problem.list_of_clauses_to_check.insert(Rc::clone(rc));
            }
            heuristics.unsatisfy_clause(&rc.borrow());
        });
    }
}

/// Called after assigning a variable, or performing a backtrack.
/// Returns true if no more implications can be made; returns false if a
/// variable is implied to be both On and Off.
pub fn boolean_constraint_propagation(
    problem: &mut Problem,
    solution: &mut SolutionStack,
    heuristics: &mut impl Heuristics,
    prof: &mut SolverProfiler,
) -> bool {
    let mut implied_assignments = BTreeMap::<Variable, Polarity>::new();

    while problem.list_of_clauses_to_check.len() > 0 || implied_assignments.len() > 0 {
        // Examine each clause, we either find a substitute variable to watch, or
        // are forced to assign the other watch variable.
        while let Some(rc) = problem.list_of_clauses_to_check.pop_first() {
            let mut c = rc.borrow_mut();
            trace!(target: "bcp", "Examining clause {}", c.id);

            let substitution_result = c.try_substitute_watch_literal(problem);
            match substitution_result {
                BCPSubstituteWatchLiteralResult::UnitClauseUnsat => {
                    trace!(target:"bcp", "Clause {} is unit clause and UNSAT", c.id);
                    trace!(target: "bcp", "{:?}", c);
                    return false;
                }
                BCPSubstituteWatchLiteralResult::ForcedAssignment { l } => {
                    let mut conflict = false;

                    implied_assignments.entry(l.variable)
                        .and_modify(|p|{ // we aren't really modifying anything
                            if *p != l.polarity {
                                // conflict!
                                trace!(target: "bcp", "Clause {}: Variable {:?} implied to be both polarities", c.id, l.variable);
                                trace!(target: "bcp", "{:?}", c);
                                conflict =true;
                            } else {
                                trace!(target: "bcp", "Clause {}: Variable {:?} implied to be {:?}", c.id, l.variable, l.polarity);
                                trace!(target: "bcp", "{:?}", c);
                            }
                        })
                        .or_insert_with(||{
                            trace!(target: "bcp", "Clause {}: Variable {:?} implied to be {:?}", c.id, l.variable, l.polarity);
                            trace!(target: "bcp", "{:?}", c);
                            l.polarity
                        });
                    if conflict {
                        return false;
                    }
                }
                _ => {}
            }
        }

        // At this point, we have finished examining all clauses affected by a
        // literal assignment, but we end up with a list of more implied assignments.
        // We try those implied assignments one at a time.
        if let Some((v, p)) = implied_assignments.pop_first() {
            solution.push_step(v, p, SolutionStepType::ForcedAtBCP);
            trace!(target: "bcp", "Assinging variable {:?}", v);
            trace!(target: "bcp", "solution stack: {:?}", solution);

            heuristics.assign_variable(v);
            mark_variable_assigned(problem, v);
            update_literal_info(
                problem,
                v,
                p,
                UpdateLiteralInfoCause::BCPImplication,
                heuristics,
            ); // adds clauses to list_of_clauses_to_check
            prof.bump_implied_decisions();
        }
    }

    return true;
}

/// Returns true if all conflicts (if any) were successfully resolved. Returns false if
/// the problem is UNSAT (i.e., we have tried both the on- and off-assignment for
/// a variable but neither works).
#[tailcall]
pub fn update_clause_state_and_resolve_conflict(
    problem: &mut Problem,
    solution_stack: &mut SolutionStack,
    heuristics: &mut impl Heuristics,
    prof: &mut SolverProfiler,
) -> bool {
    if !heuristics.use_bcp() {
        // do we even have an unsatiafiable clause?
        let mut found_unsat = false;
        while let Some(rc) = problem.list_of_clauses_to_check.pop_first() {
            let c = rc.borrow_mut();
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

            if new_status == ClauseState::Satisfied {
                heuristics.satisfy_clause(&c);
            }

            if new_status == ClauseState::Unsatisfiable {
                // One unsat clause is enough, we have to keep backtracking
                found_unsat = true;

                // register conflict clause with heuristics
                heuristics.add_conflict_clause(&c);
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

                heuristics.unassign_variable(var);

                // Update the list_of_variables
                // May panic in the unlikely event var does not exist in
                // list_of_variables
                mark_variable_unassigned(problem, var);
                prof.bump_backtracked_decisions();

                // update the list_of_literal_infos
                if let Some(li) = problem.list_of_literal_infos.get(&Literal {
                    variable: var,
                    polarity: Polarity::On,
                }) {
                    let status_ref = &mut li.borrow_mut().status;
                    assert!(*status_ref != LiteralState::Unknown);
                    *status_ref = LiteralState::Unknown;
                }

                if let Some(li) = problem.list_of_literal_infos.get(&Literal {
                    variable: var,
                    polarity: Polarity::Off,
                }) {
                    let status_ref = &mut li.borrow_mut().status;
                    assert!(*status_ref != LiteralState::Unknown);
                    *status_ref = LiteralState::Unknown;
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

        update_literal_info(
            problem,
            var,
            new_pol,
            UpdateLiteralInfoCause::Backtrack,
            heuristics,
        );
        prof.bump_flipped_decisions();

        trace!(target: "backtrack", "solution stack: {:?}", solution_stack);
        // panic_if_incoherent(problem, solution_stack);

        if heuristics.use_bcp() {
            return true;
        }

        // recursively call into this function to resolve any new conflicts
        return update_clause_state_and_resolve_conflict(problem, solution_stack, heuristics, prof);
    }
}

///
/// OBSOLETE METHODS BELOW
///

/// Returns a variable that is unresolved, and a recommendation for which
/// polarity to use. If all variables have been resolved, returns None.  
pub fn get_one_unresolved_var(problem: &Problem) -> Option<(Variable, Polarity)> {
    // heuristic: pick an unassigned variable that appears in the most amount
    // of clauses.
    let tuple_result: Option<(Variable, usize, usize)> = problem
        .list_of_variables
        .iter()
        .filter(|(_v, vs)| **vs == VariableState::Unassigned)
        .map(|(v, _vs)| {
            let mut on_count: usize = 0;
            let mut off_count: usize = 0;
            if let Some(li) = problem.list_of_literal_infos.get(&Literal {
                variable: *v,
                polarity: Polarity::On,
            }) {
                on_count = li.borrow().list_of_clauses.len();
            }
            if let Some(li) = problem.list_of_literal_infos.get(&Literal {
                variable: *v,
                polarity: Polarity::Off,
            }) {
                off_count = li.borrow().list_of_clauses.len();
            }
            (*v, on_count, off_count)
        })
        .max_by_key(|(_v, on_count, off_count)| on_count + off_count);

    if let Some((v, on_count, off_count)) = tuple_result {
        if on_count > off_count {
            return Some((v, Polarity::On));
        } else {
            return Some((v, Polarity::Off));
        }
    } else {
        return None;
    }
}

/// Obsolete
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
            let status_ref = &li.borrow().status;
            if *status_ref == LiteralState::Unknown && *vs == VariableState::Unassigned {
            } else if *status_ref == LiteralState::Sat && *vs == VariableState::Assigned {
            } else if *status_ref == LiteralState::Unsat && *vs == VariableState::Assigned {
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
            let status_ref = &li.borrow().status;
            if *status_ref == LiteralState::Unknown && *vs == VariableState::Unassigned {
            } else if *status_ref == LiteralState::Sat && *vs == VariableState::Assigned {
            } else if *status_ref == LiteralState::Unsat && *vs == VariableState::Assigned {
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
        .for_each(|_c| {
            // assert!(c.recalculate_clause_state(problem) == c.status);
        });
}
