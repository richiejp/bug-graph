extern crate failure;
extern crate stdweb;
#[macro_use]
extern crate yew;

use failure::Error;
use stdweb::web;
use yew::prelude::*;
use yew::format::{Text, Binary};
use yew::services::websocket::{WebSocketService, WebSocketTask, WebSocketStatus};

struct Context {
    ws: WebSocketService,
}

impl AsMut<WebSocketService> for Context {
    fn as_mut(&mut self) -> &mut WebSocketService {
        &mut self.ws
    }
}

enum WsMsg {
    Text(String),
    Bin(Vec<u8>),
    Err(Error),
}

impl WsMsg {
    pub fn txt<S: Into<String>>(msg: S) -> WsMsg {
        let s = msg.into();
        WsMsg::Text(s)
    }
}

impl From<Binary> for WsMsg {
    fn from(val: Binary) -> Self {
        match val {
            Ok(b) => WsMsg::Bin(b),
            Err(e) => WsMsg::Err(e),
        }
    }
}

impl From<Text> for WsMsg {
    fn from(val: Text) -> Self {
        match val {
            Ok(s) => WsMsg::Text(format!("Received: {}", s)),
            Err(e) => WsMsg::Err(e),
        }
    }
}

struct Model {
    ws: Option<WebSocketTask>,
    log: Vec<WsMsg>,
}

enum Msg {
    Recv(WsMsg),
    Stat(WebSocketStatus),
}

impl<C> Component<C> for Model
where
    C: AsMut<WebSocketService> + 'static,
{
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, env: &mut Env<C, Self>) -> Self {
        let url = ws_url();
        let cb = env.send_back(|text: WsMsg| Msg::Recv(text));
        let evt = env.send_back(|status| Msg::Stat(status));
        let wss: &mut WebSocketService = env.as_mut();
        let task = wss.connect(&url, cb, evt);

        Model {
            ws: Some(task),
            log: vec![ WsMsg::txt("Test") ],
        }
    }

    fn update(&mut self, msg: Self::Message, _env: &mut Env<C, Self>) -> ShouldRender {
        match msg {
            Msg::Stat(s) => match s {
                WebSocketStatus::Opened => {
                    self.log.push(WsMsg::txt("Opened websocket"));
                },
                WebSocketStatus::Closed => {
                    self.log.push(WsMsg::txt("Closed websocket"));
                    self.ws = None;
                },
                WebSocketStatus::Error => {
                    self.log.push(WsMsg::txt("Error on websocket"));
                },
            },
            Msg::Recv(res) => self.log.push(res),
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
                <div class="container",>{
                    for self.log.iter().map(|m| render_message(m))
                }</div>
            </section>
        }
    }
}

fn render_message<C>(msg: &WsMsg) -> Html<C, Model>
where
    C: AsMut<WebSocketService> + 'static,
{
    match msg {
        WsMsg::Text(ref s) => html! {
            <div class=("notification", "is-info"),>{
                s
            }</div>
        },
        WsMsg::Bin(_) => html! {
            <div class=("notification", "is-warning"),>{
                "Received a binary message; no idea what to do with it..."
            }</div>
       },
       WsMsg::Err(e) => html! {
            <div class=("notification", "is-danger"),>{
                format!("Error while receiving: {}", e)
            }</div>
       },
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
