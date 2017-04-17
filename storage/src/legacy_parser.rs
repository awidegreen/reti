use nom::{digit, multispace, not_line_ending, IResult};
use chrono::{NaiveDate, NaiveTime};

use data::{Part, Day};
use std;

#[derive(Debug,PartialEq,Eq,Clone)]
pub enum ParserError {
    IgnoreLine,
    DayParseError,
    EmptyLine
}

//impl From<ErrorKind> for ParserError {
    //fn from(err: ErrorKind) -> ParserError {
        //ParserError::DayParseError(err)
    //}
//}

named!(number<u16>,
    map_res!(
        map_res!(
            digit,
            std::str::from_utf8
        ),
        std::str::FromStr::from_str
    )
);


named!(date<NaiveDate>,
    do_parse!(
        y: number >>
        tag!("-") >>
        m: number >>
        tag!("-") >>
        d: number >>
        (NaiveDate::from_ymd(y as i32, m as u32, d as u32))
    )
);

named!(time<NaiveTime>,
    do_parse!(
        h: number >>
        tag!(":") >>
        m: number >>
        (NaiveTime::from_hms(h as u32, m as u32, 0))
    )
);


named!(comment<String>,
    do_parse!(
        tag!("#") >>
        text: map_res!(
            not_line_ending,
            std::str::from_utf8
        ) >>
        (text.trim().to_string())
    )
);

named!(unsigned_float <f32>, map_res!(
    map_res!(
        recognize!(
            alt_complete!(
                delimited!(opt!(digit), tag!("."), digit) |
                preceded!(digit, tag!(".")) |
                digit
            )
        ),
        std::str::from_utf8
    ),
    std::str::FromStr::from_str
));

named!(factor<f32>,
    do_parse!(
        tag!("-") >>
        f: unsigned_float >>
        (f)
    )
);

named!(part<Part>,
    do_parse!(
        start: time     >>
        tag!("-")       >>
        stop:  time     >>
        factor: opt!(complete!(factor)) >>
        (Part {
            start: start,
            stop: Some(stop),
            factor: factor
        })
    )
);

named!(parts<Vec<Part>>,
    map!(
        separated_list!(multispace, part),
        |vec: Vec<_>| vec.into_iter().collect()
    )
);

named!(day<Day>,
    do_parse!(
        d: date  >>
        multispace >>
        p: parts >>
        opt!(complete!(multispace)) >>
        c: opt!(complete!(comment)) >>
        (Day {
            date: d,
            parts: p,
            comment: c
        })
    )
);


pub fn parse_line(line: &str) -> Result<Day, ParserError> {
    if line.len() == 0 {
        return Err(ParserError::EmptyLine)
    }

    if comment(line.as_bytes()).to_result().is_ok() {
        return Err(ParserError::IgnoreLine)
    }

    match day(line.as_bytes()) {
        IResult::Done(_, d) => Ok(d),
        IResult::Error(_) => {
            //println!("Error: {:?}", e);
            Err(ParserError::DayParseError)
        },
        IResult::Incomplete(_) => {
            //println!("Incomplete: {:?}", e);
            Err(ParserError::DayParseError)
        }
    }
}

pub fn parse_date(ymd: &str) -> Option<NaiveDate> {
    match date(ymd.as_bytes()) {
        IResult::Done(_,o) => Some(o),
        _ => None
    }
}

pub fn parse_time(t: &str) -> Option<NaiveTime> {
    match time(t.as_bytes()) {
        IResult::Done(_,o) => Some(o),
        _ => None
    }
}

pub fn parse_part(t: &str) -> Option<Part> {
    match part(t.as_bytes()) {
        IResult::Done(_,o) => Some(o),
        _ => None
    }
}


#[cfg(test)]
mod test {
    use chrono::{NaiveTime,NaiveDate};
    use nom::IResult;
    use data::{Part, Day};

