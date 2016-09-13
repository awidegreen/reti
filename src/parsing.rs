use std::io;
//use std::io::prelude::*;
use std::str::FromStr;
use std::num;

use chrono::*;

use data::*;
use rustc_serialize::json;

// result & error things
#[derive(Debug)]
pub enum DataParseError {
    Io(io::Error),
    ChronoParseError(ParseError),
    Number(num::ParseIntError),
    Json(json::DecoderError),
    DayFormat,
    Line,
    Format,
    Logic,
    Comment,
}
impl From<num::ParseIntError> for DataParseError {
    fn from(err: num::ParseIntError) -> DataParseError {
        DataParseError::Number(err)
    }
}
impl From<ParseError> for DataParseError {
    fn from(err: ParseError) -> DataParseError {
        DataParseError::ChronoParseError(err)
    }
}
impl From<io::Error> for DataParseError {
    fn from(err: io::Error) -> DataParseError {
        DataParseError::Io(err)
    }
}

impl From<json::DecoderError> for DataParseError {
    fn from(err: json::DecoderError) -> DataParseError {
        DataParseError::Json(err)
    }
}
pub type DataResult<T> = Result<T, DataParseError>;
//-----

pub fn parse_date(ymd: &str) -> DataResult<NaiveDate> {
    let ymd : Vec<&str> = ymd.trim().split('-').collect();

    let now = UTC::now();
    let (year, month, day) = match ymd.len() {
        3 => {
            let year  = try!(i32::from_str(ymd[0]));
            let month = try!(u32::from_str(ymd[1]));
            let day   = try!(u32::from_str(ymd[2]));
            (year, month, day)
        },
        2 => {
            let year  = now.year() as i32;
            let month = try!(u32::from_str(ymd[0]));
            let day   = try!(u32::from_str(ymd[1]));
            (year, month, day)
        },
        1 => {
            let year  = now.year() as i32;
            let month = now.month() as u32;
            let day   = try!(u32::from_str(ymd[0]));
            (year, month, day)

        },
        _ => return Err(DataParseError::DayFormat)
    };

     match NaiveDate::from_ymd_opt(year as i32, month, day) {
        Some(date) => Ok(date),
        None => Err(DataParseError::DayFormat)
    }
}

pub fn parse_day_line_from_legacy(line: &str) -> DataResult<Day> {
    let line = line.trim();

    if line.starts_with("#") {
        return Err(DataParseError::Comment);
    }

    let (line, comment) = match line.find('#') {
        Some(pos) => {
            let comment = Some((&line[pos+1..]).trim().to_string());
            (&line[..pos], comment)
        },
        None => (line, None)
    };

    let elements : Vec<&str> = line.split_whitespace().collect();

    if elements.len() < 1 {
        return Err(DataParseError::Line)
    }

    let date = match parse_date(elements[0]) {
        Ok(date) => {
            //elements.pop();
            date
        },
        Err(_) =>
            match parse_part_from_legacy(elements[0]) {
                Ok(_) => UTC::today().naive_local(),
                Err(_) => return Err(DataParseError::Format)
            }
    };

    let mut parts: Vec<Part> = vec![];
    for e in &elements {
        match parse_part_from_legacy(e) {
            Ok(p) => parts.push(p),
            _ => continue,
            //Err(err) => println!("Unable to parse part: '{}' ({:?})", e, err)
        }
    }

    Ok(Day { date: date, parts: parts, comment: comment })
}

pub fn parse_part_from_legacy(span: &str) -> DataResult<Part> {
    let start_stop : Vec<&str> = span.split('-').collect();
    if start_stop.len() < 2 || start_stop.len() > 3 {
        return Err(DataParseError::Format)
    }
    let start = try!(parse_time_from_str(start_stop[0]));
    let stop  = try!(parse_time_from_str(start_stop[1]));

    if stop < start {
        println!("Stop {:?} is earlier then start {:?}!", stop, start);
        return Err(DataParseError::Logic)
    }

    let factor: f32 = match start_stop.len() {
        3 => {
            match start_stop[2].parse::<f32>() {
                Ok(x) => x,
                _ => 1.0,
            }
        },
        _ => 1.0
    };

    //let comment = "";

    let part = Part {
        start: start,
        stop: Some(stop),
        factor: Some(factor),
    };
    Ok(part)
}

