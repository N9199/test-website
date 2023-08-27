use yew::{Component, Html, html, Context};
use yew::html::{ImplicitClone, Scope};

use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;

mod new_game_menu;
mod board;
use board::AppRender as AppRender;

pub struct WeakComponentLink<COMP: Component>(Rc<RefCell<Option<Scope<COMP>>>>);

impl<COMP: Component> Clone for WeakComponentLink<COMP> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
impl<COMP: Component> ImplicitClone for WeakComponentLink<COMP> {}

impl<COMP: Component> Default for WeakComponentLink<COMP> {
    fn default() -> Self {
        Self(Rc::default())
    }
}

impl<COMP: Component> Deref for WeakComponentLink<COMP> {
    type Target = Rc<RefCell<Option<Scope<COMP>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<COMP: Component> PartialEq for WeakComponentLink<COMP> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}


struct Model {
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Model{}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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