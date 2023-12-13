use ewu_ics_cal::{parser, utils::{self, cache}};
use vercel_runtime::{run, Body, Error as VercelError, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), VercelError> {
    run(generate).await
}

pub async fn generate(req: Request) -> Result<Response<Body>, VercelError> {
    let calendar_remote_path = utils::get_calendar_path(&req)?;
    let calendar = utils::fetch_calendar_details(&calendar_remote_path).await?;

    let ics = parser::generate_ics(calendar.entries);
    let filename = format!("{} - {}.ics", calendar.semester, calendar.revised_date);

    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/calendar")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(ics.to_string().into())?;

    cache(&mut resp);

    Ok(resp)
}
