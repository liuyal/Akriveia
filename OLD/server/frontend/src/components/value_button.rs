
use yew::prelude::*;

pub enum Msg {
    Click,
}

pub struct ValueButton<T> {
    pub display: Option<String>,
    pub value: T,
    pub on_click: Callback<T>,
    pub disabled: bool,
    pub border: bool,
    pub style: String,
    pub icon: String,
}

#[derive(Properties)]
pub struct ValueButtonProps<T> {
    #[props(required)]
    pub value: T,
    #[props(required)]
    pub on_click: Callback<T>,
    pub disabled: bool,
    pub border: bool,
    pub display: Option<String>,
    pub icon: String,
    pub style: String,
}

impl <T: 'static> Component for ValueButton<T>
    where T: std::default::Default + std::clone::Clone + std::cmp::PartialEq + std::fmt::Display
{
    type Message = Msg;
    type Properties = ValueButtonProps<T>;

    fn create(props: Self::Properties, mut _link: ComponentLink<Self>) -> Self {
        ValueButton {
            value: props.value,
            on_click: props.on_click,
            disabled: props.disabled,
            border: props.border,
            display: props.display,
            style: props.style,
            icon: props.icon,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
                self.on_click.emit(self.value.clone());
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.value = props.value;
        self.on_click = props.on_click;
        self.disabled = props.disabled;
        self.border = props.border;
        self.icon = props.icon;
        self.style = props.style;
        true
    }
}

impl <T: 'static> Renderable<ValueButton<T>> for ValueButton<T>
    where T: std::default::Default + std::clone::Clone + std::cmp::PartialEq + std::fmt::Display
{
    fn view(&self) -> Html<Self> {
        let bold = if self.border {
            "font-weight-bold"
        } else {
            ""
        };

        let space_between = if self.icon.is_empty() {""} else {" "};

        let cls = format!("btn {} btn-sm {} spacing", self.style, bold);

        html! {
            <>
                <button
                    type="button"
                    disabled={self.disabled},
                    onclick=|_| Msg::Click,
                    class={cls},
                >
                    <i class={&self.icon} aria-hidden="true"></i>
                    { space_between }
                    { self.display.as_ref().unwrap_or(&self.value.to_string()) }
                </button>
            </>
        }
    }
}

pub struct DisplayButton<T> {
    pub display: String,
    pub value: T,
    pub on_click: Callback<T>,
    pub disabled: bool,
    pub border: bool,
    pub icon: String,
    pub style: String,
}

#[derive(Properties)]
pub struct DisplayButtonProps<T> {
    #[props(required)]
    pub value: T,
    #[props(required)]
    pub on_click: Callback<T>,
    pub disabled: bool,
    pub border: bool,
    pub display: String,
    pub icon: String,
    #[props(required)]
    pub style: String,
}

impl <T: 'static> Component for DisplayButton<T>
    where T: std::default::Default + std::clone::Clone + std::cmp::PartialEq
{
    type Message = Msg;
    type Properties = DisplayButtonProps<T>;

    fn create(props: Self::Properties, mut _link: ComponentLink<Self>) -> Self {
        DisplayButton {
            value: props.value,
            on_click: props.on_click,
            disabled: props.disabled,
            border: props.border,
            display: props.display,
            icon: props.icon,
            style: props.style,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
                self.on_click.emit(self.value.clone());
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.value = props.value;
        self.on_click = props.on_click;
        self.disabled = props.disabled;
        self.border = props.border;
        self.icon = props.icon;
        self.style = props.style;
        true
    }
}

impl <T: 'static> Renderable<DisplayButton<T>> for DisplayButton<T>
    where T: std::default::Default + std::clone::Clone + std::cmp::PartialEq
{
    fn view(&self) -> Html<Self> {
        let bold = if self.border {
            "font-weight-bold"
        } else {
            ""
        };

        let space_between = if self.icon.is_empty() {""} else {" "};

        let cls = format!("{} {} spacing", self.style, bold);
        html! {
            <>
                <button
                    type="button"
                    disabled={self.disabled},
                    onclick=|_| Msg::Click,
                    class={cls},
                >
                    <i class={ &self.icon } aria-hidden="true"></i>
                    { space_between }
                    { &self.display }
                </button>
            </>
        }
    }
}
