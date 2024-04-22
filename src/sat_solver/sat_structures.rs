use super::*;
use std::ops::Not;

/////////////////////////////////////////
/// Implementation of SAT data structures
/////////////////////////////////////////

impl fmt::Debug for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "var#{}", self.index)
    }
}

impl Not for Polarity {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Polarity::Off => Polarity::On,
            Polarity::On => Polarity::Off,
        }
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // "a" and "b'" would look like "0" and "1'"
        write!(
            f,
            "{}{}",
            self.variable.index,
            if self.polarity == Polarity::On {
                ""
            } else {
                "^"
            }
        )
    }
}

impl Clause{
    pub fn recalculate_clause_state(&self, problem: &Problem) -> ClauseState {
        let mut encountered_unknown_state = false;

        for l in &self.list_of_literals {
            let ls = problem.list_of_literal_infos[l].status;
            if ls == LiteralState::Sat {return ClauseState::Satisfied;}
            else if ls == LiteralState::Unknown {encountered_unknown_state = true;}
        }

        match encountered_unknown_state {
            true => ClauseState::Unresolved,
            false => ClauseState::Unsatisfiable,
        }
    }
}

impl fmt::Debug for SolutionStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.assignment_type {
                SolutionStepType::FreeChoiceFirstTry => "t",
                SolutionStepType::FreeChoiceSecondTry => "T",
                SolutionStepType::ForcedAtBCP => "x",
                SolutionStepType::ForcedAtInit => "I"
            }
        )?;
        write!(f, "{}", self.assignment.variable.index)?;
        write!(
            f,
            "{}",
            match self.assignment.polarity {
                Polarity::On => "",
                Polarity::Off => "^",
            }
        )
    }
}

// hard-code a SAT problem so I can try the baseline DPLL algorithm.
pub fn get_sample_problem() -> Problem {
    // f = (a + b + c) (a' + b') (b + c')
    // one example solution: a=1, b=0, c=0
    let v_a = Variable { index: 0 };
    let v_b = Variable { index: 1 };
    let v_c = Variable { index: 2 };

    let mut _list_of_variables = BTreeMap::from([
        (v_a, VariableState::Unassigned),
        (v_b, VariableState::Unassigned),
        (v_c, VariableState::Unassigned),
    ]);

    let mut _list_of_clauses = vec![
        Rc::new(RefCell::new(Clause {
            id: CLAUSE_COUNTER.inc(),
            list_of_literals: vec![
                Literal {
                    variable: v_a,
                    polarity: Polarity::On,
                },
                Literal {
                    variable: v_b,
                    polarity: Polarity::On,
                },
                Literal {
                    variable: v_c,
                    polarity: Polarity::On,
                },
            ],
            // status: ClauseState::Unresolved,
        })),
        Rc::new(RefCell::new(Clause {
            id: CLAUSE_COUNTER.inc(),
            list_of_literals: vec![
                Literal {
                    variable: v_a,
                    polarity: Polarity::Off,
                },
                Literal {
                    variable: v_b,
                    polarity: Polarity::Off,
                },
            ],
            // status: ClauseState::Unresolved,
        })),
        Rc::new(RefCell::new(Clause {
            id: CLAUSE_COUNTER.inc(),
            list_of_literals: vec![
                Literal {
                    variable: v_b,
                    polarity: Polarity::On,
                },
                Literal {
                    variable: v_c,
                    polarity: Polarity::Off,
                },
            ],
            // status: ClauseState::Unresolved,
        })),
    ];

    // To populate the list for LiteralInfo:
    // Create one LiteralInfo for each literal.
    // Then iterate over the clauses: for each literal in a clause, update its
    // entry.
    let mut _list_of_literal_infos: BTreeMap<Literal, LiteralInfo> = BTreeMap::new();
    for c in &_list_of_clauses {
        for l in &(**c).borrow().list_of_literals {
            _list_of_literal_infos
                .entry(l.clone())
                .and_modify(|e| e.list_of_clauses.push(Rc::clone(c)))
                .or_insert(LiteralInfo {
                    list_of_clauses: vec![Rc::clone(c)],
                    status: LiteralState::Unknown,
                });
        }
    }

    // println!("After the loop, list_of_literal_infos is: {:#?}", _list_of_literal_infos);

    Problem {
        list_of_variables: _list_of_variables,
        list_of_literal_infos: _list_of_literal_infos,
        list_of_clauses: _list_of_clauses,
        list_of_clauses_to_update: BTreeSet::new()
    }
}

impl SolutionStack {
    pub fn push_free_choice_first_try(&mut self, var: Variable, pol: Polarity) {
        let step = SolutionStep {
            assignment: Assignment {
                variable: var,
                polarity: pol,
            },
            assignment_type: SolutionStepType::FreeChoiceFirstTry,
        };
        self.stack.push(step);
    }
}
