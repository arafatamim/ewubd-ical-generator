use std::{error::Error, fs::File, io::Write};

pub fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).expect("path not provided");
    let body = reqwest::blocking::get(format!("https://www.ewubd.edu{}", path))?.text()?;
    let cal = ewu_ics_cal::generate_ics(&body).expect("could not generate calendar");
    let mut output = File::create("ewubd_calendar.ics")?;
    write!(&mut output, "{}", cal)?;
    println!("iCal file saved at ewubd_calendar.ics");

    Ok(())
}
