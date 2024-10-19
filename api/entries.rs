use chrono::NaiveDateTime;
use ewu_ics_cal::utils::{self, cache_headers, last_modified_header};
use reqwest::header::{CONTENT_TYPE, IF_MODIFIED_SINCE};
use serde_json::json;
use vercel_runtime::{run, Body, Error as VercelError, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), VercelError> {
    run(entries).await
}

pub async fn entries(req: Request) -> Result<Response<Body>, VercelError> {
    let calendar_remote_path = utils::get_calendar_path(&req)?;
    let cal = utils::fetch_calendar_details(&calendar_remote_path).await?;

    let if_modified_since = req.headers().get(IF_MODIFIED_SINCE).map(|x| {
        NaiveDateTime::parse_from_str(x.to_str().unwrap(), "%a, %d %b %Y %H:%M:%S GMT")
            .unwrap()
            .date()
    });

    match if_modified_since {
        Some(date) if date >= cal.revised_date => Ok(Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::Empty)?),
        _ => {
            let mut resp = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "application/json")
                .body(json!(cal).to_string().into())?;

            cache_headers(last_modified_header(&mut resp, cal.revised_date));

            Ok(resp)
        }
    }
}
