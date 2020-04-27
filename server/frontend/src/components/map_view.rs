use common::*;
use crate::canvas::{ Canvas, };
use crate::util::*;
use std::time::Duration;
use stdweb::web::{ Node, html_element::ImageElement, Date, };
use super::user_message::UserMessage;
use super::value_button::{ ValueButton, DisplayButton, };
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalService, IntervalTask, };
use yew::virtual_dom::vnode::VNode;

const REALTIME_USER_POLL_RATE: Duration = Duration::from_millis(1000);

pub enum Msg {
    CheckImage,
    ChooseMap(i32),
    Ignore,
    ToggleGrid,
    ViewDistance(ShortAddress),

    RequestGetBeaconsForMap(i32),
    RequestGetMap(i32),
    RequestGetMaps,
    RequestRealtimeUser,

    ResponseGetBeaconsForMap(JsonResponse<Vec<Beacon>>),
    ResponseGetMap(JsonResponse<Map>),
    ResponseGetMaps(JsonResponse<Vec<Map>>),
    ResponseRealtimeUser(JsonResponse<Vec<RealtimeUserData>>),
}

pub struct MapViewComponent {
    beacons: Vec<Beacon>,
    canvas: Canvas,
    current_map: Option<Map>,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task_beacons: Option<FetchTask>,
    fetch_task_realtime_users: Option<FetchTask>,
    get_fetch_task: Option<FetchTask>,
    get_many_fetch_task: Option<FetchTask>,
    interval_service: IntervalService,
    interval_service_task_user: Option<IntervalTask>,
    interval_service_task_beacon: Option<IntervalTask>,
    interval_service_task_blueprint: Option<IntervalTask>,
    legend_canvas: Canvas,
    map_img: Option<ImageElement>,
    maps: Vec<Map>,
    realtime_users: Vec<RealtimeUserData>,
    self_link: ComponentLink<MapViewComponent>,
    show_distance: Option<ShortAddress>,
    user_msg: UserMessage<Self>,
    user_type: WebUserType,
    show_grid: bool,
}

impl JsonResponseHandler for MapViewComponent {}

impl MapViewComponent {
    fn start_service(&mut self) {
        if let Some(map) = &self.current_map {
            let id = map.id;
            self.interval_service_task_beacon = Some(
                self.interval_service.spawn(REALTIME_USER_POLL_RATE, self.self_link.send_back(move |_| Msg::RequestGetBeaconsForMap(id)))
            );
        }
        self.interval_service_task_user = Some(
            self.interval_service.spawn(REALTIME_USER_POLL_RATE, self.self_link.send_back(|_| Msg::RequestRealtimeUser))
        );
    }

    fn end_service(&mut self) {
        self.interval_service_task_user = None;
        self.interval_service_task_beacon = None;
    }

    fn render(&mut self) {
        if let Some(map) = &self.current_map {
            self.canvas.reset(map, &self.map_img, self.show_grid);

            self.canvas.draw_users(map, &self.realtime_users, self.show_distance);
            if self.user_type == WebUserType::Admin {
                self.canvas.draw_beacons(map, &self.beacons.iter().collect());
            }
            self.legend_canvas.legend(80, map.bounds.y as u32, self.user_type);
        }
    }

    fn load_img(&mut self) {
        if let Some(map) = &self.current_map {
            let img = ImageElement::new();
            img.set_src(&format!("{}#{}", map_blueprint_url(&map.id.to_string()), Date::now()));
            let callback = self.self_link.send_back(|_| Msg::CheckImage);
            self.interval_service_task_blueprint = Some(self.interval_service.spawn(Duration::from_millis(100), callback));
            self.map_img = Some(img);
        }
    }
}

#[derive(Properties)]
pub struct MapViewProps {
    pub emergency: bool,
    pub opt_id: Option<i32>,
    #[props(required)]
    pub user_type: WebUserType,
}

