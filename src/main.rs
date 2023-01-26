#[macro_use]
extern crate rocket;

use std::{error::Error, io::Read, sync::atomic::AtomicUsize};

use rocket::{futures::{FutureExt, TryFutureExt}, response::status::NotFound};
use rocket_dyn_templates::{context, Template};
use scraper::Html;

async fn fetch_calendar_page() -> Result<Html, Box<dyn Error>> {
    let resp = reqwest::get("https://www.ewubd.edu/academic-calendar")
        .await?
        .text()
        .await?;
    let doc = Html::parse_document(&resp);
    Ok(doc)
}

#[get("/generate?<calendar_path>")]
async fn generate_calendar(calendar_path: &str) -> Result<String, NotFound<String>> {
    let url = format!("https://www.ewubd.edu{}", calendar_path);

    let cal = reqwest::get(url)
        .and_then(|x| x.text())
        .await
        .map(|resp| Html::parse_document(&resp))
        .map_err(|err| err.into())
        .and_then(|doc| ewu_ics_cal::generate_ics(&doc))
        .map_err(|e| NotFound(e.to_string()));

    cal
}

#[get("/")]
async fn index() -> Template {
    match fetch_calendar_page().await {
        Ok(doc) => {
            let cals = ewu_ics_cal::collect_all_calendars(&doc);
            Template::render("index", context! {title: "EWU iCal Generator", cals})
        }
        Err(e) => Template::render("error", context! {}),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, generate_calendar])
        .attach(Template::fairing())
}
