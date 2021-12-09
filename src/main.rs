use yew::{html, Component, ComponentLink, Html, ShouldRender};

mod board;
use board::BoardRender as BoardRender;


struct Model {
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
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