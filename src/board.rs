use crate::new_game_menu::{NewGameMenu, NewGameMenuMsg};
use crate::solver::Solver;

use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

use gloo_console::{debug, info};
use gloo_timers::callback::Interval;
use itertools::iproduct;
use rand::seq::SliceRandom;
use rand::thread_rng;
use yew::html::Scope;
use yew::{html, Component, Context, Event, Html, MouseEvent, TouchEvent};

use wasm_timer::Instant;

#[derive(Clone, PartialEq, Debug, Copy)]
enum GameState {
    InProgress,
    Won,
    Lost,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BoardCellState {
    Discovered = 0,
    Blank = 1,
    Flagged = 2,
    Question = 3,
    Exploded = 4,
    Other,
}

#[derive(Clone, Eq)]
pub struct BoardCell {
    cell: u8,
    x: usize,
    y: usize,
}

impl PartialEq for BoardCell {
    fn eq(&self, other: &Self) -> bool {
        match (self.state(), other.state()) {
            (BoardCellState::Discovered, BoardCellState::Discovered) => {
                self.value() == other.value()
            }
            (BoardCellState::Blank, BoardCellState::Blank) => true,
            (BoardCellState::Blank, BoardCellState::Flagged) => true,
            (BoardCellState::Blank, BoardCellState::Question) => true,
            (BoardCellState::Flagged, BoardCellState::Blank) => true,
            (BoardCellState::Flagged, BoardCellState::Flagged) => true,
            (BoardCellState::Flagged, BoardCellState::Question) => true,
            (BoardCellState::Question, BoardCellState::Blank) => true,
            (BoardCellState::Question, BoardCellState::Flagged) => true,
            (BoardCellState::Question, BoardCellState::Question) => true,
            (_, _) => false,
        }
    }
}

impl fmt::Display for BoardCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self.state() {
            BoardCellState::Question => String::from("?"),
            BoardCellState::Flagged => String::from("f"),
            BoardCellState::Blank => String::from(" "),
            BoardCellState::Discovered => match self.value() {
                1..=8 => self.value().to_string(),
                15 => String::from("m"),
                0 => String::from(" "),
                _ => String::from(" "),
            },
            BoardCellState::Exploded => String::from("e"),
            BoardCellState::Other => String::from("e"),
        };
        write!(f, "{}", out)
    }
}

impl fmt::Debug for BoardCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = format!("({}, {:?})", self.value(), self.state());
        write!(f, "{}", out)
    }
}

impl BoardCell {
    pub fn from_char(c: char,x:usize, y:usize) -> Self {
        if c.is_numeric() {
            Self::from_raw_parts(c.to_digit(10).unwrap() as u8, BoardCellState::Discoveredx,x,y)
        } else if c == 'm' {
            Self::from_raw_parts(15, BoardCellState::Blankx,x,y)
        } else if c == '?' {
            Self::from_raw_parts(0, BoardCellState::Blankx,x,y)
        } else {
            Self::from_raw_parts(0, BoardCellState::Otherx,x,y)
        }
    }
    fn from_raw_parts(value: u8, state: BoardCellState, x: usize, y: usize) -> Self {
        Self {
            cell: ((state as u8) << 4) + value,x,y
        }
    }
    pub fn state(&self) -> BoardCellState {
        match self.cell >> 4 {
            0 => BoardCellState::Discovered,
            1 => BoardCellState::Blank,
            2 => BoardCellState::Flagged,
            3 => BoardCellState::Question,
            4 => BoardCellState::Exploded,
            _ => BoardCellState::Other,
        }
    }
    pub fn value(&self) -> u8 {
        self.cell & ((1 << 4) - 1)
    }
    pub fn click(&mut self) -> bool {
        if self.state() == BoardCellState::Blank {
            self.cell = self.value();
            //info!(format!("{:?}", self));
            if self.value() == 0 {
                return true;
            }
        }
        false
    }

    pub fn flag(&mut self) -> i8 {
        if self.state() != BoardCellState::Discovered {
            self.cell = self.value() + (((self.state() as u8) % 3 + 1) << 4);
        }
        match self.state() {
            BoardCellState::Question => -1,
            BoardCellState::Flagged => 1,
            _ => 0,
        }
    }

