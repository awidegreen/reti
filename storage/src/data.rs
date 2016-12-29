use rustc_serialize::json;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;

use chrono::*;

use parsing::*;

#[derive(RustcDecodable, RustcEncodable, PartialEq, Debug)]
pub struct Data {
   pub years: Vec<Year>,
   pub fee_per_hour: f32,
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Debug)]
pub struct Year {
   pub year: u16,
   pub days: Vec<Day>,
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Debug)]
pub struct Day {
    pub date: NaiveDate,
    pub parts: Vec<Part>,
    pub comment: Option<String>,
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Debug)]
pub struct Part {
    pub start: NaiveTime,
    pub stop: Option<NaiveTime>,
    pub factor: Option<f32>,
}

impl Year {
    fn new(year: u16) -> Year {
        Year { year: year, days: vec![] }
    }

    fn get_day(&self, m: u8, d: u8) -> Option<&Day> {
        self.days.iter().find(|ref day|
                                day.date.month() as u8 == m &&
                                day.date.day()   as u8 == d)
    }

    fn get_day_mut(&mut self, m: u8, d: u8) -> Option<&mut Day> {
        self.days.iter_mut().find(|ref mut day|
                                    day.date.month() as u8 == m &&
                                    day.date.day()   as u8 == d)
    }

    fn add_day(&mut self, day: Day) -> bool {
        let m = day.date.month() as u8;
        let d = day.date.day() as u8;
        let has_day = self.get_day(m, d).is_some();
        if !has_day {
            self.days.push(day);
        }
        !has_day
    }

    pub fn get_months(&self) -> Vec<Month> {
        let mut months = BTreeMap::<u32, Month>::new();

        for ref d in self.days.iter() {
            let m = d.date.month();
            let month = months.entry(m).or_insert_with(|| {
                    Month::new(vec![])
                });
            month.days.push(d);
        }

        months.values().cloned().collect()
    }
}

impl Day {
    fn new(day: NaiveDate) -> Day {
        Day { date: day, parts: vec![], comment: None }
    }

    pub fn new_today() -> Day {
        let today = UTC::today().naive_local();
        Day { date: today, parts: vec![], comment: None }
    }

    pub fn worked(&self) -> Duration {
        let mut d = Duration::zero();

        for p in &self.parts {
            d = d + match p.worked() {
                Some(w) => w,
                None => continue
            };
            //if !p.stop.is_some() { continue }
            //d = d + (p.stop.unwrap() - p.start);
        }
        d
    }

    pub fn earned(&self, fee: f32) -> f32 {
        let mut result: f32 = 0.0;

        for p in &self.parts {
            let factor = match p.factor {
                Some(f) => f,
                _ => 1.0
            };
            result = result + match p.worked() {
                Some(worked) => (worked.num_minutes() as f32/60.0) * (factor*fee),
                _ => continue
            };
        }
        result
    }

    fn does_intersect(&self, part: &Part) -> bool{
        for p in &self.parts {
            if p.stop.is_none() || part.stop.is_none() { return true}

            if part.start > p.start {
                if p.stop.unwrap() > part.start {
                    return true
                }
            } else {
                if part.stop.unwrap() > p.start {
                    return true
                }
            }
        }
        return false
    }

    fn clear_parts(&mut self) {
        self.parts.clear();
    }

    fn merge_day(&mut self, other: Day) -> bool {
        if other.parts.len() == 0 {
            println!("No parts to merge!");
            return false;
        }

        if self.date != other.date {
            println!("Nothing to do, not the same day! Self: {} - other: {}",
                     self.date, other.date);
            return false
        }

        let len = self.parts.len();
        for other_part in other.parts.into_iter() {
            if !self.does_intersect(&other_part) {
                self.parts.push(other_part);
            } else {
                println!("{:?} does clash with existing part in day: {}",
                        &other_part, self.date);
            }
        }
        if self.comment.is_none() && other.comment.is_some() {
            self.comment = other.comment
        }
        len != self.parts.len()
    }

