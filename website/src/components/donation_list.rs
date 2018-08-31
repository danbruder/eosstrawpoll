use agents::chain::*;
use components::Link;
use prelude::*;
use std::cmp::min;

pub struct DonationList {
    props: Props,
    donations: ChainData<Vec<Donation>>,
    _chain_agent: Box<Bridge<ChainAgent>>,
}

pub enum Msg {
    Chain(ChainOutput),
}

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub context: Context,
    pub lower_bound: Option<String>,
    pub upper_bound: Option<String>,
    pub limit: Option<usize>,
    pub chain: Chain,
}

impl Component for DonationList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut chain_agent = ChainAgent::new(props.chain.clone(), link.send_back(Msg::Chain));
        chain_agent.send(ChainInput::GetNewDonations);
        DonationList {
            props,
            donations: ChainData::default(),
            _chain_agent: chain_agent,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Chain(output) => match output {
                ChainOutput::NewDonations(donations) => {
                    self.donations = donations;
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

impl Renderable<DonationList> for DonationList {
    fn view(&self) -> Html<Self> {
        match &self.donations {
            ChainData::NotAsked => self.view_empty(),
            ChainData::Loading => self.view_loading(),
            ChainData::Success(data) => {
                if data.is_empty() {
                    self.view_empty()
                } else {
                    self.view_items(&data)
                }
            }
            ChainData::Failure(error) => self.view_error(error),
        }
    }
}

impl DonationList {
    fn view_loading(&self) -> Html<Self> {
        html! {
            <div class="donation_list_loading", >
                { "Loading..." }
            </div>
        }
    }

    fn view_error(&self, error: &ChainError) -> Html<Self> {
        html! {
            <div class="donation_list_loading", >
                { "Error: " }{ format!("{:#?}", error) }
            </div>
        }
    }

    fn view_items(&self, donations: &[Donation]) -> Html<Self> {
        let limit = min(donations.len(), self.props.limit.unwrap_or_else(|| 10));
        html! {
            <ul class="donation_list_items", >
                { for donations[0..limit].iter().map(|donation| self.view_item(donation)) }
            </ul>
        }
    }

    fn view_item(&self, donation: &Donation) -> Html<Self> {
        let donor_route = Route::Profile(
            self.props.chain.to_chain_id_prefix(),
            donation.account.clone(),
        );
        let donated = donation.donated as f64;
        html! {
            <li class="donation_list_item", >
                <Link: class="donation_account",
                    route=donor_route,
                    text=donation.account.clone(),
                />
                <div class="donation_donated", >
                    { format!("{:.*} {}", 4, donated / 10000., self.props.chain.core_symbol) }
                </div>
            </li>
        }
    }

    fn view_empty(&self) -> Html<Self> {
        html! {
            <div class="donation_list_empty", >
                { "Empty" }
            </div>
        }
    }
}
