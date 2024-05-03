use core::fmt;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct SolverProfiler {
    // counters
    free_decisions: u64,
    implied_decisions: u64,
    backtracked_decisions: u64,
    flipped_decisions: u64,
    conflict_clauses: u64,

    // timers
    duration: Duration,
    start_time: Instant,
}

impl SolverProfiler {
    pub fn new() -> SolverProfiler {
        SolverProfiler {
            free_decisions: 0,
            implied_decisions: 0,
            backtracked_decisions: 0,
            flipped_decisions: 0,
            conflict_clauses: 0,
            start_time: Instant::now(),
            duration: Duration::new(0, 0),
        }
    }
    pub fn reset_start_time(&mut self) {
        self.start_time = Instant::now();
    }
    pub fn calc_duration_till_now(&mut self) {
        self.duration = Instant::now().duration_since(self.start_time);
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
    pub fn bump_conflict_clauses(&mut self) {
        self.conflict_clauses += 1;
    }
}

impl fmt::Display for SolverProfiler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "free_decisions: {}, implied_decisions: {}, backtracked_decisions: {}, flipped_decisions: {}, duration: {}us, conflict_clauses: {}", 
        self.free_decisions, self.implied_decisions, self.backtracked_decisions, self.flipped_decisions, self.duration.as_micros(), self.conflict_clauses)
    }
}
