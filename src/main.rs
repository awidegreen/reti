extern crate chrono;
extern crate rustc_serialize;
extern crate tempfile;

#[macro_use]
extern crate clap;

mod printer;
mod data;
mod parsing;

use chrono::*;
use clap::{App, SubCommand, ArgMatches};
use std::io::BufReader;
use std::io::prelude::*;
use std::process::{Command, exit};
use std::env;

fn get_args<'a>() -> ArgMatches<'a> {
    App::new("reti")
        .about("The cli time reporting tool")
        .author("Armin Widegreen <armin.widegreen@gmail.com>")
        .version(crate_version!())
        .args_from_usage(
            "--save-pretty 'Save the json file read-able if a subcommand executes save.'
            -f, --file=[FILE] 'The file where the data is stored.'"
            )
        .subcommand(SubCommand::with_name("init")
                    .about("Initializes a new storage file. CAUTION: \
                           this will overwrite existing data!")
                    .args_from_usage(
                        "<storage_file> 'Which file shall be written!'
                        <legacy_file> 'import data from the legacy file!'"))
        .subcommand(SubCommand::with_name("import")
                    .about("Import from legacy representation. Parts with intersecting times will be disregarded.")
                    .args_from_usage("<legacy_file> 'file with legacy format!'"))
        .subcommand(SubCommand::with_name("get")
                    .about("Gets attributes for the current store.")
                    .subcommand(SubCommand::with_name("fee")
                                .about("Gets the fee per hour.")
                                .args_from_usage(""))
                    )
        .subcommand(SubCommand::with_name("set")
                    .about("Sets ttributes for the current store.")
                    .subcommand(SubCommand::with_name("fee")
                                .about("Sets the fee per hour which is the base for all parts!")
                                .args_from_usage("<value> 'The fee value as f32.'"))
                    )
        .subcommand(SubCommand::with_name("add")
                    .about("Everything related to add data to the store.")
                    .subcommand(SubCommand::with_name("part")
                                .about("Add a part of the day. A part can \
                                       consits only of the starting point, \
                                       therefore 'stop' is optional. If only the \
                                       stop should be recored use '_' for start")
                                .args_from_usage(
                                    "<start> 'The format is: HH:MM (default: now), use _ if only stop shall be recorded!'
                                    [stop] 'Format: HH:MM, optional hence only start will be recored'"
                                    ))
                    .subcommand(SubCommand::with_name("parse")
                                .about("Lets you add entries based on free text, either as parameter or via stdin.")
                                .args_from_usage(
                                    "<data>... 'The data that will be attempt to be parsed and added to the store.'"
                                    ))
                    )
        .subcommand(SubCommand::with_name("edit")
                    .about("Edit a specific day.").
                    args_from_usage("[date] 'can have the format: [yyyy-][mm-]dd, \
                                    if year or year and month are missing, current will be assumed.'"))
        .subcommand(SubCommand::with_name("show")
                    .about("Show recorded times")
                    .args_from_usage(
                        "-w, --worked 'Show worked time'
                        -d, --days 'Show all days'
                        -b, --breaks 'Shows time of breaks'
                        -p. --parts 'Show all parts a of a day'
                        -v, --verbose"
                        )
                    .subcommand(SubCommand::with_name("year")
                                .args_from_usage(
                                    "[years]... 'Space separated list of years to show (default: current)'"
                                    ))
                    .subcommand(SubCommand::with_name("month")
                                .args_from_usage(
                                    "-y, --year [year] 'Specify a year (default: current)'
                                    [months]... 'Space separated list of month to show (default: current)'"
                                    ))
                    .subcommand(SubCommand::with_name("week")
                                .args_from_usage(
                                    "[weeks]... 'Space separated list of weeks to show (default: current)'"
                                    ))
                    .subcommand(SubCommand::with_name("day")
                                .args_from_usage(
                                    "-y, --year [year] 'Specify a year (default: current)'
                                    -m, --month [month] 'Specify a month (defautl: current)'
                                    [days]... 'Space separated list of days to show (default: today)'"
                                    )))
        .get_matches()
}

fn main() {
    let args = get_args();

    let pretty_json = args.is_present("save-pretty");

    if let Some(ref matches) = args.subcommand_matches("init") {
        subcmd_init(matches, pretty_json);
        return;
    }

    let storage_file = value_t!(args, "file", String).unwrap_or("times.json".to_string());
    println!("Use storage_file: {}", storage_file);

    let mut store = match data::Storage::from_file(&storage_file) {
        Ok(store) => store,
        Err(e) => {
            println!("{:?}", e);
            exit(-1);
        }
    };

    if let Some(ref matches) = args.subcommand_matches("show") {
        subcmd_show(&store, matches);
    }

    let mut do_write = false;
    if let Some(ref matches) = args.subcommand_matches("import") {
        subcmd_import(&mut store, matches);
        do_write = true;
    }

    if let Some(ref matches) = args.subcommand_matches("get") {
        subcmd_get(&mut store, matches)
    }

    if let Some(ref matches) = args.subcommand_matches("set") {
        if subcmd_set(&mut store, matches) {
            do_write = true;
        } else {
            println!("Setting did not succeed, nothing will be saved!");
        }
    }

    if let Some(ref matches) = args.subcommand_matches("add") {
        if subcmd_add(&mut store, matches) {
            do_write = true;
        } else {
            println!("Add did not work, nothing will be saved!");
        }
    }

    if let Some(ref matches) = args.subcommand_matches("edit") {
        if subcmd_edit(&mut store, matches) {
            do_write = true;
        } else {
            println!("Edit canceled, nothing will be saved!");
        }
    }


    if do_write {
        if !store.save(&storage_file, pretty_json) {
            println!("Unable to write file: {}", &storage_file);
        }
    }

}

fn subcmd_import(store: &mut data::Storage, matches: &ArgMatches) {
    let leg_file = value_t!(matches, "legacy_file", String)
        .unwrap_or_else(|e| e.exit());

    if !store.import_legacy(&leg_file) {
        println!("Unable to import data!")
    }
}

fn subcmd_edit(store: &mut data::Storage, matches: &ArgMatches) -> bool {
    let date = value_t!(matches, "date", String).unwrap_or(String::new());

    let s = if !date.is_empty() {
        let date = match parsing::parse_date(&date) {
            Ok(x) => x,
            _ => return false,
        };

        let day = match store.get_day(
            date.year() as u16,
            date.month() as u8,
            date.day() as u8) {
            Some(x) => x,
            _ => {
                println!("Unable to get date: {:?}", date);
                return false
            }
        };
        day.as_legacy()
    } else {
        String::from("# 2016-04-25   08:00-12:00  13:00-17:00")
    };

    let mut file = tempfile::NamedTempFile::new().unwrap();
    let _ = write!(file, "{}\n", &s);

    match Command::new(env::var("EDITOR").unwrap_or("vim".to_string()))
        .arg(file.path().to_str().unwrap()).status() {
        Err(e) => {
            println!("Error occured: '{:?}'", e);
            return false
        },
        Ok(x) => if !x.success() {
            println!("Editor exit was failure!");
            return false
        }
    }

    let mut f = BufReader::new(file.try_clone().unwrap());
    let _ = f.seek(std::io::SeekFrom::Start(0));

    for (i, line) in f.lines().enumerate() {
        let line = match line { Ok(l) => l, Err(_) => continue };
        if line.trim().is_empty() {
            println!("ignore empty line: {}", line);
            continue
        }

        let day = match parsing::parse_day_line_from_legacy(&line) {
            Ok(day) => day,
            Err(e) => {
                println!("ignore {}: '{}' {:?}", i, line, e);
                continue
            }
        };

        store.add_day_force(day);
    }

    return true
}

fn subcmd_get(store: &data::Storage, matches: &ArgMatches) {
    if let Some(ref _matches) = matches.subcommand_matches("fee") {
        println!("Current fee: {}", store.get_fee());
    }
}

fn subcmd_set(store: &mut data::Storage, matches: &ArgMatches) -> bool {
    if let Some(ref matches) = matches.subcommand_matches("fee") {
        let fee = value_t!(matches, "value", f32).unwrap_or_else(|e| e.exit());
        store.set_fee(fee);
        return true
    }
    return false
}

fn subcmd_add(store: &mut data::Storage, matches: &ArgMatches) -> bool {
    if let Some(ref matches) = matches.subcommand_matches("part") {
        let start = value_t!(matches, "start", String).unwrap_or_else(|e| e.exit());
        let start = parsing::parse_time_from_str(&start);
        if start.is_err() {
            println!("Unable to parse start as time: format HH:MM or HHMM");
            return false;
        }
        let start = start.unwrap();
        let mut part = data::Part { start: start, stop: None, factor: None};

        if let Ok(stop) = value_t!(matches, "stop",  String) {
            if let Ok(stop) = parsing::parse_time_from_str(&stop) {
                part.stop = Some(stop);
            } else {
                println!("Unable to parse stop as time: format HH:MM or HHMM");
                return false;
            }
        }

        let date = UTC::today().naive_local();
        return store.add_part(date, part)
    }

    if let Some(ref matches) = matches.subcommand_matches("parse") {
        let data = values_t!(matches, "data", String).unwrap_or_else(|e| e.exit());

        match parsing::parse_day_line_from_legacy(&data.join(" ")) {
            Ok(day) => {
                return store.add_day(day)
            },
            Err(_) => {
                println!("Unable to parse data");
                return false
            }
        }

    }
    return false
}

fn subcmd_init(matches: &ArgMatches, pretty: bool) {
    let mut store = data::Storage::new();

    let leg_file = value_t!(matches, "legacy_file", String)
        .unwrap_or_else(|e| e.exit());

    let storage_file = value_t!(matches, "storage_file", String)
        .unwrap_or("times.json".to_string());

    if !store.import_legacy(&leg_file) {
        println!("Unable to import data!")
    }

    if !store.save(&storage_file, pretty) {
        println!("Unable to write file: {}", &storage_file);
    }
}

fn subcmd_show(store: &data::Storage, matches: &ArgMatches) {
    let show_days  = matches.is_present("days");
    let mut worked = matches.is_present("worked");
    let breaks     = matches.is_present("breaks");
    let verbose    = matches.is_present("verbose");
    let parts      = matches.is_present("parts");
    let today      = chrono::UTC::today();

    if !breaks { worked = true; }

    if let Some(ref matches) = matches.subcommand_matches("year") {
        let vals_num : Vec<u16> = if matches.is_present("years") {
            values_t!(matches, "years", u16)
                .unwrap_or_else(|e| e.exit())
        } else {
            let c = today.year() as u16;
            if verbose {
                println!("Assume current year: {}", c);
            }
            vec![c]
        };

        let mut vals : Vec<&data::Year> = vec![];
        for y in vals_num {
            match store.get_year(y) {
                Some(y) => vals.push(y),
                None => {
                    println!("Year {} not available!", y);
                }
            }
        }
        let p = printer::Printer::with_years(vals)
                        .set_fee(store.get_fee())
                        .show_days(show_days)
                        .show_worked(worked)
                        .show_breaks(breaks)
                        .show_parts(parts)
                        .show_verbose(verbose);
        p.print();
    }

    if let Some(ref matches) = matches.subcommand_matches("month") {
        let y = if matches.is_present("year") {
            value_t!(matches, "year", u16)
                .unwrap_or_else(|e| e.exit())
        } else {
            today.year() as u16
        };

        let vals_num : Vec<u8> = if matches.is_present("months") {
            values_t!(matches, "months", u8)
                .unwrap_or_else(|e| e.exit())
        } else {
            let c = today.month() as u8;
            if verbose {
                println!("Assume current month: {}", c);
            }
            vec![c]
        };

        let mut vals : Vec<data::Month> = vec![];
        for x in vals_num {
            match store.get_month(y, x) {
                Some(x) => vals.push(x),
                None => {
                    println!("Month {} not available for year {}!", x, y);
                }
            }
        }
        let p = printer::Printer::with_months(vals)
                        .set_fee(store.get_fee())
                        .show_days(show_days)
                        .show_worked(worked)
                        .show_breaks(breaks)
                        .show_parts(parts)
                        .show_verbose(verbose);
        p.print();
        return
    }

    if let Some(ref matches) = matches.subcommand_matches("week") {
        let vals_num : Vec<u16> = if matches.is_present("weeks") {
            values_t!(matches, "weeks", u16)
                .unwrap_or_else(|e| e.exit())
        } else {
            let c = today.isoweekdate().1 as u16;
            if verbose {
                println!("Assume current week: {}", c);
            }
            vec![c]
        };

        let mut vals : Vec<&data::Year> = vec![];
        for y in vals_num {
            match store.get_year(y) {
                Some(y) => vals.push(y),
                None => {
                    println!("Year {} not available!", y);
                }
            }
        }
        let p = printer::Printer::with_years(vals)
                        .set_fee(store.get_fee())
                        .show_days(show_days)
                        .show_worked(worked)
                        .show_breaks(breaks)
                        .show_parts(parts)
                        .show_verbose(verbose);
        p.print();
    }

    if let Some(ref matches) = matches.subcommand_matches("day") {
        let y = if matches.is_present("year") {
            value_t!(matches, "year", u16)
                .unwrap_or_else(|e| e.exit())
        } else {
            today.year() as u16
        };

        let m = if matches.is_present("month") {
            value_t!(matches, "month", u8)
                .unwrap_or_else(|e| e.exit())
        } else {
            today.month() as u8
        };

        let vals_num : Vec<u8> = if matches.is_present("days") {
            values_t!(matches, "days", u8)
                .unwrap_or_else(|e| e.exit())
        } else {
            let c = today.day() as u8;
            if verbose {
                println!("Assume current day: {}", c);
            }
            vec![c]
        };

        let mut vals : Vec<&data::Day> = vec![];
        for x in vals_num {
            match store.get_day(y, m, x) {
                Some(x) => vals.push(x),
                None => {
                    println!("Day {} not available for month {} in year {}!", x, m,  y);
                }
            }
        }

        let p = printer::Printer::with_days(vals)
                        .show_worked(worked)
                        .show_breaks(breaks)
                        .show_parts(parts)
                        .show_verbose(verbose);
        p.print();
        return

    }
}

