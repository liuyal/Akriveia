use common::*;
use crate::canvas::{ Canvas, screen_space };
use crate::util::{ self, WebUserType, JsonResponseHandler, };
use std::time::Duration;
use stdweb::traits::*;
use stdweb::web::event::{ ClickEvent, };
use stdweb::web::{ Node, html_element::ImageElement, };
use super::root;
use yew::IMouseEvent;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::virtual_dom::vnode::VNode;
use super::user_message::UserMessage;

pub enum Coord {
    X,
    Y
}

pub enum Msg {
    AddAnotherMap,
    CanvasClick(ClickEvent),
    ChangeRootPage(root::Page),
    CheckImage,
    FileLoaded(FileData),
    Ignore,
    InputBound(usize, String),
    InputFile(File),
    InputName(String),
    InputNote(String),
    InputScale(String),
    ManualBeaconPlacement(usize, Coord, String),
    ToggleBeaconPlacement(i32),

    RequestAddUpdateMap,
    RequestGetMap(i32),
    RequestGetBeaconsForMap(i32),
    RequestPutBeacon(i32),

    ResponseAddMap(util::JsonResponse<Map>),
    ResponseGetBeaconsForMap(util::JsonResponse<Vec<Beacon>>),
    ResponseGetMap(util::JsonResponse<Map>),
    ResponseUpdateMap(util::JsonResponse<Map>),
    ResponsePutBeacon(util::JsonResponse<Beacon>),
    ResponseUpdateBlueprint(util::BinResponse<()>),
}

struct BeaconData {
    raw_x: String,
    raw_y: String,
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    pub map: Map,
    pub attached_beacons: Vec<(Beacon, BeaconData)>,
    pub opt_id: Option<i32>,
    pub raw_bounds: [String; 2],
    pub raw_scale: String,
    pub current_beacon: Option<i32>,
    pub blueprint: Option<FileData>,
}

impl Data {
    fn new() -> Data {
        Data {
            map: Map::new(),
            attached_beacons: Vec::new(),
            opt_id: None,
            raw_bounds: ["0".to_string(), "0".to_string()],
            raw_scale: "1".to_string(),
            current_beacon: None,
            blueprint: None,
        }
    }
}

impl MapAddUpdate {
    // NOTE: copypasta from beacon_addupdate
    fn validate_beacon(&mut self, index: usize, suppress: bool) -> bool {
        let mut success = match self.data.attached_beacons[index].1.raw_x.parse::<f64>() {
            Ok(coord) => {
                self.data.attached_beacons[index].0.coordinates.x = coord;
                true
            },
            Err(e) => {
                if !suppress {
                    self.user_msg.error_messages.push(format!("failed to parse x coordinate of beacon {}: {}", self.data.attached_beacons[index].0.name, e));
                }
                false
            },
        };

        success = success && match self.data.attached_beacons[index].1.raw_y.parse::<f64>() {
            Ok(coord) => {
                self.data.attached_beacons[index].0.coordinates.y = coord;
                true
            },
            Err(e) => {
                if !suppress {
                    self.user_msg.error_messages.push(format!("failed to parse y coordinate of beacon {}: {}", self.data.attached_beacons[index].0.name, e));
                }
                false
            },
        };

        success
    }

    fn validate(&mut self) -> bool {
        let mut success = match self.data.raw_bounds[0].parse::<i32>() {
            Ok(coord) => {
                self.data.map.bounds[0] = coord;
                true
            },
            Err(e) => {
                self.user_msg.error_messages.push(format!("failed to parse x coordinate: {}", e));
                false
            },
        };

        success = success && match self.data.raw_bounds[1].parse::<i32>() {
            Ok(coord) => {
                self.data.map.bounds[1] = coord;
                true
            },
            Err(e) => {
                self.user_msg.error_messages.push(format!("failed to parse y coordinate: {}", e));
                false
            },
        };

        success = success && match self.data.raw_scale.parse::<f64>() {
            Ok(scale) => {
                self.data.map.scale = scale;
                true
            },
            Err(e) => {
                self.user_msg.error_messages.push(format!("failed to parse scale: {}", e));
                false
            },
        };

        success
    }
}

