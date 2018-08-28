use agents::router::RouterAgent;
use agents::scatter::{
    self, ScatterAction, ScatterAgent, ScatterError, ScatterInput, ScatterOutput,
};
use components::Link;
use context::Context;
use failure::Error;
use route::Route;
use services::eos::{self, EosService};
use stdweb::traits::IEvent;
use stdweb::web::document;
use traits::Page;
use types::*;
use yew::prelude::*;
use yew::services::fetch::FetchTask;

pub struct PollVotingPage {
    eos: EosService,
    context: Context,
    task: Option<FetchTask>,
    poll: Option<Result<Poll, Error>>,
    creator: String,
    slug: String,
    scatter_agent: Box<Bridge<ScatterAgent>>,
    scatter_connected: Option<Result<(), ScatterError>>,
    scatter_identity: Option<Result<scatter::Identity, ScatterError>>,
    pushed: Option<Result<scatter::PushedTransaction, ScatterError>>,
    choices: Vec<Choice>,
    writein_input: String,
    submitting: bool,
    link: ComponentLink<PollVotingPage>,
}

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub context: Context,
    pub creator: String,
    pub slug: String,
    pub chain_id_prefix: String,
}

pub enum Msg {
    Polls(Result<eos::TableRows<Poll>, Error>),
    Scatter(ScatterOutput),
    ToggleChoice(Choice),
    SetWriteinInput(String),
    AddWritein,
    Vote,
}

