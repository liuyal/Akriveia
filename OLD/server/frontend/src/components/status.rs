use common::*;
use crate::util::*;
use std::collections::HashMap;
use std::time::Duration;
use super::root;
use super::value_button::{ ValueButton, DisplayButton, };
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use super::user_message::UserMessage;

const POLL_RATE: Duration = Duration::from_millis(1000);

#[derive(PartialEq, Copy, Clone)]
pub enum PageState {
    BeaconStatus,
    UserStatus,
}

pub enum Msg {
    ChangeRootPage(root::Page),
    ChangeStatus(PageState),

    RequestCommandBeacon(BeaconRequest),
    RequestGetBeacons,
    RequestGetBeaconsStatus,
    RequestGetMap(i32),
    RequestGetMaps,
    RequestGetUser(i32),
    RequestGetUsers,
    RequestGetUsersStatus,

    ResponseCommandBeacon(JsonResponse<()>),
    ResponseGetBeacons(JsonResponse<Vec<Beacon>>),
    ResponseGetBeaconsStatus(JsonResponse<Vec<RealtimeBeacon>>),
    ResponseGetMap(JsonResponse<Map>),
    ResponseGetMaps(JsonResponse<Vec<Map>>),
    ResponseGetUser(JsonResponse<TrackedUser>),
    ResponseGetUsers(JsonResponse<Vec<TrackedUser>>),
    ResponseGetUsersStatus(JsonResponse<Vec<RealtimeUserData>>),
}

pub struct Status {
    beacons: HashMap<i32, Beacon>,
    change_page: Callback<root::Page>,
    fetch_service: FetchService,
    interval_service: IntervalService,
    interval_service_task: Option<IntervalTask>,
    maps: HashMap<i32, Map>,
    self_link: ComponentLink<Self>,
    state: PageState,
    user_msg: UserMessage<Self>,
    user_type: WebUserType,
    users: HashMap<i32, TrackedUser>,

    // ugh.
    fetch_commands: Option<FetchTask>,
    fetch_beacons: Option<FetchTask>,
    fetch_beacons_status: Option<FetchTask>,
    fetch_users: Option<FetchTask>,
    fetch_user: Option<FetchTask>,
    fetch_user_status: Option<FetchTask>,
    fetch_maps: Option<FetchTask>,
    fetch_map: Option<FetchTask>,
}

impl JsonResponseHandler for Status {}

impl Status {
    fn restart_service(&mut self) {
        let callback = match self.state {
            PageState::UserStatus => self.self_link.send_back(|_| Msg::RequestGetUsersStatus),
            PageState::BeaconStatus => self.self_link.send_back(|_| Msg::RequestGetBeaconsStatus),
        };
        self.interval_service_task = Some(self.interval_service.spawn(POLL_RATE, callback));
    }
}

#[derive(Properties)]
pub struct StatusProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
    #[props(required)]
    pub state: PageState,
    #[props(required)]
    pub user_type: WebUserType,
}

