use common::*;
use crate::util::{ self, WebUserType, JsonResponseHandler, };
use super::root;
use super::user_message::UserMessage;
use super::status::{ self };
use yew::Callback;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };

pub enum Msg {
    AddAnotherBeacon,
    ChangeRootPage(root::Page),
    InputCoordinate(usize, String),
    InputFloorName(Option<i32>),
    InputMacAddress(String),
    InputName(String),
    InputNote(String),

    RequestAddUpdateBeacon,
    RequestGetAvailMaps,
    RequestGetBeacon(i32),

    ResponseAddBeacon(util::JsonResponse<Beacon>),
    ResponseGetAvailMaps(util::JsonResponse<Vec<Map>>),
    ResponseGetBeacon(util::JsonResponse<Beacon>),
    ResponseUpdateBeacon(util::JsonResponse<Beacon>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    pub beacon: Beacon,
    // the mac address needs to be parsed (and validated) as a mac address.
    // keep the raw string from the user in case the parsing fails.
    pub avail_floors: Vec<Map>,
    pub id: Option<i32>,
    pub raw_coord0: String,
    pub raw_coord1: String,
    pub raw_mac: String,
}

impl Data {
    fn new() -> Data {
        Data {
            beacon: Beacon::new(),
            avail_floors: Vec::new(),
            id: None,
            raw_coord0: "0".to_string(),
            raw_coord1: "0".to_string(),
            raw_mac: MacAddress8::nil().to_hex_string(),
        }
    }
}

impl BeaconAddUpdate {
    fn validate(&mut self) -> bool {
        let mut success = match MacAddress8::parse_str(&self.data.raw_mac) {
            Ok(m) => {
                self.data.beacon.mac_address = m;
                true
            },
            Err(e) => {
                self.user_msg.error_messages.push(format!("failed to parse mac address: {}", e));
                false
            },
        };

        success = success && match self.data.raw_coord0.parse::<f64>() {
            Ok(coord) => {
                self.data.beacon.coordinates[0] = coord;
                true
            },
            Err(e) => {
                self.user_msg.error_messages.push(format!("failed to parse x coordinate: {}", e));
                false
            },
        };

        success = success && match self.data.raw_coord1.parse::<f64>() {
            Ok(coord) => {
                self.data.beacon.coordinates[1] = coord;
                true
            },
            Err(e) => {
                self.user_msg.error_messages.push(format!("failed to parse y coordinate: {}", e));
                false
            },
        };

        success
    }
}

pub struct BeaconAddUpdate {
    user_msg: UserMessage<Self>,
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    // TODO more robust way of handling concurrent requests
    get_fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
    user_type: WebUserType,
    change_page: Callback<root::Page>,
}

impl JsonResponseHandler for BeaconAddUpdate {}

#[derive(Properties)]
pub struct BeaconAddUpdateProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
    pub id: Option<i32>,
    #[props(required)]
    pub user_type: WebUserType,
}

