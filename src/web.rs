
//use actix::prelude::*;
use actix_web::*;
use actix_web::fs::NamedFile;
use actix_web::http::Method;

fn index(_req: HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open("res/static/index.html")?)
}

fn static_file(file: Path<String>) -> Result<NamedFile> {
    Ok(NamedFile::open(format!("res/static/{}", *file))?)
}

pub fn new() -> App<()>
{
    App::new()
        .resource("/",
                  |r| r.method(Method::GET).f(index))
        // For now non capture groups (?: ...) confuse the actix-web parser
        // and numbered capture groups confuse the router because they produce
        // surplus matches
        .resource(r"/{file:[a-z.-]+\.[cjswam]+}",
                  |r| r.method(Method::GET).with(static_file))
}
