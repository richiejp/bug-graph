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

use serde_json;
use actix::prelude::*;
use actix::fut::{ok, err};
use actix_web::*;
use actix_web::fs::NamedFile;
use actix_web::http::Method;
use failure::Error;

use repo::{Repo, GetSetVerts, Search};
use protocol::{ClientServer, ServerClient};

pub struct AppState {
    repo: Addr<Syn, Repo>,
}

struct Ws {
    repo: Addr<Syn, Repo>,
}

impl Ws {
    fn repo_query<Q, F>(&self, query: Q, err_msg: &'static str,
                        ctx: &mut <Self as Actor>::Context, resp_fn: F)
    where
        Q: Message + Send + 'static,
        Q::Result: Send,
        Repo: Handler<Q>,
        F: 'static + FnOnce(Q::Result) -> ServerClient
    {
        let fut = self.repo.send(query).into_actor(self).from_err::<Error>();

        ctx.spawn(fut.and_then(|res, _, ctx| {
            match serde_json::to_vec(&resp_fn(res)) {
                Ok(json) => ok(ctx.binary(json)),
                Err(e) => err(e.into()),
            }
        }).then(move |res, _, ctx| {
            if let Err(e) = res {
                error!("{}: {}", err_msg, e);
                ctx.stop();
            }
            ok(())
        }));
    }

    fn handle_client_msg(&self,
                         msg: Result<ClientServer, serde_json::Error>,
                         ctx: &mut <Self as Actor>::Context)
                         -> Result<(), Error>
    {
        match msg? {
            ClientServer::SetQuery(uuid) => {
                let err = "Could not send set list";
                self.repo_query(GetSetVerts(uuid), err, ctx, |res| ServerClient::SetList(res));
            },
            ClientServer::Search(term) => {
                let err = "Search failed";
                self.repo_query(Search(term), err, ctx, |res| ServerClient::Search(res));
            }
        }
        Ok(())
    }
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let hello = serde_json::to_vec(
            &ServerClient::info_notice("Hello from WS server!")
        ).expect("Creating static hello message");
        ctx.binary(hello);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        let msg: Result<ClientServer, _> = match msg {
            ws::Message::Text(text) => {
                info!("Websocket received: {}", text);
                serde_json::from_str(&text)
            },
            ws::Message::Binary(bin) => {
                info!("Websocket received binary");
                serde_json::from_slice(bin.as_ref())
            },
            _ => {
                warn!("Received unexpected web socket msg: {:?}", msg);
                ctx.stop();
                return;
            }
        };
        if let Err(e) = self.handle_client_msg(msg, ctx) {
            error!("Could not handle client websocket request: {}", e);
            ctx.stop();
        }
    }
}

fn index(_req: HttpRequest<AppState>) -> Result<NamedFile> {
    Ok(NamedFile::open("res/static/index.html")?)
}

fn ws_index(req: HttpRequest<AppState>) -> Result<HttpResponse> {
    let repo = req.state().repo.clone();
    ws::start(req, Ws { repo: repo })
}

fn static_file(file: Path<String>) -> Result<NamedFile> {
    Ok(NamedFile::open(format!("res/static/{}", *file))?)
}

pub fn new(repo: Addr<Syn, Repo>) -> App<AppState>
{
    App::with_state(AppState{ repo: repo })
        .resource("/", |r| r.method(Method::GET).f(index))
        .resource("/ws/", |r| r.f(ws_index))
        // For now non capture groups (?: ...) confuse the actix-web parser
        // and numbered capture groups confuse the router because they produce
        // surplus matches
        .resource(r"/{file:[a-z.-]+\.[cjswam]+}",
                  |r| r.method(Method::GET).with(static_file))
}