    pub fn as_legacy(&self) -> String {
        format!("{}   {}{}",
                self.date.format("%Y-%m-%d").to_string(),
                self.parts.iter().map(|x| x.as_legacy()).collect::<Vec<String>>().join("  "),
                match self.comment {
                    Some(ref c) => format!("   # {}", c),
                    None => "".to_string()
                }
                )
    }
}

pub struct Week<'a> {
    pub days: Vec<&'a Day>,
}

impl<'a> Week<'a> {
    fn new(days: Vec<&'a Day>) -> Week<'a> {
        Week { days: days }
    }

    pub fn earned(&self, fee: f32) -> f32 {
        let mut r = 0.0_f32;
        for day in &self.days {
            r = r + day.earned(fee);
        }
        r
    }

    pub fn worked(&self) -> Duration {
        let mut d = Duration::zero();

        for day in &self.days {
            d = d + day.worked()
        }
        d
    }

    pub fn as_num(&self) -> String {
        assert!(self.days.len() > 0);
        self.days[0].date.format("%W").to_string()
    }
}

//#[derive(RustcDecodable, Clone)]
#[derive(Clone)]
pub struct Month<'a> {
    pub days: Vec<&'a Day>,
}

impl<'a> Month<'a> {
    fn new(days: Vec<&'a Day>) -> Month<'a> {
        Month {  days: days }
    }

    pub fn worked(&self) -> Duration {
        let mut d = Duration::zero();

        for day in &self.days {
            d = d + day.worked()
        }
        d
    }

    pub fn earned(&self, fee: f32) -> f32 {
        let mut r = 0.0_f32;
        for day in &self.days {
            r = r + day.earned(fee);
        }
        r
    }

    pub fn as_num(&self) -> String {
        assert!(self.days.len() > 0);
        self.days[0].date.format("%m").to_string()
    }

    pub fn as_name(&self) -> String {
        assert!(self.days.len() > 0);
        self.days[0].date.format("%B").to_string()
    }
}

impl Part {
    fn as_legacy(&self) -> String {
        format!("{}-{}-{}",
                self.start.format("%H:%M").to_string(),
                match self.stop {
                    Some(x) => x.format("%H:%M").to_string(),
                    _ => "".to_string(),
                },
                self.factor.unwrap_or(1.0)
               )
    }

