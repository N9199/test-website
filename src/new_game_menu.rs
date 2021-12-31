use crate::board::{AppRender, AppRenderMsg};

use std::fmt;
use std::rc::Rc;

use strum::{EnumIter, IntoEnumIterator};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub enum NewGameMenuMsg {
    ToggleVisibility,
}

#[derive(Debug, EnumIter)]
enum Difficulty {
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
}

impl Component for NewGameMenu {
    type Message = NewGameMenuMsg;
    type Properties = NewGameMenuProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link = Rc::new(link);
        let send_back_info = props.par_link.callback_once( AppRenderMsg::MenuLink);
        send_back_info.emit(link.clone());
        Self {
            visible: false,
            props,
            link,
            values: (9, 9, 10),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            NewGameMenuMsg::ToggleVisibility => {
                self.visible ^= true;
                true
            }
        }
    }

    fn view(&self) -> Html {
        let display = if self.visible {"block"}else{"none"};
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
        let name = diff.to_string();
        let difficulty = self
            .props
            .par_link
            .callback(move |_| AppRenderMsg::Difficulty(cols, rows, mines));
        let span_style = "text-align: right; vertical-align: middle;float: right; padding-right: 5px;";
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
