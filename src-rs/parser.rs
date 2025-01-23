use chrono::prelude::*;
use ics::{
    components::{Parameter, Property},
    properties::{CalScale, DtEnd, DtStart, LastModified, Location, Method, Name, Summary},
    Event, ICalendar, Standard, TimeZone as ICSTimeZone,
};
use regex::Regex;
use scraper::{Html, Selector};
use serde::Serialize;
use std::error::Error;

use crate::date_parser::parse_date_range;

#[derive(Serialize, Debug)]
pub struct CalendarDetails {
    pub calendar_name: String,
    pub semester: String,
    pub revised_date: NaiveDate,
    pub entries: Vec<Entry>,
    pub year: i32,
}

#[derive(Serialize, Debug)]
pub struct Entry {
    pub date: (NaiveDate, Option<NaiveDate>),
    pub revised_date: NaiveDate,
    pub event: String,
}

#[derive(Debug, Serialize)]
pub enum Semester {
    Spring(i32),
    Summer(i32),
    Fall(i32),
}

#[derive(Debug, Serialize)]
pub struct Calendar {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct Program {
    pub program_type: String,
    pub calendars: Vec<Calendar>,
}

#[derive(Debug, Serialize)]
pub struct CalendarList {
    pub year: String,
    pub programs: Vec<Program>,
}

/// Computes the year event occurs.
/// If event month is behind publish month,
/// then assumes that event year is later than publish year,
/// or else assumes that event year is the same as publish year
fn with_event_year(event_date: NaiveDate, publish_date: NaiveDate) -> NaiveDate {
    let month_diff = event_date.month() as i32 - publish_date.month() as i32;
    let event_year = if month_diff.is_negative() {
        publish_date.year() + 1
    } else {
        publish_date.year()
    };
    event_date.with_year(event_year).unwrap()
}

pub fn collect_all_calendars(doc: &Html) -> Vec<CalendarList> {
    let years = get_years(doc);

    years
        .iter()
        .map(|year| {
            let programs = get_programs(doc, year);
            CalendarList {
                year: year.to_owned(),
                programs,
            }
        })
        .collect()
}

pub fn get_years(doc: &Html) -> Vec<String> {
    let year_selector = Selector::parse(".training-program-tab li").unwrap();

    let years = doc
        .select(&year_selector)
        .map(|el| el.text().collect::<String>())
        .collect::<Vec<String>>();

    years
}

pub fn get_programs(doc: &Html, year: &str) -> Vec<Program> {
    let tab_selector = Selector::parse(&format!(".tab-content > [id=\"{}\"]", year)).unwrap();
    let panel_heading_selector = Selector::parse(".panel-heading").unwrap();
    let panel_body_selector = Selector::parse(".panel-body").unwrap();
    let calendar_semester_selector = Selector::parse("ul > li > a").unwrap();

    let year_tab = doc.select(&tab_selector).next().unwrap();

    let panels = year_tab
        .select(&panel_heading_selector)
        .map(|el| el.text().collect::<String>().trim().to_owned());

    let calendars_panel = year_tab.select(&panel_body_selector).map(|el| {
        el.select(&calendar_semester_selector).map(|el| Calendar {
            name: el.text().collect::<String>().trim().to_owned(),
            url: el.value().attr("href").unwrap().to_owned(),
        })
    });

    let calendars: Vec<Program> = panels
        .zip(calendars_panel)
        .map(|(program, calendars)| Program {
            program_type: program,
            calendars: calendars
                .filter(|cal| !cal.name.to_lowercase().contains("exam")) // filter out exam calendars
                .collect::<Vec<Calendar>>(),
        })
        .collect::<Vec<Program>>();

    calendars
}

pub fn generate_calendar<'a>(doc: &'a Html) -> Result<CalendarDetails, Box<dyn Error + 'a>> {
    let general_selector = Selector::parse(".row > .col-md-9").unwrap();
    let calendar_name_selector = Selector::parse(".row > .col-md-9 h3:nth-of-type(1)").unwrap();
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let date_selector = Selector::parse("td:nth-of-type(1)").unwrap();
    let event_selector = Selector::parse("td:nth-of-type(3)").unwrap();

    let revise_date_regex = Regex::new(r"\{((\d?\d)\s(\w+)\s(\d{4}))\}").unwrap();
    let semester_regex = Regex::new(r"(?i)(spring|summer|fall)\s(\d{4})").unwrap();

    let raw_doc = doc
        .select(&general_selector)
        .next()
        .expect("Couldn't extract raw document")
        .text()
        .collect::<String>()
        .trim()
        .to_string();
    let revise_date_raw = revise_date_regex
        .captures(&raw_doc)
        .expect("Calendar revise date not found")[1]
        .to_string();

    let semester_capture = semester_regex
        .captures(&raw_doc)
        .expect("Semester not found");
    let semester = semester_capture[1].to_string();
    let year = semester_capture[2]
        .parse::<i32>()
        .expect("Couldn't decode year");
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

    let revised_date = NaiveDate::parse_from_str(&revise_date_raw, "%d %B %Y")?;
    let publish_date = match sem {
        Semester::Spring(year) => NaiveDate::from_ymd_opt(year, 1, 1).unwrap(), // rough approximations                                                                                // approximations
        Semester::Summer(year) => NaiveDate::from_ymd_opt(year, 5, 1).unwrap(),
        Semester::Fall(year) => NaiveDate::from_ymd_opt(year, 9, 1).unwrap(),
    };

    let mut entries: Vec<Entry> = vec![];
    for row in table.select(&row_selector) {
        let date_str = row
            .select(&date_selector)
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .collect::<String>();

        if date_str.is_empty() {
            continue;
        }

        if date_str.starts_with("Date") {
            continue;
        }

        let date_range = parse_date_range(&date_str)?;
        // let date_range = DateRange {
        //     start: DatePart {
        //         month: Some(1),
        //         day: 1,
        //     },
        //     end: Some(DatePart {
        //         month: Some(1),
        //         day: 1,
        //     }),
        // };
        let start_date_part = date_range.start;
        let end_date_part = date_range.end;

        let start_date = NaiveDate::from_ymd_opt(
            1972,
            start_date_part.month.expect("start month is empty") as u32,
            start_date_part.day as u32,
        )
        .map(|x| with_event_year(x, publish_date))
        .expect("invalid start date");

        let date = match end_date_part {
            Some(end_date_part) => {
                let end_date = NaiveDate::from_ymd_opt(
                    1972,
                    end_date_part
                        .month
                        .or(start_date_part.month)
                        .expect("both start and end month is empty") as u32,
                    end_date_part.day as u32,
                )
                .map(|x| with_event_year(x, publish_date))
                .expect("invalid end date");

                (start_date, Some(end_date))
            }
            None => (start_date, None),
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

    let calendar_name = doc
        .select(&calendar_name_selector)
        .map(|el| el.text().collect::<String>().trim().to_owned())
        .collect::<String>();

    Ok(CalendarDetails {
        calendar_name,
        revised_date,
        semester,
        year,
        entries,
    })
}

pub fn generate_ics(calendar_details: CalendarDetails) -> String {
    let mut calendar = ICalendar::new("2.0", "icalendar");

    let timezone = ICSTimeZone::standard(
        "Asia/Dhaka",
        Standard::new("19700101T000000", "+0600", "+0600"),
    );
    calendar.add_timezone(timezone);
    let cal_name = format!(
        "{} {} {}",
        calendar_details.semester, calendar_details.year, calendar_details.calendar_name
    );
    calendar.push(Name::new(&cal_name));
    calendar.push(Property::new("X-WR-CALNAME", &cal_name));
    calendar.push(CalScale::new("GREGORIAN"));
    calendar.push(Method::new("PUBLISH"));

    let entries = calendar_details.entries;

    for entry in entries {
        let ev_hash = xxhash_rust::xxh3::xxh3_64(entry.event.as_bytes());
        let mut event = Event::new(
            format!("{:x}", ev_hash),
            Utc::now().format("%Y%m%dT000000").to_string(),
        );
        let mut dtstart = DtStart::new(entry.date.0.format("%Y%m%d").to_string());
        dtstart.add(Parameter::new("VALUE", "DATE"));
        event.push(dtstart);

        let mut dtend = DtEnd::new(
            entry
                .date
                .1
                .or(Some(entry.date.0))
                .map(|date| date.format("%Y%m%d").to_string())
                .unwrap(),
        );
        dtend.add(Parameter::new("VALUE", "DATE"));
        event.push(dtend);

        event.push(LastModified::new(
            entry.revised_date.format("%Y%m%dT000000").to_string(),
        ));
        event.push(Summary::new(entry.event));
        event.push(Location::new("East West University, Dhaka"));
        calendar.add_event(event);
    }

    calendar.to_string()
}

// TODO: implement SEQUENCE property
