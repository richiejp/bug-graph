#[macro_use]
extern crate yew;

use yew::prelude::*;

struct Context;
struct Model;
struct Msg;

impl<C> Component<C> for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _env: &mut Env<C, Self>) -> Self {
        Model
    }

    fn update(&mut self, _msg: Self::Message, _env: &mut Env<C, Self>) -> ShouldRender {
        true
    }
}

impl<C: 'static> Renderable<C, Model> for Model {
    fn view(&self) -> Html<C, Self> {
        html! {
            <section class="section",>
                <div class="container",>
                <h1 class="title",>{ "Bug Graph" }</h1>
                <p class="subtitle",>{ "Ruining the pychology of bugs everywhere!" }</p>
                </div>
            </section>
        }
    }
}

fn main() {
    yew::initialize();

    let app: App<_, Model> = App::new(Context);
    app.mount_to_body();
    yew::run_loop();
}
