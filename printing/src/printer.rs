use chrono::Duration;
use reti_storage::data;
use std::collections::HashMap;
use std::fmt;

pub struct Printer<'a> {
    years: Vec<&'a data::Year>,
    months: Vec<data::Month<'a>>,
    weeks: Vec<data::Week<'a>>,
    days: Vec<&'a data::Day>,

    worked: bool,
    breaks: bool,
    verbose: bool,
    show_days: bool,
    parts: bool,
    fee: f32,
}

impl<'a> Printer<'a> {
    pub fn with_years(years: Vec<&'a data::Year>) -> Printer<'a> {
        Printer {
            years,
            months: vec![],
            weeks: vec![],
            days: vec![],
            worked: false,
            breaks: false,
            verbose: false,
            show_days: false,
            parts: false,
            fee: 0.0,
        }
    }

    pub fn with_months(months: Vec<data::Month<'a>>) -> Printer<'a> {
        Printer {
            years: vec![],
            months,
            weeks: vec![],
            days: vec![],
            worked: false,
            breaks: false,
            verbose: false,
            show_days: false,
            parts: false,
            fee: 0.0,
        }
    }

    pub fn with_weeks(weeks: Vec<data::Week<'a>>) -> Printer<'a> {
        Printer {
            years: vec![],
            months: vec![],
            weeks,
            days: vec![],
            worked: false,
            breaks: false,
            verbose: false,
            show_days: false,
            parts: false,
            fee: 0.0,
        }
    }

    pub fn with_days(days: Vec<&'a data::Day>) -> Printer<'a> {
        Printer {
            years: vec![],
            months: vec![],
            weeks: vec![],
            days,
            worked: false,
            breaks: false,
            verbose: false,
            show_days: false,
            parts: false,
            fee: 0.0,
        }
    }

    pub fn set_fee(mut self, val: f32) -> Self {
        self.fee = val;
        self
    }

    pub fn show_worked(mut self, val: bool) -> Self {
        self.worked = val;
        self
    }
    pub fn show_breaks(mut self, val: bool) -> Self {
        self.breaks = val;
        self
    }
    pub fn show_days(mut self, val: bool) -> Self {
        self.show_days = val;
        self
    }
    pub fn show_verbose(mut self, val: bool) -> Self {
        self.verbose = val;
        self
    }
    pub fn show_parts(mut self, val: bool) -> Self {
        self.parts = val;
        self
    }

    //pub fn print(&self) {
    //writeln!(
    //&mut std::io::stdout(),
    //"Assumed fee per hour: {:.2}",
    //self.fee
    //);
    //if self.years.len() > 0 {
    //self.fmt_years(&mut std::io::stdout(), &self.years);
    //}
    //if self.months.len() > 0 {
    //self.fmt_months(&self.months);
    //}
    //if self.weeks.len() > 0 {
    //self.fmt_weeks(&self.weeks);
    //}
    //if self.days.len() > 0 {
    //self.fmt_days(&self.days);
    //}
    //}

    fn fmt_day(&self, f: &mut fmt::Formatter, day: &'a data::Day) -> fmt::Result {
        write!(f, "  {}", day.date)?;
        if self.verbose {
            write!(f, " ({})", day.date.format("%a"))?;
        }
        if self.worked {
            let w = day.worked().num_minutes() as f64 / 60.0;
            write!(f, " worked: {: >5.2}h", w)?;
        }
        if self.breaks {
            //let b = 0;
            write!(f, " TODO!!! breaks: FOOBARh")?;
        }
        if self.parts {
            let s = &day
                .parts
                .iter()
                .map(|ref x| {
                    let s = format!(
                        "{}-{} f: {:.1}",
                        x.start.format("%H:%M"),
                        x.stop.unwrap().format("%H:%M"),
                        x.factor.unwrap_or(1.0)
                    );
                    s
                })
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, " ({})", s)?;
        }
        if self.verbose {
            write!(f, " ({} parts)", day.parts.len())?;
            write!(f, " earned: {:.2}", day.earned(self.fee))?;
        }
        if let Some(ref c) = day.comment {
            writeln!(f, "  ({})", &c)
        } else {
            writeln!(f)
        }
    }

    fn fmt_days(&self, f: &mut fmt::Formatter, days: &[&'a data::Day]) -> fmt::Result {
        for d in days {
            self.fmt_day(f, d)?
        }
        Ok(())
    }

    fn fmt_week(&self, f: &mut fmt::Formatter, week: &'a data::Week) -> fmt::Result {
        writeln!(f, "Week: {}", week.as_num())?;

        if self.verbose {
            writeln!(f, "Days recorded: {}", week.days.len())?;
        }

        if self.show_days {
            self.fmt_days(f, &week.days)?;
        }

        if self.worked {
            let w = week.worked().num_minutes() as f64 / 60.0;
            writeln!(f, "total worked: {:.2}h", w)?;
            writeln!(
                f,
                "avg worked per day: {:.2}h/day",
                w / week.days.len() as f64
            )?;
        }
        if self.breaks {
            writeln!(f, "The breaks shall go here!")?;
        }

        if self.verbose {
            writeln!(f, "total earned: {:.2}", week.earned(self.fee))?;
        }

        Ok(())
    }

    fn fmt_weeks(&self, f: &mut fmt::Formatter, weeks: &[data::Week]) -> fmt::Result {
        for week in weeks {
            self.fmt_week(f, week)?;
        }
        Ok(())
    }

    fn fmt_month(&self, f: &mut fmt::Formatter, month: &'a data::Month) -> fmt::Result {
        writeln!(f, "Month: {} - {}", month.as_num(), month.as_name())?;

        if self.verbose {
            writeln!(f, "Days recorded: {}", month.days.len())?;
        }

        if self.show_days {
            self.fmt_days(f, &month.days)?;
        }

        if self.worked {
            let w = month.worked().num_minutes() as f64 / 60.0;
            writeln!(f, "total worked: {:.2}h", w)?;
            writeln!(
                f,
                "avg worked per day: {:.2}h/day",
                w / month.days.len() as f64
            )?;
        }
        if self.breaks {
            writeln!(f, "The breaks shall go here!")?
        }

        if self.verbose {
            let mut times_fac: HashMap<usize, Duration> = HashMap::new();
            for d in &month.days {
                for ref p in &d.parts {
                    let f = (p.factor.unwrap_or(1.0) * 10.0) as usize;
                    if let Some(worked) = p.worked() {
                        if let Some(x) = times_fac.get_mut(&f) {
                            *x = *x + worked;
                        } else {
                            times_fac.insert(f, worked);
                        }
                    }
                }
            }

            for (k, v) in &times_fac {
                let w = v.num_minutes() as f64 / 60.0;
                writeln!(f, "Worked factor {:.1}: {:.2}h", (*k as f32 / 10.0), w)?
            }

            writeln!(f, "total earned: {:.2}", month.earned(self.fee))?
        }

        Ok(())
    }

    fn fmt_months(&self, f: &mut fmt::Formatter, months: &[data::Month]) -> fmt::Result {
        for month in months {
            self.fmt_month(f, month)?;
        }
        Ok(())
    }

    fn fmt_year(&self, f: &mut fmt::Formatter, year: &'a data::Year) -> fmt::Result {
        let months = year.get_months();
        writeln!(f, "Year: {}, {} month(s) recorded", year.year, months.len())?;

        let mut worked: f64 = 0.0;
        let mut earned: f64 = 0.0;
        for m in months {
            worked += m.worked().num_minutes() as f64 / 60.0;
            earned += m.earned(self.fee) as f64;
            self.fmt_month(f, &m)?;
            writeln!(f, "-------")?;
        }
        if self.worked {
            writeln!(
                f,
                "Accumulated worked: {:.2}h - earned: {:.2}",
                worked, earned
            )?;
        }
        Ok(())
    }

    fn fmt_years(&self, f: &mut fmt::Formatter, years: &[&'a data::Year]) -> fmt::Result {
        for year in years {
            self.fmt_year(f, year)?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for Printer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Assumed fee per hour: {:.2}", self.fee)?;
        if !self.years.is_empty() {
            self.fmt_years(f, &self.years)?;
        }
        if !self.months.is_empty() {
            self.fmt_months(f, &self.months)?;
        }
        if !self.weeks.is_empty() {
            self.fmt_weeks(f, &self.weeks)?;
        }
        if !self.days.is_empty() {
            self.fmt_days(f, &self.days)?;
        }
        Ok(())
    }
}
