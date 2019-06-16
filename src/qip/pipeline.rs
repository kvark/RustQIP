extern crate num;

use std::collections::{BinaryHeap, VecDeque};

use num::complex::Complex;

use super::qubits::*;
use super::state_ops::*;

pub type StateBuilder<QS: QuantumState> = fn(Vec<&Qubit>) -> QS;
pub type MeasuredResultReference = u32;

pub trait QuantumState {
    // Function to mutate self into the state with op applied.
    fn apply_op(&mut self, op: &QubitOp);
}

pub struct LocalQuantumState {
    // A bundle with the quantum state data.
    n: u64,
    state: Vec<Complex<f64>>,
    arena: Vec<Complex<f64>>,
}

impl LocalQuantumState {
    fn new(n: u64) -> LocalQuantumState {
        let mut cvec: Vec<Complex<f64>> = (0..(1 << n)).map(|_| Complex::<f64> {
            re: 0.0,
            im: 0.0,
        }).collect();
        cvec[0].re = 1.0;

        LocalQuantumState {
            n,
            state: cvec.clone(),
            arena: cvec,
        }
    }
}

impl QuantumState for LocalQuantumState {
    fn apply_op(&mut self, op: &QubitOp) {
        apply_op(self.n, op, &self.state, &mut self.arena, 0, 0, self.n > PARALLEL_THRESHOLD);
        std::mem::swap(&mut self.state, &mut self.arena);
    }
}

fn fold_apply_op<QS: QuantumState>(mut s: QS, op: &QubitOp) -> QS {
    s.apply_op(op);
    s
}

pub fn run(q: &Qubit) -> LocalQuantumState {
    run_with_state(q, |qs| {
        let n: u64 = qs.iter().map(|q| q.indices.len() as u64).sum();
        LocalQuantumState::new(n)
    })
}

pub fn run_with_state<QS: QuantumState>(q: &Qubit, state_builder: StateBuilder<QS>) -> QS {
    let (frontier, ops) = get_opfns_and_frontier(q);
    let initial_state = state_builder(frontier);
    ops.into_iter().fold(initial_state, fold_apply_op)
}

fn get_opfns_and_frontier(q: &Qubit) -> (Vec<&Qubit>, Vec<&QubitOp>) {
    let mut heap = BinaryHeap::new();
    heap.push(q);
    let mut frontier_qubits: Vec<&Qubit> = vec![];
    let mut fn_queue = VecDeque::new();
    while heap.len() > 0 {
        if let Some(q) = heap.pop() {
            match &q.parent {
                Some(parent) => {
                    match &parent {
                        Parent::Owned(parents, op) => {
                            // This fixes linting issues.
                            let parents: &Vec<Qubit> = parents;
                            let op: &Option<QubitOp> = op;
                            if let Some(op) = op {
                                fn_queue.push_front(op);
                            }
                            heap.extend(parents.iter());
                        }
                        Parent::Shared(parent) => {
                            let parent = parent.as_ref();
                            if !qubit_in_heap(parent, &heap) {
                                heap.push(parent);
                            }
                        }
                    }
                }
                None => frontier_qubits.push(q)
            }
        }
    }
    (frontier_qubits, fn_queue.into_iter().collect())
}

fn qubit_in_heap(q: &Qubit, heap: &BinaryHeap<&Qubit>) -> bool {
    for hq in heap {
        if hq == &q {
            return true;
        }
    }
    false
}