use common::*;
use crate::util::*;
use std::collections::{ VecDeque, BTreeSet };
use std::time::Duration;
use super::user_message::UserMessage;
use super::value_button::ValueButton;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalTask, IntervalService, };

const DIAGNOSTIC_POLLING_RATE: Duration = Duration::from_millis(1000);
const MAX_BUFFER_SIZE: usize = 0x50;

pub enum Msg {
    ClearBuffer,
    ToggleBeaconSelected(MacAddress8),

    RequestDiagnostics,

    ResponseDiagnostics(JsonResponse<common::DiagnosticData>),
}

pub struct Diagnostics {
    active_beacons: BTreeSet<MacAddress8>,
    diagnostic_data: VecDeque<common::TagData>,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    interval_service: Option<IntervalService>,
    interval_service_task: Option<IntervalTask>,
    selected_beacons: BTreeSet<MacAddress8>,
    self_link: ComponentLink<Diagnostics>,
    user_msg: UserMessage<Self>,
}

impl JsonResponseHandler for Diagnostics {}

#[derive(Properties)]
pub struct DiagnosticsProps {
    pub emergency: bool,
}

impl Diagnostics {
    fn start_service(&mut self) {
        let mut interval_service = IntervalService::new();
        self.interval_service_task = Some(interval_service.spawn(DIAGNOSTIC_POLLING_RATE, self.self_link.send_back(|_| Msg::RequestDiagnostics)));
        self.interval_service = Some(interval_service);
    }

    fn end_service(&mut self) {
        self.interval_service = None;
        self.interval_service_task = None;
    }
}

impl Component for Diagnostics {
    type Message = Msg;
    type Properties = DiagnosticsProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut result = Diagnostics {
            active_beacons: BTreeSet::new(),
            diagnostic_data: VecDeque::new(),
            emergency: props.emergency,
            fetch_service: FetchService::new(),
            fetch_task: None,
            interval_service: None,
            interval_service_task: None,
            selected_beacons: BTreeSet::new(),
            self_link: link,
            user_msg: UserMessage::new(),
        };

        if result.emergency {
            result.start_service();
        }
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ClearBuffer => {
                self.diagnostic_data = VecDeque::new();
            },
            Msg::ToggleBeaconSelected(b_mac) => {
                if self.selected_beacons.contains(&b_mac) {
                    self.selected_beacons.remove(&b_mac);
                } else {
                    self.selected_beacons.insert(b_mac.clone());
                }
            },
            Msg::RequestDiagnostics => {
                self.user_msg.reset();
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &system_diagnostics_url(),
                    self.self_link,
                    Msg::ResponseDiagnostics
                );
            },
            Msg::ResponseDiagnostics(response) => {
                self.handle_response(
                    response,
                    |s, diagnostics_data| {
                        for point in diagnostics_data.tag_data.into_iter() {
                            if !s.active_beacons.contains(&point.beacon_mac) {
                                s.active_beacons.insert(point.beacon_mac.clone());
                                s.selected_beacons.insert(point.beacon_mac.clone());
                            }
                            s.diagnostic_data.push_front(point);
                        }
                        s.diagnostic_data.truncate(MAX_BUFFER_SIZE);
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to obtain diagnostics, reason: {}", e));
                    },
                );
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.emergency = props.emergency;

        if self.emergency {
            self.start_service();
        } else {
            self.end_service();
            self.diagnostic_data = VecDeque::new();
        }

        true
    }
}

impl Renderable<Diagnostics> for Diagnostics {
    fn view(&self) -> Html<Self> {
        if self.diagnostic_data.len() > 0 {
            let mut beacon_selections = self.active_beacons.iter().map(|b_mac| {
                let set_border = self.selected_beacons.contains(b_mac);
                html! {
                    <ValueButton<String>
                        on_click=|value: String| Msg::ToggleBeaconSelected(MacAddress8::parse_str(&value).unwrap()),
                        border=set_border,
                        value={b_mac.to_hex_string()}
                        style= { if set_border {"btn-secondary"} else {"btn-outline-secondary"} },
                    />
                }
            });
            let filtered: Vec<common::TagData> = self.diagnostic_data.iter().filter(|point| self.selected_beacons.contains(&point.beacon_mac)).cloned().collect();

            let mut diagnostic_rows = filtered.iter().map(|row| {
                html! {
                    <tr>
                        <td>{ &row.beacon_mac }</td>
                        <td>{ &row.tag_mac }</td>
                        <td>{ &row.tag_distance }</td>
                        <td>{ format_timestamp(&row.timestamp) }</td>
                    </tr>
                }
            });

            html! {
                <>
                    <button
                        type="button" class="btn btn-lg btn-warning"
                        onclick=|_| Msg::ClearBuffer,
                    >
                        <i class="fa fa-recycle" aria-hidden="true"></i>
                        { " Reset Data" }
                    </button>
                    { self.user_msg.view() }
                    <div class="content-wrapper">
                        <div class="boxedForm">
                            <div>
                                <h2>{ "Diagnostics" }</h2>
                                <tr>{ for beacon_selections }</tr>
                            </div>
                            <table class="table table-striped">
                                <thead>
                                    <tr>
                                        <th>{ "Beacon Mac" }</th>
                                        <th>{ "User Mac" }</th>
                                        <th>{ "Distance" }</th>
                                        <th>{ "Timestamp" }</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    { for diagnostic_rows }
                                </tbody>
                            </table>
                        </div>
                    </div>
                </>
            }
        } else {
            html! {
                <>
                    <div class="content-wrapper">
                        <div class="boxedForm">
                            <h2>{ "Diagnostics" }</h2>
                            <h4>{ "No diagnostics yet..." }</h4>
                        </div>
                    </div>
                </>
            }
        }
    }
}
