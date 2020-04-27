use common::*;
use crate::util::*;
use std::time::Duration;
use super::beacon_addupdate::BeaconAddUpdate;
use super::beacon_list::BeaconList;
use super::diagnostics::Diagnostics;
use super::emergency_buttons::EmergencyButtons;
use super::login::{ self, Login, };
use super::map_addupdate::MapAddUpdate;
use super::map_list::MapList;
use super::map_view::MapViewComponent;
use super::status::{ self, Status, };
use super::system_settings::SystemSettings;
use super::user_addupdate::UserAddUpdate;
use super::user_list::UserList;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalService, IntervalTask, };

const POLL_RATE: Duration = Duration::from_millis(2000);

#[derive(PartialEq)]
pub enum Page {
    BeaconAddUpdate(Option<i32>),
    BeaconList,
    Diagnostics,
    Login(login::AutoAction),
    MapAddUpdate(Option<i32>),
    MapList,
    MapView(Option<i32>),
    Status(status::PageState),
    SystemSettings,
    UserAddUpdate(Option<i32>),
    UserList,
    Restarting,
}

pub struct RootComponent {
    user_type: WebUserType,
    current_page: Page,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<RootComponent>,

    interval_service: IntervalService,
    interval_ping_task: Option<IntervalTask>,
}

impl JsonResponseHandler for RootComponent {}

pub enum Msg {
    // page changes
    ChangePage(Page),
    ChangeWebUserType(WebUserType),

    // requests
    RequestPostEmergency(bool),
    RequestGetEmergency,
    RequestGetPing,

