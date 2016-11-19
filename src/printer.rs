use data;

pub struct Printer<'a> {
    years:  Vec<&'a data::Year>,
    months: Vec<data::Month<'a>>,
    weeks:  Vec<data::Week<'a>>,
    days:   Vec<&'a data::Day>,

    worked:    bool,
    breaks:    bool,
    verbose:   bool,
    show_days: bool,
    parts:     bool,
    fee:       f32,
}

impl<'a> Printer<'a> {
    pub fn with_years(years: Vec<&'a data::Year>) -> Printer<'a> {
        Printer {
            years: years,
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
            months: months,
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
            weeks: weeks,
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
            days: days,
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

    pub fn print(&self) {
        println!("Assumed fee per hour: {:.2}", self.fee);
        if self.years.len() > 0 {
            self.print_years(&self.years);
        }
        if self.months.len() > 0 {
            self.print_months(&self.months);
        }
        if self.weeks.len() > 0 {
            self.print_weeks(&self.weeks);
        }
        if self.days.len() > 0 {
            self.print_days(&self.days);
        }
    }

    fn print_day(&self, day: &'a data::Day) {
        print!("  {}", day.date);
        if self.verbose {
            print!(" ({})", day.date.format("%a"));
        }
        if self.worked {
            let w = day.worked().num_minutes() as f64/60.0;
            print!(" worked: {: >5.2}h", w);
        }
        if self.breaks {
            //let b = 0;
            print!(" breaks: {}h", "TODO!");
        }
        if self.parts {
            let s = &day.parts
                .iter()
                .map(|ref x| {
                    let s = format!("{}-{} f: {:.1}",
                            x.start.format("%H:%M"),
                            x.stop.unwrap().format("%H:%M"),
                            x.factor.unwrap_or(1.0));
                    s
                })
                .collect::<Vec<String>>()
                .join(", ");
            print!(" ({})", s);
        }
        if self.verbose {
            print!(" ({} parts)", day.parts.len());
            print!(" earned: {:.2}", day.earned(self.fee));
        }
        if let Some(ref c) = day.comment {
             println!("  ({})", &c);
        } else { println!(""); }
    }

    fn print_days(&self, days: &Vec<&'a data::Day>) {
        for d in days {
            self.print_day(d);
        }
    }

    fn print_week(&self, week: &'a data::Week) {
        println!("Week: {}", week.as_num());

        if self.verbose {
            println!("Days recorded: {}", week.days.len());
        }

        if self.show_days {
            self.print_days(&week.days);
        }

        if self.worked {
            let w = week.worked().num_minutes() as f64/60.0;
            println!("total worked: {:.2}h", w);
            println!("avg worked per day: {:.2}h/day", w/week.days.len() as f64);
        }
        if self.breaks {
            println!("The breaks shall go here!");
        }

        if self.verbose {
            println!("total earned: {:.2}", week.earned(self.fee));
        }
    }

    fn print_weeks(&self, weeks: &Vec<data::Week>) {
        for week in weeks {
            self.print_week(week);
        }
    }

    fn print_month(&self, month: &'a data::Month) {
        println!("Month: {} - {}", month.as_num(), month.as_name());

        if self.verbose {
            println!("Days recorded: {}", month.days.len());
        }

        if self.show_days {
            self.print_days(&month.days);
        }

        if self.worked {
            let w = month.worked().num_minutes() as f64/60.0;
            println!("total worked: {:.2}h", w);
            println!("avg worked per day: {:.2}h/day", w/month.days.len() as f64);
        }
        if self.breaks {
            println!("The breaks shall go here!");
        }

        if self.verbose {
            println!("total earned: {:.2}", month.earned(self.fee));
        }
    }

    fn print_months(&self, months: &Vec<data::Month>) {
        for month in months {
            self.print_month(month);
        }
    }

    fn print_year(&self, year: &'a data::Year) {
        let months = year.get_months();
        println!("Year: {}, {} month(s) recorded", year.year, months.len());

        let mut worked : f64 = 0.0;
        let mut earned : f64 = 0.0;
        for m in months {
            worked += m.worked().num_minutes() as f64/60.0;
            earned += m.earned(self.fee) as f64;
            self.print_month(&m);
            println!("-------");
        }
        if self.worked {
            println!("Accumulated worked: {:.2}h - earned: {:.2}", worked, earned);
        }
    }

    fn print_years(&self, years: &Vec<&'a data::Year>) {
        for year in years {
            self.print_year(year)
        }
    }
}