pub struct MapAddUpdate {
    user_msg: UserMessage<Self>,
    binary_fetch_task: Option<FetchTask>,
    canvas: Canvas,
    change_page: Callback<root::Page>,
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    file_reader: ReaderService,
    file_task: Option<ReaderTask>,
    get_fetch_task: Option<FetchTask>,
    interval_service: IntervalService,
    interval_service_task: Option<IntervalTask>,
    map_img: Option<ImageElement>,
    self_link: ComponentLink<Self>,
    user_type: WebUserType,
}

impl JsonResponseHandler for MapAddUpdate {}

#[derive(Properties)]
pub struct MapAddUpdateProps {
    pub opt_id: Option<i32>,
    #[props(required)]
    pub user_type: WebUserType,
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for MapAddUpdate {
    type Message = Msg;
    type Properties = MapAddUpdateProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        if let Some(id) = props.opt_id {
            link.send_self(Msg::RequestGetMap(id));
            link.send_self(Msg::RequestGetBeaconsForMap(id));
        }
        let data = Data::new();

        let click_callback = link.send_back(|event| Msg::CanvasClick(event));
        let mut result = MapAddUpdate {
            user_msg: UserMessage::new(),
            binary_fetch_task: None,
            canvas: Canvas::new("addupdate_canvas", click_callback),
            change_page: props.change_page,
            data,
            fetch_service: FetchService::new(),
            fetch_task: None,
            file_reader: ReaderService::new(),
            file_task: None,
            get_fetch_task: None,
            interval_service: IntervalService::new(),
            interval_service_task: None,
            map_img: None,
            self_link: link,
            user_type: props.user_type,
        };

        result.canvas.reset(&result.data.map, &result.map_img);
        result.canvas.draw_beacons(&result.data.map, &result.data.attached_beacons.iter().map(|(b, _d)| b).collect());
        result.data.opt_id = props.opt_id;
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {
            },
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
            Msg::CheckImage => {
                // The is necessary to force a rerender when the image finally loads,
                // it would be nice to use an onload() callback, but that does not seem to
                // work.
                // once the map is loaded, we dont need to check it anymore.
                if let Some(img) = &self.map_img {
                    if img.complete() {
                        self.interval_service_task = None;
                    }
                }
            },
            Msg::InputFile(file) => {
                let callback = self.self_link.send_back(Msg::FileLoaded);
                let task = self.file_reader.read_file(file, callback);
                self.file_task = Some(task);
            },
            Msg::FileLoaded(data) => {
                self.data.blueprint = Some(data);
                self.file_task = None;
            },
            Msg::AddAnotherMap => {
                self.data = Data::new();
            }
            Msg::InputName(name) => {
                self.data.map.name = name;
            },
            Msg::InputNote(note) => {
                self.data.map.note = Some(note);
            },
            Msg::InputBound(index, value) => {
                self.data.raw_bounds[index] = value;
            },
            Msg::InputScale(value) => {
                self.data.raw_scale = value;
            },
            Msg::ToggleBeaconPlacement(beacon_id) => {
                match self.data.current_beacon {
                    Some(id) if beacon_id == id => {
                        self.data.current_beacon = None;
                    }
                    _ => {
                        self.data.current_beacon = Some(beacon_id);
                    },
                }
            },
            Msg::ManualBeaconPlacement(index, coord_type, value) => {
                self.user_msg.error_messages = Vec::new();
                match coord_type {
                    Coord::X => {
                        self.data.attached_beacons[index].1.raw_x = value;
                    },
                    Coord::Y => {
                        self.data.attached_beacons[index].1.raw_y = value;
                    },
                }
                self.validate_beacon(index, true);
            },
            Msg::CanvasClick(event) => {
                let canvas_bound = self.canvas.canvas.get_bounding_client_rect();
                match self.data.current_beacon {
                    Some(id) => {
                        match self.data.attached_beacons.iter().position(|(beacon, _bdata)| beacon.id == id) {
                            Some(index) => {
                                let pix_coords = na::Vector2::new(event.client_x() - canvas_bound.get_left() as i32, event.client_y() - canvas_bound.get_top() as i32);
                                let world_coords = screen_space(&self.data.map, pix_coords.x as f64, pix_coords.y as f64);
                                let coords = na::Vector2::new(world_coords.x / self.data.map.scale as f64, world_coords.y / self.data.map.scale as f64);
                                self.data.attached_beacons[index].1.raw_x = coords.x.to_string();
                                self.data.attached_beacons[index].1.raw_y = coords.y.to_string();
                                self.data.attached_beacons[index].0.coordinates = coords;
                                self.canvas.reset(&self.data.map, &self.map_img);
                                self.canvas.draw_beacons(&self.data.map, &self.data.attached_beacons.iter().map(|(b, _bdata)| b).collect());
                            },
                            _ => {
                                Log!("invalid current beacon");
                            },
                        }
                    },
                    _ => {
                        Log!("ignoring input location because a beacon has not been selected");
                    }
                }
            },
            Msg::RequestPutBeacon(id) => {
                self.user_msg.error_messages = Vec::new();
                match self.data.attached_beacons.iter().position(|(beacon, _bdata)| beacon.id == id) {
                    Some(index) => {
                        if self.validate_beacon(index, false) {
                            self.fetch_task = put_request!(
                                self.fetch_service,
                                &beacon_url(&id.to_string()),
                                self.data.attached_beacons[index].0,
                                self.self_link,
                                Msg::ResponsePutBeacon
                            );
                        }
                    },
                    _ => {
                        Log!("could not save invalid beacon");
                    },
                }
            },
            Msg::RequestGetBeaconsForMap(id) => {
                self.user_msg.error_messages = Vec::new();
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &beacons_for_map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeaconsForMap
                );
            },
            Msg::RequestGetMap(id) => {
                self.user_msg.error_messages = Vec::new();
                self.get_fetch_task = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::RequestAddUpdateMap => {
                self.user_msg.reset();
                let success = self.validate();

                match self.data.opt_id {
                    Some(id) if success => {
                        //ensure the id does not mismatch.
                        self.data.map.id = id;

                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &map_url(&self.data.map.id.to_string()),
                            self.data.map,
                            self.self_link,
                            Msg::ResponseUpdateMap
                        );
                    },
                    None if success => {
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &map_url(""),
                            self.data.map,
                            self.self_link,
                            Msg::ResponseAddMap
                        );
                    },
                    _ => { }
                }
            },
            Msg::ResponsePutBeacon(response) => {
                self.handle_response(
                    response,
                    |s, beacon| {
                        match s.data.attached_beacons.iter().position(|(b, _bdata)| beacon.id == b.id) {
                            Some(index) => {
                                s.user_msg.success_message = Some("successfully updated attached beacon".to_string());
                                s.data.attached_beacons[index].1.raw_x = beacon.coordinates.x.to_string();
                                s.data.attached_beacons[index].1.raw_y = beacon.coordinates.y.to_string();
                                s.data.attached_beacons[index].0 = beacon;
                            },
                            _ => {
                                s.user_msg.error_messages.push("failed to update attached beacon, reason: beacon is no longer attached to this map".to_owned());
                            },
                        }
                    },
                    |s, error| {
                        s.user_msg.error_messages.push(format!("failed to update attached beacon, reason: {}", error.reason));
                    },
                );
            },
            Msg::ResponseGetBeaconsForMap(response) => {
                self.handle_response(
                    response,
                    |s, beacons| {
                        s.data.attached_beacons = beacons.into_iter().map(|beacon| {
                            let raw_x = beacon.coordinates.x.to_string();
                            let raw_y = beacon.coordinates.y.to_string();
                            (beacon, BeaconData { raw_x, raw_y })
                        }).collect();
                    },
                    |s, error| {
                        s.user_msg.error_messages.push(format!("failed to obtain available floors list, reason: {}", error.reason));
                    },
                );
            },
            Msg::ResponseUpdateMap(response) => {
                self.handle_response(
                    response,
                    |s, map| {
                        s.user_msg.success_message = Some("successfully updated map".to_owned());
                        s.data.map = map;

                        if let Some(file) = &s.data.blueprint {
                            s.binary_fetch_task = put_image!(
                                s.fetch_service,
                                &map_blueprint_url(&s.data.map.id.to_string()),
                                file.content.clone(),
                                s.self_link,
                                Msg::ResponseUpdateBlueprint
                            );
                        }
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to update map, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetMap(response) => {
                self.handle_response(
                    response,
                    |s, map| {
                            s.data.map = map;
                            s.data.raw_bounds[0] = s.data.map.bounds[0].to_string();
                            s.data.raw_bounds[1] = s.data.map.bounds[1].to_string();
                            s.data.raw_scale = s.data.map.scale.to_string();
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to find map, reason: {}", e));
                    },
                );
            },
            Msg::ResponseUpdateBlueprint(response) => {
                let (meta, _body) = response.into_parts();
                if meta.status.is_success() {
                    self.user_msg.success_message = Some("successfully updated image".to_owned());
                    let img = ImageElement::new();
                    img.set_src(&map_blueprint_url(&self.data.map.id.to_string()));
                    let callback = self.self_link.send_back(|_| Msg::CheckImage);
                    self.interval_service_task = Some(self.interval_service.spawn(Duration::from_millis(100), callback));
                    self.map_img = Some(img);
                } else {
                    self.user_msg.error_messages.push("failed to find map".to_owned());
                }
            },
            Msg::ResponseAddMap(response) => {
                self.handle_response(
                    response,
                    |s, map| {
                        s.user_msg.success_message = Some("successfully added map".to_owned());
                        s.data.map = map;
                        s.data.opt_id = Some(s.data.map.id);

                        if let Some(file) = &s.data.blueprint {
                            s.binary_fetch_task = put_image!(
                                s.fetch_service,
                                &map_blueprint_url(&s.data.map.id.to_string()),
                                file.content.clone(),
                                s.self_link,
                                Msg::ResponseUpdateBlueprint
                            );
                        }
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to add map, reason: {}", e));
                    },
                );
            },
        }

        self.canvas.reset(&self.data.map, &self.map_img);
        self.canvas.draw_beacons(&self.data.map, &self.data.attached_beacons.iter().map(|(b, _bdata)| b).collect());
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.data.opt_id = props.opt_id;
        self.user_type = props.user_type;
        if let Some(id) = props.opt_id {
            self.self_link.send_self(Msg::RequestGetMap(id));
            self.self_link.send_self(Msg::RequestGetBeaconsForMap(id));
        }
        true
    }
}

