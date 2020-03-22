use common::*;
use crate::util::*;
use super::root;
use super::value_button::{ ValueButton, DisplayButton };
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::prelude::*;
use super::user_message::UserMessage;

pub enum Msg {
    ChangeRootPage(root::Page),

    RequestDeleteBeacon(i32),
    RequestGetBeacons,
    RequestCommandBeacon(BeaconRequest),

    ResponseGetBeacons(JsonResponse<Vec<(Beacon, Option<Map>)>>),
    ResponseDeleteBeacon(JsonResponse<()>),
    ResponseCommandBeacon(JsonResponse<()>),
}
pub struct BeaconList {
    change_page: Callback<root::Page>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    list: Vec<(Beacon, Option<Map>)>,
    self_link: ComponentLink<Self>,
    user_msg: UserMessage<Self>,
}

impl JsonResponseHandler for BeaconList {}

#[derive(Properties)]
pub struct BeaconListProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for BeaconList {
    type Message = Msg;
    type Properties = BeaconListProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetBeacons);
        let result = BeaconList {
            fetch_service: FetchService::new(),
            list: Vec::new(),
            fetch_task: None,
            self_link: link,
            change_page: props.change_page,
            user_msg: UserMessage::new(),
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestGetBeacons => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &format!("{}?prefetch=true", beacons_url()),
                    self.self_link,
                    Msg::ResponseGetBeacons
                );
            },
            Msg::RequestCommandBeacon(command) => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &beacon_command_url(),
                    command,
                    self.self_link,
                    Msg::ResponseCommandBeacon
                );
            },
            Msg::RequestDeleteBeacon(id) => {
                self.fetch_task = delete_request!(
                    self.fetch_service,
                    &beacon_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseDeleteBeacon
                );
            },
            Msg::ResponseGetBeacons(response) => {
                self.handle_response(
                    response,
                    |s, mut beacons_and_maps| {
                        beacons_and_maps.sort_unstable_by(|(beacon_a, _), (beacon_b, _)| beacon_a.name.cmp(&beacon_b.name));
                        s.list = beacons_and_maps;
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to obtain beacon list, reason: {}", e));
                    },
                );
            },
            Msg::ResponseCommandBeacon(response) => {
                self.handle_response(
                    response,
                    |s, _| {
                        s.user_msg.success_message = Some("Successfully sent command".to_owned());
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to command beacon, reason: {}", e));
                    },
                );
            },
            Msg::ResponseDeleteBeacon(response) => {
                self.handle_response(
                    response,
                    |s, _| {
                        s.user_msg.success_message = Some("Successfully deleted beacon".to_owned());
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to delete beacon, reason: {}", e));
                    },
                );
                // now that the beacon is deleted, get the updated list
                self.self_link.send_self(Msg::RequestGetBeacons);
            },
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        self.self_link.send_self(Msg::RequestGetBeacons);
        true
    }
}

impl Renderable<BeaconList> for BeaconList {
    fn view(&self) -> Html<Self> {
        let mut rows = self.list.iter().map(|(beacon, opt_map)| {
            let map_name = opt_map.as_ref().map(|m| m.name.as_ref()).unwrap_or("");
            html! {
                <tr>
                    <td>{ &beacon.mac_address.to_hex_string() }</td>
                    <td>{ format!("{},{}", &beacon.coordinates.x, &beacon.coordinates.y) }</td>
                    <td>{ map_name }</td>
                    <td>{ &beacon.name }</td>
                    <td>{ beacon.note.clone().unwrap_or(String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Edit".to_owned()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::BeaconAddUpdate(Some(value))),
                            border=false,
                            icon="fa fa-pencil-square-o",
                            style="btn-primary",
                            value={beacon.id}
                        />
                        <ValueButton<i32>
                            display=Some("Delete".to_owned()),
                            on_click=|value: i32| Msg::RequestDeleteBeacon(value),
                            border=false,
                            icon="fa fa-trash",
                            style="btn-secondary",
                            value=beacon.id
                        />
                        <DisplayButton<BeaconRequest>
                            display="Ping".to_owned(),
                            on_click=|value| Msg::RequestCommandBeacon(value),
                            border=false,
                            value=BeaconRequest::Ping(Some(beacon.mac_address)),
                            icon="fa fa-signal",
                            style="btn btn-sm btn-info",
                        />
                        <DisplayButton<BeaconRequest>
                            display="Reboot".to_owned(),
                            on_click=|value| Msg::RequestCommandBeacon(value),
                            border=false,
                            value=BeaconRequest::Reboot(Some(beacon.mac_address)),
                            icon="fa fa-refresh",
                            style="btn btn-sm btn-info",
                        />
                    </td>
                </tr>
            }
        });

        html! {
            <>
                { self.user_msg.view() }
                <div class="content-wrapper">
                    <div class="boxedForm">
                        <div class="d-flex justify-content-between">
                            <h2>{ "Beacon List"}</h2>
                            <button
                                class="btn btn-success logoutPlacement my-1",
                                onclick=|_| Msg::ChangeRootPage(root::Page::BeaconAddUpdate(None)),
                            >
                                {"Add Beacon"}
                            </button>
                        </div>
                        <table class="table table-striped">
                            <thead>
                                <tr>
                                    <th>{ "Mac" }</th>
                                    <th>{ "Coordinates" }</th>
                                    <th>{ "Floor" }</th>
                                    <th>{ "Name" }</th>
                                    <th>{ "Note" }</th>
                                    <th>{ "Actions" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for rows }
                            </tbody>
                        </table>
                    </div>
                </div>
            </>
        }
    }
}
