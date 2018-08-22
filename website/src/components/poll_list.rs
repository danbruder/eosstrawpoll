use agents::tables::*;
use components::{Link, RelativeTime};
use context::Context;
use route::Route;
use types::Poll;
use yew::prelude::*;

pub struct PollList {
    props: Props,
    polls: Option<Result<Vec<Poll>, String>>,
    table: PollsTable,
    tables: Box<Bridge<TablesAgent>>,
}

#[derive(PartialEq, Clone)]
pub enum PollsTable {
    Polls,
    PopularPolls,
    NewPolls,
}

impl Default for PollsTable {
    fn default() -> PollsTable {
        PollsTable::Polls
    }
}

#[derive(PartialEq, Clone)]
pub enum PollsOrder {
    Id,
    Created,
    Popularity,
}

impl Default for PollsOrder {
    fn default() -> PollsOrder {
        PollsOrder::Id
    }
}

pub enum Msg {
    Tables(TablesOutput),
}

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub context: Context,
    pub scope: String,
    pub table: Option<PollsTable>,
    pub lower_bound: Option<String>,
    pub upper_bound: Option<String>,
    pub limit: Option<u32>,
    pub order: Option<PollsOrder>,
}

impl Component for PollList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let tables_config = props.context.tables_config();
        let mut tables = TablesAgent::new(tables_config, link.send_back(Msg::Tables));

        let table = props.clone().table.unwrap_or_else(|| PollsTable::Polls);

        if table == PollsTable::NewPolls {
            tables.send(TablesInput::GetNewPolls);
        } else if table == PollsTable::PopularPolls {
            tables.send(TablesInput::GetPopularPolls);
        }

        PollList {
            props,
            polls: None,
            tables,
            table,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tables(output) => match (&self.table, output) {
                (PollsTable::NewPolls, TablesOutput::NewPolls(polls)) => {
                    self.polls = Some(polls);
                    if let Some(Ok(ref mut polls)) = self.polls {
                        polls.sort_by(|a, b| b.create_time.cmp(&a.create_time));
                    }
                    true
                }
                (PollsTable::PopularPolls, TablesOutput::PopularPolls(polls)) => {
                    self.polls = Some(polls);
                    if let Some(Ok(ref mut polls)) = self.polls {
                        polls.sort_by(|a, b| {
                            let a_pop: f64 = a.popularity.parse().unwrap();
                            let b_pop: f64 = b.popularity.parse().unwrap();
                            b_pop.partial_cmp(&a_pop).unwrap()
                        });
                    }
                    true
                }
                _ => false,
            },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl Renderable<PollList> for PollList {
    fn view(&self) -> Html<Self> {
        match &self.polls {
            Some(result) => match result {
                Ok(table) => {
                    if table.is_empty() {
                        self.view_empty()
                    } else {
                        self.view_items(&table)
                    }
                }
                Err(error) => self.view_error(error),
            },
            None => self.view_loading(),
        }
    }
}

impl PollList {
    fn view_loading(&self) -> Html<Self> {
        html! {
            <div class="poll_list -loading", >
                { "Loading..." }
            </div>
        }
    }

    fn view_error(&self, error: &str) -> Html<Self> {
        html! {
            <div class="poll_list -error", >
                { "Error: " }{ error }
            </div>
        }
    }

    fn view_empty(&self) -> Html<Self> {
        html! {
            <div class="poll_list -empty", >
                { "Empty" }
            </div>
        }
    }

    fn view_items(&self, polls: &[Poll]) -> Html<Self> {
        html! {
            <ul class="poll_list -loaded", >
                { for polls.iter().map(|poll| self.view_item(poll)) }
            </ul>
        }
    }

    fn view_item(&self, poll: &Poll) -> Html<Self> {
        let poll_route = Route::Poll(poll.creator.clone(), poll.slug.clone());
        let creator_route = Route::Profile(poll.creator.clone());
        html! {
            <li class="poll", >
                <Link: class="poll_title",
                    route=poll_route,
                    text=poll.title.clone(),
                />
                <div class="poll_details", >
                    <Link: class="poll_creator",
                        route=creator_route,
                        text=poll.creator.clone(),
                    />
                    <div class="poll_open_time", >
                        { if poll.is_open() { "Opened " } else { "Opens " } }
                        <RelativeTime: timestamp=poll.open_time, />
                    </div>
                    <div class="poll_votes", >
                        { &poll.votes.len() } { " votes" }
                    </div>
                </div>
            </li>
        }
    }
}