    fn render(&self, link: &Scope<AppRender>) -> Html {
        let (x, y) = (self.x, self.y);
        let left_click = link.callback(move |_| AppRenderMsg::Clicked(x, y, true));
        let right_click = link.callback(move |e: MouseEvent| {
            e.prevent_default();
            AppRenderMsg::Clicked(x, y, false)
        });
        let tap_start = link.callback(move |_| AppRenderMsg::TapStart(x, y));
        let tap_end = link.callback(move |e: TouchEvent| {
            e.prevent_default();
            AppRenderMsg::TapEnd(x, y)
        });
        let prevent_select = link.callback(|e: Event| {
            e.prevent_default();
            AppRenderMsg::Nothing
        });
        let s = match self.state() {
            BoardCellState::Discovered => "cell1",
            BoardCellState::Exploded => "cell1",
            BoardCellState::Blank => "cell0",
            _ => "cell0",
        };
        html! {
            <td class={s} onclick={left_click} oncontextmenu={right_click} ontouchstart={tap_start} ontouchend={tap_end} onselectstart={prevent_select.clone()} onselect={prevent_select.clone()}>{format!("{}", self)}</td>
        }
    }
}

impl Default for BoardCell {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Board {
    board: Vec<Vec<BoardCell>>,
    pub rows: u16,
    pub cols: u16,
    pub mines: u16,
    pub game_state: GameState,
    start: bool,
    clicked_cells: usize,
    flagged_mines: i16,
    start_time: Option<Instant>,
    display_time: u16,
    flag: bool,
    solver: Option<Solver>,
}

impl Board {
    fn new(rows: u16, cols: u16, mines: u16) -> Self {
        Board {
            board: (0..rows as _)
                .map(|x| {
                    (0..cols as _)
                        .map(|y| BoardCell {
                            cell: (1 << 4),
                            x,
                            y,
                        })
                        .collect()
                })
                .collect(),
            rows,
            cols,
            mines,
            game_state: GameState::InProgress,
            start: false,
            clicked_cells: 0,
            flagged_mines: 0,
            start_time: None,
            display_time: 0,
            flag: false,
            solver: None,
        }
    }

    fn start(&mut self, x: usize, y: usize, flag: bool) {
        //populate board
        debug!("Fill Board");
        let mut rng = thread_rng();
        let _place = x * (self.cols as usize) + y;
        debug!("Create Mines");
        let mut places = iproduct!(-1..=1, -1..=1)
            .map(|(dx, dy)| (x as i32 + dx, y as i32 + dy))
            .filter(|(x, y)| 0 <= *x && *x < self.rows as i32 && 0 <= *y && *y < self.cols as i32)
            .map(|(x, y)| (x * self.cols as i32 + y) as usize)
            .collect::<Vec<usize>>();
        places.sort_unstable();
        let places = {
            let mut temp: Vec<(usize, usize)> = vec![(0, 0)];
            let (mut start, mut len, mut next) = (
                (self.rows * self.cols) as _,
                0,
                (self.rows * self.cols) as _,
            );
            for e in places {
                if e == next {
                    len += 1;
                    next += 1;
                } else {
                    if start != (self.rows * self.cols) as usize {
                        temp.push((start, len));
                    }
                    start = e;
                    len = 1;
                    next = e + 1;
                }
            }
            temp.push((start, len));
            temp.push(((self.cols * self.rows) as _, 0));
            temp
        };
        //info!(format!("{:?}", places));
        let mut pos = (0..((self.rows * self.cols) as usize
            - places.iter().fold(0, |acc, (_, x)| acc + x))) //Counting is hard
            .collect::<Vec<usize>>()
            .choose_multiple(&mut rng, self.mines as _)
            .copied()
            .collect::<Vec<usize>>();
        pos.sort_unstable();
        let mut delta = 0;
        let mut i = 0;
        let pos = pos
            .iter()
            .map(|a| {
                while places[i].0 <= (*a) + delta {
                    delta += places[i].1;
                    i += 1;
                }
                //info!(format!("{} {} {}", a, delta, i));
                (*a) + delta * (flag as usize)
            })
            .map(|a| {
                //info!(format!("a:{} x:{}", a, a/self.m));
                (a / (self.cols as usize), a % (self.cols as usize))
            })
            .collect::<Vec<(usize, usize)>>();
        //info!(format!("self.m:{}", self.m));
        //info!(format!("pos:{:?}", pos));
        debug!("Place Mines");
        for (x, y) in pos {
            self.board[x][y].cell = 15 + ((self.board[x][y].state() as u8) << 4);
            for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                let x1 = x as i32 + dx;
                let y1 = y as i32 + dy;
                if 0 <= x1 && x1 < self.rows as i32 && 0 <= y1 && y1 < self.cols as i32 {
                    let x1 = x1 as usize;
                    let y1 = y1 as usize;
                    if self.board[x1][y1].value() != 15 {
                        self.board[x1][y1].cell += 1;
                        //info!(format!("({},{}): {}", x1, y1, self.board[x1][y1].flags()),);
                    }
                }
            }
        }
        ConsoleService::debug("Finish Board Filling");

