use std::cell::RefCell;
use std::rc::Rc;
use crate::sat_structures::*;

// If the problem is UNSAT, we will not return anything but throw an exception. 
pub fn dpll(mut p: Problem) -> SolutionStack {
  let mut solution = SolutionStack{ stack : vec![]};

  // resolve all variables before we return a solution
  let mut unresolved_var = p.get_one_unresolved_var();

  while let Some(var) = unresolved_var {
    p.mark_variable_assigned(var);
    println!("dpll marking variable {:?} as assigned.", var);
    unresolved_var = p.get_one_unresolved_var();
  }

  solution
}