impl Component for Status {
    type Message = Msg;
    type Properties = StatusProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetBeacons);
        link.send_self(Msg::RequestGetUsers);
        link.send_self(Msg::RequestGetMaps);
        let mut result = Status {
            beacons: HashMap::new(),
            change_page: props.change_page,
            fetch_service: FetchService::new(),
            interval_service: IntervalService::new(),
            interval_service_task: None,
            maps: HashMap::new(),
            self_link: link,
            state: props.state,
            user_msg: UserMessage::new(),
            user_type: props.user_type,
            users: HashMap::new(),

            fetch_beacons: None,
            fetch_commands: None,
            fetch_beacons_status: None,
            fetch_map: None,
            fetch_maps: None,
            fetch_user: None,
            fetch_user_status: None,
            fetch_users: None,
        };

        result.restart_service();
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeStatus(state) => {
                self.state = state;
                self.restart_service();
            }
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
            Msg::RequestGetMaps => {
                self.user_msg.reset();
                self.fetch_maps = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetMaps
                );
            },
            Msg::RequestGetMap(id) => {
                self.user_msg.reset();
                self.fetch_map = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::RequestGetBeacons => {
                self.user_msg.reset();
                self.fetch_beacons = get_request!(
                    self.fetch_service,
                    &beacons_url(),
                    self.self_link,
                    Msg::ResponseGetBeacons
                );
            },
            Msg::RequestGetBeaconsStatus => {
                self.user_msg.reset();
                self.fetch_beacons_status = get_request!(
                    self.fetch_service,
                    &beacons_status_url(),
                    self.self_link,
                    Msg::ResponseGetBeaconsStatus
                );
            },
            Msg::RequestCommandBeacon(command) => {
                self.user_msg.reset();
                self.fetch_commands = post_request!(
                    self.fetch_service,
                    &beacon_command_url(),
                    command,
                    self.self_link,
                    Msg::ResponseCommandBeacon
                );
            },
            Msg::RequestGetUsers => {
                self.user_msg.reset();
                self.fetch_users = get_request!(
                    self.fetch_service,
                    &users_url(),
                    self.self_link,
                    Msg::ResponseGetUsers
                );
            },
            Msg::RequestGetUsersStatus => {
                self.user_msg.reset();
                self.fetch_user_status = get_request!(
                    self.fetch_service,
                    &users_status_url(),
                    self.self_link,
                    Msg::ResponseGetUsersStatus
                );
            },
            Msg::RequestGetUser(id) => {
                self.user_msg.reset();
                self.fetch_user = get_request!(
                    self.fetch_service,
                    &user_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetUser
                );
            },
            Msg::ResponseGetMaps(response) => {
                self.handle_response(
                    response,
                    |s, maps| {
                        for map in maps {
                            s.maps.insert(map.id, map);
                        }
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to obtain maps list, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetMap(response) => {
                self.handle_response(
                    response,
                    |s, map| {
                        s.maps.insert(map.id, map);
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to get map, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetBeacons(response) => {
                self.handle_response(
                    response,
                    |s, beacons| {
                        for b in beacons {
                            // dont add any beacons that dont have floors, first responders dont
                            // need to see beacons that dont have any use.
                            if b.map_id.is_some() {
                                if let Some(mid) = b.map_id {
                                    if !s.maps.contains_key(&mid) {
                                        let mid = mid.clone();
                                        s.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                    }
                                }
                                s.beacons.insert(b.id, b);
                            }
                        }
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to obtain beacon list, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetBeaconsStatus(response) => {
                self.handle_response(
                    response,
                    |s, realtime_beacons| {
                        for rb in realtime_beacons {
                            match s.beacons.get_mut(&rb.id) {
                                Some(b) => {
                                    b.merge(rb);
                                },
                                None => {
                                    // just drop the realtime data for now until
                                    // the user object is retrieved, more realtime data
                                    // will come eventually and the UI user likely wont
                                    // notice.
                                    s.self_link
                                        .send_back(move |_: ()| Msg::RequestGetUser(rb.id));
                                }
                            }
                        }
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to get beacons status, reason: {}", e));
                    },
                );
            },
            Msg::ResponseCommandBeacon(response) => {
                let (meta, Json(_body)) = response.into_parts();
                if meta.status.is_success() {
                    self.user_msg.success_message = Some("Successfully sent command".to_string());
                } else {
                    self.user_msg.error_messages.push("failed to send command".to_owned());
                }
            },
            Msg::ResponseGetUser(response) => {
                self.handle_response(
                    response,
                    |s, user| {
                        if let Some(mid) = user.map_id {
                            if !s.maps.contains_key(&mid) {
                                let mid = mid.clone();
                                s.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                            }
                        }
                        s.users.insert(user.id, user);
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to get user, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetUsers(response) => {
                self.handle_response(
                    response,
                    |s, users| {
                        for user in users {
                            if let Some(mid) = user.map_id {
                                if !s.maps.contains_key(&mid) {
                                    let mid = mid.clone();
                                    s.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                }
                            }
                            s.users.insert(user.id, user);
                        }
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to get user list, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetUsersStatus(response) => {
                self.handle_response(
                    response,
                    |s, users| {
                        for ru in users {
                            match s.users.get_mut(&ru.id) {
                                Some(u) => {
                                    u.merge(ru);
                                },
                                None => {
                                    // just drop the realtime data for now until
                                    // the user object is retrieved, more realtime data
                                    // will come eventually and the UI user likely wont
                                    // notice.
                                    s.self_link
                                        .send_back(move |_: ()| Msg::RequestGetUser(ru.id));
                                }
                            }
                        }
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to get user status, reason: {}", e));
                    },
                );
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.state != props.state {
            self.self_link.send_self(Msg::ChangeStatus(props.state));
        }
        true
    }
}

lazy_static! {
    static ref DEFAULT_MAP: Map = Map::new();
}

impl Status {

    fn beacon_table(&self) -> Html<Self> {
        let mut rows = self.beacons.iter().map(|(_id, beacon)| {
            let (map, valid_map) = match beacon.map_id {
                Some(mid) => {
                    match self.maps.get(&mid) {
                        Some(map) => (map, true),
                        None => {
                            (&*DEFAULT_MAP, true) // render default map until the correct one loads
                        }
                    }
                },
                None => {
                    (&*DEFAULT_MAP, false) // this beacon doesnt have a map
                }
            };

            let command_buttons = match self.user_type {
                WebUserType::Admin => html! {
                    <>
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
                    </>
                },
                _ => html! {},
            };

            html! {
                <tr>
                    <td>{ &beacon.name }</td>
                    <td>{ &beacon.state }</td>
                    <td>{ format_timestamp(&beacon.last_active) }</td>
                    <td>{ format!("{:.3},{:.3}", &beacon.coordinates.x, &beacon.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ &beacon.mac_address.to_hex_string() }</td>
                    <td>{ beacon.note.as_ref().unwrap_or(&String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Details".to_owned()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::BeaconAddUpdate(Some(value))),
                            border=false,
                            icon = "fa fa-book",
                            style="btn-primary",
                            value={beacon.id},
                        />
                        <DisplayButton<Option<i32>>
                            display="Map".to_owned(),
                            on_click=|opt_map_id: Option<i32>| Msg::ChangeRootPage(root::Page::MapView(opt_map_id)),
                            border=false,
                            disabled=!valid_map,
                            value={beacon.map_id},
                            icon = "fa fa-external-link",
                            style="btn btn-sm btn-secondary",
                        />
                        { command_buttons }
                    </td>
                </tr>
            }
        });

        html! {
            <>
                <div class="content-wrapper">
                    <div class="boxedForm">
                        <h2>{ "Beacon Status" }</h2>
                        <table class="table table-striped">
                            <thead>
                                <tr>
                                    <th>{ "Name" }</th>
                                    <th>{ "State" }</th>
                                    <th>{ "Last Active" }</th>
                                    <th>{ "Coordinates" }</th>
                                    <th>{ "Floor" }</th>
                                    <th>{ "Mac" }</th>
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

    fn user_table(&self) -> Html<Self> {
        let mut rows = self.users.iter().map(|(_id, user)| {
            let (map, valid_map) = match user.map_id {
                Some(mid) => {
                    match self.maps.get(&mid) {
                        Some(map) => (map, true),
                        None => {
                            (&*DEFAULT_MAP, false) // render default map until the correct one loads
                        }
                    }
                },
                None => {
                    (&*DEFAULT_MAP, false) // this user doesnt have a map
                }
            };

            html! {
                <tr>
                    <td>{ &user.name }</td>
                    <td>{ format!("{:.3},{:.3}", &user.coordinates.x, &user.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ format_timestamp(&user.last_active) }</td>
                    <td>{ &user.note.as_ref().unwrap_or(&String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Details".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::UserAddUpdate(Some(value))),
                            border=false,
                            icon = "fa fa-book",
                            style="btn-primary",
                            value={user.id}
                        />
                        <DisplayButton<Option<i32>>
                            display="Map".to_string(),
                            on_click=|opt_map_id: Option<i32>| Msg::ChangeRootPage(root::Page::MapView(opt_map_id)),
                            border=false,
                            disabled=!valid_map,
                            style="btn btn-sm btn-secondary",
                            icon = "fa fa-external-link",
                            value={user.map_id},
                        />
                    </td>
                </tr>
            }
        });

        html! {
            <>
                <div class="content-wrapper">
                    <div class="boxedForm">
                        <h2>{ "User Status" }</h2>
                        <table class="table table-striped">
                            <thead>
                                <tr>
                                    <th>{ "Name" }</th>
                                    <th>{ "Coordinates" }</th>
                                    <th>{ "Floor" }</th>
                                    <th>{ "Last Seen" }</th>
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

impl Renderable<Status> for Status {
    fn view(&self) -> Html<Self> {
        let table = match self.state {
            PageState::BeaconStatus => self.beacon_table(),
            PageState::UserStatus => self.user_table(),
        };

        html! {
            <>
                { self.user_msg.view() }
                { table }
            </>

        }
    }
}