impl Component for MapViewComponent {
    type Message = Msg;
    type Properties = MapViewProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        if let Some(id) = props.opt_id {
            link.send_self(Msg::RequestGetMap(id));
            link.send_self(Msg::RequestGetBeaconsForMap(id));
        }
        link.send_self(Msg::RequestGetMaps);
        let click_callback = link.send_back(|_event| Msg::Ignore);

        let mut result = MapViewComponent {
            beacons: Vec::new(),
            canvas: Canvas::new("map_canvas", click_callback.clone()),
            current_map: None,
            emergency: props.emergency,
            fetch_service: FetchService::new(),
            fetch_task_beacons: None,
            fetch_task_realtime_users: None,
            get_fetch_task: None,
            get_many_fetch_task: None,
            interval_service: IntervalService::new(),
            interval_service_task_user: None,
            interval_service_task_beacon: None,
            interval_service_task_blueprint: None,
            legend_canvas: Canvas::new("legend_canvas", click_callback),
            map_img: None,
            maps: Vec::new(),
            realtime_users: Vec::new(),
            self_link: link,
            show_distance: None,
            user_msg: UserMessage::new(),
            user_type: props.user_type,
            show_grid: false,
        };

        if props.emergency {
            result.start_service();
        }

        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleGrid => {
                self.show_grid = !self.show_grid;
            }
            Msg::CheckImage => {
                // This is necessary to force a rerender when the image finally loads,
                // it would be nice to use an onload() callback, but that does not seem to
                // work.
                // once the map is loaded, we dont need to check it anymore.
                if let Some(img) = &self.map_img {
                    if img.complete() && img.width() > 0 && img.height() > 0 {
                        self.interval_service_task_blueprint = None;
                    }
                }
            },
            Msg::ViewDistance(selected_tag_mac) => {
                match &self.show_distance {
                    Some(current_tag) => {
                        if current_tag == &selected_tag_mac {
                            self.show_distance = None;
                        } else {
                            self.show_distance = Some(selected_tag_mac);
                        }
                    },
                    None => {
                        self.show_distance = Some(selected_tag_mac);
                    }
                }
            },
            Msg::ChooseMap(id) => {
                self.current_map = match &self.current_map {
                    Some(curr) if curr.id == id => {
                        None
                    },
                    _ => {
                        match self.maps.iter().find(|map| map.id == id) {
                            Some(map) => {
                                Some(map.clone())
                            },
                            None => None,
                        }
                    }
                };

                self.load_img();
                if let Some(map) = &self.current_map {
                    self.self_link.send_self(Msg::RequestGetMap(map.id));
                }

                if self.emergency {
                    self.start_service();
                }
            },
            Msg::RequestRealtimeUser => {
                self.user_msg.reset();
                self.fetch_task_realtime_users = get_request!(
                    self.fetch_service,
                    &users_status_url(),
                    self.self_link,
                    Msg::ResponseRealtimeUser
                );
            },
            Msg::RequestGetBeaconsForMap(id) => {
                self.user_msg.reset();
                self.fetch_task_beacons = get_request!(
                    self.fetch_service,
                    &beacons_for_map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeaconsForMap
                );
            },
            Msg::RequestGetMap(id) => {
                self.user_msg.reset();
                self.get_fetch_task = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::RequestGetMaps => {
                self.user_msg.reset();
                self.get_many_fetch_task = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetMaps
                );
            },
            Msg::ResponseRealtimeUser(response) => {
                self.handle_response(
                    response,
                    |s, users| {
                        let current_mid = s.current_map.as_ref().map(|m| m.id);
                        s.realtime_users = users.into_iter().filter(|u| u.map_id == current_mid).collect();
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to request realtime user data, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetBeaconsForMap(response) => {
                self.handle_response(
                    response,
                    |s, beacons| {
                        s.beacons = beacons;
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to obtain list of beacons for this map, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetMap(response) => {
                self.handle_response(
                    response,
                    |s, map| {
                        s.current_map = Some(map.clone());
                        s.maps.iter_mut().find(|m| m.id == map.id).and_then(|m| {
                            *m = map;
                            Some(())
                        });
                        s.load_img();
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to get map, reason: {}", e));
                    },
                );
            },
            Msg::ResponseGetMaps(response) => {
                self.handle_response(
                    response,
                    |s, mut maps| {
                        maps.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                        s.maps = maps;
                        if s.maps.len() > 0 {
                            s.self_link.send_self(Msg::ChooseMap(s.maps[0].id));
                        }
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to get maps, reason: {}", e));
                    },
                );
            },
            Msg::Ignore => {
            },
        }
        self.render();
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        // do not overwrite the canvas or context.
        self.emergency = props.emergency;

        if self.emergency {
            self.end_service();
            self.start_service();
        } else {
            self.end_service();
            self.realtime_users = Vec::new();
        }
        self.render();
        true
    }
}

impl Renderable<MapViewComponent> for MapViewComponent {
    fn view(&self) -> Html<Self> {
        let current_map_id = match &self.current_map {
            Some(map) => map.id,
            None => -1,
        };

        let mut maps = self.maps.iter().map(|map| {
            let map_id = map.id;
            let map_name = map.name.clone();
            html! {
                <ValueButton<i32>
                    on_click=move |value: i32| Msg::ChooseMap(map_id),
                    border=map.id == current_map_id,
                    value={map.id},
                    style={ if map.id==current_map_id {"btn-primary"} else {"btn-outline-primary"} },
                    display=Some(map_name),
                />
            }
        });

        let mut realtime_users = self.realtime_users.iter().map(|user| {
            let set_border = match &self.show_distance {
                Some(selected) => &user.addr == selected,
                None => false,
            };
            html! {
                <tr>
                    <td>{&user.addr}</td>
                    <td>{&user.name}</td>
                    <td>{format_timestamp(&user.last_active) }</td>
                    {
                        match self.user_type {
                            WebUserType::Admin => html! {
                                <td>
                                    <DisplayButton<String>
                                        on_click=|value: String| Msg::ViewDistance(ShortAddress::parse_str(&value).unwrap()),
                                        border=set_border,
                                        value={user.addr.to_string()},
                                        icon="fa fa-map-marker",
                                        style={ if set_border {"btn btn-sm btn-secondary"} else {"btn btn-sm btn-outline-secondary"} },
                                        display="Show",
                                    />
                                </td>
                            },
                            _ => html! {}
                        }
                    }
                </tr>
            }
        });

        html! {
            <>
                { self.user_msg.view() }
                <div class="page-wrapper-map-view">
                    <div class="boxedForm">
                        <div>
                            <h3>{ "View Map" }</h3>
                            { for maps }
                        </div>
                        <div>
                            <div class="form-check">
                                <input
                                    type="checkbox",
                                    class="form-check-input",
                                    id="grid1",
                                    checked = self.show_grid,
                                    value=&self.show_grid,
                                    onclick=|_| Msg::ToggleGrid,
                                />
                                <label class="checkmark m-1" for="grid1">{ "Show Gridlines" }</label>
                            </div>
                        </div>
                        { VNode::VRef(Node::from(self.legend_canvas.canvas.to_owned()).to_owned()) }
                        { VNode::VRef(Node::from(self.canvas.canvas.to_owned()).to_owned()) }
                        <div class="tinyBoxForm align-top">
                            <h4>{"User Status"}</h4>
                            <table class="table table-sm table-striped">
                                <tr>
                                    <th>{"Address"}</th>
                                    <th>{"Name"}</th>
                                    <th>{"Last Seen"}</th>
                                    {
                                        match self.user_type {
                                            WebUserType::Admin => html! {
                                                <th>{ "Intersection" }</th>
                                            },
                                            _ => html! {},
                                        }
                                    }
                                </tr>
                                { for realtime_users }
                            </table>
                        </div>
                    </div>
                </div>
            </>
        }
    }
}
