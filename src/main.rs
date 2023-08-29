use yew::{html, Component, Context, Html};

mod board;
mod new_game_menu;
use board::AppRender;

struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Model {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <AppRender />
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