    #[test]
    fn test_parse_comment() {
        let r =  super::comment("#foo bar\n".as_bytes());
        println!("{:?}", r);
        assert_eq!(r, IResult::Done(&b"\n"[..], "foo bar".to_string()));
        assert_eq!(
            super::comment("# foo bar  \n".as_bytes()),
                IResult::Done(&b"\n"[..], "foo bar".to_string()));
        assert_eq!(
            super::comment("# foo bar  ".as_bytes()),
            IResult::Done(&b""[..], "foo bar".to_string()));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(
            super::number("2017".as_bytes()), IResult::Done(&b""[..], 2017));
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(super::date("2017-03-21".as_bytes()),
                IResult::Done(&b""[..], NaiveDate::from_ymd(2017, 3, 21)));
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(super::time("16:43".as_bytes()),
                IResult::Done(&b""[..], NaiveTime::from_hms(16, 43, 0)));
    }

    #[test]
    fn test_parse_factor() {
        assert_eq!(super::factor("-1.0".as_bytes()),
                IResult::Done(&b""[..], 1.0));
        assert_eq!(super::factor("-1".as_bytes()),
                IResult::Done(&b""[..], 1.0));
    }

    #[test]
    fn test_parse_part_with_factor() {
        let exp_part = Part {
            start: NaiveTime::from_hms(8,0,0),
            stop:  Some(NaiveTime::from_hms(11, 30, 0)),
            factor: Some(1.0)
        };
        let r = super::part("08:00-11:30-1".as_bytes());
        assert_eq!(r, IResult::Done(&b""[..], exp_part));
    }

    #[test]
    fn test_parse_part_no_factor() {
        let exp_part = Part {
            start: NaiveTime::from_hms(8,0,0),
            stop:  Some(NaiveTime::from_hms(11, 30, 0)),
            factor: None
        };
        let r = super::part("08:00-11:30".as_bytes());
        assert_eq!(r, IResult::Done(&b""[..], exp_part));
    }

    #[test]
    fn test_parse_parts() {

        let p1 = Part {
            start: NaiveTime::from_hms(8,0,0),
            stop:  Some(NaiveTime::from_hms(11, 30, 0)),
            factor: Some(2.0)
        };
        let p2 = Part {
            start: NaiveTime::from_hms(12,30,0),
            stop:  Some(NaiveTime::from_hms(17, 59, 0)),
            factor: None
        };

        let exp_parts = vec![p1, p2];

        let r = super::parts("08:00-11:30-2   12:30-17:59".as_bytes());
        assert_eq!(r, IResult::Done(&b""[..], exp_parts));
    }

    #[test]
    fn test_parse_day() {

        let p1 = Part {
            start: NaiveTime::from_hms(8,0,0),
            stop:  Some(NaiveTime::from_hms(11, 30, 0)),
            factor: Some(2.0)
        };
        let p2 = Part {
            start: NaiveTime::from_hms(12,30,0),
            stop:  Some(NaiveTime::from_hms(17, 59, 0)),
            factor: None
        };

        let exp_parts = vec![p1, p2];

        let exp_day = Day {
            date: NaiveDate::from_ymd(2017, 03, 20),
            parts: exp_parts,
            comment: Some("foo bar".to_string())
        };
        let r =
            super::day("2017-03-20    08:00-11:30-2 12:30-17:59  #foo bar".as_bytes());
        println!("{:?}", r);
        assert_eq!(r, IResult::Done(&b""[..], exp_day));
    }

    #[test]
    fn test_parse_line() {

        let p1 = Part {
            start: NaiveTime::from_hms(8,0,0),
            stop:  Some(NaiveTime::from_hms(11, 30, 0)),
            factor: None
        };
        let p2 = Part {
            start: NaiveTime::from_hms(12,30,0),
            stop:  Some(NaiveTime::from_hms(17, 59, 0)),
            factor: None
        };

        let exp_parts = vec![p1, p2];

        let exp_day = Day {
            date: NaiveDate::from_ymd(2017, 03, 20),
            parts: exp_parts,
            comment: None
        };
        let r =
            super::day("2017-03-20    08:00-11:30  12:30-17:59".as_bytes());
        println!("{:?}", r);
        assert_eq!(r, IResult::Done(&b""[..], exp_day));
    }
}
