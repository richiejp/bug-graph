use std::time::Instant;
use std::cell::RefCell;
use std::fmt;

use futures::Future;
use actix::prelude::*;
use log::{self, Record, Level, Metadata, LevelFilter};

static FACADE: JournalFacade = JournalFacade;

thread_local!(
    static JOURNAL: RefCell<Option<Addr<Syn, Journal>>> = RefCell::new(None);
);

#[derive(Message)]
pub struct Log {
    pub src: String,
    pub msg: String,
}

pub struct Journal {
    genesis: Instant,
}

impl Default for Journal {
    fn default() -> Self {
        Journal {
            genesis: Instant::now(),
        }
    }
}

impl Actor for Journal {
    type Context = Context<Self>;
}

impl Supervised for Journal {}
impl SystemService for Journal {}

impl Handler<Log> for Journal {
    type Result = ();

    fn handle(&mut self, log: Log, _ctx: &mut Context<Self>) {
        let d = self.genesis.elapsed();
        eprintln!("{:>+4}:{:<04}[{}] {}",
                  d.as_secs(),
                  d.subsec_nanos() / 100_000,
                  log.src,
                  log.msg);
    }
}

struct JournalFacade;

impl log::Log for JournalFacade {

    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, r: &Record) {
        if Arbiter::name() == "Arbiter is not running" {
            eprintln!("<SysDown> [{:5} {}] {}",
                      r.level(), r.module_path().unwrap_or("Unknown"),
                      r.args());
        }
        let arb = Arbiter::handle();
        let journal = JOURNAL.with(|cell| {
            if let Some(ref j) = *cell.borrow() {
                return j.clone();
            }
            let j = Arbiter::system_registry().get::<Journal>();
            *cell.borrow_mut() = Some(j.clone());
            j
        });
        let log = Log {
            src: format!("{:5} {}", r.level(), r.module_path().unwrap_or("Unknown")),
            msg: fmt::format(*r.args()),
        };

        match r.level() {
            Level::Error => journal.do_send(log),
            _ => arb.spawn(journal.send(log).map_err(|_| ())),
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_max_level(LevelFilter::Trace);
    if let Err(e) = log::set_logger(&FACADE) {
        eprintln!("Init logger failed: {}", e);
    }
}