        self.solver = Solver::from_board(&self.board).into();
        self.solver.as_mut().unwrap().start();
        self.start = true;
        self.start_time = Some(Instant::now());
        debug!("Finish Board Filling");
    }

    fn flag(&mut self, x: usize, y: usize) {
        if self.board[x][y].state() == BoardCellState::Discovered {
            self.click(x, y);
        }
        if !self.start {
            self.start(x, y, false);
        }
        self.flagged_mines += self.board[x][y].flag() as i16;
    }

    fn click(&mut self, x: usize, y: usize) {
        debug!("Clicked");
        if !self.start {
            self.start(x, y, true);
        }
        let mut q = VecDeque::new();
        let mut set = HashSet::new();
        debug!("Check if flagged");
        if self.board[x][y].state() == BoardCellState::Discovered {
            let mut count = 0;
            for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                let x1 = x as i32 + dx;
                let y1 = y as i32 + dy;
                if 0 <= x1 && x1 < self.rows as i32 && 0 <= y1 && y1 < self.cols as i32 {
                    let x1 = x1 as usize;
                    let y1 = y1 as usize;
                    if self.board[x1][y1].state() == BoardCellState::Flagged {
                        count += 1;
                    }
                }
            }
            if count == self.board[x][y].value() {
                for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                    let x1 = x as i32 + dx;
                    let y1 = y as i32 + dy;
                    if 0 <= x1 && x1 < self.rows as i32 && 0 <= y1 && y1 < self.cols as i32 {
                        let x1 = x1 as usize;
                        let y1 = y1 as usize;
                        if self.board[x1][y1].state() == BoardCellState::Blank {
                            q.push_back((x1, y1));
                            set.insert((x1, y1));
                        }
                    }
                }
            }
        }
        debug!("Check if clickable");
        if self.board[x][y].flags() == 1 {
            q.push_back((x, y));
            set.insert((x, y));
        }
        debug!("Check all discovered values");
        //Maybe optimize in future
        while let Some((x, y)) = q.pop_front() {
            //BFS
            if self.board[x][y].value() == 15 {
                self.board[x][y].click();
                self.game_state = GameState::Lost;
                self.board[x][y].cell = 15 + (4 << 4);
                return;
            }
            if self.board[x][y].state() == BoardCellState::Blank {
                self.clicked_cells += 1;
            }
            if self.board[x][y].click() {
                for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                    let x1 = x as i32 + dx;
                    let y1 = y as i32 + dy;
                    if 0 <= x1 && x1 < self.rows as i32 && 0 <= y1 && y1 < self.cols as i32 {
                        let x1 = x1 as usize;
                        let y1 = y1 as usize;
                        if self.board[x1][y1].state() == BoardCellState::Blank
                            && !set.contains(&(x1, y1))
                        {
                            q.push_back((x1, y1));
                            set.insert((x1, y1));
                        }
                    }
                }
            }
        }
        debug!("Check if game is won");
        if self.clicked_cells + (self.mines as usize) == (self.cols * self.rows) as usize {
            self.game_state = GameState::Won;
        }
        debug!("Finish Click");
    }

    fn time(&self) -> u16 {
        match self.game_state {
            GameState::InProgress => (match self.start_time {
                Some(start_time) => Instant::now() - start_time,
                None => Duration::ZERO,
            })
            .as_secs()
            .try_into()
            .unwrap_or_default(),
            _ => self.display_time,
        }
    }

    fn update(&mut self) {
        self.display_time = self.time();
        if self.game_state != GameState::InProgress {
            for x in 0..self.rows as _ {
                for y in 0..self.cols as _ {
                    if self.board[x][y].value() != 15 {
                        self.board[x][y].click();
                    } else if self.game_state == GameState::Won {
                        self.board[x][y].cell = 15 + ((BoardCellState::Flagged as u8) << 4);
                    } else if self.board[x][y].state() != BoardCellState::Exploded {
                        self.board[x][y].cell = 15;
                    }
                }
            }
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new(9, 9, 10)
    }
}

#[derive(Debug)]
pub enum AppRenderMsg {
    Clicked(usize, usize, bool), //(x,y,is_left)
    Difficulty(u16, u16, u16),   //cols, rows, mines
    NewGame,
    Menu,
    ToggleFlag,
    MenuLink(Scope<NewGameMenu>),
    UpdateTime,
    TapStart(usize, usize),
    TapEnd(usize, usize),
    Nothing,
}

