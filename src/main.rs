#[macro_use]
extern crate rocket;

use std::error::Error;

use ewu_ics_cal::CalendarDetails;
use rocket::{
    futures::TryFutureExt,
    http::{ContentType, Header,},
    response::status,
};
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

struct ContentDisposition {
    file_name: String,
}

impl From<ContentDisposition> for Header<'static> {
    fn from(value: ContentDisposition) -> Self {
        Header {
            name: "Content-Disposition".into(),
            value: format!("attachment; filename=\"{}\"", value.file_name).into(),
        }
    }
}

#[derive(Responder)]
struct CalendarResponse {
    inner: String,
    content_type: ContentType,
    content_disposition: ContentDisposition,
}

async fn fetch_calendar_details(path: &str) -> Result<CalendarDetails, Box<dyn Error>> {
    let url = format!("https://www.ewubd.edu{}", path);

    let cal = reqwest::get(url)
        .and_then(|x| x.text())
        .await
        .map(|resp| Html::parse_document(&resp))
        .map_err(|err| err.into())
        .and_then(|doc| ewu_ics_cal::generate_calendar(&doc));

    cal
}

#[get("/generate?<calendar_path>")]
async fn generate_calendar(
    calendar_path: &str,
) -> Result<CalendarResponse, status::NotFound<String>> {
    let cal = fetch_calendar_details(calendar_path).await;
    let ics = cal.map(|cal| {
        (
            format!("{} - {}.ics", cal.semester, cal.revised_date.to_string()),
            ewu_ics_cal::generate_ics(cal.entries),
        )
    });

    match ics {
        Ok((file_name, ics)) => Ok(CalendarResponse {
            inner: ics,
            content_type: ContentType::new("text", "calendar"),
            content_disposition: ContentDisposition { file_name },
        }),
        Err(err) => Err(status::NotFound(err.to_string())),
    }
}

#[get("/info?<calendar_path>")]
async fn info(calendar_path: &str) -> Template {
    let cal = fetch_calendar_details(calendar_path).await;

    match cal {
        Ok(cal) => Template::render(
            "info",
            context! {
                title: cal.calendar_name,
                entries: cal.entries,
                path: calendar_path,
                semester: cal.semester
            },
        ),
        Err(error) => Template::render("error", context! {error: error.to_string()}),
    }
}

#[get("/")]
async fn index() -> Template {
    match fetch_calendar_page().await {
        Ok(doc) => {
            let cals = ewu_ics_cal::collect_all_calendars(&doc);
            Template::render("index", context! {title: "EWU iCal Generator", cals})
        }
        Err(error) => Template::render(
            "error",
            context! {
                error: error.to_string()
            },
        ),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, info, generate_calendar])
        .attach(Template::fairing())
}
