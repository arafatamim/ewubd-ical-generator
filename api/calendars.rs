use ewu_ics_cal::utils::cache_headers;
use scraper::Html;
use serde_json::to_string;
use vercel_runtime::{http::bad_request, run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(calendars).await
}

async fn fetch_calendar_page() -> Result<Html, Error> {
    let resp = reqwest::get("https://www.ewubd.edu/academic-calendar")
        .await?
        .text()
        .await?;
    let doc = Html::parse_document(&resp);
    Ok(doc)
}

pub async fn calendars(_req: Request) -> Result<Response<Body>, Error> {
    match fetch_calendar_page().await {
        Ok(doc) => {
            let cals = ewu_ics_cal::parser::collect_all_calendars(&doc);

            let mut response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(to_string(&cals)?.into())?;

            cache_headers(&mut response);
            Ok(response)
        }
        Err(error) => bad_request(error.to_string()),
    }
}
