use itertools::iproduct;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use std::fmt;
use yew::prelude::*;
use yew::services::ConsoleService;
use yew::{html, html::ImplicitClone, Component, ComponentLink, Html, Properties, ShouldRender};

use std::collections::VecDeque;

#[derive(Debug)]
enum Msg {
    Clicked(usize, usize),
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub enum GameState{
    InProgess,
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

#[derive(Clone, PartialEq, Deserialize)]
pub struct Board {
    board: Vec<Vec<BoardCell>>,
    pub n: usize,
    pub m: usize,
    pub mines: usize,
    pub game_state: GameState,
    start: bool,
    clicked_cells: usize,
    flagged_mines: usize,
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
            8 => String::from("?"),
            4 => String::from("f"),
            2 => String::from(" "),
            1 => match self.value() {
                1..=8 => self.value().to_string(),
                15 => String::from("m"),
                0 => String::from("b"),
                _ => String::from(" "),
            },
            _ => String::from("e"),
        };
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
        //ConsoleService::info(format!("{}, {}", self.flags(), self.value()).as_ref());
        if self.flags() == 2 {
            self.cell &= !(1 << 5);
            self.cell |= 1 << 4;
            if self.value() == 0 {
                return true;
            }
        }
        //ConsoleService::info(format!("{}, {}", self.flags(), self.value()).as_ref());
        false
    }
}

impl Board {
    fn new(n: usize, m: usize, mines: usize) -> Self {
        Board {
            board: (0..n)
                .map(|x| {
                    (0..m)
                        .map(|y| BoardCell {
                            cell: (1 << 5),
                            x,
                            y,
                        })
                        .collect()
                })
                .collect(),
            n,
            m,
            mines,
            game_state: GameState::InProgess,
            start: false,
            clicked_cells: 0,
            flagged_mines: 0,
        }
    }

    fn click(&mut self, x: usize, y: usize) {
        ConsoleService::info(format!("{:?}", self.game_state).as_ref());
        if self.start == false {
            //populate board
            let mut rng = rand::thread_rng();
            let place = x * self.m + y;
            let pos: Vec<(usize, usize)> = (0..(self.n * self.m))
                .collect::<Vec<usize>>()
                .choose_multiple(&mut rng, self.mines)
                .map(|a| (*a) + (a >= &place) as usize)
                .map(|a| (a / self.m, a % self.m))
                .collect();
            //ConsoleService::info(format!("{:?}", pos).as_ref());
            for (x, y) in pos {
                self.board[x][y].cell |= 15;
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
        let mut q: VecDeque<(usize, usize)> = VecDeque::new();
        q.push_back((x, y));
        //Maybe optimize in future
        while let Some((x, y)) = q.pop_front() {
            if self.board[x][y].value() == 15{
                self.board[x][y].click();
                self.game_state = GameState::Lost;
                return;
            }
            if self.board[x][y].flags() == 2 {
                self.clicked_cells += 1;
            }
            if self.board[x][y].click() {
                for (dx, dy) in iproduct!(-1..=1, -1..=1) {
                    let x1 = x as i32 + dx;
                    let y1 = y as i32 + dy;
                    if 0 <= x1 && x1 < self.n as i32 && 0 <= y1 && y1 < self.m as i32 {
                        let x1 = x1 as usize;
                        let y1 = y1 as usize;
                        if self.board[x1][y1].flags() == 2 {
                            q.push_back((x1, y1));
                        }
                    }
                }
            }
        }
        if self.clicked_cells +self.mines == self.m*self.n{
            self.game_state = GameState::Won;
            return;
        }
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
        match msg {
            Msg::Clicked(x, y) => self.board.click(x, y),
        };
        true
    }

    fn view(&self) -> Html {
        html! {
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
                                .map(|bcell| {
                                    let (x,y) = (bcell.x,bcell.y);
                                    let cb = self.link.callback(move |_|Msg::Clicked(x,y));
                                    html!{
                                        <td class={"cell0"} onclick=cb>{format!("{}", bcell)}</td>
                                    }
                                })
                                .collect::<Html>()}
                            </tr>
                        }
                    })
                    .collect::<Html>()}
                </tbody>
            </table>
        }
    }
}
