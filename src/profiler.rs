#[derive(Debug)]
pub struct SolverProfiler {
    free_decisions: u64,
    implied_decisions: u64,
    backtracked_decisions: u64,
    flipped_decisions: u64,
}

impl SolverProfiler {
    pub fn new() -> SolverProfiler {
        SolverProfiler {
            free_decisions: 0,
            implied_decisions: 0,
            backtracked_decisions: 0,
            flipped_decisions: 0,
        }
    }
    pub fn bump_free_decisions(&mut self) {
        self.free_decisions += 1;
    }
    pub fn bump_implied_decisions(&mut self) {
        self.implied_decisions += 1;
    }
    pub fn bump_backtracked_decisions(&mut self) {
        self.backtracked_decisions += 1;
    }
    pub fn bump_flipped_decisions(&mut self) {
        self.flipped_decisions += 1;
    }
}