impl MapAddUpdate {
    fn render_beacon_placement(&self) -> Html<Self> {
        let mut beacon_placement_rows = self.data.attached_beacons.iter().enumerate().map(|(index, (beacon, bdata))| {
            let beacon_id = beacon.id;
            let this_beacon_selected = match self.data.current_beacon {
                Some(id) => id == beacon_id,
                _ => false,
            };

            html! {
                <tr>
                    <td class="formLabel">
                        { &beacon.name }
                    </td>
                    <td>
                        <input
                            type="text",
                            class="coordinates",
                            value=&bdata.raw_x,
                            oninput=|e| Msg::ManualBeaconPlacement(index, Coord::X, e.value),
                        />
                        <input
                            type="text",
                            class="coordinates",
                            value=&bdata.raw_y,
                            oninput=|e| Msg::ManualBeaconPlacement(index, Coord::Y, e.value),
                        />
                    </td>
                    <td>
                        <button
                            class="btn btn-sm btn-success mx-1",
                            onclick=|_| Msg::RequestPutBeacon(beacon_id),
                        >
                            { "Save" }
                        </button>
                        <button
                            class={ if this_beacon_selected { "btn btn-sm btn-secondary mx-1 selected" }
                            else { "btn btn-sm mx-1 btn-warning" } },
                            onclick=|_| Msg::ToggleBeaconPlacement(beacon_id),
                        >
                            { "Toggle Placement" }
                        </button>
                    </td>
                </tr>
            }
        });

        match self.data.opt_id {
            Some(_) => {
                if self.data.attached_beacons.len() > 0 {
                    html! {
                        <>
                            <h3>{ "Beacon Placement" }</h3>
                            <table>
                                <tr>
                                    <td class="formLabel">
                                        { "Name" }
                                    </td>
                                    <td class="formLabel">
                                        { "Location" }
                                    </td>
                                    <td class="formLabel">
                                        { "Actions" }
                                    </td>
                                </tr>
                                { for beacon_placement_rows }
                            </table>
                            <div>
                                { VNode::VRef(Node::from(self.canvas.canvas.to_owned()).to_owned()) }
                            </div>
                        </>
                    }
                } else {
                    html! {
                        <p>{ "No Attached Beacons for this Map." }</p>
                    }
                }
            },
            None => {
                html! { }
            },
        }
    }
}

