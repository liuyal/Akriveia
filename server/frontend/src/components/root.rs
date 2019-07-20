
use common;
use crate::util;
use failure::Error;
use std::convert::TryFrom;
use std::time::Duration;
use stdweb::web::{ CanvasRenderingContext2d, Element, HtmlElement, Node, };
use stdweb::web::html_element::CanvasElement;
use stdweb::web;
use yew::format::{ Nothing, Json };
use yew::services::console::ConsoleService;
use yew::services::fetch::{ FetchService, FetchTask, Request, Response, };
use yew::services::interval::*;
use yew::virtual_dom::vnode::VNode;
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };

use super::map_view::MapViewComponent;

#[derive(PartialEq)]
pub enum Page {
    Diagnostics,
    FrontPage,
    Login,
    Map,
}

fn get_canvas() -> CanvasElement {
    unsafe {
       js! (
            return document.querySelector("canvas");
        ).into_reference_unchecked().unwrap()
    }
}

fn get_canvas_by_id(id: &str) -> CanvasElement {
    unsafe {
       js! (
            return document.getElementById(id);
        ).into_reference_unchecked().unwrap()
    }
}

fn get_context(canvas: &CanvasElement) -> CanvasRenderingContext2d {
    unsafe {
        js! (
            return @{canvas}.getContext("2d");
        ).into_reference_unchecked().unwrap()
    }
}

fn set_canvas_visibility(canvas: &CanvasElement, visible: bool) {
    let visibility = if visible { "block" } else { "none" };
    js! {
        @{canvas}.style.display = @{visibility};
    }
}

pub struct RootComponent {
    current_page: Page,
    data: Option<common::HelloFrontEnd>,
    diagnostic_data: Vec<common::TagData>,
    diagnostic_service: Option<IntervalService>,
    diagnostic_service_task: Option<IntervalTask>,
    fetch_in_flight: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<RootComponent>,
    map_canvas: Option<CanvasElement>,
}

pub enum Msg {
    Ignore,
    // page changes
    ChangePage(Page),

    // requests
    RequestPing,
    RequestDiagnostics,
    RequestEmergency,
    RequestEndEmergency,

    // responses
    ResponsePing(util::Response<common::HelloFrontEnd>),
    ResponseDiagnostics(util::Response<common::DiagnosticData>),
    ResponseEmergency(util::Response<()>),
    ResponseEndEmergency(util::Response<()>),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        RootComponent {
            current_page: Page::Login,
            data: None,
            diagnostic_service: None,
            diagnostic_service_task: None,
            diagnostic_data: Vec::new(),
            fetch_service: FetchService::new(),
            fetch_in_flight: false,
            fetch_task: None,
            link: link,
            map_canvas: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangePage(page) => {
                self.current_page = page;
                match self.current_page {
                    Page::Diagnostics => {
                        let mut interval_service = IntervalService::new();
                        self.diagnostic_service_task = Some(interval_service.spawn(Duration::from_millis(1000), self.link.send_back(|_| Msg::RequestDiagnostics)));
                        self.diagnostic_service = Some(interval_service);
                    },
                    _ => {
                        self.diagnostic_service = None;
                        self.diagnostic_service_task = None;
                    }
                }
            },

            // requests
            Msg::RequestPing => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    common::PING,
                    self.link,
                    Msg::ResponsePing
                );
            },
            Msg::RequestEmergency => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    common::EMERGENCY,
                    (),
                    self.link,
                    Msg::ResponseEmergency
                );
            },
            Msg::RequestEndEmergency => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    common::END_EMERGENCY,
                    (),
                    self.link,
                    Msg::ResponseEndEmergency
                );
            },
            Msg::RequestDiagnostics => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    common::DIAGNOSTICS,
                    self.link,
                    Msg::ResponseDiagnostics
                );
            },


            // responses
            Msg::ResponsePing(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::HelloFrontEnd { data }) => {
                            println!("success {:?}", data);
                        }
                        _ => { }
                    }

                } else {
                    Log!("response - failed to ping");
                }
            },
            Msg::ResponseDiagnostics(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::DiagnosticData { mut tag_data }) => {
                            self.diagnostic_data.append(&mut tag_data);
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to request diagnostics");
                }
            },
            Msg::ResponseEmergency(_response) => {
                println!("emergency response");
            },
            Msg::ResponseEndEmergency(_response) => {
                println!("endemergency response");
            },

            Msg::Ignore => {
                // do nothing
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
                        <div>
                            <button onclick=|_| Msg::RequestEmergency,>{ "Start Emergency" }</button>
                            <button onclick=|_| Msg::RequestEndEmergency,>{ "End Emergency" }</button>
                        </div>
                        <h>{ "Diagnostics" }</h>
                        { self.render_diagnostics() }
                    </div>
                }
            }
            Page::Login => {
                html! {
                    <div>
                        { self.navigation() }
                        <h>{ "Login" }</h>
                        { self.view_data() }
                    </div>
                }
            }
            Page::Map => {
                html! {
                    <div>
                        { self.navigation() }
                        <h>{ "Map" }</h>
                        <MapViewComponent/>
                    </div>
                }
            }
            Page::FrontPage => {
                html! {
                    <div>
                        <h>{ "FrontPage" }</h>
                        <button onclick=|_| Msg::ChangePage(Page::Login),>{ "Click" }</button>
                    </div>
                }
            }
        }
    }
}

impl RootComponent {
    fn navigation(&self) -> Html<Self> {
        html! {
            <div>
                <button onclick=|_| Msg::ChangePage(Page::Login), disabled={self.current_page == Page::Login},>{ "Login Page" }</button>
                <button onclick=|_| Msg::ChangePage(Page::Diagnostics), disabled={self.current_page == Page::Diagnostics},>{ "Diagnostics" }</button>
                <button onclick=|_| Msg::ChangePage(Page::Map), disabled={self.current_page == Page::Map},>{ "Map" }</button>
            </div>
        }

    }

    fn view_data(&self) -> Html<RootComponent> {
        html! {
            <p>{ "Its empty in here." }</p>
        }
    }

    fn render_diagnostics(&self) -> Html<RootComponent> {
        if self.diagnostic_data.len() > 0 {
            html! {
                <table> {
                    for self.diagnostic_data.iter().map(|row| {
                        match row.tag_distance {
                            common::DataType::RSSI(strength) => {
                                html! {
                                    <tr>{ format!("name: {}\tmac: {}\trssi: {}", &row.tag_name, &row.tag_mac, strength ) } </tr>
                                }
                            },
                            common::DataType::TOF(distance) => {
                                html! {
                                    <tr>{ format!("name: {}\tmac: {}\ttof: {}", &row.tag_name, &row.tag_mac, distance ) } </tr>
                                }
                            },
                        }
                    })
                } </table>
            }
        } else {
            html! {
                <p>{ "No diagnostics yet..." }</p>
            }
        }
    }
}
