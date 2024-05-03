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

impl Not for Literal{
    type Output = Self;
    fn not(self) -> Self::Output {
        let new_p = match self.polarity {
            Polarity::Off => Polarity::On,
            Polarity::On => Polarity::Off,
        };
        Literal{variable: self.variable, polarity: new_p}
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct _Clause<'a> {
    pub _id: &'a u32,
    // pub status: ClauseState,
    pub _list_of_literals: &'a Vec<Literal>,
    pub _watch_literals: &'a [Literal; 2],
}

impl<'a> _Clause<'a>{
    pub fn from(c: &'a Clause)->Self{
        _Clause{
            _id: &c.id,
            _list_of_literals: &c.list_of_literals,
            _watch_literals: &c.watch_literals,
        }
    }
}

impl fmt::Debug for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The point of this struct is to remove the strong reference to
        // LiteralInfo objects. When debug-printing, Rc objects are dereferenced
        // and printed. Printing a Clause object that holds Rc pointers to
        // LiteralInfos will cause the LiteralInfo to be printed, which in turn
        // cause more Clauses to be printed...

        let c = _Clause::from(self);
        write!(f, "{:?}", c)
    }
}

impl PartialEq for Clause {
    fn eq(&self, other: &Clause) -> bool{
        let lhs = _Clause::from(self);
        let rhs = _Clause::from(other);
        return lhs == rhs;
    }
}

impl Eq for Clause{}

impl PartialOrd for Clause{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Clause{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering{
        let lhs = _Clause::from(self);
        let rhs = _Clause::from(other);
        return _Clause::cmp(&lhs, &rhs);
    }
}

impl Clause {
    pub fn recalculate_clause_state(&self, problem: &Problem) -> ClauseState {
        let mut encountered_unknown_state = false;

        //for l in &self.list_of_literals {
        for li in &self.list_of_literal_infos {
            //let ls = problem.list_of_literal_infos[l].borrow().status;
            let ls = li.upgrade().unwrap().borrow().status;
            if ls == LiteralState::Sat {
                return ClauseState::Satisfied;
            } else if ls == LiteralState::Unknown {
                encountered_unknown_state = true;
            }
        }

        match encountered_unknown_state {
            true => ClauseState::Unresolved,
            false => ClauseState::Unsatisfiable,
        }
    }

    pub fn hits_watch_literals(&self, l: Literal) -> bool {
        if self.watch_literals[0] != l && self.watch_literals[1] != l {
            return false;
        } else {
            return true;
        }
    }

