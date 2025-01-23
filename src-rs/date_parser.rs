use std::error::Error;

use chumsky::{
    error::Simple,
    primitive::{choice, just, take_until},
    text, Parser,
};

// use nom::{
//     branch::alt,
//     bytes::complete::{tag, take_till},
//     character::complete::{digit1, space1},
//     combinator::{map, opt},
//     multi::separated_list1,
//     sequence::{preceded, tuple},
//     IResult, Parser,
// };

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DatePart {
    pub month: Option<u8>,
    pub day: u8,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DateRange {
    pub start: DatePart,
    pub end: Option<DatePart>,
}

pub fn parse_month() -> impl Parser<char, u8, Error = Simple<char>> {
    choice((
        choice((just("January"), just("Jan"))).to(1),
        choice((just("February"), just("Feb"))).to(2),
        choice((just("March"), just("Mar"))).to(3),
        choice((just("April"), just("Apr"))).to(4),
        just("May").to(5),
        choice((just("June"), just("Jun"))).to(6),
        choice((just("July"), just("Jul"))).to(7),
        choice((just("August"), just("Aug"))).to(8),
        choice((just("September"), just("Sept"), just("Sep"))).to(9),
        choice((just("October"), just("Oct"))).to(10),
        choice((just("November"), just("Nov"))).to(11),
        choice((just("December"), just("Dec"))).to(12),
    ))
}

/// eg: January 27, Feb 16
pub fn parse_date_part() -> impl Parser<char, DatePart, Error = Simple<char>> {
    parse_month()
        .or_not()
        .then(
            take_until(text::digits(10))
                .map(|(_, day): (Vec<char>, String)| day.parse::<u8>().unwrap()),
        )
        .map(|(month, day)| DatePart { month, day })
}

/// parses multiple date parts between a separator
pub fn parse_date_parts() -> impl Parser<char, Vec<DatePart>, Error = Simple<char>> {
    parse_date_part().separated_by(choice((just(" - "), just("-"), just(" "))))
}

/*
 * Nom parser works but gives bizarre borrow checker error

pub fn parse_month<'a>(i: &'a str) -> IResult<&'a str, u8> {
    alt((
        alt((tag("January"), tag("Jan"))).map(|_| 1),
        alt((tag("February"), tag("Feb"))).map(|_| 2),
        alt((tag("March"), tag("Mar"))).map(|_| 3),
        alt((tag("April"), tag("Apr"))).map(|_| 4),
        tag("May").map(|_| 5),
        alt((tag("June"), tag("Jun"))).map(|_| 6),
        alt((tag("July"), tag("Jul"))).map(|_| 7),
        alt((tag("August"), tag("Aug"))).map(|_| 8),
        alt((tag("September"), tag("Sept"), tag("Sep"))).map(|_| 9),
        alt((tag("October"), tag("Oct"))).map(|_| 10),
        alt((tag("November"), tag("Nov"))).map(|_| 11),
        alt((tag("December"), tag("Dec"))).map(|_| 12),
    ))(i)
}

pub fn parse_date_part<'a>(i: &'a str) -> IResult<&'a str, DatePart> {
    map(
        tuple((
            opt(parse_month),
            preceded(
                take_till(|c: char| c.is_ascii_digit()),
                map(digit1, |d: &str| d.parse::<u8>().unwrap()),
            ),
        )),
        |(month, day)| DatePart { month, day },
    )(i)
}

pub fn parse_date_parts<'a>(i: &'a str) -> IResult<&'a str, Vec<DatePart>> {
    separated_list1(alt((tag(" - "), tag("-"), space1)), parse_date_part)(i)
}
*/

/// converts date parts into a readable struct
pub fn parse_date_range(i: &str) -> Result<DateRange, Box<dyn Error>> {
    let date_parts = parse_date_parts().parse(i).expect("Couldn't parse date");

    if date_parts.len() > 2 || date_parts.is_empty() {
        return Err("Invalid date range".into());
    }

    Ok(DateRange {
        start: date_parts[0],
        end: if date_parts.len() == 2 {
            Some(date_parts[1])
        } else {
            None
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let cases = [
            (
                "May 31 June 06",
                vec![
                    DatePart {
                        month: Some(5),
                        day: 31,
                    },
                    DatePart {
                        month: Some(6),
                        day: 6,
                    },
                ],
            ),
            (
                "March 08-14",
                vec![
                    DatePart {
                        month: Some(3),
                        day: 8,
                    },
                    DatePart {
                        month: None,
                        day: 14,
                    },
                ],
            ),
            (
                "Sept. 26-Oct. 01",
                vec![
                    DatePart {
                        month: Some(9),
                        day: 26,
                    },
                    DatePart {
                        month: Some(10),
                        day: 1,
                    },
                ],
            ),
            (
                "May 31 - June 06",
                vec![
                    DatePart {
                        month: Some(5),
                        day: 31,
                    },
                    DatePart {
                        month: Some(6),
                        day: 6,
                    },
                ],
            ),
            (
                "September 23",
                vec![DatePart {
                    month: Some(9),
                    day: 23,
                }],
            ),
        ];

        for (input, expected) in cases.iter() {
            let result = parse_date_parts().parse(*input).unwrap();
            assert_eq!(*expected, result);
        }
    }
}
