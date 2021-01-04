use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new("reti")
        .about("The cli time reporting tool")
        .author("Armin Widegreen <armin.widegreen@gmail.com>")
        .version(crate_version!())
        .global_setting(AppSettings::InferSubcommands)
        .args_from_usage(
            "--save-pretty 'Save the json file read-able if a subcommand executes save.'
            -f, --file=[FILE] 'The file where the data is stored.'"
            )
        .subcommand(SubCommand::with_name("init")
                    .about("Initializes a new storage file. CAUTION: \
                           this will overwrite existing data!")
                    .args_from_usage(
                        "<storage_file> 'Which file shall be written!'
                        [legacy_file] 'import data from the legacy file!'"))
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
                    .about("Sets attributes for the current store.")
                    .subcommand(SubCommand::with_name("fee")
                                .about("Sets the fee per hour which is the base for all parts!")
                                .args_from_usage("<value> 'The fee value as f32.'"))
                    )
        .subcommand(SubCommand::with_name("rm")
                    .about("Removes given days from the current store")
                    .args_from_usage(
                        "-f, --force 'Enforce removal.'
                        [dates]... 'List of days that should be removed, space separated'"
                        ))
        .subcommand(SubCommand::with_name("add")
                    .about("Everything related to add data to the store.")
                    .subcommand(SubCommand::with_name("part")
                                .about("Add a part of the day. A part can \
                                       consist only of the starting point, \
                                       therefore 'stop' is optional. If only the \
                                       stop should be recorded use '_' for start")
                                .args_from_usage(
                                    "<start> 'The format is: HH:MM (default: now), use _ if only stop shall be recorded!'
                                    [stop] 'Format: HH:MM, optional hence only start will be recorded'"
                                    ))
                    .subcommand(SubCommand::with_name("parts")
                                .about("Add a parts of the day.")
                                .args_from_usage(
                                    "<parts>... 'The format is: HH:MM-HH:MM[-factor] (START-STOP[-FACTOR]).'"
                                    ))
                    .subcommand(SubCommand::with_name("parse")
                                .about("Lets you add entries based on free text, either as parameter or via stdin.")
                                .args_from_usage(
                                    "<data>... 'The data that will be attempted to be parsed and added to the store.'"
                                    ))
                    )
        .subcommand(SubCommand::with_name("edit")
                    .about("Edit a specific day.").
                    args_from_usage("[dates]... 'can have the format: [yyyy-][mm-]dd, \
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
                                    "-y, --year [year] 'Specify a year (default: current)'
                                    [weeks]... 'Space separated list of weeks to show (default: current)'"
                                    ))
                    .subcommand(SubCommand::with_name("day")
                                .args_from_usage(
                                    "-y, --year [year] 'Specify a year (default: current)'
                                    -m, --month [month] 'Specify a month (default: current)'
                                    [days]... 'Space separated list of days to show (default: today)'"
                                    )))
        .subcommand(SubCommand::with_name("completions")
            .about("Generates completion scripts for your shell")
            .setting(AppSettings::Hidden)
            .arg(Arg::with_name("SHELL")
                .required(true)
                .possible_values(&["bash", "fish", "zsh"])
                .help("The shell to generate the script for")))
}
