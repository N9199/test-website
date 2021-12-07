use itertools::iproduct;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use std::fmt;
use yew::services::ConsoleService;
use yew::{html, html::ImplicitClone, Component, ComponentLink, Html, Properties, ShouldRender};

use yew::web_sys::MouseEvent;

use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub enum Msg {
    LeftClicked(usize, usize),
    RightClicked(usize, usize),
    Restart,
}

#[derive(Clone, PartialEq, Deserialize, Debug, Copy)]
pub enum GameState {
    InProgress,
    Won,
    Lost,
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <BoardRender />
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct Board {
    board: Vec<Vec<BoardCell>>,
    pub n: usize,
    pub m: usize,
    pub mines: usize,
    pub game_state: GameState,
    start: bool,
    clicked_cells: usize,
    flagged_mines: i16,
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
    fn left_click(&mut self) -> bool {
        if self.flags() == 1 {
            self.cell = self.value();
            ConsoleService::info(format!("{:?}", self).as_ref());
            if self.value() == 0 {
                return true;
            }
        }
        false
    }

    fn right_click(&mut self) -> i8 {
        if self.flags() != 0 {
            self.cell = self.value() + ((self.flags() % 3 + 1) << 4);
        }
        match self.flags() {
            3 => -1,
            2 => 1,
            _ => 0,
        }
    }

    fn render(&self, link: &ComponentLink<BoardRender>) -> Html{
        let (x,y) = (self.x,self.y);
        let left_click = link.callback(move |_|Msg::LeftClicked(x,y));
        let right_click = link.callback(move |e: MouseEvent| {e.prevent_default(); Msg::RightClicked(x,y)});
        let s = match self.flags(){
            0 => "cell1",
            4 => "cell1",
            1 => "cell0",
            _ => "cell0",
        };
        html!{
            <td class={s} onclick=left_click oncontextmenu=right_click id={"noContextMenu"}>{format!("{}", self)}</td>
        }
    }
}

impl Board {
    fn new(n: usize, m: usize, mines: usize) -> Self {
        Board {
            board: (0..n)
                .map(|x| {
                    (0..m)
                        .map(|y| BoardCell {
                            cell: (1 << 4),
                            x,
                            y,
                        })
                        .collect()
                })
                .collect(),
            n,
            m,
            mines,
            game_state: GameState::InProgress,
            start: false,
            clicked_cells: 0,
            flagged_mines: 0,
        }
    }

    fn start(&mut self, x: usize, y: usize, flag: bool) {
        //populate board
        let mut rng = rand::thread_rng();
        let place = x * self.m + y;
        let pos: Vec<(usize, usize)> = (0..(self.n * self.m - 1)) //Counting is hard
            .collect::<Vec<usize>>()
            .choose_multiple(&mut rng, self.mines)
            .map(|a| (*a) + ((a >= &place) && flag) as usize)
            .map(|a| (a / self.m, a % self.m))
            .collect();
        //ConsoleService::info(format!("{:?}", pos).as_ref());
        for (x, y) in pos {
            ConsoleService::info(format!("{} {}", x, y).as_ref());
            self.board[x][y].cell = 15 + (self.board[x][y].flags() << 4);
            for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                let x1 = x as i32 + dx;
                let y1 = y as i32 + dy;
                if 0 <= x1 && x1 < self.n as i32 && 0 <= y1 && y1 < self.m as i32 {
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
    }

    fn left_click(&mut self, x: usize, y: usize) {
        ConsoleService::info(format!("{:?}", self.game_state).as_ref());
        if self.start == false {
            self.start(x, y, true);
        }
        let mut q: VecDeque<(usize, usize)> = VecDeque::new();
        if self.board[x][y].flags() == 0 {
            let mut count = 0;
            for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                let x1 = x as i32 + dx;
                let y1 = y as i32 + dy;
                if 0 <= x1 && x1 < self.n as i32 && 0 <= y1 && y1 < self.m as i32 {
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
                    if 0 <= x1 && x1 < self.n as i32 && 0 <= y1 && y1 < self.m as i32 {
                        let x1 = x1 as usize;
                        let y1 = y1 as usize;
                        if self.board[x1][y1].flags() == 1 {
                            q.push_back((x1, y1));
                        }
                    }
                }
            }
        }
        if self.board[x][y].flags() == 1 {
            q.push_back((x, y));
        }
        //Maybe optimize in future
        while let Some((x, y)) = q.pop_front() {
            if self.board[x][y].value() == 15 {
                self.board[x][y].left_click();
                self.game_state = GameState::Lost;
                self.board[x][y].cell = 15 + (4 << 4);
                return;
            }
            if self.board[x][y].flags() == 1 {
                self.clicked_cells += 1;
            }
            if self.board[x][y].left_click() {
                for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                    let x1 = x as i32 + dx;
                    let y1 = y as i32 + dy;
                    if 0 <= x1 && x1 < self.n as i32 && 0 <= y1 && y1 < self.m as i32 {
                        let x1 = x1 as usize;
                        let y1 = y1 as usize;
                        if self.board[x1][y1].flags() == 1 {
                            q.push_back((x1, y1));
                        }
                    }
                }
            }
        }
        if self.clicked_cells + self.mines == self.m * self.n {
            self.game_state = GameState::Won;
            return;
        }
    }

    fn right_click(&mut self, x: usize, y: usize) {
        if self.start == false {
            self.start(x, y, false);
        }
        self.flagged_mines += self.board[x][y].right_click() as i16;
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new(9, 9, 10)
    }
}

#[derive(Clone)]
struct BoardRender {
    link: ComponentLink<Self>,
    board: Board,
}

impl Component for BoardRender {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            board: Board::default(),
        }
    }
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        ConsoleService::info(format!("{:?}", msg).as_ref());

        match (msg, self.board.game_state) {
            (Msg::LeftClicked(x, y), GameState::InProgress) => self.board.left_click(x, y),
            (Msg::RightClicked(x, y), GameState::InProgress) => self.board.right_click(x, y),
            (Msg::Restart, _) => {
                self.board = Board::default();
            }
            (_, _) => (),
        };

        if self.board.game_state != GameState::InProgress {
            for x in 0..self.board.n {
                for y in 0..self.board.m {
                    if self.board.board[x][y].value() != 15 {
                        self.board.board[x][y].left_click();
                    } else if self.board.game_state == GameState::Won {
                        self.board.board[x][y].cell = 15 + (2 << 4);
                    } else if self.board.board[x][y].flags() != 4 {
                        self.board.board[x][y].cell = 15;
                    }
                }
            }
        }
        true
    }

    fn view(&self) -> Html {
        let restart = self.link.callback(move |_| Msg::Restart);
        html! {
            <>
            <div>
                <a class={"button"} onclick=restart>{"R"}</a>
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
                                    .map(|bcell| bcell.render(&(self.link)))
                                    .collect::<Html>()}
                                </tr>
                            }
                        })
                        .collect::<Html>()}
                    </tbody>
                </table>
            </div>
            </>
        }
    }
}
