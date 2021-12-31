use crate::new_game_menu::{NewGameMenu, NewGameMenuMsg};

use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::rc::Rc;

use itertools::iproduct;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use yew::services::ConsoleService;
use yew::virtual_dom::VChild;
use yew::web_sys::MouseEvent;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

//use instant::Instant;

#[derive(Clone, PartialEq, Deserialize, Debug, Copy)]
enum GameState {
    InProgress,
    Won,
    Lost,
}

#[derive(Clone, PartialEq, Deserialize)]
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
        let s = match self.flags() {
            0 => "cell1",
            4 => "cell1",
            1 => "cell0",
            _ => "cell0",
        };
        html! {
            <td class={s} onclick=left_click oncontextmenu=right_click>{format!("{}", self)}</td>
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
struct Board {
    board: Vec<Vec<BoardCell>>,
    pub rows: usize,
    pub cols: usize,
    pub mines: usize,
    pub game_state: GameState,
    start: bool,
    clicked_cells: usize,
    flagged_mines: i16,
    //#[serde(skip_deserializing)]
    //start_time: Option<Instant>,
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
            //start_time: None,
            display_time: 0,
            flag: false,
        }
    }

    fn reset(&mut self){
        for x in 0..self.rows{
            for y in 0..self.cols{
                self.board[x][y].cell = 1<<4;
            }
        }
        self.game_state = GameState::InProgress;
        self.start = false;
        self.clicked_cells = 0;
        self.flagged_mines = 0;
        self.display_time = 0;
        self.flag = false;
    }

    fn start(&mut self, x: usize, y: usize, flag: bool) {
        //populate board
        let mut rng = thread_rng();
        let _place = x * self.cols + y;
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
        //self.start_time = Some(Instant::now());
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
        if !self.start {
            self.start(x, y, true);
        }
        let mut q = VecDeque::new();
        let mut set = HashSet::new();
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
        if self.board[x][y].flags() == 1 {
            q.push_back((x, y));
            set.insert((x, y));
        }
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

        if self.clicked_cells + self.mines == self.cols * self.rows {
            self.game_state = GameState::Won;
        }
    }

    fn time(&self) -> u16 {
        match self.game_state {
            GameState::InProgress => 0, //(self.start_time.unwrap_or(Instant::now())-Instant::now()).as_secs(),
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
    Restart,
    Menu,
    ToggleFlag,
    MenuLink(Rc<ComponentLink<NewGameMenu>>),
}

#[derive(Clone)]
pub struct AppRender {
    link: Rc<ComponentLink<Self>>,
    board: Board,
    new_game_menu: Option<Rc<ComponentLink<NewGameMenu>>>,
}

impl Component for AppRender {
    type Message = AppRenderMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: Rc::new(link),
            board: Board::default(),
            new_game_menu: None,
        }
    }
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        ConsoleService::info(format!("{:?}", msg).as_ref());

        match (msg, self.board.game_state) {
            (AppRenderMsg::Clicked(x, y, flag), GameState::InProgress) => {
                match flag ^ self.board.flag {
                    true => self.board.click(x, y),
                    false => self.board.flag(x, y),
                }
            }
            (AppRenderMsg::Restart, _) => self.board.reset(),
            (AppRenderMsg::Menu, _) => {
                if let Some(menu) = &self.new_game_menu {
                    let toggle_menu = menu.callback(|_| NewGameMenuMsg::ToggleVisibility);
                    toggle_menu.emit("");
                }
            }
            (AppRenderMsg::ToggleFlag, _) => self.board.flag ^= true,
            (AppRenderMsg::MenuLink(link), _) => self.new_game_menu = Some(link),
            (AppRenderMsg::Difficulty(rows, cols, mines), _) => {
                self.board = Board::new(rows, cols, mines)
            }
            (_, _) => (),
        };
        self.board.update();
        true
    }

    fn view(&self) -> Html {
        let restart = self.link.callback(move |_| AppRenderMsg::Restart);
        let toggle_flag = self.link.callback(move |_| AppRenderMsg::ToggleFlag);
        let menu = self.link.callback(move |_| AppRenderMsg::Menu);
        html! {
            <>
            <div style={"position: absolute; top: 30%"}>
                <div class={"title_bar"} style={format!("min-width: {}px",self.board.cols*32+2)}>
                    <div class={"item"}>
                        <span class={"button"} onclick=toggle_flag>{"T"}</span>
                    </div>
                    <div class={"item"}>
                        <Display number = self.board.mines as i16 - self.board.flagged_mines/>
                        <span class={"button"} onclick=restart>{"R"}</span>
                        <Display number = self.board.display_time as i16/>
                    </div>
                    <div class={"item"}>
                        <span class={"button"} onclick=menu>{"S"}</span>
                    </div>
                </div>
                <div>
                    <table class={"board"}>
                        <tbody>
                        {self.board
                            .board
                            .iter()
                            .map(|row| {
                                html! {
                                    <tr>
                                    {row
                                        .iter()
                                        .map(|cell| cell.render(&(self.link)))
                                        .collect::<Html>()}
                                    </tr>
                                }
                            })
                            .collect::<Html>()}
                        </tbody>
                    </table>
                </div>
            </div>
            <NewGameMenu par_link=self.link.clone()/>
            </>
        }
    }
}

#[derive(PartialEq, Clone, Properties)]
pub struct DisplayProps {
    number: i16,
}

pub struct Display {
    props: DisplayProps,
}

impl Component for Display {
    type Message = ();
    type Properties = DisplayProps;
    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self) -> Html {
        html! {
            <span class={"display"}>{format!("{:03}", self.props.number.min(999).max(-99))}</span>
        }
    }
}