    // responses
    ResponsePostEmergency(JsonResponse<bool>),
    ResponseGetEmergency(JsonResponse<bool>),
    ResponseGetPing(JsonResponse<()>),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetEmergency);
        let root = RootComponent {
            current_page: Page::Login(login::AutoAction::Login),
            emergency: false,
            fetch_service: FetchService::new(),
            fetch_task: None,
            interval_ping_task: None,
            interval_service: IntervalService::new(),
            link: link,
            user_type: WebUserType::Responder,
        };
        root
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangePage(page) => {
                self.current_page = page;
                match self.current_page {
                    Page::Restarting => {
                        self.interval_ping_task = Some(
                            self.interval_service.spawn(POLL_RATE, self.link.send_back(move |_| Msg::RequestGetPing))
                        );
                    },
                    _ => {
                    },
                }
            },
            Msg::ChangeWebUserType(user_type) => {
                self.user_type = user_type;
            },

            // requests
            Msg::RequestPostEmergency(is_emergency) => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &system_emergency_url(),
                    SystemCommandResponse::new(is_emergency),
                    self.link,
                    Msg::ResponsePostEmergency
                );
            },
            Msg::RequestGetEmergency => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &system_emergency_url(),
                    self.link,
                    Msg::ResponseGetEmergency
                );
            },
            Msg::RequestGetPing => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &system_ping_url(),
                    self.link,
                    Msg::ResponseGetPing
                );
            },
            // responses
            Msg::ResponsePostEmergency(response) => {
                self.handle_response(
                    response,
                    |s, resp| {
                        s.emergency = resp;
                    },
                    |_s, e| {
                        Log!("response - failed to post start emergency, {}", e);
                    },
                );
            },
            Msg::ResponseGetEmergency(response) => {
                self.handle_response(
                    response,
                    |s, resp| {
                        s.emergency = resp;
                    },
                    |_s, e| {
                        Log!("response - failed to request emergency status, {}", e);
                    },
                );
            },
            Msg::ResponseGetPing(response) => {
                self.handle_response(
                    response,
                    |s, _resp| {
                        s.interval_ping_task = None;
                        s.link.send_self(Msg::ChangePage(Page::Login(login::AutoAction::Login)));
                    },
                    |_s, _e| {
                        // this might happen if the server is down while we ping it, this is
                        // totally fine
                    },
                );
            },
        }
        true
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        match self.current_page {
            Page::Diagnostics => {
                html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <Diagnostics
                                emergency={self.emergency}
                            />
                        </div>
                    </div>
                }
            },
            Page::Status(state) => {
                html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <Status
                                change_page=|page| Msg::ChangePage(page),
                                state=state,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::Login(auto_action) => {
                html! {
                    <div>
                        <Login
                            change_page=|page| Msg::ChangePage(page),
                            change_user_type=|user_type| Msg::ChangeWebUserType(user_type),
                            auto_action=auto_action,
                        />
                    </div>
                }
            },
            Page::MapView(opt_id) => {
                html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                        </div>
                        <div class="container-fluid">
                            <MapViewComponent
                                emergency={self.emergency},
                                opt_id=opt_id,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::BeaconList => {
               html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <BeaconList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::BeaconAddUpdate(id) => {
               html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <BeaconAddUpdate
                                id=id,
                                user_type=self.user_type,
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::UserList => {
                html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <UserList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::UserAddUpdate(id) => {
                html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <UserAddUpdate
                                id=id,
                                user_type=self.user_type,
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::MapList => {
               html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <MapList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::MapAddUpdate(opt_id) => {
               html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <MapAddUpdate
                                opt_id=opt_id,
                                user_type=self.user_type,
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::SystemSettings => {
                html! {
                    <div>
                        { self.navigation() }
                        <div class="container-fluid">
                            <SystemSettings
                                user_type=self.user_type,
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::Restarting => {
                html! {
                    <div class="content-wrapper">
                        <div class="boxedForm">
                            <div class="restarting-notify">
                                <img src="/images/icon_780_720.png" width="256" height="256" style="margin:30px"/>
                                <h2>{"Restarting Server"}</h2>
                                <h4>{"You will be redirected to the main page momentarily..."}</h4>
                                <i class="fa fa-spinner fa-pulse fa-3x fa-fw"></i>
                            </div>
                        </div>
                    </div>
                }
            },
        }
    }
}

impl RootComponent {
    fn navigation(&self) -> Html<Self> {
        let space = {" "};
        let view_map = html! {
            <>
                <a
                    class = match self.current_page {
                        Page::MapView {..} => {"nav-link navBarText active"},
                        _ => {"nav-link navBarText"},
                    }
                    onclick=|_| Msg::ChangePage(Page::MapView(None)),
                    disabled={
                        match self.current_page {
                            Page::MapView { .. } => true,
                            _ => false,
                        }
                    },
                >
                    { "View Map" }
                </a>
            </>
        };

        let show_status = html! {
            <>
                <a
                    class = match self.current_page {
                        Page::Status {..} => {"nav-link navBarText active"},
                        _ => {"nav-link navBarText"},
                    }
                    id="navbarDropdown",
                    role="button",
                    onclick=|_| Msg::ChangePage(Page::Status(status::PageState::UserStatus)),
                >
                    { "Status" }
                </a>
                <div class="dropdown-content">
                    <a
                        class="dropdown-item navBarText",
                        onclick=|_| Msg::ChangePage(Page::Status(status::PageState::UserStatus)),
                        disabled={
                            match self.current_page {
                                Page::Status(status::PageState::UserStatus) => true,
                                _ => false,
                            }
                        },
                    >
                        { "User Status" }
                    </a>
                    <a
                        class="dropdown-item navBarText",
                        onclick=|_| Msg::ChangePage(Page::Status(status::PageState::BeaconStatus)),
                        disabled={
                            match self.current_page {
                                Page::Status(status::PageState::BeaconStatus) => true,
                                _ => false,
                            }
                        },
                    >
                        { "Beacon Status" }
                    </a>
                </div>
            </>
        };

        let select_user = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a
                        class = match self.current_page {
                            Page::UserList => {"nav-link dropdown navBarText active"},
                            Page::UserAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        }
                        role="button",
                        data-toggle="dropdown",
                        onclick=|_| Msg::ChangePage(Page::UserList)
                    >
                            { "User" }
                    </a>
                    <div class="dropdown-content">
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::UserList),
                            disabled={self.current_page == Page::UserList},>
                                { "User List" }
                        </a>
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::UserAddUpdate(None)),
                            disabled={
                                match self.current_page {
                                    // match ignoring the fields
                                    Page::UserAddUpdate {..} => true,
                                    _=> false,
                                }
                            },
                        >
                            { "Add User" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! { },
        };

        let select_beacon = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a
                        class = match self.current_page {
                            Page::BeaconList => {"nav-link dropdown navBarText active"},
                            Page::BeaconAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        }
                        role="button",
                        data-toggle="dropdown",
                        onclick=|_| Msg::ChangePage(Page::BeaconList),
                    >
                        { "Beacons" }
                    </a>
                    <div class="dropdown-content">
                            <a
                                class="dropdown-item navBarText",
                                onclick=|_| Msg::ChangePage(Page::BeaconList),
                                disabled={self.current_page == Page::BeaconList},
                            >
                                { "Beacon List" }
                            </a>
                            <a
                                class="dropdown-item navBarText",
                                onclick=|_| Msg::ChangePage(Page::BeaconAddUpdate(None)),
                                disabled={
                                    match self.current_page {
                                        // match ignoring the fields
                                        Page::BeaconAddUpdate {..} => true,
                                        _ => false,
                                    }
                                },
                            >
                                { "Add Beacon" }
                            </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! { }
        };

        let select_map = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a
                        class = match self.current_page {
                            Page::MapList => {"nav-link dropdown navBarText active"},
                            Page::MapAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        }
                        role="button",
                        data-toggle="dropdown",
                        onclick=|_| Msg::ChangePage(Page::MapList),
                    >
                            { "Maps" }
                    </a>
                    <div class="dropdown-content">
                        <a
                            class = "dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::MapList),
                            disabled={self.current_page == Page::MapList},
                        >
                                { "Map List" }
                        </a>
                        <a
                            class = "dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::MapAddUpdate(None)),
                            disabled={
                                match self.current_page {
                                    // match ignoring the fields
                                    Page::MapAddUpdate {..} => true,
                                    _ => false,
                                }
                            },
                        >
                            { "Add Map" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! { }
        };

        let select_system = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a
                        class = match self.current_page {
                            Page::SystemSettings => {"nav-link navBarText active"}
                            Page::Diagnostics {..} => {"nav-link navBarText active"},
                            _ => {"nav-link navBarText"},
                        },
                        role="button"
                        onclick=|_| Msg::ChangePage(Page::SystemSettings),
                    >
                        { "System" }
                    </a>
                    <div class="dropdown-content">
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::SystemSettings),
                            disabled={self.current_page == Page::SystemSettings},>
                                { "System Settings" }
                        </a>
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::Diagnostics),
                            disabled={self.current_page == Page::Diagnostics},>
                                { "Diagnostics" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! { }
        };

        let login_type = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <button
                        class="btn btn-danger btn-sm nav-link logoutPlacement ml-auto",
                        onclick=|_| Msg::ChangePage(Page::Login(login::AutoAction::Logout)),
                        disabled={
                            match self.current_page {
                                Page::Login{..} => true,
                                _ => false,
                            }
                        },
                    >
                        { "Logout" }
                        { space }
                        <i class="fa fa-sign-out" aria-hidden="true"></i>
                    </button>
                    <a class="loginTypeHeader">{ "ADMIN" }</a>
                </>
            },
            WebUserType::Responder => html! {
                <>
                    <button
                        class="btn btn-success btn-sm nav-link logoutPlacement ml-auto",
                        onclick=|_| Msg::ChangePage(Page::Login(login::AutoAction::Nothing)),
                        disabled={
                            match self.current_page {
                                Page::Login{..} => true,
                                _ => false,
                            }
                        },
                    >
                        <i class="fa fa-sign-in" aria-hidden="true"></i>
                        { space }
                        { "Login" }
                    </button>
                    <a class="loginTypeHeader">{ "FIRST RESPONDER" }</a>
                </>
            }
        };
        html! {
            <nav class="navbar navbar-expand-sm navbarColour">
                <a class="navbar-brand">
                    <img src="/images/icon.PNG" width="52" height="48" class="d-inline-block align-top" alt=""/>
                </a>
                <div class="navbarJustify">
                    <ul class="nav navbarText">
                        <li class="my-auto">
                            {view_map}
                        </li>
                        <li class="dropdown my-auto">
                            { show_status }
                        </li>
                        <li class="dropdown my-auto">
                            { select_map }
                        </li>
                        <li class="dropdown my-auto">
                            { select_beacon }
                        </li>
                        <li class="dropdown my-auto">
                            { select_user }
                        </li>
                        <li class="dropdown my-auto">
                            { select_system }
                        </li>
                    </ul>
                </div>
                {login_type}
            </nav>
        }
    }
}
