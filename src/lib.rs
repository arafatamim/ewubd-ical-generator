mod wasm;

use chrono::prelude::*;
use scraper::{Html, Selector};
use ics::{
    components::Parameter,
    properties::{DtEnd, DtStart, LastModified, Location, Summary},
    Event, ICalendar, Standard, TimeZone as ICSTimeZone,
};
use std::error::Error;

pub struct Entry {
    pub date: (NaiveDate, NaiveDate),
    pub revised_date: NaiveDate,
    pub event: String,
}

#[derive(Debug)]
pub enum Semester {
    Spring(i32),
    Summer(i32),
    Fall(i32),
}

fn clean_raw_date(date: &str) -> String {
    date.replace('.', "").replace("Sept ", "Sep ")
}

fn with_event_year(event_date: NaiveDate, publish_date: NaiveDate) -> NaiveDate {
    let month_diff = event_date.month() as i32 - publish_date.month() as i32;
    let event_year = if month_diff.is_negative() {
        publish_date.year() + 1
    } else {
        publish_date.year()
    };
    event_date.with_year(event_year).unwrap()
}

pub fn generate_entries_from_html(doc: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
    let doc = Html::parse_document(doc);

    let revise_date_selector =
        Selector::parse(".row > .col-md-9 > h4 > strong:nth-of-type(3)").unwrap();
    let semester_selector = Selector::parse(".row h3:nth-of-type(2)").unwrap();
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let date_selector = Selector::parse("td:nth-of-type(1)").unwrap();
    let event_selector = Selector::parse("td:nth-of-type(3)").unwrap();

    let revise_date_raw = doc
        .select(&revise_date_selector)
        .next()
        .ok_or("Calendar revised date not found")?
        .text()
        .collect::<String>()
        .trim()
        .to_string();
    let semester = doc
        .select(&semester_selector)
        .next()
        .ok_or("Could not decode semester")?
        .text()
        .collect::<String>()
        .trim()
        .to_string();
    let year = semester
        .split_whitespace()
        .nth(1)
        .expect("Year not found")
        .parse::<i32>()
        .expect("year not valid");
    let sem = match semester
        .split_whitespace()
        .next()
        .expect("Error decoding semester")
    {
        "Spring" => Semester::Spring(year),
        "Summer" => Semester::Summer(year),
        "Fall" => Semester::Fall(year),
        _ => unreachable!(),
    };
    let table = doc
        .select(&table_selector)
        .next()
        .ok_or("dates not found")?;

    let revised_date = NaiveDate::parse_from_str(&revise_date_raw, "{%d %B %Y}")?;
    let publish_date = match sem {
        Semester::Spring(year) => NaiveDate::from_ymd(year, 1, 1),
        Semester::Summer(year) => NaiveDate::from_ymd(year, 5, 1),
        Semester::Fall(year) => NaiveDate::from_ymd(year, 9, 1),
    };

    let mut entries: Vec<Entry> = vec![];
    for row in table.select(&row_selector) {
        let date = row
            .select(&date_selector)
            .map(|a| a.text().collect::<String>().trim().to_owned())
            .collect::<String>();
        if date.is_empty() {
            continue;
        }

        let format_1 = "%B %d %Y";
        let format_2 = "%d %B %Y";

        let mut days = date.split('-');
        let raw_start_date = days.next().map(clean_raw_date);
        let raw_end_date = days.next().map(clean_raw_date);
        let date = match (raw_start_date, raw_end_date) {
            (Some(x), Some(y)) => {
                let start_day_raw = format!("{} 1970", x);
                let end_day_raw = format!("{} 1970", y);

                let start_date = NaiveDate::parse_from_str(&start_day_raw, format_1)
                    .or_else(|_| NaiveDate::parse_from_str(&start_day_raw, format_2))
                    .map(|x| with_event_year(x, publish_date))?;

                let end_date = NaiveDate::parse_from_str(&end_day_raw, format_1)
                    .or_else(|_| NaiveDate::parse_from_str(&end_day_raw, format_2))
                    .or_else(|_| -> Result<NaiveDate, Box<dyn Error>> {
                        let single_day = y.parse::<i32>()?;
                        let date = NaiveDate::from_ymd(1970, start_date.month(), single_day as u32);
                        Ok(date)
                    })
                    .map(|x| with_event_year(x, publish_date))?;

                (start_date, end_date)
            }
            (Some(x), None) => {
                let raw_date = format!("{} 1970", x);
                let date = NaiveDate::parse_from_str(&raw_date, format_1)
                    .or_else(|_| NaiveDate::parse_from_str(&raw_date, format_2))
                    .map(|x| with_event_year(x, publish_date))?;
                (date, date)
            }
            _ => continue,
        };

        let event = row
            .select(&event_selector)
            .map(|a| {
                a.text()
                    .fold(String::new(), |a, b| a.trim().to_string() + " " + b.trim())
                    .trim()
                    .to_owned()
            })
            .collect::<String>();

        entries.push(Entry {
            date,
            event,
            revised_date,
        });
    }

    Ok(entries)
}

pub fn generate_ics(doc: &str) -> Result<String, Box<dyn Error>> {
    let mut calendar = ICalendar::new("2.0", "icalendar");

    let entries = generate_entries_from_html(doc)?;

    let timezone = ICSTimeZone::standard(
        "Asia/Dhaka",
        Standard::new("19700101T000000", "+0600", "+0600"),
    );
    calendar.add_timezone(timezone);
    for entry in entries {
        let ev_hash = xxhash_rust::xxh3::xxh3_64(entry.event.as_bytes());
        let mut event = Event::new(
            format!("{:x}", ev_hash),
            Utc::now().format("%Y%m%dT000000").to_string(),
        );
        let mut dtstart = DtStart::new(entry.date.0.format("%Y%m%d").to_string());
        dtstart.add(Parameter::new("VALUE", "DATE"));
        event.push(dtstart);

        let mut dtend = DtEnd::new(entry.date.1.format("%Y%m%d").to_string());
        dtend.add(Parameter::new("VALUE", "DATE"));
        event.push(dtend);

        event.push(LastModified::new(
            entry.revised_date.format("%Y%m%dT000000").to_string(),
        ));
        event.push(Summary::new(entry.event));
        event.push(Location::new("East West University, Dhaka"));
        calendar.add_event(event);
    }

    Ok(calendar.to_string())
}

// TODO: implement SEQUENCE property
