use super::parser;
use scraper::Html;
use std::error::Error;
use vercel_runtime::{Body, Error as VercelError, Request, Response};

pub async fn fetch_calendar_page() -> Result<Html, Box<dyn Error>> {
    let resp = reqwest::get("https://www.ewubd.edu/academic-calendar")
        .await?
        .text()
        .await?;
    let doc = Html::parse_document(&resp);
    Ok(doc)
}

pub async fn fetch_calendar_details(
    path: &str,
) -> Result<parser::CalendarDetails, Box<dyn Error + Send + Sync>> {
    let url = format!("https://www.ewubd.edu{}", path);

    let raw_doc = reqwest::get(url).await?.text().await?;
    let parsed_doc = Html::parse_document(&raw_doc);

    parser::generate_calendar(&parsed_doc).map_err(|e| VercelError::from(e.to_string()))
}

pub fn get_calendar_path(req: &Request) -> Result<String, Box<dyn Error + Send + Sync>> {
    let calendar_path = req
        .uri()
        .query()
        .ok_or(VercelError::from("Calendar path not found"))
        .and_then(|x| queryst::parse(x).map_err(|e| VercelError::from(e.message)))
        .and_then(|v| {
            v.find("calendar_path")
                .ok_or(VercelError::from("Calendar path not found"))
                .map(|v| v.to_owned())
        })?;

    let calendar_path = calendar_path.as_str().unwrap().into();

    Ok(calendar_path)
}

pub fn cache(res: &mut Response<Body>) {
    res.headers_mut()
        .insert("Cache-Control", "max-age=259200, public".parse().unwrap());
}
