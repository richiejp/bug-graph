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

use std::time::Instant;
use std::cell::RefCell;
use std::fmt;

use futures::Future;
use actix::prelude::*;
use log::{self, Record, Level, Metadata, LevelFilter};

static FACADE: JournalFacade = JournalFacade;

thread_local!(
    static JOURNAL: RefCell<Option<Addr<Journal>>> = RefCell::new(None);
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
        let journal = JOURNAL.with(|cell| {
            if let Some(ref j) = *cell.borrow() {
                return j.clone();
            }
            let j = System::current().registry().get::<Journal>();
            *cell.borrow_mut() = Some(j.clone());
            j
        });
        let log = Log {
            src: format!("{:5} {}", r.level(), r.module_path().unwrap_or("Unknown")),
            msg: fmt::format(*r.args()),
        };

        match r.level() {
            Level::Error => journal.do_send(log),
            _ => Arbiter::spawn(journal.send(log).map_err(|_| ())),
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
