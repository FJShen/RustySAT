# RUSTY SAT
Welcome to RustySAT, a SAT Solver written in Rust! This work is done by Andrew
Gan and Fangjia Shen. 

## Features
- Boolean Constraint Propagation (Fangjia)
- Variable Heuristics (Andrew)

## How to Compile and Run

### To Compile
The script file `setup.sh` first installs a Rust toolchain then compiles the
SAT solver. Nothing here requires sudo (you do need internet to download things). Run the command:
```
bash setup.sh
```

**Note**: At the beginning of installation, you will be
prompted with a message that looks like this. Just choose the standard
installation. 
```
Current installation options:

   default host triple: x86_64-unknown-linux-gnu
     default toolchain: stable (default)
               profile: default
  modify PATH variable: yes

1) Proceed with standard installation (default - just press enter)
2) Customize installation
3) Cancel installation
```

The last command in the script file compiles the solver: `cargo build --release`.

### To Run
**CLI Interface**

Our CLI interface follows the course project requirement (of course!)

- Default configuration with BCP and VSIDS `./target/release/sat_solver <cnf_file>` 
- To disable BCP: Add `--no-bcp` 
- To disabled VSIDS : Add `--heuristics x`
- To confirm SAT/UNSAT : Add `--check [--satisfiable]`

**Output format**:
- SAT
```
RESULT: SAT
ASSIGNMENT: 1=0 2=1 3=1 4=0 5=0 ....
```

- UNSAT
```
RESULT: UNSAT
```

## Code Structure
- src/
  - main.rs: Entry point of the solver. Definition of CLI argument parser.
  - parser.rs: Implementation of CNF file lexer and parser.
  - sat_solver.rs: Top level file for module `sat_solver`. Definition of all
    data structures used by the DPLL algorithm (and the two-watch-literals
    data structure used by BCP).
  - sat_solver/
    - sat_structures.rs: Auxiliary methods for the 
      data structures (e.g. debug-print format and checking if a clause is unsatisfiable). 
    - dpll.rs: Routines for the DPLL algorithm. Routines for `Boolean Constraint
      Propagation`. 
  - heuristics.rs: Top level file for the module `heuristics`. 
  - heuristics/
    - heuristics.rs: Declaration of the `Heuristics` trait (heuristics for picking a variable to assign). 
    - ascending.rs: Implements `Ascending`, a baseline implementation of the
      `Heuristics` trait that recommends unassigned variables starting with the smallest index.
    - dlis.rs: Implements `Dynamic Largest Individual Sum`, heuristics which recommends the literal  
      that appears most frequently among unresolved clauses.
    - vsids.rs: Implements `Variable State Independent Decaying Sum`, heuristics which prioritises 
      literals that appeared in recently discovered conflict clauses.
  - profiler.rs: Counters for the number of free/implied/backtracked/flipped
    decisions made during the solving of a problem. Timer for run time.

## Data Structures 

### Variable
Aliased to a 32-bit unsigned integer. 

### Literal
- A `Variable`; 
- A boolean field representing the polarity of a literal. 

For example, `a`
and `a'` have an identical `Variable` field but different polarities. 

### LiteralInfo
- An enum field representing the status of a literal: `Satisfied`, `Unsatisfied`, or
`Unresolved` (variable is not assigned yet);

- A `Vec` of  references (i.e. pointers) to heap-allocated `Clause` objects,
  each corresponding to a clause where
this literal appears. 

**note**: This object alone does not know which literal it is describing.
Higher-level data structures are required to associate a `Literal` object with
a `LiteralInfo` object.   

### Clause
- A 32-bit unsigned integer id for a clause;

- A `Vec` of `Literal`
objects that make up the clause;

- A `Vec` of references (i.e. pointers) to
heap-allocated `LiteralInfo` objects, each corresponding to a `Literal` object
that appears in this clause;

- A two-element array of `Literal` objects, representing the two watch literals. 

### Problem
**Purpose**: Represents the sat problem and current states of each variable and
literal in the solving process.

- list_of_variables: A `HashMap` from `Variable` to a `VariableState` enum (Assigned/Unassigned).

- list_of_literal_infos: A `HashMap` from `Literal` to a reference (i.e. pointer)
to a heap-allocated `LiteralInfo` object

