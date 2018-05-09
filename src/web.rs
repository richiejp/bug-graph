
use actix::prelude::*;
use actix_web::*;
use actix_web::fs::NamedFile;
use actix_web::http::Method;

pub struct Ws;

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.text("Hello from WS server!");
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Text(text) => {
                ctx.text(format!("You sent me: {}", text));
            }
            _ => ()
        }
    }
}

fn index(_req: HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open("res/static/index.html")?)
}

fn ws_index(req: HttpRequest) -> Result<HttpResponse> {
    ws::start(req, Ws)
}

fn static_file(file: Path<String>) -> Result<NamedFile> {
    Ok(NamedFile::open(format!("res/static/{}", *file))?)
}

pub fn new() -> App<()>
{
    App::new()
        .resource("/", |r| r.method(Method::GET).f(index))
        .resource("/ws/", |r| r.f(ws_index))
        // For now non capture groups (?: ...) confuse the actix-web parser
        // and numbered capture groups confuse the router because they produce
        // surplus matches
        .resource(r"/{file:[a-z.-]+\.[cjswam]+}",
                  |r| r.method(Method::GET).with(static_file))
}
