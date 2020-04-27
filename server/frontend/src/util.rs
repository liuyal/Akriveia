use chrono::offset::FixedOffset;
use chrono::{ DateTime, Utc, format::DelayedFormat, format::StrftimeItems, };
use common::*;
use failure::Fallible;
use stdweb::web::Date;
use yew::format::Json;
use yew::services::fetch::{ Response as FetchResponse, };

pub type JsonResponse<T> = FetchResponse<Json<Fallible<Result<T, WebError>>>>;
pub type BinResponse<T> = FetchResponse<Json<Fallible<T>>>;

#[derive(Clone, Copy, PartialEq)]
pub enum WebUserType {
    Admin,
    Responder,
}

pub fn format_timestamp<'a>(stamp: &DateTime<Utc>) -> DelayedFormat<StrftimeItems<'a>> {
    let offset_minutes = Date::new().get_timezone_offset();
    let off = FixedOffset::west(offset_minutes * 60);
    let zoned_stamp = stamp.with_timezone(&off);
    zoned_stamp.format("%c")
}

macro_rules! Log {
    ($($arg:tt)*) => (
        {
            use yew::services::ConsoleService;
            let mut console = ConsoleService::new();
            console.log(format!($($arg)*).as_str());
        }
    )
}

macro_rules! put_image {
    ($fetch_service:expr, $url:expr, $body:expr, $link:expr, $msg:expr) => {
        match yew::services::fetch::Request::put($url)
            .header("Content-Type", "image/png,image/jpg")
            .header("Accept", "application/json")
            .body(Ok($body))
        {
            Ok(req) => Some($fetch_service.fetch_binary(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! post_request {
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match yew::services::fetch::Request::post($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Json(&$request))
        {
            Ok(req) => {
                $success();
                Some($fetch_service.fetch(req, $link.send_back($msg)))
            },
            Err(e) => {
                $error(e);
                None
            },
        };
    };
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr) => {
        match yew::services::fetch::Request::post($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Json(&$request))
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! put_request {
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match yew::services::fetch::Request::put($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Json(&$request))
        {
            Ok(req) => {
                $success();
                Some($fetch_service.fetch(req, $link.send_back($msg)))
            },
            Err(e) => {
                $error(e);
                None
            },
        };
    };
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr) => {
        match yew::services::fetch::Request::put($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Json(&$request))
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! get_request {
    ($fetch_service:expr, $url:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match yew::services::fetch::Request::get($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Nothing)
        {
            Ok(req) => {
                $success();
                Some($fetch_service.fetch(req, $link.send_back($msg)))
            },
            Err(e) => {
                $error(e);
                None
            },
        };
    };
    ($fetch_service:expr, $url:expr, $link:expr, $msg:expr) => {
        match yew::services::fetch::Request::get($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Nothing)
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) => None,
        };
    };
}

macro_rules! delete_request {
    ($fetch_service:expr, $url:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match yew::services::fetch::Request::delete($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Nothing)
        {
            Ok(req) => {
                $success();
                Some($fetch_service.fetch(req, $link.send_back($msg)))
            },
            Err(e) => {
                $error(e);
                None
            },
        };
    };
    ($fetch_service:expr, $url:expr, $link:expr, $msg:expr) => {
        match yew::services::fetch::Request::delete($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Nothing)
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) => None,
        };
    };
}

pub trait JsonResponseHandler {
    fn handle_response<T, S, F>(&mut self, response: JsonResponse<T>, success: S, failure: F)
        where
        S: Fn(&mut Self, T),
        F: Fn(&mut Self, WebError),
    {
        let (_meta, Json(body)) = response.into_parts();
        match body {
            Ok(Ok(value)) => {
                success(self, value)
            },
            Ok(Err(err)) => {
                failure(self, err)
            },
            Err(err) => {
                let user_err = WebError {
                    t: AkErrorType::ConnectionError,
                    reason: err.to_string(),
                };
                failure(self, user_err)
            },
        }
    }
}
