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

use std::fmt;

use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum Flavor {
    Error,
    Warn,
    Info,
    Success,
}

/// For the user's information
#[derive(Serialize, Deserialize)]
pub struct Notice {
    pub msg: String,
    pub flavor: Flavor,
}

macro_rules! make_notices {
    ( $( $mname:ident, $tname:ident );+ ) => { $(
        #[allow(dead_code)]
        pub fn $mname<S: Into<String>>(msg: S) -> Notice {
            Notice {
                msg: msg.into(),
                flavor: Flavor::$tname,
            }
        }
    )+ }
}

impl Notice {
    #[allow(dead_code)]
    pub fn css_class(&self) -> &'static str {
        match self.flavor {
            Flavor::Error => "is-error",
            Flavor::Warn => "is-warning",
            Flavor::Info => "is-info",
            Flavor::Success => "is-success",
        }
    }

    make_notices! {
        error, Error;
        warn, Warn;
        info, Info;
        succ, Success
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VertInfo(pub String, pub Uuid);

#[derive(Clone, Serialize, Deserialize)]
pub struct ResultInMatrix {
    pub test_case: u32,
    pub passes: u32,
    pub fails: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ResultMatrix {
    pub test_cases: Vec<VertInfo>,
    pub results: Vec<(VertInfo, Vec<ResultInMatrix>)>,
}

/// Server to Client message
#[derive(Serialize, Deserialize)]
pub enum ServerClient {
    Notify(Notice),
    SetList(Vec<(String, Uuid)>),
    Search(String, Vec<(String, Uuid)>),
    ResultMatrix(Uuid, ResultMatrix),
}

impl ServerClient {
    pub fn info_notice<S: Into<String>>(msg: S) -> ServerClient {
        ServerClient::Notify(Notice {
            msg: msg.into(),
            flavor: Flavor::Info,
        })
    }
}

/// Client to Server message
#[derive(Serialize, Deserialize)]
pub enum ClientServer {
    SetQuery(Option<Uuid>),
    Search(String),
    ResultMatrix(Uuid),
}

impl fmt::Display for ClientServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ClientServer::*;

        match self {
            SetQuery(Some(uuid)) => write!(f, "SetQuery({})", uuid),
            SetQuery(None) => write!(f, "SetQuery(All)"),
            Search(term) => write!(f, "Search({})", term),
            ResultMatrix(uuid) => write!(f, "ResultMatrix({})", uuid),
        }
    }
}