impl Renderable<MapAddUpdate> for MapAddUpdate {
    fn view(&self) -> Html<Self> {
        let title_name = match self.data.opt_id {
            Some(_id) => "Update Map",
            None => "Add Map",
        };

        let add_another_map = match &self.data.opt_id {
            Some(_) => {
                html! {
                    <button
                        type="button",
                        class="btn btn-lg btn-primary align",
                        onclick=|_| Msg::AddAnotherMap,
                    >
                        { "Add Another" }
                    </button>
                }
            },
            None => {
                html! { }
            },
        };

        let note = self.data.map.note.clone().unwrap_or(String::new());

        html! {
            <>
                { self.user_msg.view() }
                <div class="boxedForm">
                    <h2>{ title_name }</h2>
                    <table>
                        <tr>
                            <td class="formLabel">{"Name: " }</td>
                            <td>
                                <input
                                    type="text",
                                    value=&self.data.map.name,
                                    oninput=|e| Msg::InputName(e.value),
                                />
                            </td>
                        </tr>
                        <tr>
                            <td class="formLabel">{ "Dimensions(m): " }</td>
                            <td>
                                <input
                                    class="coordinates",
                                    type="text",
                                    value=&self.data.raw_bounds[0],
                                    oninput=|e| Msg::InputBound(0, e.value),
                                />
                                <input
                                    class="coordinates",
                                    type="text",
                                    value=&self.data.raw_bounds[1],
                                    oninput=|e| Msg::InputBound(1, e.value),
                                />
                            </td>
                        </tr>
                        <tr>
                            <td class="formLabel">{ "Scale(px/m): " }</td>
                            <td>
                                <input
                                    type="text",
                                    value=&self.data.raw_scale,
                                    oninput=|e| Msg::InputScale(e.value),
                                />
                            </td>
                        </tr>
                        <tr>
                            <td class="formLabel">{ "Notes: " }</td>
                            <td>
                                <textarea
                                    class="formAlign",
                                    rows=5,
                                    cols=38,
                                    value=note,
                                    placeholder="Add Important Information",
                                    oninput=|e| Msg::InputNote(e.value),
                                />
                            </td>
                        </tr>
                        <tr>
                            <td class="formLabel">{ "Blueprint: " }</td>
                            <td>
                                <input
                                    type="file",
                                    class="formAlign",
                                    onchange=|value| {
                                        if let ChangeData::Files(file_names) = value {
                                            match file_names.iter().next() {
                                                Some(file_name) => Msg::InputFile(file_name),
                                                None => Msg::Ignore,
                                            }
                                        } else {
                                            Msg::Ignore
                                        }
                                    },
                                />
                            </td>
                        </tr>
                    </table>
                    { self.render_beacon_placement() }
                    <div class="formButtons">
                        {
                            match self.user_type {
                                WebUserType::Admin => html! {
                                    <>
                                        <button
                                            type="button",
                                            class="btn btn-lg btn-success align",
                                            onclick=|_| Msg::RequestAddUpdateMap,
                                        >
                                            { title_name }
                                        </button>
                                        { add_another_map }
                                    </>
                                },
                                WebUserType::Responder => html! { },
                            }
                        }
                        <button
                            type="button",
                            class="btn btn-lg btn-danger align",
                            onclick=|_| Msg::ChangeRootPage(root::Page::MapList),
                        >
                                { "Cancel" }
                        </button>
                    </div>
                </div>
            </>
        }
    }
}
