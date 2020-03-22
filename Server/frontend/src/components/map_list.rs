use common::*;
use crate::util::*;
use super::root;
use super::value_button::ValueButton;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::prelude::*;
use super::user_message::UserMessage;

pub enum Msg {
    ChangeRootPage(root::Page),

    RequestDeleteMap(i32),
    RequestGetMaps,

    ResponseGetMaps(JsonResponse<Vec<Map>>),
    ResponseDeleteMap(JsonResponse<()>),
}

pub struct MapList {
    change_page: Callback<root::Page>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    list: Vec<Map>,
    self_link: ComponentLink<Self>,
    user_msg: UserMessage<Self>,
}

impl JsonResponseHandler for MapList {}

#[derive(Properties)]
pub struct MapListProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for MapList {
    type Message = Msg;
    type Properties = MapListProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetMaps);
        let result = MapList {
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
            Msg::RequestGetMaps => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetMaps
                );
            },
            Msg::RequestDeleteMap(id) => {
                self.fetch_task = delete_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseDeleteMap
                );
            },
            Msg::ResponseGetMaps(response) => {
                self.handle_response(
                    response,
                    |s, mut maps| {
                        maps.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                        s.list = maps;
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to obtain map list, reason: {}", e));
                    },
                );
            },
            Msg::ResponseDeleteMap(response) => {
                self.handle_response(
                    response,
                    |s, _| {
                        s.user_msg.success_message = Some("successfully deleted map".to_owned());
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to delete map, reason: {}", e));
                    },
                );
                // now that the map is deleted, get the updated list
                self.self_link.send_self(Msg::RequestGetMaps);
            },
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        self.self_link.send_self(Msg::RequestGetMaps);
        true
    }
}

impl Renderable<MapList> for MapList {
    fn view(&self) -> Html<Self> {
        let mut rows = self.list.iter().map(|map| {
            html! {
                <tr>
                    <td>{ &map.name }</td>
                    <td>{ format!("{},{}", &map.bounds.x, &map.bounds.y) }</td>
                    <td>{ map.scale }</td>
                    <td>{ map.note.clone().unwrap_or(String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Edit".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::MapAddUpdate(Some(value))),
                            border=false,
                            icon="fa fa-pencil-square-o",
                            style="btn-primary",
                            value=map.id,
                        />
                        <ValueButton<i32>
                            display=Some("Delete".to_string()),
                            on_click=|value: i32| Msg::RequestDeleteMap(value),
                            border=false,
                            icon="fa fa-trash",
                            style="btn-secondary",
                            value=map.id,
                        />
                        <ValueButton<i32>
                            display=Some("View".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::MapView(Some(value))),
                            border=false,
                            icon="fa fa-external-link",
                            style="btn-warning",
                            value=map.id
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
                            <h2>{ "Map List"}</h2>
                            <button
                                class="btn btn-success logoutPlacement my-1",
                                onclick=|_| Msg::ChangeRootPage(root::Page::MapAddUpdate(None)),
                            >
                                {"Add Map"}
                            </button>
                        </div>
                        <table class="table table-striped">
                            <thead>
                                <tr>
                                    <th>{ "Name" }</th>
                                    <th>{ "Bounds" }</th>
                                    <th>{ "Scale" }</th>
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
