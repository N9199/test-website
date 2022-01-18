use crate::new_game_menu::{NewGameMenu, NewGameMenuMsg};

use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

use itertools::iproduct;
use rand::seq::SliceRandom;
use rand::thread_rng;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::services::{ConsoleService, Task};
use yew::{html, Component, ComponentLink, Event, Html, MouseEvent, ShouldRender, TouchEvent};

use wasm_timer::Instant;

#[derive(Clone, PartialEq, Debug, Copy)]
enum GameState {
    InProgress,
    Won,
    Lost,
}

#[derive(Clone, PartialEq)]
struct BoardCell {
    pub cell: u8,
    x: usize,
    y: usize,
}

impl fmt::Display for BoardCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self.flags() {
            3 => String::from("?"),
            2 => String::from("f"),
            1 => String::from(" "),
            0 => match self.value() {
                1..=8 => self.value().to_string(),
                15 => String::from("m"),
                0 => String::from(" "),
                _ => String::from(" "),
            },
            _ => String::from("e"),
        };
        write!(f, "{}", out)
    }
}

impl fmt::Debug for BoardCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = format!(
            "{:?}: ({}, {})",
            (self.x, self.y),
            self.value(),
            self.flags()
        );
        write!(f, "{}", out)
    }
}

impl BoardCell {
    fn flags(&self) -> u8 {
        self.cell >> 4
    }
    fn value(&self) -> u8 {
        self.cell & ((1 << 4) - 1)
    }
    fn click(&mut self) -> bool {
        if self.flags() == 1 {
            self.cell = self.value();
            //ConsoleService::info(format!("{:?}", self).as_ref());
            if self.value() == 0 {
                return true;
            }
        }
        false
    }

    fn flag(&mut self) -> i8 {
        if self.flags() != 0 {
            self.cell = self.value() + ((self.flags() % 3 + 1) << 4);
        }
        match self.flags() {
            3 => -1,
            2 => 1,
            _ => 0,
        }
    }

