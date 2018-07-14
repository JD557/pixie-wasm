extern crate stdweb;
#[macro_use]
extern crate yew;
extern crate csv;
extern crate pixie_rust;

use pixie_rust::recommender::Recommender;
use std::collections::HashMap;
use std::time::Duration;
use stdweb::web::Date;
use stdweb::web::XhrReadyState;
use stdweb::web::XmlHttpRequest;
use yew::prelude::*;
use yew::services::{ConsoleService, IntervalService, Task};

pub struct Model {
    console: ConsoleService,
    data_request: XmlHttpRequest,
    data_loaded: bool,
    data_load_task: Box<Task>,
    ratings: HashMap<String, f32>,
    query: String,
    suggested_query: String,
    recommendation: String,
    recommender: Recommender<String>,
}

pub enum Msg {
    GetRecommendation(String),
    LoadData,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut interval = IntervalService::new();
        let load_data_callback = link.send_back(|_| Msg::LoadData);
        let data_load_task = interval.spawn(Duration::from_secs(1), load_data_callback.into());
        Model {
            console: ConsoleService::new(),
            data_request: XmlHttpRequest::new(),
            data_loaded: false,
            data_load_task: Box::new(data_load_task),
            ratings: HashMap::new(),
            query: String::from(""),
            suggested_query: String::from(""),
            recommendation: String::from(""),
            recommender: Recommender::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::LoadData => {
                let mut update_required = false;
                if !self.data_loaded {
                    if self.data_request.ready_state() == XhrReadyState::Unsent {
                        self.console.log("Loading Data...");
                        self.data_request.open("GET", "anime.csv").unwrap();
                        self.data_request.send().unwrap();
                    }
                    if self.data_request.ready_state() == XhrReadyState::Done {
                        self.console.log("Parsing CSV...");
                        let csv = self.data_request.response_text().unwrap().unwrap();
                        let mut csv_reader = csv::Reader::from_reader(csv.as_bytes());
                        for entry_res in csv_reader.records() {
                            let entry = entry_res.unwrap();
                            let name = entry.get(1).unwrap();
                            let categories_str = entry.get(2).unwrap();
                            let rating = entry.get(5).unwrap().parse::<f32>().unwrap_or(0.0);
                            self.ratings.insert(String::from(name), rating);
                            self.recommender.add_object(&String::from(name));
                            let categories = categories_str.split(",");
                            for cat in categories {
                                let trimmed = cat.trim();
                                self.recommender.add_tag(trimmed);
                                self.recommender.tag_object(&String::from(name), trimmed);
                            }
                        }
                        self.console.log("Data Loaded!");
                        self.data_loaded = true;
                        update_required = true;
                    }
                }
                update_required
            }
            Msg::GetRecommendation(str) => {
                let new_recommendation = self.recommender
                    .object_recommendations(
                        &vec![str.clone()],
                        15,
                        500,
                        &(|_, _| 1.0),
                        &(|_, to| match to {
                            name => self.ratings.get(name).unwrap_or(&0.0).clone(),
                        }),
                    )
                    .iter()
                    .next()
                    .unwrap_or(&String::from(""))
                    .clone();

                let mut new_suggestion = self.suggested_query.clone();

                if str.is_empty() {
                    new_suggestion = String::from("");
                } else if str.len() > 3 {
                    new_suggestion = self.ratings
                        .keys()
                        .find(|name| name.to_lowercase().contains(&str.to_lowercase()))
                        .unwrap_or(&String::from(""))
                        .clone();
                }

                if self.recommendation != new_recommendation
                    || self.suggested_query != new_suggestion
                {
                    self.recommendation = new_recommendation;
                    self.suggested_query = new_suggestion;
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let data_message: String = if self.data_loaded {
            String::from("Data loaded")
        } else {
            String::from("Loading data...")
        };
        let mut recommendation = self.recommendation.clone();
        if !recommendation.is_empty() {
            recommendation = recommendation
                + " ("
                + &format!("{}", self.ratings.get(&self.recommendation).unwrap())
                + ")";
        }
        html! {
            <div>
                <nav class="menu",>
                    <strong>{"Anime name (Case sensitive): "}</strong>
                    <input value=&self.query,
                        oninput=|e| Msg::GetRecommendation(e.value),
                        placeholder="Query",/>
                    { " " }
                    { self.suggested_query.clone() }
                </nav>
                <p><strong>{ "Data status: " }</strong>{ data_message }</p>
                <p><strong>{ "Recommendation: " }</strong>{ recommendation }</p>
                <p><strong>{ "Last component update: " }</strong>{ Date::new().to_string() }</p>
            </div>
        }
    }
}
