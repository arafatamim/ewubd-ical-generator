use std::error::Error;

use ewu_ics_cal::{parser, utils};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let calendar =
        utils::fetch_calendar_details("/academic-calendar-details/spring-2024-graduate").await?;
    let ics = parser::generate_ics(calendar);

    println!("{ics}");

    Ok(())
}