    fn render(&self, link: &ComponentLink<AppRender>) -> Html {
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
        let prevent_select = link.callback( |e: Event| {
            e.prevent_default();
            AppRenderMsg::Nothing
        });
        let s = match self.flags() {
            0 => "cell1",
            4 => "cell1",
            1 => "cell0",
            _ => "cell0",
        };
        html! {
            <td class={s} onclick=left_click oncontextmenu=right_click ontouchstart=tap_start ontouchend=tap_end onselectstart=prevent_select.clone() onselect=prevent_select.clone()>{format!("{}", self)}</td>
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Board {
    board: Vec<Vec<BoardCell>>,
    pub rows: usize,
    pub cols: usize,
    pub mines: usize,
    pub game_state: GameState,
    start: bool,
    clicked_cells: usize,
    flagged_mines: i16,
    start_time: Option<Instant>,
    display_time: u16,
    flag: bool,
}

impl Board {
    fn new(rows: usize, cols: usize, mines: usize) -> Self {
        Board {
            board: (0..rows)
                .map(|x| {
                    (0..cols)
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
        }
    }

    fn start(&mut self, x: usize, y: usize, flag: bool) {
        //populate board
        ConsoleService::debug("Fill Board");
        let mut rng = thread_rng();
        let _place = x * self.cols + y;
        ConsoleService::debug("Create Mines");
        let mut places = iproduct!(-1..=1, -1..=1)
            .map(|(dx, dy)| (x as i32 + dx, y as i32 + dy))
            .filter(|(x, y)| 0 <= *x && *x < self.rows as i32 && 0 <= *y && *y < self.cols as i32)
            .map(|(x, y)| (x * self.cols as i32 + y) as usize)
            .collect::<Vec<usize>>();
        places.sort_unstable();
        let places = {
            let mut temp: Vec<(usize, usize)> = vec![(0, 0)];
            let (mut start, mut len, mut next) = (self.rows * self.cols, 0, self.rows * self.cols);
            for e in places {
                if e == next {
                    len += 1;
                    next += 1;
                } else {
                    if start != self.rows * self.cols {
                        temp.push((start, len));
                    }
                    start = e;
                    len = 1;
                    next = e + 1;
                }
            }
            temp.push((start, len));
            temp.push((self.cols * self.rows, 0));
            temp
        };
        //ConsoleService::info(format!("{:?}", places).as_ref());
        let mut pos = (0..(self.rows * self.cols - places.iter().fold(0, |acc, (_, x)| acc + x))) //Counting is hard
            .collect::<Vec<usize>>()
            .choose_multiple(&mut rng, self.mines)
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
                //ConsoleService::info(format!("{} {} {}", a, delta, i).as_ref());
                (*a) + delta * (flag as usize)
            })
            .map(|a| {
                //ConsoleService::info(format!("a:{} x:{}", a, a/self.m).as_ref());
                (a / self.cols, a % self.cols)
            })
            .collect::<Vec<(usize, usize)>>();
        //ConsoleService::info(format!("self.m:{}", self.m).as_ref());
        //ConsoleService::info(format!("pos:{:?}", pos).as_ref());
        ConsoleService::debug("Place Mines");
        for (x, y) in pos {
            self.board[x][y].cell = 15 + (self.board[x][y].flags() << 4);
            for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                let x1 = x as i32 + dx;
                let y1 = y as i32 + dy;
                if 0 <= x1 && x1 < self.rows as i32 && 0 <= y1 && y1 < self.cols as i32 {
                    let x1 = x1 as usize;
                    let y1 = y1 as usize;
                    if self.board[x1][y1].value() != 15 {
                        self.board[x1][y1].cell += 1;
                        //ConsoleService::info(format!("({},{}): {}", x1, y1, self.board[x1][y1].flags()).as_ref(),);
                    }
                }
            }
        }
        self.start = true;
        self.start_time = Some(Instant::now());
        ConsoleService::debug("Finish Board Filling");
    }

    fn flag(&mut self, x: usize, y: usize) {
        if self.board[x][y].flags() == 0 {
            self.click(x, y);
        }
        if !self.start {
            self.start(x, y, false);
        }
        self.flagged_mines += self.board[x][y].flag() as i16;
    }

    fn click(&mut self, x: usize, y: usize) {
        ConsoleService::debug("Clicked");
        if !self.start {
            self.start(x, y, true);
        }
        let mut q = VecDeque::new();
        let mut set = HashSet::new();
        ConsoleService::debug("Check if flagged");
        if self.board[x][y].flags() == 0 {
            let mut count = 0;
            for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                let x1 = x as i32 + dx;
                let y1 = y as i32 + dy;
                if 0 <= x1 && x1 < self.rows as i32 && 0 <= y1 && y1 < self.cols as i32 {
                    let x1 = x1 as usize;
                    let y1 = y1 as usize;
                    if self.board[x1][y1].flags() == 2 {
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
                        if self.board[x1][y1].flags() == 1 {
                            q.push_back((x1, y1));
                            set.insert((x1, y1));
                        }
                    }
                }
            }
        }
        ConsoleService::debug("Check if clickable");
        if self.board[x][y].flags() == 1 {
            q.push_back((x, y));
            set.insert((x, y));
        }
        ConsoleService::debug("Check all discovered values");
        //Maybe optimize in future
        while let Some((x, y)) = q.pop_front() {
            //BFS
            if self.board[x][y].value() == 15 {
                self.board[x][y].click();
                self.game_state = GameState::Lost;
                self.board[x][y].cell = 15 + (4 << 4);
                return;
            }
            if self.board[x][y].flags() == 1 {
                self.clicked_cells += 1;
            }
            if self.board[x][y].click() {
                for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                    let x1 = x as i32 + dx;
                    let y1 = y as i32 + dy;
                    if 0 <= x1 && x1 < self.rows as i32 && 0 <= y1 && y1 < self.cols as i32 {
                        let x1 = x1 as usize;
                        let y1 = y1 as usize;
                        if self.board[x1][y1].flags() == 1 && !set.contains(&(x1, y1)) {
                            q.push_back((x1, y1));
                            set.insert((x1, y1));
                        }
                    }
                }
            }
        }
        ConsoleService::debug("Check if game is won");
        if self.clicked_cells + self.mines == self.cols * self.rows {
            self.game_state = GameState::Won;
        }
        ConsoleService::debug("Finish Click");
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
            for x in 0..self.rows {
                for y in 0..self.cols {
                    if self.board[x][y].value() != 15 {
                        self.board[x][y].click();
                    } else if self.game_state == GameState::Won {
                        self.board[x][y].cell = 15 + (2 << 4);
                    } else if self.board[x][y].flags() != 4 {
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
    Clicked(usize, usize, bool),     //(x,y,is_left)
    Difficulty(usize, usize, usize), //cols, rows, mines
    NewGame,
    Menu,
    ToggleFlag,
    MenuLink(Rc<ComponentLink<NewGameMenu>>),
    UpdateTime,
    TapStart(usize, usize),
    TapEnd(usize, usize),
    Nothing,
}

pub struct AppRender {
    link: Rc<ComponentLink<Self>>,
    board: Board,
    new_game_menu: Option<Rc<ComponentLink<NewGameMenu>>>,
    new_game_menu_visible: bool,
    last_tap: (usize, usize, Option<Instant>),
    _clock_updater: IntervalTask,
}

impl Component for AppRender {
    type Message = AppRenderMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let clock_handle = IntervalService::spawn(
            Duration::from_millis(500),
            link.callback(|_| AppRenderMsg::UpdateTime),
        );
        Self {
            link: Rc::new(link),
            board: Board::default(),
            new_game_menu: None,
            new_game_menu_visible: false,
            last_tap: (0, 0, None),
            _clock_updater: clock_handle,
        }
    }
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let debug_msg = format!("{:?}", msg);
        let time = if let Some(start_time) = self.board.start_time {
            Instant::now() - start_time
        } else {
            Duration::ZERO
        };
        ConsoleService::debug(format!("Starting: {} ({})", debug_msg, time.as_millis()).as_ref());

        match (msg, self.board.game_state, self.new_game_menu_visible) {
            (AppRenderMsg::Clicked(x, y, flag), GameState::InProgress, false) => {
                match flag ^ self.board.flag {
                    true => self.board.click(x, y),
                    false => self.board.flag(x, y),
                }
            }
            (AppRenderMsg::NewGame, _, _) => {
                if let Some(menu) = &self.new_game_menu {
                    let toggle_menu = menu.callback(|_| NewGameMenuMsg::ToggleVisibility);
                    toggle_menu.emit("");
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
        ConsoleService::debug(format!("Finished: {} ({})", debug_msg, time.as_millis()).as_ref());
        true
    }

    fn view(&self) -> Html {
        let restart = self.link.callback(move |_| AppRenderMsg::NewGame);
        let toggle_flag = self.link.callback(move |_| AppRenderMsg::ToggleFlag);
        let menu = self.link.callback(move |_| AppRenderMsg::Menu);
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
                        <div class={"button"} onclick=toggle_flag>{"T"}</div>
                    </div>
                    <div class={"item"} style={"display: flex; flex-direction: row;"}>
                        {display(self.board.mines as i16 - self.board.flagged_mines)}
                        <div class={"button"} onclick=restart>{restart_button}</div>
                        {display(self.board.time() as i16)}
                    </div>
                    <div class={"item"}>
                        <div class={"button"} onclick=menu>{"S"}</div>
                    </div>
                </div>
                <div>
                    <table class={"board"}>
                        <tbody>
                        {board_display(&self.board.board, &self.link)}
                        </tbody>
                    </table>
                </div>
            </div>
            <NewGameMenu par_link=Rc::downgrade(&self.link)/>
            </>
        }
    }
}

fn display(number: i16) -> Html {
    html! {
        <div class={"display"}>{format!("{:03}", number.min(999).max(-99))}</div>
    }
}

fn board_display(board: &Vec<Vec<BoardCell>>, link: &ComponentLink<AppRender>) -> Html {
    html! {
       board
            .iter()
            .map(|row| {
                html! {
                    <tr>
                    {row
                        .iter()
                        .map(|cell| cell.render(link))
                        .collect::<Html>()}
                    </tr>
                }
            })
            .collect::<Html>()
    }
}
