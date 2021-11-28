use std::cmp::Ordering;
// Note items in board are in range [1,64], if an item is 0, then it's treated as a blank space.
// 1 must always be part of the original input

type Pos = (usize, usize);

pub struct Board {
    pub board: [[u8; 8]; 8],
    rsum: [u16; 8],
    csum: [u16; 8],
    cfree: [u8; 8],
    rfree: [u8; 8],
}

impl Board {
    fn new() {}

    fn update(self: &mut Self, val: u8, (x, y): Pos, flag: bool) {
        if flag {
            return;
        }
        self.rsum[y] += val as u16;
        self.csum[x] += val as u16;
        self.rfree[x] -= 1;
        self.cfree[y] -= 1;
        self.board[x][y] = val;
    }
    fn undo(self: &mut Self, val: u8, (x, y): Pos, flag: bool) {
        if flag {
            return;
        }
        self.rsum[y] -= val as u16;
        self.csum[x] -= val as u16;
        self.rfree[x] += 1;
        self.cfree[y] += 1;
        self.board[x][y] = 0;
    }
    pub fn set(self: &mut Self, val: u8, x: usize, y: usize) {
        self.update(val, (x, y), false);
    }
}
const SUM: u16 = 260;
const REM_SUM: [u16; 8] = [64, 127, 189, 250, 310, 369, 427, 484]; //Someday it will be explicit in construction with const fn

fn check(curr_board: &Board, (x, y): (i16, i16), next: u8) -> bool {
    if x < 0 || x > 7 || y < 0 || y > 7 {
        return false;
    }
    let (x, y): Pos = (x as usize, y as usize);
    if curr_board.board[x][y] != 0 && curr_board.board[x][y] != next {
        return false;
    }
    let next = next as u16;
    if curr_board.csum[x] + next > SUM
        || curr_board.rsum[y] + next > SUM
        || curr_board.csum[x] + next + REM_SUM[curr_board.rfree[x] as usize] < SUM
        || curr_board.rsum[y] + next + REM_SUM[curr_board.cfree[y] as usize] < SUM
    {
        return false;
    }
    true
}

fn backtracking(curr_board: &mut Board, (x, y): Pos) -> bool {
    let next = curr_board.board[x][y] + 1;
    if next > 64 {
        return true; //Some flag or something to make it so it stops.
    }
    let (dx, dy): (i16, i16) = (1, 2);
    for _ in 0..4 {
        let (n_x, n_y) = (x as i16 + dx, y as i16 + dy);
        if check(curr_board, (n_x, n_y), next) {
            let (n_x, n_y) = (n_x as usize, n_y as usize);
            let flag = curr_board.board[n_x][n_y] == next;
            curr_board.update(next, (n_x, n_y), flag);
            if backtracking(curr_board, (n_x, n_y)) {
                return true;
            }
            curr_board.undo(next, (n_x, n_y), flag);
        }
        let (dx, dy) = (-dy, dx);
        let (a, b) = match 0.cmp(&(dx * dy)) {
            Ordering::Less => (1, -1),
            Ordering::Greater => (-1, 1),
            Ordering::Equal => (1, 1),
        };
        let (n_x, n_y) = (x as i16 + a * dx, y as i16 + b * dy);
        if check(curr_board, (n_x, n_y), next) {
            let (n_x, n_y) = (n_x as usize, n_y as usize);
            let flag = curr_board.board[n_x][n_y] == next;
            curr_board.update(next, (n_x, n_y), flag);
            if backtracking(curr_board, (n_x, n_y)) {
                return true;
            }
            curr_board.undo(next, (n_x, n_y), flag);
        }
    }
    false
}
