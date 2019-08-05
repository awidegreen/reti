# reti - Time recording in Rust

`reti` allows you do **re**cord **ti**me via the CLI by tracking periods of time
during a day. All data is stored in a json file (*store*).

## Background

Some time ago, I started to feel the need of tracking hours I've worked.
I started off by tracking the times in a text-file and wrote a simple python
script for parsing, counting and summing up the entries. The entries were line
based, which means each line contains exactly on day with all time periods,
factor and comment.

```
# date          parts with factor            comment
# yyyy-mm-dd    separated by ' '
#               (value of last '-'
#               i.e. '...-1', 1 if
#               not present)

2016-04-02     08:00-12:00-1 13:00-17:00 19:00-21:00-1.5  # long day

# the line above consits of 3 parts, where the last part (19-21) has a factor of
# 1.5 of the base fee
```
The date can also only be specified as `mm-dd` then the current year is assumed.

Apart from this getting a bit clunky, I also wanted to continue to work with
*Rust* which let me to the idea of implementing something proper and storing the
data in another format.

However, since the format specified above is pretty straight forward to
understand and edit in *your-favorite-editor*, `reti` continues to use the format
when editing entries.

## Features

As mentioned, `reti` works on single files, which can be specified with `-f`

```sh
# use the 'year2016.json' for all reti subcommands
$ reti -f year2016.json show
```

### recording

Each day consists of periods of time (part), for example periods worked before
and after lunch, where lunch is considered a break. Each period can be assigned
a factor if for example the period shall be counted as overtime. This factor is
based on the file-specific base-fee which can be set via `reti set fee <val>`.

```sh
# show help for adding
$ reti add help

# help for adding parts
$ reti add part -h

# record a period (part) for the current day (from 08 to 12).
$ reti add part 08:00 12:00

# add via parse will parse the provided data as legacy format
$ reti add parse 04-02 08:00-12:00

```

### show

The recorded data (per *store*) can by shown via the `show` subcommand. This
includes showing a day, week, month and year with different levels of details
(see `reti show help`).

Issue `reti show help` for a detailed description, some examples:

```sh
# show a summary of the current mont
$ reti show month

# get a verbose (-v) summary of year 2016 in the 'bla.json' file.
# all days (-d) and all their parts (-p) will be shown
$ reti -f bla.json show -p -v -d year 2016

```

### import

The import of files using the "legacy" format is still available ... editing a
text-file per hand is easier then writing json.

### edit

Existing entries can be changed using the `edit` subcommand. All requested
entries will be queried and showm in the *legacy* format (per line) within a
temp-file.  Once saved and the $EDITOR is exited, the entries will be parsed
and overwritten. If a line should be disregarded, either leave it untouched,
comment it out (`# ...`) or delete the line from the file.

```sh
# edit the current day in $EDITOR from file foo.json
$ reti -f foo.json edit
```

### `get` and `set` file properties

In order to allow reti to do fee calculations, one can set the base fee:

```sh
# set fee configured for file bla.json
$ reti -f bla.json set fee 50
```
The fee is not bound to any currency - keep it simple. Each part of a day can be
a factor of that base fee, where the default factor is `1`.

```sh
# get the current fee for bla.json
$ reti -f bla.json get fee
```
## Getting started

Create a new store file.

```sh
# init file 2016.json (empty)
$ reti init 2016.json

# alternative init a file with a legacy (e.g. under examples/test_leg_format.txt)
$ reti init 2016..json examples/test_leg_format.txt

# show the whole year
$ reti -f 2016.json show -p -d year 2016

# set some arbitrary fee per hour
$ reti -f 2016.json set fee 250
# show year should show some proper calculation now
$ reti -f 2016.json show -p -d year 2016

# edit a specific date, e.g. change end time and comment to 3h
$ reti -f 2016.json edit 2016-08-27
# check if everything was updates
$ reti -f 2016.json show -p -d year 2016

# edit mulitple days, only September
$ reti -f 2016.json edit 2016-09-01 2016-09-02
```

## Configuration file

`reti` supports a configuration file which can pre-define certain properties
which are used when `reti` starts. The config file should be located under
`$XDG_CONFIG_DIR/reti/reti.toml` (e.g. `$HOME/.config/reti/reti.conf`)

* `storage-file`: path to default json storage file (string)
* `save-pretty`: specifies if the json file shall be written readable (bool)

The properties can be overwritten with command line parameters, see help.

Example config file: `$HOME/.config/reti/reti.toml`
```toml
storage-file = "/home/awidegreen/reti_2018.json"
```

## Disclaimer

**Use at your own risk**

## License

Copyright (C) 2018 by Armin Widegreen

This is free software, licensed under The [ISC License](LICENSE).
