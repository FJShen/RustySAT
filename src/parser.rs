use crate::heuristics::heuristics::Heuristics;
use log::info;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use crate::sat_solver::*;
use std::collections::{BTreeSet, HashMap};

pub fn parse(filename: &String, heuristics: &mut impl Heuristics) -> Problem {
    let path = Path::new(filename);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("cannot open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut buffer = String::new();
    match file.read_to_string(&mut buffer) {
        Err(why) => panic!("failed to read {}: {}", display, why),
        Ok(_) => info!(target: "parser", "successfully read {}", display),
    };

    let mut circuit = Problem {
        list_of_variables: HashMap::<Variable, VariableState>::new(),
        list_of_literal_infos: HashMap::<Literal, Rc<RefCell<LiteralInfo>>>::new(),
        list_of_clauses: Vec::<Rc<RefCell<Clause>>>::new(),
        list_of_clauses_to_check: BTreeSet::new(),
    };

    // CLAUSE LOOP
    let mut clause_id: u32 = 0;
    for line in buffer.split("\n").map(|s| s.trim()) {
        if line.is_empty(){
            continue;
        }
        if line.starts_with("0") { 
            continue; 
        }
        if line.starts_with("%") {
            continue;
        }
        if line.starts_with("c") || line.starts_with("p") {
            continue;
        }
        if circuit.list_of_clauses.len() < (clause_id + 1) as usize {
            circuit.list_of_clauses.push(Rc::new(RefCell::new(Clause {
                id: clause_id as u32,
                // status: ClauseState::Unresolved,
                list_of_literals: Vec::<Literal>::new(),
                list_of_literal_infos: vec![],
                watch_literals: [NULL_LITERAL; 2],
            })));
        }
        // LITERAL LOOP
        let mut clause_lit_count = 0;
        for literal_str in line.split_whitespace() {
            let literal_val: i32 = match literal_str.parse() {
                Ok(val) => val,
                Err(_) => break,
            };
            if literal_val == 0 {
                clause_id += 1;
                continue;
            }
            clause_lit_count += 1;

            // register new variable
            assert!(
                literal_val != 0,
                "Variable index must be non-zero, because zero is for NULL_VARIABLE, we got {}",
                literal_val
            );
            let variable = Variable {
                index: literal_val.abs() as u32,
            };
            circuit
                .list_of_variables
                .insert(variable, VariableState::Unassigned);

            // register new literal
            let literal = Literal {
                variable: variable,
                polarity: if literal_val > 0 {
                    Polarity::On
                } else {
                    Polarity::Off
                },
            };
            let current_clause = circuit.list_of_clauses.last().unwrap();
            circuit
                .list_of_literal_infos
                .entry(literal.clone())
                .and_modify(|e| {
                    (**e)
                        .borrow_mut()
                        .list_of_clauses
                        .push(Rc::clone(current_clause))
                })
                .or_insert({
                    let l = LiteralInfo {
                        list_of_clauses: vec![Rc::clone(current_clause)],
                        status: LiteralState::Unknown,
                    };
                    Rc::new(RefCell::new(l))
                });
            let mut clause = (**current_clause).borrow_mut();
            clause.list_of_literals.push(literal);
            clause.watch_literals[clause_lit_count % 2] = literal;
        }
        let current_clause = circuit.list_of_clauses.last().unwrap();
        heuristics.add_parsed_clause(&current_clause.borrow());
    }

    circuit.list_of_clauses.iter().for_each(|rc| {
        let mut clause_ref = (**rc).borrow_mut();
        clause_ref.list_of_literal_infos = clause_ref
            .list_of_literals
            .iter()
            .map(|l| Rc::clone(&circuit.list_of_literal_infos[l]))
            .collect()
    });

    circuit
}