    fn worked(&self) -> Option<Duration> {
        if !self.stop.is_some() { return None }
        Some(self.stop.unwrap() - self.start)
    }
}

pub struct Storage {
    data : Data,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            data : Data {
                years: vec![],
                fee_per_hour: 0.0,
            },
        }
    }

    pub fn from_file(file: &str) -> DataResult<Storage> {
        let mut file = File::open(file).unwrap();
        let mut s = String::new();
        let _ = file.read_to_string(&mut s).unwrap();

        let data = try!(json::decode(&s));
        Ok(Storage {
            data: data
        })
    }

    pub fn set_fee(&mut self, fee: f32) {
        self.data.fee_per_hour = fee
    }

    pub fn get_fee(&self) -> f32 {
        self.data.fee_per_hour
    }

    pub fn import_legacy(&mut self, file: &str) -> bool {
        let f = match File::open(file) {
            Ok(x) => x,
            _ => return false
        };
        let f = BufReader::new(f);

        for line in f.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    println!("Unable to process line: {}", e);
                    continue
                }
            };
            match parse_day_line_from_legacy(&line) {
                Ok(day) => { self.add_day(day); },
                Err(e) =>
                    println!("Unable to parse line: '{}', Err: {:?}", line, e),
            }
        };

        true
    }

    pub fn save(&self, file: &str, readable: bool) -> bool {
        let mut f = File::create(file).unwrap();

        if readable {
            let encoded = json::as_pretty_json(&self.data);
            return write!(f, "{}", encoded).is_ok()
        } else {
            let encoded = json::as_json(&self.data);
            return write!(f, "{}", encoded).is_ok()
        }
    }

    pub fn get_week(&self, y: u16, w: u32) -> Option<Week> {
        if let Some(year) = self.get_year(y) {
            let days : Vec<&Day> = year.days.iter()
                .filter(|&x| x.date.isoweekdate().1 == w)
                .collect();
            if days.len() > 0 {
                return Some(Week::new(days))
            }
        }
        None
    }

    pub fn get_month(&self, y: u16,  m: u8) -> Option<Month> {
        if let Some(year) = self.get_year(y) {
            let days : Vec<&Day> = year.days.iter()
                .filter(|&x| x.date.month() == u32::from(m))
                .collect();
            if days.len() > 0 {
                return Some(Month::new(days))
            }
        }
        None
    }

    pub fn get_day(&self, y: u16, m: u8, d: u8) -> Option<&Day> {
        if let Some(month) = self.get_month(y, m) {
            if let Some(day) = month.days.iter().find(|&&x| x.date.day() == d as u32) {
                return Some(day)
            }
        }
       None
    }

    /// Removes a day from the store based chrono::NaiveDate
    pub fn remove_day_nd(&mut self, date: NaiveDate) -> bool {
        if let Some(mut year) = self.get_year_mut(date.year() as u16) {
            let size = year.days.len();
            year.days.retain(|x| x.date != date);
            return size > year.days.len();
        }
        false
    }

    pub fn get_year(&self, y: u16) -> Option<&Year> {
        self.data.years
            .iter()
            .find(|&x| x.year == y)
    }

    fn get_year_mut(&mut self, y: u16) -> Option<&mut Year>{
        self.data.years.iter_mut().find(|ref mut year| year.year == y)
    }

    pub fn add_part(&mut self, date: NaiveDate, part: Part) -> bool {
        if part.stop.is_some() && part.stop.unwrap() < part.start {
            println!("Well, did you stopped working before you started?");
            return false;
        }

        let y = date.year() as u16;
        let m = date.month() as u8;
        let d = date.day() as u8;
        let mut new_day = Day::new(date);

        new_day.parts.push(part);

        if let Some(year) = self.get_year_mut(y) {
            if let Some(day) = year.get_day_mut(m, d) {
                return day.merge_day(new_day);
            }
            return year.add_day(new_day);
        }
        let mut year = Year::new(y);
        return year.add_day(new_day)
    }

    pub fn add_day(&mut self, day: Day) -> bool {
        let y = day.date.year() as u16;
        let m = day.date.month() as u8;
        let d = day.date.day() as u8;

        if let Some(year) = self.get_year_mut(y) {
            if let Some(existing_day) = year.get_day_mut(m, d) {
                return existing_day.merge_day(day)
            }
            return year.add_day(day)
        }
        let mut year = Year::new(y);
        year.add_day(day);
        self.data.years.push(year);
        true
    }

    pub fn add_day_force(&mut self, day: Day) -> bool {
        let y = day.date.year() as u16;
        let m = day.date.month() as u8;
        let d = day.date.day() as u8;

        if day.parts.len() == 0 {
            println!("No parts specified!");
            return false
        }

        if let Some(year) = self.get_year_mut(y) {
            if let Some(existing_day) = year.get_day_mut(m, d) {
                existing_day.clear_parts();
                existing_day.comment = None;
                return existing_day.merge_day(day)
            }
            return year.add_day(day)
        }
        let mut year = Year::new(y);
        year.add_day(day)
    }
}

#[test]
fn test_day_worked() {
    // test only one part!
    let l = String::from("05-23     10:00-11:30");
    let d = parse_day_line_from_legacy(&l).unwrap();

    let worked = d.worked();
    assert_eq!(worked.num_minutes(), 90);

    let l = String::from("05-23     10:00-11:30 --- 13:00-18:00");
    let d = parse_day_line_from_legacy(&l).unwrap();

    let worked = d.worked();
    assert_eq!(worked.num_minutes(), 90 + 300);
}

