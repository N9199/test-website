use crate::board::{AppRender, AppRenderMsg};

use std::fmt;
use std::rc::Rc;

use strum::{EnumIter, IntoEnumIterator};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub enum NewGameMenuMsg {
    ToggleVisibility,
    Difficulty(Difficulty),
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
    const fn value(&self) -> (usize, usize, usize) {
        match self {
            Difficulty::Easy => (9, 9, 10),
            Difficulty::Medium => (16, 16, 40),
            Difficulty::Hard => (16, 30, 99),
            Difficulty::Extreme => (24, 30, 180),
            Difficulty::Custom => (0, 0, 0),
        }
    }
}

#[derive(Clone, Properties)]
pub struct NewGameMenuProps {
    pub par_link: Rc<ComponentLink<AppRender>>,
}

pub struct NewGameMenu {
    visible: bool,
    props: NewGameMenuProps,
    link: Rc<ComponentLink<NewGameMenu>>,
    values: (usize, usize, usize),
    selected_diff: Difficulty,
    curr_diff: Difficulty,
}

impl Component for NewGameMenu {
    type Message = NewGameMenuMsg;
    type Properties = NewGameMenuProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link = Rc::new(link);
        let send_back_info = props.par_link.callback_once(AppRenderMsg::MenuLink);
        send_back_info.emit(link.clone());
        Self {
            visible: false,
            props,
            link,
            values: (9, 9, 10),
            selected_diff: Difficulty::Easy,
            curr_diff: Difficulty::Easy,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            NewGameMenuMsg::ToggleVisibility => {
                self.visible ^= true;
            }
            NewGameMenuMsg::Difficulty(diff) => {
                self.selected_diff = diff;
            }
        }
        true
    }

    fn view(&self) -> Html {
        let display = if self.visible { "block" } else { "none" };
        let curr_diff = self.curr_diff;
        let set_curr_diff = self
            .link
            .callback(move |_| NewGameMenuMsg::Difficulty(curr_diff));
        let close = self.link.callback(move |e| {
            set_curr_diff.emit(e);
            NewGameMenuMsg::ToggleVisibility
        });
        let close2 = self.link.callback(|_| NewGameMenuMsg::ToggleVisibility);
        let (rows, cols, mines) = match self.selected_diff {
            Difficulty::Custom => self.values,
            _ => self.selected_diff.value(),
        };
        let new_game = self.props.par_link.callback(move |e| {
            close2.emit(e);
            AppRenderMsg::Difficulty(rows, cols, mines)
        });
        html! {
            <div class={"menu"} style={format!("display: {}", display)}>
                <div class={"item"}>
                    <span>//Add Dead and Happy icon
                        <p style={"text-align: center"}>{"New Game"}</p>
                    </span>
                </div>
                <div class={"item"}>
                    {Difficulty::iter().map(|x|self.create_button(x)).collect::<Html>()}
                </div>
                <div class={"item"} style={"display: flex;"}>
                    <div class={"button"} style={"flex: 1;"} onclick=close>{"X"}</div>
                    <div class={"button"} style={"flex: 1;"} onclick=new_game>{"Y"}</div>
                </div>
            </div>
        }
    }
}

impl NewGameMenu {
    fn create_button(&self, diff: Difficulty) -> Html {
        let (cols, rows, mines) = match diff {
            Difficulty::Custom => self.values,
            _ => diff.value(),
        };
        let mut name = diff.to_string();
        if diff == self.selected_diff{
            name += "*";
        }
        let difficulty = self
            .link
            .callback(move |_| NewGameMenuMsg::Difficulty(diff));
        let span_style =
            "text-align: right; vertical-align: middle;float: right; padding-right: 5px;";
        let div_style = "height: 20px; min-width: 100; position: relative;";
        let stuff = [
            (name, "15%"),
            (format!("{}x{} {}mines", cols, rows, mines), "25%"),
        ];
        html! {
            <div class={"button"} onclick=difficulty style={"height: 55px"}>
            {stuff.iter().map(|(text, alignment)|html!{<div style={format!("{} top: {};", div_style, alignment)}><span style={span_style}>{text}</span></div>}).collect::<Html>()}
            </div>
        }
    }
}
