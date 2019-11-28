
use yew::prelude::*;
use yew::Component;

pub enum Msg {
    RequestEmergency,
    RequestEndEmergency,
}

pub struct EmergencyButtons {
    is_emergency: bool,
    on_emergency: Callback<()>,
    on_end_emergency: Callback<()>,
}

#[derive(Properties)]
pub struct EmergencyButtonsProps {
    pub is_emergency: bool,
    #[props(required)]
    pub on_emergency: Callback<()>,
    #[props(required)]
    pub on_end_emergency: Callback<()>,
}

impl Component for EmergencyButtons {
    type Message = Msg;
    type Properties = EmergencyButtonsProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let result = EmergencyButtons {
            is_emergency: props.is_emergency,
            on_emergency: props.on_emergency,
            on_end_emergency: props.on_end_emergency,
        };

        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestEmergency => {
                self.on_emergency.emit(())
            },
            Msg::RequestEndEmergency => self.on_end_emergency.emit(()),
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.is_emergency = props.is_emergency;
        self.on_emergency = props.on_emergency;
        self.on_end_emergency = props.on_end_emergency;
        true
    }
}

impl Renderable<EmergencyButtons> for EmergencyButtons {
    fn view(&self) -> Html<Self> {

        html! {
            <>
                <button type="button" class="btn btn-lg btn-success e-buttons"
                    onclick=|_| Msg::RequestEmergency,
                    disabled={self.is_emergency},
                >
                    <i
                        class={ if self.is_emergency {"fa fa-refresh fa-spin"} else {"fa fa-hourglass-start"} },
                        aria-hidden="true">
                    </i>
                    { " Start Tracking" }
                </button>
                <button type="button" class="btn btn-lg btn-danger e-buttons"
                    onclick=|_| Msg::RequestEndEmergency,
                    disabled={!self.is_emergency},
                >
                    <i class="fa fa-hourglass-end" aria-hidden="true"></i>
                    { " End Tracking" }
                </button>
            </>
        }
    }
}
