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

use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;

use failure::Error;
use stdweb::web;
use yew::prelude::*;
use yew::virtual_dom::{VNode, VList};
use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketTask, WebSocketStatus};
use uuid::Uuid;

use protocol::{Notice, ClientServer, ServerClient, ResultMatrix, ResultInMatrix};
use search::Search;

#[derive(Clone,Copy,PartialEq,Eq)]
enum AppTab {
    Explore,
    Compare,
}

impl fmt::Display for AppTab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AppTab::*;

        match self {
            Explore => write!(f, "Explore"),
            Compare => write!(f, "Compare"),
        }
    }
}

struct Model {
    link: ComponentLink<Model>,
    wss: WebSocketService,
    ws: Option<WebSocketTask>,
    notices: Vec<Notice>,
    sets: Option<Vec<(String, Uuid)>>,
    tab: AppTab,
    cmp_term: Rc<RefCell<String>>,
    cmp_completions: Rc<Vec<(String, Uuid)>>,
    cmp_matrix: Option<ResultMatrix>,
    cmp_selected: Uuid,
}

enum Msg {
    Recv(Result<ServerClient, Error>),
    Stat(WebSocketStatus),
    Send(ClientServer),
    DelNotice(usize),
    ToTab(AppTab),
}

impl Component for Model
{
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let url = ws_url();
        let cb = link.send_back(|Json(msg)| Msg::Recv(msg));
        let evt = link.send_back(|status| Msg::Stat(status));
        let mut wss = WebSocketService::new();
        let ws = wss.connect(&url, cb, evt);

        Model {
            link,
            wss,
            ws: Some(ws),
            notices: Vec::default(),
            sets: None,
            tab: AppTab::Explore,
            cmp_term: Rc::new(RefCell::new("".to_string())),
            cmp_completions: Rc::new(Vec::default()),
            cmp_selected: Uuid::default(),
            cmp_matrix: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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
            Msg::Recv(res) => match res {
                Ok(ServerClient::Notify(n)) => { self.notices.push(n); true },
                Ok(ServerClient::SetList(l)) => { self.sets = Some(l); true },
                Ok(ServerClient::Search(t, r)) => if t == *self.cmp_term.borrow() {
                    self.cmp_completions = Rc::new(r);
                    true
                } else {
                    false
                },
                Ok(ServerClient::ResultMatrix(uuid, m)) => if uuid == self.cmp_selected {
                    self.cmp_matrix = Some(m);
                    true
                } else {
                    false
                },
                Err(e) => {
                    self.notices.push(
                        Notice::error(format!("Could not parse message from server: {}", e))
                    );
                    true
                },
            },
            Msg::Send(m) => {
                if let ClientServer::ResultMatrix(t) = m {
                    self.cmp_selected = t;
                }
                self.ws.as_mut().unwrap().send_binary(Json(&m));
                self.notices.push(Notice::info(format!("Requesting set {}", &m)));
                true
            },
            Msg::DelNotice(i) => { self.notices.remove(i); true },
            Msg::ToTab(t) => { self.tab = t; true },
        }
    }
}

impl Renderable<Model> for Model
{
    fn view(&self) -> Html<Self> {
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
           <div class=("container","is-fluid"),>{
               match self.tab {
                   AppTab::Explore => html! {
                       <div class="columns",>
                        <div class=("column","is-narrow"),>
                         <button class="button",
                                 onclick=|_| Msg::Send(ClientServer::SetQuery(None)),>{
                             "Get sets"
                         }</button>
                        </div>
                        <div class=("column", "is-centered"),>{
                            self.render_set_list()
                        }</div>
                       </div>
                   },
                   AppTab::Compare => html! {
                       <>
                       <div class="container",>
                        <Search: term=Rc::clone(&self.cmp_term),
                                 completions=Some(Rc::clone(&self.cmp_completions)),
                                 onneed_more=|t| Msg::Send(ClientServer::Search(t)),
                                 onmatch=|t| Msg::Send(ClientServer::ResultMatrix(t)),/>
                       </div>
                       <div class=("container","is-fluid"),>{ self.render_matrix() }</div>
                       </>
                   },
               }
           }</div>
          </section>
          <footer class="footer",><div class="container",>{
             for self.notices.iter().enumerate().map(|(i, m)| render_notice(i, m))
          }</div></footer>
        </>}
    }
}

impl Model {

    fn render_matrix_cells(&self, results: &[ResultInMatrix], test_count: usize) -> Html<Model> {
        let mut html = VList::new();
        let mut i = 0;

        for result in results {
            let td = if result.test_case == i {
                let score = format!("{}/{}", result.passes, result.fails);
                if result.fails == 0 {
                    html! {
                        <td class="has-background-success",>{ score }</td>
                    }
                } else if result.passes >= result.fails {
                    html! {
                        <td class="has-background-warning",>{ score }</td>
                    }
                } else {
                    html! {
                        <td class="has-background-danger",>{ score }</td>
                    }
                }
            } else if result.test_case > i {
                html! {
                    <td>{ "_" }</td>
                }
            } else {
                panic!("Test results are out-of-order?");
            };

            i += 1;
            html.add_child(td);
        }

        for _j in i..(test_count as u32) {
            html.add_child(html! { <td>{ "_" }</td> });
        }

        VNode::from(html)
    }
    
    fn render_matrix_rows(&self, matrix: &ResultMatrix) -> Html<Model> {
        let test_count = matrix.test_cases.len();
        html! {
            {
                for matrix.results.iter().map(|(build, results)| {
                    html! {
                        <tr>
                         <td>{ &build.0 }</td>
                         { self.render_matrix_cells(results, test_count) }
                        </tr>
                    }
                })
            }
        }
    }

    fn render_matrix(&self) -> Html<Model> {
        if let Some(ref matrix) = self.cmp_matrix {
            html! {
                <table class=("table","is-narrow"),>
                 <thead><tr>
                  <th>{ "" }</th>
                  {
                      for matrix.test_cases.iter().map(|t| html! {
                          <th>{ &t.0 }</th>
                      })
                  }
                  </tr></thead>
                  {
                      self.render_matrix_rows(matrix)
                  }
                </table>
            }
        } else {
            html! { <p>{ "Type in a fully qualified test name in the box above" }</p> }
        }
    }

    fn render_tabs(&self) -> impl Iterator<Item=Html<Model>>
    {
        use AppTab::*;

        let cur = self.tab;

        (&[Explore, Compare]).iter().map(move |tab| {
            if cur == *tab {
                html! {
                    <li class="is-active",><a>{ *tab }</a></li>
                }
            } else {
                html! {
                    <li><a onclick=|_| Msg::ToTab(*tab),>{ *tab }</a></li>
                }
            }
        })
    }

    fn render_set_list(&self) -> Html<Model>
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
                            let uuid = *uuid;
                            html! {
                                <tr>
                                    <td>{ name }</td>
                                    <td>
                                    <a onclick=|_| Msg::Send(ClientServer::SetQuery(Some(uuid))),>{
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

fn render_notice(i: usize, notice: &Notice) -> Html<Model>
{
    html! {
        <div class=("notification", notice.css_class()),>
            <button class="delete", onclick=|_| Msg::DelNotice(i),/>
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

    let app = App::<Model>::new();

    app.mount_to_body();
    yew::run_loop();
}
