use common::*;
use crate::util::*;
use std::net::Ipv4Addr;
use stdweb::web;
use super::root;
use super::user_message::UserMessage;
use super::value_button::DisplayButton;
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, Properties, };

pub enum Msg {
    ChangeRootPage(root::Page),
    InputIp(String),

    RequestRestart(SystemCommand),
    RequestSetIp,

    ResponseRestart(JsonResponse<()>),
    ResponseSetIp(JsonResponse<()>),
}

pub struct SystemSettings {
    user_msg: UserMessage<Self>,
    fetch_service: FetchService,
    self_link: ComponentLink<Self>,
    user_type: WebUserType,
    ip_raw: String,
    change_page: Callback<root::Page>,

    fetch_task: Option<FetchTask>,
    fetch_task_command: Option<FetchTask>,
}

impl JsonResponseHandler for SystemSettings {}

#[derive(Properties)]
pub struct SystemSettingsProps {
    #[props(required)]
    pub user_type: WebUserType,
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for SystemSettings {
    type Message = Msg;
    type Properties = SystemSettingsProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let result = SystemSettings {
            change_page: props.change_page,
            fetch_service: FetchService::new(),
            ip_raw: String::new(),
            self_link: link,
            user_msg: UserMessage::new(),
            user_type: props.user_type,

            fetch_task: None,
            fetch_task_command: None,
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
            Msg::RequestRestart(command) => {
                let reset_prompt = "Are you sure you wish to rebuild the database? This action will cause the server to restart and cannot be undone.";

                let confirmed = match command {
                    SystemCommand::StartNormal => web::window().confirm("Are you sure you wish to restart?"),
                    SystemCommand::RebuildDB => web::window().confirm(reset_prompt),
                    SystemCommand::RebuildDemoDB => web::window().confirm(reset_prompt),
                };

                if confirmed {
                    self.fetch_task = post_request! (
                        self.fetch_service,
                        &system_restart_url(),
                        command,
                        self.self_link,
                        Msg::ResponseRestart
                    );
                }
            },
            Msg::InputIp(ip) => {
                self.ip_raw = ip;
            }
            Msg::RequestSetIp => {
                self.user_msg.reset();
                let ret_ip: Result<Ipv4Addr, _> = self.ip_raw.parse();

                match ret_ip {
                    Ok(ip) => {
                        self.fetch_task_command = post_request! (
                            self.fetch_service,
                            &beacon_command_url(),
                            BeaconRequest::SetIp(ip),
                            self.self_link,
                            Msg::ResponseSetIp
                        );
                    },
                    Err(e) => {
                        self.user_msg.error_messages.push(format!("failed to send setip command, reason: {}", e));
                    },
                }

            },
            Msg::ResponseRestart(response) => {
                let (meta, Json(_body)) = response.into_parts();
                if meta.status.is_success() {
                    self.user_msg.success_message = Some("successfully sent restart command".to_owned());
                    self.self_link.send_self(Msg::ChangeRootPage(root::Page::Restarting));
                } else {
                    self.user_msg.error_messages.push("failed to send restart command".to_string());
                }
            },
            Msg::ResponseSetIp(response) => {
                self.handle_response(
                    response,
                    |s, _| {
                        s.user_msg.success_message = Some("Successfully sent setip command".to_owned());
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to send setip, reason: {}", e));
                    },
                );
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.user_type = props.user_type;
        true
    }
}

// The front-end layout in HTML
impl Renderable<SystemSettings> for SystemSettings {
    fn view(&self) -> Html<Self> {

        html! {
            <>
                { self.user_msg.view() }
                <div class="content-wrapper">
                    <div/>
                    <div class="boxedForm">
                        <h2>{ "System Settings" }</h2>
                        <div class="d-flex">
                            <DisplayButton<()>
                                value=(),
                                style="btn btn-lg btn-info mr-3 my-auto",
                                on_click=|_| Msg::RequestRestart(SystemCommand::StartNormal),
                                icon="fa fa-refresh",
                                display="Restart Server",
                            />
                            <DisplayButton<()>
                                value=(),
                                style="btn btn-lg btn-info ml-3 my-auto",
                                on_click=|_| Msg::RequestRestart(SystemCommand::RebuildDB),
                                icon="fa fa-power-off",
                                display="Reset Database",
                            />
                            <DisplayButton<()>
                                value=(),
                                style="btn btn-lg btn-info ml-3 my-auto",
                                on_click=|_| Msg::RequestRestart(SystemCommand::RebuildDemoDB),
                                icon="fa fa-power-off",
                                display="Reset Database with Demo Data",
                            />
                        </div>

                        <div class="d-flex justify-content-start">
                            <DisplayButton<()>
                                value=(),
                                style="btn btn-lg btn-secondary my-auto",
                                on_click=|_| Msg::RequestSetIp,
                                display="Set IP Address",
                                icon="fa fa-wifi",
                            />
                            <input
                                type="text",
                                class="fixedLength",
                                placeholder="IP address",
                                oninput=|event| Msg::InputIp(event.value),
                            />
                        </div>
                    </div>
                </div>
            </>
        }
    }
}