pub fn parse_time_from_str(time: &str) -> DataResult<NaiveTime> {
    match NaiveTime::parse_from_str(time, "%H:%M") {
        Ok(t) => Ok(t),
        _     => Ok(try!(NaiveTime::parse_from_str(time, "%H%M")))
    }
}

#[test]
fn test_parse_date() {
    let now = UTC::today();

    let d = parse_date("08  ").unwrap();
    assert_eq!(now.year(), d.year());
    assert_eq!(now.month(), d.month());
    assert_eq!(08, d.day());

    let d = parse_date("06-08").unwrap();
    assert_eq!(now.year(), d.year());
    assert_eq!(06, d.month());
    assert_eq!(08, d.day());

    let d = parse_date("2014-06-08").unwrap();
    assert_eq!(2014, d.year());
    assert_eq!(06, d.month());
    assert_eq!(08, d.day());

    let d = parse_date(" 05-23   ").unwrap();
    assert_eq!(now.year(), d.year());
    assert_eq!(05, d.month());
    assert_eq!(23, d.day());
}

#[test]
fn test_parse_legacy_day_from_line() {
    let today = UTC::today();
    let l = String::from("05-23     10:00-11:30-1.0 --- 12:30-18:00");
    let d = parse_day_line_from_legacy(&l).unwrap();

    assert_eq!(today.year(), d.date.year());
    assert_eq!(05, d.date.month());
    assert_eq!(23, d.date.day());
    assert_eq!(2, d.parts.len());

    let d = parse_day_line_from_legacy("10:00-11:30 - 12:30-18:00").unwrap();
    assert_eq!(today.year(), d.date.year());
    assert_eq!(today.month(), d.date.month());
    assert_eq!(today.day(), d.date.day());
    assert_eq!(2, d.parts.len());

    let d = parse_day_line_from_legacy("10:00-11:30 # foo bar # world ").unwrap();
    assert_eq!(today.year(), d.date.year());
    assert_eq!(today.month(), d.date.month());
    assert_eq!(today.day(), d.date.day());
    assert_eq!(1, d.parts.len());
    assert_eq!(Some(String::from("foo bar # world")), d.comment);

    let d = parse_day_line_from_legacy("ddd 12:30-18:00 fobar");
    assert!(d.is_err());

    let d = parse_day_line_from_legacy("");
    assert!(d.is_err());

    let d = parse_day_line_from_legacy("#10:00-11:30 - 12:30-18:00");
    assert!(d.is_err());

    let d = parse_day_line_from_legacy("asdas");
    assert!(d.is_err());
}

#[test]
fn test_parse_part_from_legacy() {
    let span = "10:00-11:30";
    let e = parse_part_from_legacy(span);

    assert!(e.is_ok());
    let e = e.unwrap();

    assert_eq!(Some(1.0), e.factor);

    // start
    assert_eq!(10, e.start.hour());
    assert_eq!(00, e.start.minute());
    assert_eq!(00, e.start.second());

    assert!(e.stop.is_some());
    let stop = e.stop.unwrap();

    // stop
    assert_eq!(11, stop.hour());
    assert_eq!(30, stop.minute());
    assert_eq!(00, stop.second());

    let span = "10:00-11:30-0.5";
    let e = parse_part_from_legacy(span);
    assert!(e.is_ok());
    let e = e.unwrap();
    // start
    assert_eq!(10, e.start.hour());
    assert_eq!(00, e.start.minute());
    assert_eq!(00, e.start.second());
    assert_eq!(Some(0.5), e.factor);
}

#[test]
fn test_parse_time_from_str() {
    let r = parse_time_from_str("12:13");
    assert!(r.is_ok());
    let r = r.unwrap();
    assert_eq!(12, r.hour());
    assert_eq!(13, r.minute());

    let r = parse_time_from_str("1213");
    assert!(r.is_ok());
    let r = r.unwrap();
    assert_eq!(12, r.hour());
    assert_eq!(13, r.minute());

    let r = parse_time_from_str("2513");
    assert!(r.is_err());

    let r = parse_time_from_str("2461");
    assert!(r.is_err());
}
