extern crate failure;
extern crate stdweb;
#[macro_use]
extern crate yew;

use failure::Error;
use stdweb::web;
use yew::prelude::*;
use yew::services::Task;
use yew::services::websocket::{WebSocketService, WebSocketTask, WebSocketStatus};

struct Context {
    ws: WebSocketService,
}

impl AsMut<WebSocketService> for Context {
    fn as_mut(&mut self) -> &mut WebSocketService {
        &mut self.ws
    }
}

struct Model {
    ws: Option<WebSocketTask>,
    log: Vec<String>,
}

enum Msg {
    Recv(Result<String, Error>),
    Stat(WebSocketStatus),
}

impl<C> Component<C> for Model
where
    C: AsMut<WebSocketService> + 'static,
{
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, env: &mut Env<C, Self>) -> Self {
        let wss: &mut WebSocketService = env.as_mut();
        let url = ws_url();
        let cb = env.send_back(|text| Msg::Recv(text));
        let evt = env.send_back(|status| Msg::Stat(status));
        let task = wss.connect(&url, cb, evt);

        Model {
            ws: Some(task),
            log: Vec::default(),
        }
    }

    fn update(&mut self, msg: Self::Message, env: &mut Env<C, Self>) -> ShouldRender {
        match msg {
            Msg::Stat(s) => match s {
                WebSocketStatus::Opened => self.log.push("Opened websocket".into()),
                WebSocketStatus::Closed => self.log.push("Closed websocket".into()),
                WebSocketStatus::Error => self.log.push("Error on websocket".into()),
            },
            Msg::Recv(res) => match res {
                Ok(resp) => self.log.push(format!("Server: {}", resp)),
                Err(e) => self.log.push(format!("Error receiving: {}", e)),
            }
        }
        true
    }
}

impl<C> Renderable<C, Model> for Model
where
    C: AsMut<WebSocketService> + 'static,
{
    fn view(&self) -> Html<C, Self> {
        html! {
            <section class="section",>
                <div class="container",>
                <h1 class="title",>{ "Bug Graph" }</h1>
                <p class="subtitle",>{ "Ruining the pychology of bugs everywhere!" }</p>
                </div>
                <div class="container",>{ self.render_messages() }</div>
            </section>
        }
    }
}

impl Model
{
    fn render_messages<C>(&self) -> Html<C, Model>
    where
        C: AsMut<WebSocketService> + 'static,
    {
        self.log.iter()
            html! {
                <div class="notification is-info",>{ msg }</div>
            }
        }
    }
}

fn ws_url() -> String {
    let loc = web::window().location().expect("Getting host URL");
    let proto = if "https:" == loc.protocol().expect("Getting connection protocol") {
        "wss:"
    } else {
        "ws:"
    };
    let host = loc.host().expect("Getting host");

    format!("{}//{}/ws/", proto, host)
}

fn main() {
    yew::initialize();

    let app: App<_, Model> = App::new(Context {
        ws: WebSocketService::new(),
    });

    app.mount_to_body();
    yew::run_loop();
}
