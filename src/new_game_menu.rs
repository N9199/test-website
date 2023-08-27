use crate::board::{AppRender, AppRenderMsg};

use std::fmt;

use gloo_console::log;
use js_sys::Reflect;
use strum::{EnumIter, IntoEnumIterator};
use wasm_bindgen::JsValue;
use web_sys::Event;
use yew::{html, Component, Context, Html};

pub enum NewGameMenuMsg {
    ToggleVisibility,
    Difficulty(Difficulty),
    Rows(u16),
    Cols(u16),
    Mines(u16),
}

#[derive(Copy, Clone, Debug, EnumIter, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
    Custom,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Difficulty {
    const fn value(&self) -> (u16, u16, u16) {
        match self {
            Difficulty::Easy => (9, 9, 10),
            Difficulty::Medium => (16, 16, 40),
            Difficulty::Hard => (16, 30, 99),
            Difficulty::Extreme => (24, 30, 180),
            Difficulty::Custom => (0, 0, 0),
        }
    }
}

pub struct NewGameMenu {
    visible: bool,
    rows: u16,
    cols: u16,
    mines: u16,
    selected_diff: Difficulty,
    curr_diff: Difficulty,
}

impl Component for NewGameMenu {
    type Message = NewGameMenuMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        link.get_parent()
            .expect("App should exist")
            .clone()
            .downcast::<AppRender>()
            .send_message(AppRenderMsg::MenuLink(link));
        Self {
            visible: false,
            rows: 9,
            cols: 9,
            mines: 10,
            selected_diff: Difficulty::Easy,
            curr_diff: Difficulty::Easy,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            NewGameMenuMsg::ToggleVisibility => {
                self.visible ^= true;
            }
            NewGameMenuMsg::Difficulty(diff) => {
                self.selected_diff = diff;
            }
            NewGameMenuMsg::Rows(rows) => {
                self.rows = rows;
                self.mines = self.mines.min(self.max_mines());
            }
            NewGameMenuMsg::Cols(cols) => {
                self.cols = cols;
                self.mines = self.mines.min(self.max_mines());
            }
            NewGameMenuMsg::Mines(mines) => {
                self.mines = mines;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let display = if self.visible { "block" } else { "none" };
        let curr_diff = self.curr_diff;
        let set_curr_diff = link.callback(move |_| NewGameMenuMsg::Difficulty(curr_diff));
        let close = link.callback(move |e| {
            set_curr_diff.emit(e);
            NewGameMenuMsg::ToggleVisibility
        });
        let close2 = link.callback(|_| NewGameMenuMsg::ToggleVisibility);
        let (rows, cols, mines) = match self.selected_diff {
            Difficulty::Custom => self.values(),
            _ => self.selected_diff.value(),
        };
        let new_game = link
            .get_parent()
            .expect("App should exit")
            .clone()
            .downcast::<AppRender>()
            .callback(move |e| {
                close2.emit(e);
                AppRenderMsg::Difficulty(rows, cols, mines)
            });
        let rows_change = link.callback(|e: Event| match (e.type_().as_ref(), e.target()) {
            ("change", Some(target)) => {
                match Reflect::get(&target, &JsValue::from_str("value")) {
                    Ok(value) => {
                        log!(&value);
                        NewGameMenuMsg::Rows(
                            value.as_string().expect("value should exist").parse().expect("value should be number")
                        )
                    }
                    Err(_) => NewGameMenuMsg::Rows(1),
                }
            }
            (_, _) => NewGameMenuMsg::Rows(1),
        });
        let cols_change = link.callback(|e: Event| match (e.type_().as_ref(), e.target()) {
            ("change", Some(target)) => {
                match Reflect::get(&target, &JsValue::from_str("value")) {
                    Ok(value) => {
                        log!(&value);
                        NewGameMenuMsg::Cols(
                            value.as_string().expect("value should exist").parse().expect("value should be number")
                        )
                    }
                    Err(_) => NewGameMenuMsg::Cols(1),
                }
            }
            (_, _) => NewGameMenuMsg::Cols(1),
        });
        let mines_change = link.callback(|e: Event| {
            match (e.type_().as_ref(), e.target()) {
                ("change", Some(target)) => {
                    match Reflect::get(&target, &JsValue::from_str("value")) {
                        Ok(value) => {
                            log!(&value);
                            NewGameMenuMsg::Mines(
                                value.as_string().expect("value should exist").parse().expect("value should be number")
                            )
                        }
                        Err(_) => NewGameMenuMsg::Mines(1),
                    }
                }
                (_, _) => NewGameMenuMsg::Mines(1),
            }
        });
        html! {
            <div class={"menu"} style={format!("display: {}", display)}>
                <div class={"item"}>
                    <span>//Add Dead and Happy icon
                        <p style={"text-align: center;"}>{"New Game"}</p>
                    </span>
                </div>
                <div class={"item"}>
                    {Difficulty::iter().map(|x|self.create_button(ctx, x)).collect::<Html>()}
                    <div style={"display: flex; flex-direction: row; justify-content: space-around;"}>
                        <input type={"range"} id="rows" name="rows" min="5" max="60" orient="vertical" onchange={rows_change}/>
                        <input type={"range"} id="cols" name="cols" min="5" max="60" orient="vertical" onchange={cols_change}/>
                        <input type={"range"} id="mines" name="mines" min="1" max={format!("{}",self.max_mines())} orient="vertical" onchange={mines_change}/>
                    </div>
                </div>
                <div class={"item"} style={"display: flex; justify-content: space-between;"}>
                    <div class={"button"} style={"flex: 1;"} onclick={close}>{"X"}</div>
                    <div class={"button"} style={"flex: 1;"} onclick={new_game}>{"Y"}</div>
                </div>
            </div>
        }
    }
}

impl NewGameMenu {
    fn create_button(&self, ctx: &Context<Self>, diff: Difficulty) -> Html {
        let (cols, rows, mines) = match diff {
            Difficulty::Custom => self.values(),
            _ => diff.value(),
        };
        let mut name = diff.to_string();
        if diff == self.selected_diff {
            name += "*";
        }
        let difficulty = ctx
            .link()
            .callback(move |_| NewGameMenuMsg::Difficulty(diff));
        let span_style =
            "text-align: right; vertical-align: middle;float: right; padding-right: 5px; font-size: 2em;";
        let div_style = "height: 20px; min-width: 100; position: relative;";
        let stuff = [
            (name, "15%"),
            (format!("{}x{} {}mines", cols, rows, mines), "25%"),
        ];
        html! {
            <div class={"button"} onclick={difficulty} style={"height: 55px"}>
            {stuff.iter().map(|(text, alignment)|html!{<div style={format!("{} top: {};", div_style, alignment)}><span style={span_style}>{text}</span></div>}).collect::<Html>()}
            </div>
        }
    }
    fn values(&self) -> (u16, u16, u16) {
        (self.rows, self.cols, self.mines)
    }
    fn max_mines(&self) -> u16 {
        let temp = self.rows * self.cols;
        if temp < 9 {
            0
        } else {
            (temp / 2).min(temp - 9)
        }
    }
}
