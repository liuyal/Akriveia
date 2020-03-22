use common::*;
use crate::util::*;
use super::root;
use super::value_button::ValueButton;
use yew::services::fetch::{ FetchService, FetchTask, };
use super::user_message::UserMessage;
use yew::{ Callback, Component, ComponentLink, Html, Renderable, ShouldRender, html, Properties, };

pub enum Msg {
    ChangeRootPage(root::Page),

    RequestDeleteUser(i32),
    RequestGetUsers,

    ResponseGetUsers(JsonResponse<Vec<TrackedUser>>),
    ResponseDeleteUser(JsonResponse<Vec<()>>),
}

pub struct UserList {
    change_page: Callback<root::Page>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    list: Vec<TrackedUser>,
    self_link: ComponentLink<Self>,
    user_msg: UserMessage<Self>,
}

impl JsonResponseHandler for UserList {}

#[derive(Properties)]
pub struct UserListProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for UserList {
    type Message = Msg;
    type Properties = UserListProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetUsers);
        let result = UserList {
            change_page: props.change_page,
            fetch_service: FetchService::new(),
            fetch_task: None,
            list: Vec::new(),
            self_link: link,
            user_msg: UserMessage::new(),
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestGetUsers => {
                self.user_msg.reset();
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &format!("{}?include_contacts=false", users_url()),
                    self.self_link,
                    Msg::ResponseGetUsers
                );
            },
            Msg::RequestDeleteUser(id) => {
                self.user_msg.reset();
                self.fetch_task = delete_request!(
                    self.fetch_service,
                    &user_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseDeleteUser
                );
            },
            Msg::ResponseGetUsers(response) => {
                self.handle_response(
                    response,
                    |s, mut users| {
                        users.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                        s.list = users;
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to obtain user list, reason: {}", e));
                    }
                );
            },
            Msg::ResponseDeleteUser(response) => {
                self.handle_response(
                    response,
                    |s, _| {
                        s.user_msg.success_message = Some("successfully deleted user".to_owned());
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to delete user, reason: {}", e));
                    },
                );
                self.self_link.send_self(Msg::RequestGetUsers);
            },
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        self.self_link.send_self(Msg::RequestGetUsers);
        true
    }
}

impl Renderable<UserList> for UserList {
    fn view(&self) -> Html<Self> {

        let mut rows = self.list.iter().map(|user| {

            html! {
                <tr>
                    <td>{ &user.name }</td>
                    <td>{ format!("{:.3},{:.3}", &user.coordinates.x, &user.coordinates.y) }</td>
                    <td>{ &user.mac_address.map_or(String::new(), |m| m.to_string()) }</td>
                    <td>{ user.employee_id.clone().unwrap_or(String::new()) }</td>
                    <td>{ format_timestamp(&user.last_active) }</td>
                    <td>{ user.work_phone.clone().unwrap_or(String::new()) }</td>
                    <td>{ user.mobile_phone.clone().unwrap_or(String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Edit".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::UserAddUpdate(Some(value))),
                            border=false,
                            icon="fa fa-pencil-square-o",
                            style="btn-primary",
                            value={user.id}
                        />
                        <ValueButton<i32>
                            display=Some("Delete".to_string()),
                            on_click=|value: i32| Msg::RequestDeleteUser(value),
                            border=false,
                            icon="fa fa-trash",
                            style="btn-secondary",
                            value=user.id,
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
                            <h2>{ "User List"}</h2>
                            <button
                                class="btn btn-success logoutPlacement my-1",
                                onclick=|_| Msg::ChangeRootPage(root::Page::UserAddUpdate(None)),
                            >
                                {"Add User"}
                            </button>
                        </div>
                        <table class="table table-striped">
                            <thead class="thead-light">
                                <tr>
                                    <th>{ "Name"}</th>
                                    <th>{ "Coordinates" }</th>
                                    <th>{ "Mac" }</th>
                                    <th>{ "Employee ID" }</th>
                                    <th>{ "Last Active" }</th>
                                    <th>{ "Work Phone" }</th>
                                    <th>{ "Mobile Phone" }</th>
                                    <th>{"Actions"}</th>
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