impl Component for BeaconAddUpdate {
    type Message = Msg;
    type Properties = BeaconAddUpdateProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetAvailMaps);
        if let Some(id) = props.id {
            link.send_self(Msg::RequestGetBeacon(id));
        }
        let mut result = BeaconAddUpdate {
            user_msg: UserMessage::new(),
            change_page: props.change_page,
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            get_fetch_task: None,
            self_link: link,
            user_type: props.user_type,
        };
        result.data.id = props.id;
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
            Msg::AddAnotherBeacon => {
                self.data = Data::new();
                self.self_link.send_self(Msg::RequestGetAvailMaps);
            }
            Msg::InputName(name) => {
                self.data.beacon.name = name;
            },
            Msg::InputFloorName(map_id) => {
                self.data.beacon.map_id = map_id;
            },
            Msg::InputNote(note) => {
                self.data.beacon.note = Some(note);
            },
            Msg::InputMacAddress(mac) => {
                self.data.raw_mac = mac;
            },
            Msg::InputCoordinate(index, value) => {
                match index {
                    0 => { self.data.raw_coord0 = value; },
                    1 => { self.data.raw_coord1 = value; },
                    _ => panic!("invalid coordinate index specified"),
                };
            },
            Msg::RequestGetAvailMaps => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetAvailMaps
                );
            },
            Msg::RequestGetBeacon(id) => {
                self.get_fetch_task = get_request!(
                    self.fetch_service,
                    &beacon_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeacon
                );
            },
            Msg::RequestAddUpdateBeacon => {
                self.user_msg.reset();
                let success = self.validate();

                match self.data.id {
                    Some(id) if success => {
                        //ensure the beacon id does not mismatch.
                        self.data.beacon.id = id;

                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &beacon_url(&self.data.beacon.id.to_string()),
                            self.data.beacon,
                            self.self_link,
                            Msg::ResponseUpdateBeacon
                        );
                    },
                    None if success => {
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &beacon_url(""),
                            self.data.beacon,
                            self.self_link,
                            Msg::ResponseAddBeacon
                        );
                    }
                    _ => {},
                }
            },
            Msg::ResponseGetAvailMaps(response) => {
                self.handle_response(
                    response,
                    |s, maps| {
                        s.data.avail_floors = maps;
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to obtain available floors list, reason: {}", e));
                    },
                );
            },
            Msg::ResponseUpdateBeacon(response) => {
                self.handle_response(
                    response,
                    |s, beacon| {
                        s.user_msg.success_message = Some("successfully updated beacon".to_string());
                        s.data.beacon = beacon;
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to update beacon, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetBeacon(response) => {
                self.handle_response(
                    response,
                    |s, beacon| {
                        s.data.beacon = beacon;
                        s.data.raw_mac = s.data.beacon.mac_address.to_hex_string();
                        s.data.raw_coord0 = s.data.beacon.coordinates[0].to_string();
                        s.data.raw_coord1 = s.data.beacon.coordinates[1].to_string();
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to find beacon, reason: {}", e));
                    },
                );
            },
            Msg::ResponseAddBeacon(response) => {
                self.handle_response(
                    response,
                    |s, beacon| {
                        s.user_msg.success_message = Some("successfully added beacon".to_string());
                        s.data.beacon = beacon;
                        s.data.id = Some(s.data.beacon.id);
                        s.data.raw_mac = s.data.beacon.mac_address.to_hex_string();
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to add beacon, reason: {}", e));
                    },
                );
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.data.id = props.id;
        self.user_type = props.user_type;
        true
    }
}

impl Renderable<BeaconAddUpdate> for BeaconAddUpdate {
    fn view(&self) -> Html<Self> {
        let title_name = match self.data.id {
            Some(_id) => "Update Beacon",
            None => "Add Beacon",
        };
        let add_another_button = match &self.data.id {
            Some(_) => {
                html! {
                    <button
                        type="button",
                        class="btn btn-lg btn-primary",
                        onclick=|_| Msg::AddAnotherBeacon,
                    >
                        { "Add Another" }
                    </button>
                }
            },
            None => {
                html! { <></> }
            },
        };

        let mut floor_options = self.data.avail_floors.iter().map(|floor| {
            let floor_id = floor.id;
            html! {
                <option
                    onclick=|_| Msg::InputFloorName(Some(floor_id)),
                    selected={ Some(floor_id) == self.data.beacon.map_id },
                >
                    { &floor.name }
                </option>
            }
        });

        let return_cancel = match self.user_type {
            WebUserType::Admin => html! {
                    <button
                        type="button",
                        class="btn btn-lg btn-danger align",
                        onclick=|_| Msg::ChangeRootPage(root::Page::BeaconList),
                    >
                        { "Cancel" }
                    </button>
            },
            WebUserType::Responder => html! {
                    <button
                        type="button",
                        class="btn btn-lg btn-danger align",
                        onclick=|_| Msg::ChangeRootPage(root::Page::Status(status::PageState::BeaconStatus)),
                    >
                        { "Cancel" }
                    </button>
            },
        };


        let note = self.data.beacon.note.clone().unwrap_or(String::new());

        html! {
            <>
                { self.user_msg.view() }
                <div class="content-wrapper">
                    <div class="boxedForm">
                        <h2>{ title_name }</h2>
                        <table>
                            <tr>
                                <td class="formLabel">{ "Mac Address: " }</td>
                                <td>
                                    <input
                                        type="text",
                                        value=&self.data.raw_mac,
                                        oninput=|e| Msg::InputMacAddress(e.value),
                                    />
                                </td>
                            </tr>
                            <tr>
                                <td class="formLabel">{ "Assign to Map: " }</td>
                                <td>
                                    <select class="formAlign">
                                        <option
                                            onclick=|_| Msg::InputFloorName(None),
                                            selected={ None == self.data.beacon.map_id },
                                        >
                                            { "None" }
                                        </option>
                                        { for floor_options }
                                    </select>
                                </td>
                            </tr>
                            <tr>
                                <td class="formLabel">{ "Name: " }</td>
                                <td>
                                    <input
                                        type="text",
                                        value=&self.data.beacon.name,
                                        oninput=|e| Msg::InputName(e.value),
                                    />
                                </td>
                            </tr>
                            <tr>
                                <td class="formLabel">{ "Coordinates: " }</td>
                                <td>
                                    <input
                                        type="text",
                                        class="coordinates",
                                        value=&self.data.raw_coord0,
                                        oninput=|e| Msg::InputCoordinate(0, e.value),
                                    />
                                    <input
                                        type="text",
                                        class="coordinates"
                                        value=&self.data.raw_coord1,
                                        oninput=|e| Msg::InputCoordinate(1, e.value),
                                    />
                                </td>
                            </tr>
                            <tr>
                                <td class="formLabel">{ "Note: " }</td>
                                <td>
                                    <textarea
                                        class="formAlign",
                                        rows=5,
                                        cols=36,
                                        value=note,
                                        oninput=|e| Msg::InputNote(e.value),
                                    />
                                </td>
                            </tr>
                        </table>
                        <div class="formButtons">
                            {
                                match self.user_type {
                                    WebUserType::Admin => html! {
                                        <>
                                            <button
                                                type="button",
                                                class="btn btn-lg btn-success align",
                                                onclick=|_| Msg::RequestAddUpdateBeacon,
                                            >
                                                { title_name }
                                            </button>
                                            { add_another_button }
                                        </>
                                    },
                                    WebUserType::Responder => html! { },
                                }
                            }
                            { return_cancel }
                        </div>
                    </div>
                </div>
            </>
        }
    }
}
