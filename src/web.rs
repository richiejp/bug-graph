use actix_web::{server, Application, HttpRequest};

fn index(_req: HttpRequest) -> &'static str {
    "Hello world!"
}

pub fn run(listen_addr: &str) {
    let err_msg = format!("Can not bind to {}", listen_addr);

    server::HttpServer::new(|| Application::new().resource("/", |r| r.f(index)))
        .bind(listen_addr).expect(&err_msg)
        .start();
}
