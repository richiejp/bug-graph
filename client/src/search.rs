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

use std::cell::RefCell;
use std::rc::Rc;

use yew::prelude::*;
use uuid::Uuid;

pub enum Msg {
    Term(String),
}

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub term: Rc<RefCell<String>>,
    pub completions: Option<Rc<Vec<(String, Uuid)>>>,
    pub onneed_more: Option<Callback<String>>,
}

pub struct Search {
    link: ComponentLink<Search>,
    term: Rc<RefCell<String>>,
    completions: Rc<Vec<(String, Uuid)>>,
    matches: Vec<usize>,
    onneed_more: Option<Callback<String>>,
}

impl Search {
    fn filter_compls(&mut self) {
        let term = &*self.term.borrow();
        self.matches = self.completions.iter()
            .enumerate()
            .filter(|(_, (c, _))| c.starts_with(term))
            .map(|(i, _)| i)
            .collect();
    }
}

impl Component for Search
{
    type Message = Msg;
    type Properties = Props;

    fn create(p: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut s = Search {
            link,
            term: p.term,
            completions: p.completions.unwrap_or_else(|| Rc::new(Vec::default())),
            matches: Vec::default(),
            onneed_more: p.onneed_more,
        };

        s.filter_compls();
        s
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Term(s) => if s != *self.term.borrow() {
                self.term.replace(s);
                self.filter_compls();
                if self.matches.len() < 5 {
                    if let Some(ref cb) = self.onneed_more {
                        cb.emit((*self.term.borrow()).clone());
                    }
                }
                true
            } else {
                false
            },
        }
    }

    fn change(&mut self, p: Self::Properties) -> ShouldRender {
        if let Some(c) = p.completions {
            self.completions = c;
            self.filter_compls();
            true
        } else {
            false
        }
    }
}

impl Renderable<Search> for Search {
    fn view(&self) -> Html<Self> {
        html! {
            <div class=("dropdown", "is-active"),>
             <div class="dropdown-trigger",>
              <input
                class=("input","is-rounded"), type="text", placeholder="Search term",
                value=self.term.borrow(),
                oninput=|e| Msg::Term(e.value),/>
             </div>
             <div class="dropdown-menu", role="menu",>
              <div class="dropdown-content",>{
                  for self.matches.iter().map(|i| {
                      html! {
                          <a class="dropdown-item",>{ &self.completions[*i].0 }</a>
                      }
                  })
              }</div>
             </div>
            </div>
        }
    }
}
