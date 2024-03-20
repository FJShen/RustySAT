use crate::sat_structures::*;

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
        panic_if_incoherent(&mut p, &solution);

        let resolved_all_conflicts = resolve_conflict(&mut p, &mut solution);
        if !resolved_all_conflicts {
            panic!("UNSAT");
        }
    }

    println!("all variables are assigned");

    solution
}
