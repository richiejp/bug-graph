// Copyright (C) 2018 Richard Palethorpe <richiejp@f-m.fm>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![recursion_limit="128"]

extern crate failure;
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate yew;
extern crate uuid;

mod protocol;
mod search;

use failure::Error;
use stdweb::web;
use yew::prelude::*;
use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketTask, WebSocketStatus};
use uuid::Uuid;

use protocol::{Notice, ClientServer, ServerClient};
use search::Search;

struct Context {
    ws: WebSocketService,
}

impl AsMut<WebSocketService> for Context {
    fn as_mut(&mut self) -> &mut WebSocketService {
        &mut self.ws
    }
}

#[derive(Clone,Copy,PartialEq,Eq)]
enum AppTab {
    Explore,
    Compare,
}

struct Model {
    ws: Option<WebSocketTask>,
    notices: Vec<Notice>,
    sets: Option<Vec<(String, Uuid)>>,
    tab: AppTab,
}

enum Msg {
    Recv(Result<ServerClient, Error>),
    Stat(WebSocketStatus),
    Send(ClientServer),
    DelNotice(usize),
    ToTab(AppTab),
}

impl<C> Component<C> for Model
where
    C: AsMut<WebSocketService> + 'static,
{
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, env: &mut Env<C, Self>) -> Self {
        let url = ws_url();
        let cb = env.send_back(|Json(msg)| Msg::Recv(msg));
        let evt = env.send_back(|status| Msg::Stat(status));
        let wss: &mut WebSocketService = env.as_mut();
        let task = wss.connect(&url, cb, evt);

        Model {
            ws: Some(task),
            notices: Vec::default(),
            sets: None,
            tab: AppTab::Explore,
        }
    }

    fn update(&mut self, msg: Self::Message, _env: &mut Env<C, Self>) -> ShouldRender {
        match msg {
            Msg::Stat(s) => { match s {
                WebSocketStatus::Opened => {
                    self.notices.push(Notice::succ("Opened websocket"));
                },
                WebSocketStatus::Closed => {
                    self.notices.push(Notice::info("Closed websocket"));
                    self.ws = None;
                },
                WebSocketStatus::Error => {
                    self.notices.push(Notice::error("Error on websocket"));
                },
            } true },
            Msg::Recv(res) => { match res {
                Ok(ServerClient::Notify(n)) => self.notices.push(n),
                Ok(ServerClient::SetList(l)) => self.sets = Some(l),
                Ok(ServerClient::Search(s)) => panic!("Not implemented"),
                Err(e) => self.notices.push(
                    Notice::error(format!("Could not parse message from server: {}", e))
                ),
            } true },
            Msg::Send(m) => {
                self.ws.as_mut().unwrap().send_binary(Json(&m));
                self.notices.push(Notice::info(format!("Requesting set {}", &m)));
                true
            },
            Msg::DelNotice(i) => { self.notices.remove(i); true },
            Msg::ToTab(t) => { self.tab = t; true },
        }
    }
}

impl<C> Renderable<C, Model> for Model
where
    C: AsMut<WebSocketService> + 'static,
{
    fn view(&self) -> Html<C, Self> {
        html! {<>
          <section class=("hero","is-primary","is-bold"),>
            <div class="hero-body",>
              <div class="container",>
                <h1 class="title",>{
                    "Bug Graph"
                }</h1>
                <h2 class="subtitle",>{
                    "Connecting bugs and test results"
                }</h2>
              </div>
            </div>
            <div class="hero-foot",>
              <nav class=("tabs","is-boxed"),>
                <div class="container",>
                  <ul>{
                    for self.render_tabs()
                  }</ul>
                </div>
              </nav>
            </div>
          </section>
          <section class="section",>
            <div class=("container","is-fluid"),>
              <div class="columns",>
                <div class=("column","is-narrow"),>
                  <button class="button", onclick=|_| Msg::Send(ClientServer::SetQuery(None)),>{
                    "Get sets"
                  }</button>
                </div>
               <div class=("column", "is-centered"),>{
                   match self.tab {
                       AppTab::Explore => self.render_set_list(),
                       AppTab::Compare => self.render_matrix(),
                   }
               }</div>
              </div>
            </div>
          </section>
          <footer class="footer",><div class="container",>{
             for self.notices.iter().enumerate().map(|(i, m)| render_notice(i, m))
          }</div></footer>
        </>}
    }
}

impl Model {

    fn render_matrix<C: AsMut<WebSocketService> + 'static>(&self) -> Html<C, Model> {
        html! {
            <Search: />
        }
    }

    fn render_tabs<C>(&self) -> impl Iterator<Item=Html<C, Model>>
    where
        C: AsMut<WebSocketService> + 'static
    {
        use AppTab::*;

        let cur = self.tab;

        (&[("Explore", Explore), ("Compare", Compare)]).iter().map(move |(text, val)| {
            let val = *val;
            if cur == val {
                html! {
                    <li class="is-active",><a>{ text }</a></li>
                }
            } else {
                html! {
                    <li><a onclick= move |_| Msg::ToTab(val),>{ text }</a></li>
                }
            }
        })
    }

    fn render_set_list<C: AsMut<WebSocketService> + 'static>(&self) -> Html<C, Model>
    {
        if let Some(ref l) = self.sets {
            html! {
                <table class=("table","is-hoverable"),>
                    <thead>
                    <tr>
                    <th><abbr title="Test, Product or Set name",>{
                        "Name"
                    }</abbr></th>
                    <th><abbr title="Vertex UUID",>{
                        "UUID"
                    }</abbr></th>
                    </tr>
                    </thead>
                    <tbody>{
                        for l.iter().map(|(name, uuid)| {
                            let uuid2 = *uuid;
                            html! {
                                <tr>
                                    <td>{ name }</td>
                                    <td>
                                    <a onclick= move |_| Msg::Send(ClientServer::SetQuery(Some(uuid2))),>{
                                        uuid
                                    }</a>
                                    </td>
                               </tr>
                            }
                        })
                    }</tbody>
                    </table>
            }
        } else {
            html! {
                <div class=("notification","has-text-grey"),>{
                    "Nothing to see here... yet."
                }</div>
            }
        }
    }
}

fn render_notice<C>(i: usize, notice: &Notice) -> Html<C, Model>
where
    C: AsMut<WebSocketService> + 'static,
{
    html! {
        <div class=("notification", notice.css_class()),>
            <button class="delete", onclick= move |_| Msg::DelNotice(i),/>
            { &notice.msg }
        </div>
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
