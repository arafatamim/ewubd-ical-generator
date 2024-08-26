use ewu_ics_cal::{parser, utils::{self, cache}};
use vercel_runtime::{run, Body, Error as VercelError, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), VercelError> {
    run(generate).await
}

pub async fn generate(req: Request) -> Result<Response<Body>, VercelError> {
    let calendar_remote_path = utils::get_calendar_path(&req)?;
    let calendar = utils::fetch_calendar_details(&calendar_remote_path).await?;

    let semester = calendar.semester.clone();
    let year = calendar.year;
    let revised_date = calendar.revised_date;

    let ics = parser::generate_ics(calendar);
    let filename = format!("{semester} {year} - {revised_date}.ics");

    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/calendar")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(ics.into())?;

    cache(&mut resp);

    Ok(resp)
}
