use crate::vsids::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

use std::collections::BTreeMap;
use crate::sat_solver::*;

pub fn parse(filename: &String, vsids: &mut VSIDS) -> Problem {
  let path = Path::new(filename);
  let display = path.display();

  let mut file = match File::open(&path) {
    Err(why) => panic!("cannot open {}: {}", display, why),
    Ok(file) => file,
  };

  let mut buffer = String::new();
  match file.read_to_string(&mut buffer) {
    Err(why) => panic!("failed to read {}: {}", display, why),
    Ok(_) => println!("successfully read {}", display),
  };

  let mut circuit = Problem {
    list_of_variables: BTreeMap::<Variable, VariableState>::new(),
    list_of_literal_infos: BTreeMap::<Literal, LiteralInfo>::new(),
    list_of_clauses: Vec::<Rc::<RefCell::<Clause>>>::new(),
  };

  // CLAUSE LOOP
  let mut clause_id : u32 = 0;
  for line in buffer.split("\n").map(|s| s.trim()) {
    if line.starts_with("%") {
      break;
    }
    if line.starts_with("c") || line.starts_with("p") {
      continue;
    }
    if circuit.list_of_clauses.len() < (clause_id + 1) as usize {
      circuit.list_of_clauses.push(Rc::new(RefCell::new(
        Clause {
          id: clause_id as u32,
          list_of_literals: Vec::<Literal>::new(),
          status: ClauseState::Unresolved,
        }
      )));
    }

    let current_clause = circuit.list_of_clauses.last().unwrap();

    // LITERAL LOOP
    for literal_str in line.split_whitespace() {
      let literal_val : i32 = match literal_str.parse() {
          Ok(val) => val,
          Err(_)  => break,
      };
      if literal_val == 0 {
        clause_id += 1;
        continue;
      }

      // register new variable
      let variable = Variable {
        index: literal_val.abs() as u32,
      };
      circuit.list_of_variables.insert(variable, VariableState::Unassigned);

      // register new literal
      let literal = Literal {
        variable: variable,
        polarity: if literal_val > 0 {Polarity::On} else {Polarity::Off},
      };
      circuit.list_of_literal_infos
        .entry(literal.clone())
        .and_modify(|e| e.list_of_clauses.push(Rc::clone(current_clause)))
        .or_insert(LiteralInfo {
          list_of_clauses: vec![Rc::clone(current_clause)],
          status: LiteralState::Unknown,
        });
      _ = &(**current_clause).borrow_mut().list_of_literals.push(literal);
    }
    vsids.add_clause(&(**current_clause).borrow());
  }

  circuit
}
