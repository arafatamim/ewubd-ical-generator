use ewu_ics_cal::utils::{self, cache};
use serde_json::json;
use vercel_runtime::{run, Body, Error as VercelError, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), VercelError> {
    run(entries).await
}

pub async fn entries(req: Request) -> Result<Response<Body>, VercelError> {
    let calendar_remote_path = utils::get_calendar_path(&req)?;
    let cal = utils::fetch_calendar_details(&calendar_remote_path).await?;

    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!(cal).to_string().into())?;

    cache(&mut resp);

    Ok(resp)
}
