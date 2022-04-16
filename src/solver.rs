use crate::board::{BoardCell, BoardCellState};
use arrayvec::ArrayVec;
use gloo_console::info;
use lazy_static::lazy_static;
use nalgebra::{ArrayStorage, Matrix, Matrix2};

use std::{
    collections::HashSet,
    iter::zip,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::channel,
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub enum SolvedCellState {
    Mine,
    Blank,
    Unknown,
}

#[derive(Debug)]
pub struct Solver {
    board: Arc<RwLock<SolvableBoard>>,
}

const SOLVER_THREADS_AMOUNT: usize = 1;
impl Solver {
    pub fn from_board(board: &Vec<Vec<BoardCell>>) -> Self {
        Self {
            board: Arc::new(RwLock::new(SolvableBoard::from_board(board))),
        }
    }

    pub fn start(&mut self) {
        rayon::spawn({
            let board = self.board.clone();
            move || {
                info!("Start of Main Solver Thread");
                let (sender, receiver) = channel();
                let live_solvers = Arc::new(AtomicUsize::new(0));
                loop {
                    info!("Main Solver Thread: Loop beginning");
                    if Arc::strong_count(&board) == 1 {
                        return;
                    }
                    if live_solvers.load(Ordering::Relaxed) == 0 {
                        let local_sender = sender.clone();
                        let local_board = board.read().unwrap().board.clone();
                        for _ in 0..SOLVER_THREADS_AMOUNT {
                            rayon::spawn({
                                let thread_sender = local_sender.clone();
                                let thread_board = local_board.clone();
                                let live_solvers_clone = live_solvers.clone();
                                move || {
                                    let count = live_solvers_clone.fetch_add(1, Ordering::Relaxed);
                                    info!(format!("Solver Thread #{count}"));
                                    let solver = SolverBoard::from_board(thread_board);
                                    thread_sender.send(solver.solve()).unwrap();
                                    live_solvers_clone.fetch_sub(1, Ordering::Relaxed);
                                }
                            });
                        }
                    }
                    let curr = receiver.recv().unwrap();
                    if curr.iter().any(|curr| curr.is_confirmed()) {
                        board
                            .write()
                            .unwrap()
                            .extend(curr.into_iter().filter(|x| x.is_confirmed()));
                    } else {
                        todo!()
                    }
                }
            }
        });
    }

    pub fn update(&mut self, board: &Vec<Vec<BoardCell>>) {}

    pub fn get_cell_state(&self, x: usize, y: usize) -> SolvedCellState {
        let board = self.board.read().unwrap();
        if board.verified.contains(&(x, y)) {
            match board.board[x][y].state() {
                BoardCellState::Discovered => SolvedCellState::Blank,
                BoardCellState::Flagged => SolvedCellState::Mine,
                _ => SolvedCellState::Unknown,
            }
        } else {
            SolvedCellState::Unknown
        }
    }

    pub fn is_knowable(&self) -> bool {
        self.board.read().unwrap().has_new_info
    }
}

#[derive(Debug)]
struct SolvableBoard {
    board: Vec<Vec<BoardCell>>,
    verified: HashSet<(usize, usize)>,
    has_new_info: bool,
}

impl SolvableBoard {
    fn aux_get_verified(board: &Vec<Vec<BoardCell>>) -> HashSet<(usize, usize)> {
        board
            .iter()
            .enumerate()
            .flat_map(|(x, row)| row.iter().enumerate().map(move |(y, cell)| (x, y, cell)))
            .filter_map(|(x, y, cell)| match cell.state() {
                BoardCellState::Discovered => Some((x, y)),
                _ => None,
            })
            .collect()
    }
    pub fn from_board(board: &Vec<Vec<BoardCell>>) -> Self {
        Self {
            board: board.clone(),
            verified: SolvableBoard::aux_get_verified(board),
            has_new_info: false,
        }
    }

    pub fn update(&mut self, board: &Vec<Vec<BoardCell>>) {
        self.board = board.clone();
        self.verified
            .extend(SolvableBoard::aux_get_verified(board).into_iter());
    }

    pub fn extend(&mut self, iter: impl Iterator<Item = Answer>) {
        iter.for_each(|action| match action {
            Answer::Mine(x, y) => {
                self.board[x][y].flag();
                self.verified.insert((x, y));
            }
            Answer::Blank(x, y) => {
                self.board[x][y].click();
                self.verified.insert((x, y));
            }
            _ => (),
        })
    }
}

#[derive(PartialEq, Eq)]
enum Answer {
    Mine(usize, usize),
    Blank(usize, usize),
    MaybeMine(usize, usize),
    MaybeBlank(usize, usize),
}

impl Answer {
    pub fn is_confirmed(&self) -> bool {
        match self {
            Answer::Mine(_, _) => true,
            Answer::Blank(_, _) => true,
            Answer::MaybeMine(_, _) => false,
            Answer::MaybeBlank(_, _) => false,
        }
    }
}

#[derive(Debug)]
struct SolverBoard {
    board: Vec<Vec<BoardCell>>,
}
const MAX_STATE_SIZE: usize = 5;
type State = ArrayVec<ArrayVec<BoardCell, MAX_STATE_SIZE>, MAX_STATE_SIZE>;
fn string_to_state(state: &str, height: usize, length: usize) -> State {
    assert!(state.chars().count() == height * (length + 1) - 1);
    let mut out: State = ArrayVec::new_const();
    let mut temp = ArrayVec::new_const();
    for (i, cell) in state.chars().enumerate() {
        if i % length == 0 {
            out.push(temp.take());
        } else {
            temp.push(BoardCell::from_char(cell, i % length, i / length));
        }
    }
    out
}
const STATES_LEN: usize = 1;
fn solvable_states() -> [(State, State); STATES_LEN] {
    let states = [("000 121 ???", "000 121 m0m", 3, 3)];
    assert!(states.len() == STATES_LEN);
    states.map(|(a, b, height, length)| {
        (
            string_to_state(a, height, length),
            string_to_state(b, height, length),
        )
    })
}

lazy_static! {
    static ref CACHED_SOLVABLE_STATES: [(State, State); STATES_LEN] = solvable_states();
}

impl SolverBoard {
    fn from_board(board: Vec<Vec<BoardCell>>) -> Self {
        Self { board }
    }

    fn solve(&self) -> Vec<Answer> {
        let mut out = Vec::new();
        out.extend(
            // First Pass
            CACHED_SOLVABLE_STATES.iter().flat_map(|(input, output)| {
                let (length, height) = (input.len(), input.first().unwrap().len());
                let (limit_x, limit_y) = (
                    self.board.len().checked_sub(length),
                    self.board.first().unwrap().len().checked_sub(height),
                );
                if limit_x.is_none() || limit_y.is_none() {
                    return Vec::new();
                }
                let (limit_x, limit_y) = (limit_x.unwrap(), limit_y.unwrap());
                let mut flag = true;
                let mut out = Vec::new();
                let mut curr_rot = Matrix2::identity();
                let rot = Matrix::from_array_storage(ArrayStorage([[0, 1], [-1, 0]]));
                let delta =
                    Matrix::from_array_storage(ArrayStorage([[length as i32, height as i32]]));
                for (x, y) in zip(0..limit_x, 0..limit_y) {
                    for _ in 0..4 {
                        for (dx, row) in input.iter().enumerate() {
                            for (dy, cell) in row.iter().enumerate() {
                                let mut vector = Matrix::from_array_storage(ArrayStorage([[
                                    dx as i32, dy as i32,
                                ]]));
                                vector *= 2;
                                vector -= delta;
                                vector = curr_rot * vector;
                                vector += delta;
                                vector /= 2;
                                let (dx, dy) = (vector[0] as usize, vector[1] as usize);
                                flag &= *cell == self.board[x + dx][y + dy];
                                if !flag {
                                    break;
                                }
                            }
                            if !flag {
                                break;
                            }
                        }
                        if flag {
                            for (dx, row) in output.iter().enumerate() {
                                for (dy, cell) in row.iter().enumerate() {
                                    if input[dx][dy].state() == BoardCellState::Blank {
                                        match cell.value() {
                                            15 => out.push(Answer::Mine(x + dx, y + dy)),
                                            0..=8 => out.push(Answer::Blank(x + dx, y + dy)),
                                            _ => (),
                                        }
                                    }
                                }
                            }
                        }
                        curr_rot *= rot;
                    }
                    curr_rot = Matrix2::identity();
                }
                out
            }),
        );
        out
    }
}
