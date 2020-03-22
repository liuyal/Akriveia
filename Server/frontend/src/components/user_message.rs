
use yew::prelude::*;
use std::marker::PhantomData;

pub struct UserMessage<T> {
    pub error_messages: Vec<String>,
    pub success_message: Option<String>,
    _dummy: PhantomData<T>,
}

impl <T> UserMessage<T> {
    pub fn reset(&mut self) {
        self.error_messages = Vec::new();
        self.success_message = None;
    }

    pub fn new() -> Self {
        UserMessage {
            error_messages: Vec::new(),
            success_message: None,
            _dummy: PhantomData,
        }
    }

    pub fn view(&self) -> Html<T>
        where T: yew::Component
    {
        let mut errors = self.error_messages.iter().map(|msg| {
            html! {
                <div
                    class="alert alert-danger"
                    role="alert"
                >
                    {msg}
                </div>
            }
        });

        html! {
            <div>
                {
                    match &self.success_message {
                        Some(msg) => html! {
                            <div class="alert alert-success" role="alert">
                                { format!("Success: {}", msg) }
                            </div>
                        },
                        None => html! { },
                    }
                }
                {
                    if self.error_messages.len() > 0 {
                        html! {
                            <div class="alert alert-danger" role="alert">
                               <p>{ "Failure: " }</p>
                                { for errors }
                            </div>
                        }
                   } else {
                       html! { }
                   }
                }
            </div>
        }
    }
}

