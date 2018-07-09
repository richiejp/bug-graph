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

use yew::prelude::*;
use uuid::Uuid;

pub enum Msg {
    Term(String),
}

#[derive(PartialEq, Clone, Default)]
pub struct Props {

}

pub struct Search {
    term: String,
    completions: Vec<(String, Uuid)>,
}

impl Component for Search
{
    type Message = Msg;
    type Properties = Props;

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Search {
            term: String::default(),
            completions: Vec::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Term(s) => if s != self.term {
                self.term = s;
                true
            } else {
                false
            },
        }
    }
}

impl Renderable<Search> for Search {
    fn view(&self) -> Html<Self> {
        html! {
            <input
                class=("input","is-rounded"), type="text", placeholder="Search term",
                value=&self.term,
                oninput=|e| Msg::Term(e.value),
            />
        }
    }
}