- list_of_clauses: A `Vec` of references (i.e. pointers) to heap-allocated
`Clause` objects. 

- list_of_clauses_to_check: A `BTreeSet` of references (i.e. pointers) to heap-allocated
`Clause` objects. 

---

### Assignment
- A `Variable` object;

- A boolean representing the polarity (on/off) of this assigned variable.

### SolutionStep
- A `Assignment` object;

- An enum representing the nature of this assignment:
  - Forced-at-Init: Implied in the beginning because this variable belongs to
    a unit clause.
  - Forced-at-BCP: Implied during Boolean Constraint Propagation.
  - First-try: The variable was picked at-will by some heuristics; haven't
    backtracked and flipped the polarity of this assignment.
  - Second-try: Converted from `First-try` after flipping the polarity during
    backtrack. 

### SolutionStack
- A `Vec` of `SolutionStep` objects that represents the solution stack of
assignments so far. 

## Functions
### dpll
Top-level function for the DPLL algorithm. Steps include:
1. Pre-process the problem: identify Unit Clauses and force assign their
   variables.
    - If BCP is enabled, perform BCP for each forced assignment.
2. Pick a variable to assign, using some heuristics.
3. Update the problem
    1. Update list of variables: mark one as Assigned
    2. Update list of literals: mark one literal as Sat, and its complement as
       Unsat
    3. Update the state of each Clause associated with the literals touched in
       the last step
4. Resolve conflicts by performing backtrack if any clause is unsatisfiable.
    - If BCP is enabled, first try to imply as many variables as possible. Only
     backtrack when a variable is implied to be both on and off.
      Repeat the "BCP-backtrack" loop until no more implications can be made.
5. Repeat 2~4 until no variables are left to assign. 

#### force_assignment_for_unit_clauses
Called once at the beginning of the DPLL algorithm. 

Scans the list of clauses to search for unit clauses. Put all involved literals
in a set. Then iteratively push the forced assignments onto the solution stack
while performing BCP after each assignment. 

#### update_literal_info
Called every time a variable changes state (being assigned/unassigned, or having
its assigned polarity flipped), hence this method is used in many places: when
we pick a variable to assign at-will, during BCP, during backtracking, and
during the initial force-assignment of unit clause variables. 

Aside from updating the state in `LiteralInfo`, it also inserts all
`Clause` objects referenced by the `Literal` which becomes unsatisfied into
`list_of_clauses_to_check` (when BCP is enabled, only insert if this literal is
one of the two watch variables of a clause).  

#### udpate_clause_state_and_resolve_conflict
When BCP is not enabled, it pops every `Clause` object reference from
`list_of_clauses_to_check` and calculates if it is satisfied, unresolved, or
unsatisfiable. If no clause is unsatisfiable, then there is no need to backtrack.

In the case of a unsatisfiable clause, the solver drops all remaining `Clause`
object references from `list_of_clauses_to_check` and starts backtracking. All
implied assignments and already-flipped assignments are dropped; the
last at-will assignment which has not been flipped is flipped, clauses which
contain the subsequently unsatisfied literal are added to `list_of_clauses_to_check`, and this method
is called in a tail-recursive fashion as long as one clause in the worklist proves to be
unsatisfiable.

When BCP is enabled, only a portion of code in this method is run: it merely drops implied/"already-flipped" assignments on the stack till the last "first-try"
assignment and flip. The method then returns without recursion. More on this in the
description for `boolean_constraint_propagation`.

#### boolean_constraint_propagation
When BCP is enabled, this method takes over from
`udpate_clause_state_and_resolve_conflict` the role of checking if each clause becomes
unsatisfiable. It pops clauses from `list_of_clauses_to_check` and see if they
has become unsatisfied. If a clause is still unresolved and more than one of its
literals are unresolved, the current watch literals are updated; if a clause
only has one unresolved literal remaining (it has to be one of the watch literals), force-assign that variable; if the clause is
unsatisfiable, then transfer control to
`udpate_clause_state_and_resolve_conflict` to perform backtracking.

This method also keeps a record of all implied assignments in each call. If it
detects a variable being implied to be both on and off, it transfers control to
`udpate_clause_state_and_resolve_conflict` to perform backtracking.
