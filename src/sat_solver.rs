
use crate::sat_structures::*;

// If the problem is UNSAT, we will not return anything but throw an exception. 
pub fn dpll(mut p: Problem) -> SolutionStack {
  let mut solution = SolutionStack{ stack : vec![]};

  // resolve all variables before we return a solution
  // 1. Pick a variable to assign
  // 1.1 Pick a variable
  // 1.2 Pick a polarity
  // 2. Update the problem
  // 2.1 Update list of variables: mark one as Assigned
  // 2.2 Update list of literals: mark one literal as Sat, and its complement as
  // Unsat
  // 2.3 Update the state of each Clause associated with the literals touched in
  // the last step
  // 3. Resolve conflicts if any clause is unsatisfiable
  // 4. Repeat
  while let Some((var, pol)) =  p.get_one_unresolved_var() {
    solution.push_free_choice_first_try(var, pol);
    println!("dpll marking variable {:?} as assigned.", var);
    println!("solution stack: {:?}", solution);
    p.mark_variable_assigned(var);
    p.update_literal_info_and_clauses(var, pol);
    //println!("after update, problem is {:#?}", p);
    //println!("after update, solution is {:#?}", solution);
    p.panic_if_incoherent(&solution);
    
    let resolved_all_conflicts = resolve_conflict(&mut p, &mut solution);
    if !resolved_all_conflicts{
      panic!("UNSAT");
    }
    //panic!("done");
  }

  println!("all variables are assigned");

  solution
}