pub struct AppRender {
    board: Board,
    new_game_menu: Option<Scope<NewGameMenu>>,
    new_game_menu_visible: bool,
    last_tap: (usize, usize, Option<Instant>),
    _clock_updater: Interval,
}

impl Component for AppRender {
    type Message = AppRenderMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let clock_handle = {
            let link = ctx.link().clone();
            Interval::new(500, move || link.send_message(AppRenderMsg::UpdateTime))
        };
        Self {
            board: Board::default(),
            new_game_menu: None,
            new_game_menu_visible: false,
            last_tap: (0, 0, None),
            _clock_updater: clock_handle,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let debug_msg = format!("{:?}", msg);
        let time = if let Some(start_time) = self.board.start_time {
            Instant::now() - start_time
        } else {
            Duration::ZERO
        };
        debug!(format!("Starting: {} ({})", debug_msg, time.as_millis()));
        match (msg, self.board.game_state, self.new_game_menu_visible) {
            (AppRenderMsg::Clicked(x, y, flag), GameState::InProgress, false) => {
                match flag ^ self.board.flag {
                    true => self.board.click(x, y),
                    false => self.board.flag(x, y),
                }
            }
            (AppRenderMsg::NewGame, _, _) => {
                if let Some(menu) = self.new_game_menu.as_ref() {
                    menu.send_message(NewGameMenuMsg::ToggleVisibility);
                }
                self.new_game_menu_visible = true;
            }
            (AppRenderMsg::Menu, _, _) => (),
            (AppRenderMsg::ToggleFlag, _, false) => self.board.flag ^= true,
            (AppRenderMsg::MenuLink(link), _, _) => self.new_game_menu = Some(link),
            (AppRenderMsg::Difficulty(rows, cols, mines), _, true) => {
                self.board = Board::new(rows, cols, mines);
                self.new_game_menu_visible = false;
            }
            (AppRenderMsg::TapStart(x, y), GameState::InProgress, false) => {
                self.last_tap = (x, y, Some(Instant::now()));
            }
            (AppRenderMsg::TapEnd(x, y), GameState::InProgress, false) => {
                if let Some(last_time) = self.last_tap.2 {
                    let time = (Instant::now() - last_time).as_millis();
                    if x == self.last_tap.0 && y == self.last_tap.1 {
                        let flag = time < 400; //Fiddle with value
                        match flag ^ self.board.flag {
                            true => self.board.click(x, y),
                            false => self.board.flag(x, y),
                        }
                    }
                }
            }
            (_, _, _) => (),
        };
        self.board.update();
        debug!(format!("Finished: {} ({})", debug_msg, time.as_millis()));
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let restart = link.callback(move |_| AppRenderMsg::NewGame);
        let toggle_flag = link.callback(move |_| AppRenderMsg::ToggleFlag);
        let menu = link.callback(move |_| AppRenderMsg::Menu);
        let restart_button = match self.board.game_state {
            GameState::InProgress => "R",
            GameState::Won => ":)",
            GameState::Lost => ":(",
        };
        html! {
            <>
            <div style={"position: absolute; top: 10%"}>
                <div class={"title_bar"} style={format!("min-width: {}px",self.board.cols*32+2)}>
                    <div class={"item"}>
                        <div class={"button"} onclick={toggle_flag}>{"T"}</div>
                    </div>
                    <div class={"item"} style={"display: flex; flex-direction: row;"}>
                        {display(self.board.mines as i16 - self.board.flagged_mines)}
                        <div class={"button"} onclick={restart}>{restart_button}</div>
                        {display(self.board.time() as i16)}
                    </div>
                    <div class={"item"}>
                        <div class={"button"} onclick={menu}>{"S"}</div>
                    </div>
                </div>
                <div>
                    <table class={"board"}>
                        <tbody>
                        {board_display(&self.board.board, link)}
                        </tbody>
                    </table>
                </div>
            </div>
            <NewGameMenu/>
            </>
        }
    }
}

fn display(number: i16) -> Html {
    html! {
        <div class={"display"}>{format!("{:03}", number.min(999).max(-99))}</div>
    }
}

fn board_display(board: &Vec<Vec<BoardCell>>, link: &Scope<AppRender>) -> Html {
    html! {
       board
            .iter()
            .enumerate()
            .map(|(row_index,row)| {
                html! {
                    <tr>
                    {row
                        .iter()
                        .enumerate()
                        .map(|(column_index, cell)| cell.render(link))
                        .collect::<Html>()}
                    </tr>
                }
            })
            .collect::<Html>()
    }
}
