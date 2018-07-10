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
    ChooseCompl(usize, Uuid),
    Blur,
}

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub term: Rc<RefCell<String>>,
    pub completions: Option<Rc<Vec<(String, Uuid)>>>,
    pub onneed_more: Option<Callback<String>>,
    pub onmatch: Option<Callback<Uuid>>,
}

pub struct Search {
    link: ComponentLink<Search>,
    term: Rc<RefCell<String>>,
    completions: Rc<Vec<(String, Uuid)>>,
    matches: Vec<usize>,
    show_compls: bool,
    exact_match: Option<Uuid>,
    onneed_more: Option<Callback<String>>,
    onmatch: Option<Callback<Uuid>>,
}

impl Search {
    fn filter_compls(&mut self) {
        let term = &*self.term.borrow();
        let compls = &self.completions;

        self.matches.clear();
        self.exact_match = None;
        for i in 0..self.completions.len() {
            if compls[i].0.starts_with(term) {
                self.matches.push(i);
                if &compls[i].0 == term {
                    self.exact_match = Some(compls[i].1);
                }
            }
        }
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
            show_compls: false,
            exact_match: None,
            onneed_more: p.onneed_more,
            onmatch: p.onmatch,
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
                if self.matches.len() > 0 {
                    self.show_compls = true;
                }
                if let Some(uuid) = self.exact_match {
                    if let Some(ref cb) = self.onmatch {
                        cb.emit(uuid);
                    }
                }
                true
            } else {
                false
            },
            Msg::ChooseCompl(i, uuid) => {
                if let Some(ref cb) = self.onmatch {
                    cb.emit(uuid);
                }
                self.show_compls = false;
                self.term.replace(self.completions[i].0.clone());
                true
            },
            Msg::Blur => {
                if self.show_compls {
                    self.show_compls = false;
                    true
                } else {
                    false
                }
            }
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

impl Search {
    fn render_input(&self) -> Html<Self> {
        html! {
            <div class="dropdown-trigger",>{
                if self.exact_match.is_some() {
                    html! {
                        <input
                            class=("input","is-rounded","has-text-weight-bold","is-success"),
                            type="text", value=self.term.borrow(),
                            oninput=|e| Msg::Term(e.value),
                            onblur=|_| Msg::Blur,/>
                    }
                } else {
                    html! {
                        <input
                            class=("input","is-rounded"),
                            type="text", placeholder="Search term",
                            value=self.term.borrow(),
                            oninput=|e| Msg::Term(e.value),/>
                        }
                }
            }</div>
        }
    }

    fn render_compls(&self) -> Html<Self> {
        html! {
            <div class="dropdown-menu", role="menu",>
             <div class="dropdown-content",>{
                 for self.matches.iter().map(|i| {
                     let i = *i;
                     let (ref name, uuid) = self.completions[i];

                     html! {
                         <a class="dropdown-item",
                            onclick=|_| Msg::ChooseCompl(i, uuid),>{
                                name
                         }</a>
                     }
                 })
             }</div>
            </div>
        }
    }
}

impl Renderable<Search> for Search {
    fn view(&self) -> Html<Self> {
        if self.show_compls {
            html! {
                <div class=("dropdown", "is-active"),>
                  { self.render_input() }
                  { self.render_compls() }
                </div>
            }
        } else {
            html! {
                <div class=("dropdown"),>
                { self.render_input() }
                { self.render_compls() }
                </div>
            }
        }
    }
}
