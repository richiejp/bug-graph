
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
        pub fn $mname<S: Into<String>>(msg: S) -> Notice {
            Notice {
                msg: msg.into(),
                flavor: Flavor::$tname,
            }
        }
    )+ }
}

impl Notice {
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

/// Server to Client message
#[derive(Serialize, Deserialize)]
pub enum ServerClient {
    Notify(Notice),
    TestList(Vec<String>),
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
    TestQuery,
}