#[test]
fn test_day_earned() {
    let fee = 100_f32;
    let l = String::from("05-23     10:00-12:00");
    let d = parse_day_line_from_legacy(&l).unwrap();

    let earned = d.earned(fee);
    assert_eq!(200_f32, earned);

    let l = String::from("05-24     10:00-11:00-0.5 --- 13:00-14:00-2.0");
    let d = parse_day_line_from_legacy(&l).unwrap();

    let earned = d.earned(fee);
    assert_eq!(250_f32, earned);
}

#[test]
fn test_day_does_intersect() {
    let l = String::from("05-23     08:00-12:00");
    let day = parse_day_line_from_legacy(&l).unwrap();

    let part = parse_part_from_legacy("0600-0700").unwrap();
    assert!(!day.does_intersect(&part));
    let part = parse_part_from_legacy("0600-0800").unwrap();
    assert!(!day.does_intersect(&part));
    let part = parse_part_from_legacy("0600-0900").unwrap();
    assert!(day.does_intersect(&part));
    let part = parse_part_from_legacy("0800-0900").unwrap();
    assert!(day.does_intersect(&part));
    let part = parse_part_from_legacy("0900-1100").unwrap();
    assert!(day.does_intersect(&part));
    let part = parse_part_from_legacy("0900-1200").unwrap();
    assert!(day.does_intersect(&part));
    let part = parse_part_from_legacy("1200-1400").unwrap();
    assert!(!day.does_intersect(&part));
    let part = parse_part_from_legacy("1300-1400").unwrap();
    assert!(!day.does_intersect(&part));
}

#[test]
fn test_day_merge_day() {
    let l = String::from("05-23     08:00-12:00");
    let mut day = parse_day_line_from_legacy(&l).unwrap();

    assert_eq!(1, day.parts.len());

    // not the same day!
    let l = String::from("08-05     14:00-16:00");
    let other = parse_day_line_from_legacy(&l).unwrap();
    assert!(!day.merge_day(other));
    assert_eq!(1, day.parts.len());

    // does intersect!
    let l = String::from("05-23     09:00-10:00");
    let other = parse_day_line_from_legacy(&l).unwrap();
    assert!(!day.merge_day(other));
    assert_eq!(1, day.parts.len());

    // shall work
    let l = String::from("05-23     14:00-15:00");
    let other = parse_day_line_from_legacy(&l).unwrap();
    assert!(day.merge_day(other));
    assert_eq!(2, day.parts.len());
}

#[test]
fn test_year_add_day() {
    let mut year = Year::new(2016);

    let l = String::from("05-25     08:00-12:00");
    let day = parse_day_line_from_legacy(&l).unwrap();
    assert!(year.add_day(day));
    assert_eq!(1, year.days.len());

    let l = String::from("05-23     08:00-12:00");
    let day = parse_day_line_from_legacy(&l).unwrap();
    assert!(year.add_day(day));
    assert_eq!(2, year.days.len());

    // should not work
    let l = String::from("05-23     13:00-15:00");
    let day = parse_day_line_from_legacy(&l).unwrap();
    assert!(!year.add_day(day));
    assert_eq!(2, year.days.len());
}

#[test]
fn test_year_get_day_mut() {
    let mut year = Year::new(2016);

    let l = String::from("05-25     08:00-12:00");
    let day = parse_day_line_from_legacy(&l).unwrap();
    year.add_day(day);

    assert!(year.get_day_mut(5, 1).is_none());
    let day = year.get_day_mut(5, 25);
    assert!(day.is_some());

    let day = day.unwrap();
    assert_eq!(25, day.date.day());
    assert_eq!(5, day.date.month());
}