impl Component for PollVotingPage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let creator = props.creator;

        let callback = link.send_back(Msg::Scatter);
        let scatter_agent = ScatterAgent::new("eosstrawpoll", 10000, callback);

        let context = props.context;
        let mut poll_page = PollVotingPage {
            eos: EosService::new(),
            context,
            task: None,
            poll: None,
            slug: props.slug.clone(),
            creator: creator.clone(),
            scatter_agent,
            scatter_connected: None,
            scatter_identity: None,
            pushed: None,
            choices: Vec::new(),
            writein_input: "".to_string(),
            submitting: false,
            link,
        };

        poll_page.fetch_poll();
        poll_page
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Polls(result) => {
                self.poll = match result {
                    Ok(table) => match table.rows.first() {
                        Some(poll) => Some(Ok(poll.clone())),
                        None => Some(Err(format_err!("poll not found"))),
                    },
                    Err(error) => Some(Err(error)),
                };
                self.task = None;
                self.load_choices();

                true
            }
            Msg::Scatter(output) => match output {
                ScatterOutput::GotIdentity(result) => {
                    let is_ok = result.is_ok();
                    self.scatter_identity = Some(result);
                    match (is_ok, self.submitting) {
                        (false, true) => {
                            self.submitting = false;
                            true
                        }
                        (true, true) => self.update(Msg::Vote),
                        (true, false) => {
                            self.load_choices();
                            true
                        }
                        (false, false) => true,
                    }
                }
                ScatterOutput::ForgotIdentity(_result) => {
                    self.scatter_identity = None;
                    true
                }
                ScatterOutput::Connected(result) => {
                    if result.is_ok() {
                        self.scatter_agent.send(ScatterInput::CurrentIdentity);
                    }
                    self.scatter_connected = Some(result);
                    true
                }
                ScatterOutput::PushedActions(result) => {
                    if self.submitting {
                        self.pushed = Some(result.clone());
                        self.submitting = false;
                        if result.is_ok() {
                            let route = Route::PollResults(
                                "cf057bbfb726".into(),
                                self.creator.clone(),
                                self.slug.clone(),
                            );
                            let url = route.to_string();
                            RouterAgent::redirect(url);
                            true
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                }
            },
            Msg::ToggleChoice(choice) => {
                info!("CHOICES: {:#?}, CHOICE: {:#?}", self.choices, choice);
                if self.choices.contains(&choice) {
                    self.choices.retain(|c| choice != *c);
                } else {
                    self.choices.push(choice);
                }
                if let Some(Ok(poll)) = &self.poll {
                    if self.choices.len() > poll.max_choices {
                        self.choices.remove(0);
                    }
                }
                true
            }
            Msg::Vote => {
                info!("submitting form");
                self.submitting = true;

                let voter = match self.voter() {
                    Some(voter) => voter,
                    None => {
                        let required_fields = self.context.required_fields();
                        let scatter_input = ScatterInput::GetIdentity(required_fields);
                        self.scatter_agent.send(scatter_input);
                        return true;
                    }
                };

                let network = self.context.network();
                let config = self.context.eos_config();

                let action: ScatterAction = CreateVote {
                    creator: self.creator.to_string(),
                    slug: self.slug.clone(),
                    voter: voter.clone(),
                    choices: self.choices.clone(),
                }.into();
                let actions = vec![action];

                self.scatter_agent
                    .send(ScatterInput::PushActions(network, config, actions));
                true
            }
            Msg::SetWriteinInput(input) => {
                self.writein_input = input;
                true
            }
            Msg::AddWritein => {
                self.choices
                    .push(Choice::from_writein(self.writein_input.clone()));
                self.writein_input = "".to_string();
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}

impl Page for PollVotingPage {
    fn title(&self) -> String {
        match &self.poll {
            Some(result) => match result {
                Ok(poll) => poll.title.clone(),
                Err(_error) => "Error".to_string(),
            },
            None => "Loading...".to_string(),
        }
    }
    fn class(&self) -> String {
        let state_modifier = match &self.poll {
            Some(result) => match result {
                Ok(_) => "loaded",
                Err(_) => "error",
            },
            None => "loading",
        };

        format!("poll_page poll_page_vote poll_page_{}", state_modifier)
    }
    fn content(&self) -> Html<Self> {
        match &self.poll {
            Some(result) => match result {
                Ok(poll) => self.view_ok(poll),
                Err(error) => self.view_error(error),
            },
            None => self.view_loading(),
        }
    }
}

page_view! { PollVotingPage }

impl PollVotingPage {
    fn fetch_poll(&mut self) {
        let params = eos::TableRowsParams {
            scope: self.creator.clone(),
            code: "eosstrawpoll".to_string(),
            table: "polls".to_string(),
            json: true,
            lower_bound: Some(self.slug.clone()),
            upper_bound: None,
            limit: Some(1),
            key_type: None,
            index_position: None,
        };

        let callback = self.link.send_back(Msg::Polls);
        let task = self
            .eos
            .get_table_rows(self.context.endpoint.as_str(), params, callback);
        self.task = Some(task);
    }

    fn voter(&self) -> Option<String> {
        let result = match &self.scatter_identity {
            Some(result) => result,
            None => return None,
        };

        let identity = match result {
            Ok(identity) => identity,
            Err(_error) => return None,
        };

        match identity.accounts.first() {
            Some(ref account) => Some(account.name.clone()),
            None => None,
        }
    }

    fn has_voted(&self) -> bool {
        let voter = match self.voter() {
            Some(voter) => voter,
            None => return false,
        };

        let votes = match &self.poll {
            Some(Ok(poll)) => &poll.votes,
            _ => return false,
        };

        votes.into_iter().any(|vote| vote.voter == voter)
    }

    fn load_choices(&mut self) {
        let voter = match self.voter() {
            Some(voter) => voter,
            None => return,
        };

        let votes = match &self.poll {
            Some(Ok(poll)) => &poll.votes,
            _ => return,
        };

        let filtered_votes = votes
            .iter()
            .filter(|vote| vote.voter == voter)
            .cloned()
            .collect::<Vec<Vote>>();

        if let Some(vote) = filtered_votes.first() {
            self.choices = vote.choices.to_owned();
        }
    }

    fn view_loading(&self) -> Html<Self> {
        html! {
            <div>
                <h1>{ "Loading" }</h1>
            </div>
        }
    }

    fn view_error(&self, error: &Error) -> Html<Self> {
        html! {
            <div>
                <h1>{ "Error: " }{ error }</h1>
            </div>
        }
    }

    fn vote_text(&self) -> &str {
        match (self.submitting, self.has_voted()) {
            (false, false) => "Vote",
            (true, false) => "Voting...",
            (false, true) => "Change Vote",
            (true, true) => "Changing Vote...",
        }
    }

    fn view_ok(&self, poll: &Poll) -> Html<Self> {
        //

        let results = Route::PollResults(
            "cf057bbfb726".into(),
            poll.creator.clone(),
            poll.slug.clone(),
        );
        let num_options = poll.options.len();
        let num_choices_text = match (
            poll.min_choices,
            poll.max_choices,
            poll.min_choices == poll.max_choices,
            poll.max_choices == num_options,
        ) {
            (1, 1, _, _) => "choose one option".to_string(),
            (_, _, true, true) => "rank all options".to_string(),
            (_, _, true, false) => format!("choose and rank {} options", poll.min_choices),
            (_, _, false, _) => format!(
                "choose and rank {} to {} options",
                poll.min_choices, poll.max_choices
            ),
        };
        let choose_one = poll.min_choices == 1 && poll.max_choices == 1;
        let select_text = if self.choices.len() < poll.min_choices && !self.choices.is_empty() {
            let diff = poll.min_choices - self.choices.len();
            if diff == 1 {
                "Select one more option".to_string()
            } else {
                format!("Select {} more options", diff)
            }
        } else {
            "".to_string()
        };
        let mut writeins = self.choices.clone();
        writeins.retain(|c| !c.writein.is_empty());
        html! {
            <>
                <p class="poll_num_choices", >
                    { format!("Please {}:", num_choices_text) }
                </p>
                <div class="poll_options", >
                    { for poll.options.iter().enumerate().map(|(i, option)| self.view_option(i as i16, option, choose_one)) }
                    { for writeins.iter().map(|c| self.view_option(-1, &c.writein, choose_one)) }
                </div>
                { self.view_writein_input(poll) }
                <div class="poll_actions", >
                    <button
                        disabled=self.choices.len() < poll.min_choices,
                        type="submit",
                        onclick=|e| {
                            e.prevent_default();
                            Msg::Vote
                        },
                    >
                        { self.vote_text() }
                    </button>
                    <p>{ select_text }</p>
                    <Link: route=results, text="View results", />
                </div>
            </>
        }
    }

    fn view_writein_input(&self, poll: &Poll) -> Html<Self> {
        if poll.max_writeins == 0 {
            return html! { <></> };
        }
        html! {
            <>
                <input class="poll_writein_input",
                    oninput=|e| Msg::SetWriteinInput(e.value),
                    value=&self.writein_input,
                    placeholder="Write in your own answer",
                />
                <button class="poll_writein_button",
                    onclick=|e| {
                        e.prevent_default();
                        Msg::AddWritein
                    },
                >
                    { "Add" }
                </button>
            </>
        }
    }

    fn view_option(&self, index: i16, option: &str, choose_one: bool) -> Html<Self> {
        let choice = Choice {
            option_index: index,
            writein: "".to_string(),
        };
        let is_selected = self.choices.contains(&choice);
        let input = if choose_one {
            html! {
                <input class="poll_option_checkbox",
                    type="radio",
                    name="choices",
                    onchange=|_| Msg::ToggleChoice(choice.clone()),
                    checked=is_selected,
                />
            }
        } else {
            html! {
                <input class="poll_option_checkbox",
                    type="checkbox",
                    onchange=|_| Msg::ToggleChoice(choice.clone()),
                    checked=is_selected,
                />
            }
        };
        html! {
            <label class="poll_option", >
                { input }
                <span class="poll_option_text", >
                    { option }
                </span>
            </label>
        }
    }
}