    /// Precondition: one (and only one) of the watch literals of this clause is UNSAT
    /// Postcondition: if retuning FoundSubstitute, the UNSAT watch literal is
    /// substituted with an Unknown-state literal; if returning ForcedAssignment, nothing is changed.
    pub fn try_substitute_watch_literal(
        &mut self,
        problem: &Problem,
    ) -> BCPSubstituteWatchLiteralResult {
        // Examine if the clause is already SAT (a not-watched literal is SAT),
        // also examine the two watch literals
        let mut already_sat = false;
        let (mut status_0, mut status_1) = (LiteralState::Unknown, LiteralState::Unknown);

        self.list_of_literals
            .iter()
            .zip(self.list_of_literal_infos.iter())
            .for_each(|(l, li)| {
                // let l_status = problem.list_of_literal_infos[l].borrow().status;
                let l_status = li.upgrade().unwrap().borrow().status;
                if l_status == LiteralState::Sat {
                    already_sat = true;
                }
                if &self.watch_literals[0] == l {
                    status_0 = l_status;
                } else if &self.watch_literals[1] == l {
                    status_1 = l_status;
                }
            });

        if already_sat {
            return BCPSubstituteWatchLiteralResult::ClauseIsSAT;
        }

        // which watch literal became UNSAT?
        let watch_index;

        match (status_0, status_1) {
            (LiteralState::Unsat, LiteralState::Unknown) => watch_index = 0,
            (LiteralState::Unknown, LiteralState::Unsat) => watch_index = 1,
            (LiteralState::Sat, _) | (_, LiteralState::Sat) => {
                return BCPSubstituteWatchLiteralResult::ClauseIsSAT;
            }
            (LiteralState::Unknown, LiteralState::Unknown) => {
                panic!("both of the watch literals of clause {:?} are unassigned, why are you trying to find a substitute literal to watch?", self);
            }
            (LiteralState::Unsat, LiteralState::Unsat) => {
                panic!("both of the watch literals of clause {:?} are unsat", self);
            }
        }

        let substitute_literal = self
            .list_of_literals
            .iter()
            .zip(self.list_of_literal_infos.iter())
            .filter(|(l, _)| !self.hits_watch_literals(**l))
            .find(|(_, li)| li.upgrade().unwrap().borrow().status == LiteralState::Unknown);

        match substitute_literal {
            Some((l, _)) => {
                self.watch_literals[watch_index] = *l;
                return BCPSubstituteWatchLiteralResult::FoundSubstitute;
            }
            None => {
                let other_index = 1 - watch_index;
                let other_literal = self.watch_literals[other_index];
                if other_literal == NULL_LITERAL {
                    return BCPSubstituteWatchLiteralResult::UnitClauseUnsat;
                } else {
                    return BCPSubstituteWatchLiteralResult::ForcedAssignment { l: other_literal };
                }
            }
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
                SolutionStepType::ForcedAtBCP{unit_clause_id:_} => "x",
                SolutionStepType::ForcedAtInit => "I",
                SolutionStepType::Zombied => "Z"
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
// pub fn get_sample_problem() -> Problem {
//     // f = (a + b + c) (a' + b') (b + c')
//     // one example solution: a=1, b=0, c=0
//     let v_a = Variable { index: 0 };
//     let v_b = Variable { index: 1 };
//     let v_c = Variable { index: 2 };

//     let mut _list_of_variables = BTreeMap::from([
//         (v_a, VariableState::Unassigned),
//         (v_b, VariableState::Unassigned),
//         (v_c, VariableState::Unassigned),
//     ]);

//     let mut _list_of_clauses = vec![
//         Rc::new(RefCell::new(Clause {
//             id: CLAUSE_COUNTER.inc(),
//             list_of_literals: vec![
//                 Literal {
//                     variable: v_a,
//                     polarity: Polarity::On,
//                 },
//                 Literal {
//                     variable: v_b,
//                     polarity: Polarity::On,
//                 },
//                 Literal {
//                     variable: v_c,
//                     polarity: Polarity::On,
//                 },
//             ],
//             // status: ClauseState::Unresolved,
//             watch_variables: [v_a, v_b],
//         })),
//         Rc::new(RefCell::new(Clause {
//             id: CLAUSE_COUNTER.inc(),
//             list_of_literals: vec![
//                 Literal {
//                     variable: v_a,
//                     polarity: Polarity::Off,
//                 },
//                 Literal {
//                     variable: v_b,
//                     polarity: Polarity::Off,
//                 },
//             ],
//             // status: ClauseState::Unresolved,
//             watch_variables: [v_a, v_b],
//         })),
//         Rc::new(RefCell::new(Clause {
//             id: CLAUSE_COUNTER.inc(),
//             list_of_literals: vec![
//                 Literal {
//                     variable: v_b,
//                     polarity: Polarity::On,
//                 },
//                 Literal {
//                     variable: v_c,
//                     polarity: Polarity::Off,
//                 },
//             ],
//             // status: ClauseState::Unresolved,
//             watch_variables: [v_c, v_b],
//         })),
//     ];

//     // To populate the list for LiteralInfo:
//     // Create one LiteralInfo for each literal.
//     // Then iterate over the clauses: for each literal in a clause, update its
//     // entry.
//     let mut _list_of_literal_infos: BTreeMap<Literal, LiteralInfo> = BTreeMap::new();
//     for c in &_list_of_clauses {
//         for l in &(**c).borrow().list_of_literals {
//             _list_of_literal_infos
//                 .entry(l.clone())
//                 .and_modify(|e| e.list_of_clauses.push(Rc::clone(c)))
//                 .or_insert(LiteralInfo {
//                     list_of_clauses: vec![Rc::clone(c)],
//                     status: LiteralState::Unknown,
//                 });
//         }
//     }

//     // println!("After the loop, list_of_literal_infos is: {:#?}", _list_of_literal_infos);

//     Problem {
//         list_of_variables: _list_of_variables,
//         list_of_literal_infos: _list_of_literal_infos,
//         list_of_clauses: _list_of_clauses,
//         list_of_clauses_to_check: BTreeSet::new()
//     }
// }

impl SolutionStack {
    pub fn push_free_choice_first_try(&mut self, var: Variable, pol: Polarity) {
        self.push_step(var, pol, SolutionStepType::FreeChoiceFirstTry);
    }

    pub fn push_step(&mut self, var: Variable, pol: Polarity, ass_type: SolutionStepType) {
        let step = SolutionStep {
            assignment: Assignment {
                variable: var,
                polarity: pol,
            },
            assignment_type: ass_type,
        };
        self.stack.push(step);
    }
}
