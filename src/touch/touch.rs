#![crate_name = "touch"]

/*
 * This file is part of the uutils coreutils package.
 *
 * (c) Nick Platt <platt.nicholas@gmail.com>
 *
 * For the full copyright and license information, please view the LICENSE file
 * that was distributed with this source code.
 */

extern crate getopts;
extern crate libc;
extern crate time;
extern crate filetime;

#[macro_use]
extern crate uucore;

use filetime::*;
use std::fs::{self, File};
use std::io::{Error, Write};
use std::path::Path;
use uucore::fs::UUPathExt;

static NAME: &'static str = "touch";
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

// Since touch's date/timestamp parsing doesn't account for timezone, the
// returned value from time::strptime() is UTC. We get system's timezone to
// localize the time.
macro_rules! to_local(
    ($exp:expr) => ({
        let mut tm = $exp;
        tm.tm_utcoff = time::now().tm_utcoff;
        tm
    })
);

macro_rules! local_tm_to_filetime(
    ($exp:expr) => ({
        let ts = $exp.to_timespec();
        FileTime::from_seconds_since_1970(ts.sec as u64, ts.nsec as u32)
    })
);

pub fn uumain(args: Vec<String>) -> i32 {
    let mut opts = getopts::Options::new();

    opts.optflag("a", "",               "change only the access time");
    opts.optflag("c", "no-create",      "do not create any files");
    opts.optopt( "d", "date",           "parse argument and use it instead of current time", "STRING");
    opts.optflag("h", "no-dereference", "affect each symbolic link instead of any referenced file \
                                         (only for systems that can change the timestamps of a symlink)");
    opts.optflag("m", "",               "change only the modification time");
    opts.optopt( "r", "reference",      "use this file's times instead of the current time", "FILE");
    opts.optopt( "t", "",               "use [[CC]YY]MMDDhhmm[.ss] instead of the current time", "STAMP");
    opts.optopt( "",  "time",           "change only the specified time: \"access\", \"atime\", or \
                                         \"use\" are equivalent to -a; \"modify\" or \"mtime\" are \
                                         equivalent to -m", "WORD");
    opts.optflag("h", "help",           "display this help and exit");
    opts.optflag("V", "version",        "output version information and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(e) => panic!("Invalid options\n{}", e)
    };

    if matches.opt_present("version") {
        println!("{} {}", NAME, VERSION);
        return 0;
    }

    if matches.opt_present("help") || matches.free.is_empty() {
        println!("{} {}", NAME, VERSION);
        println!("");
        println!("Usage: {} [OPTION]... FILE...", NAME);
        println!("");
        println!("{}", opts.usage("Update the access and modification times of \
                                   each FILE to the current time."));
        if matches.free.is_empty() {
            return 1;
        }
        return 0;
    }

    if matches.opt_present("date") && matches.opts_present(&["reference".to_string(), "t".to_string()]) ||
       matches.opt_present("reference") && matches.opts_present(&["date".to_string(), "t".to_string()]) ||
       matches.opt_present("t") && matches.opts_present(&["date".to_string(), "reference".to_string()]) {
        panic!("Invalid options: cannot specify reference time from more than one source");
    }

    let (mut atime, mut mtime) =
        if matches.opt_present("reference") {
            stat(&matches.opt_str("reference").unwrap()[..], !matches.opt_present("no-dereference"))
        } else if matches.opts_present(&["date".to_string(), "t".to_string()]) {
            let timestamp = if matches.opt_present("date") {
                parse_date(matches.opt_str("date").unwrap().as_ref())
            } else {
                parse_timestamp(matches.opt_str("t").unwrap().as_ref())
            };
            (timestamp, timestamp)
        } else {
            let now = local_tm_to_filetime!(time::now());
            (now, now)
        };

    for filename in matches.free.iter() {
        let path = &filename[..];

        if !Path::new(path).uu_exists() {
            // no-dereference included here for compatibility
            if matches.opts_present(&["no-create".to_string(), "no-dereference".to_string()]) {
                continue;
            }

            match File::create(path) {
                Err(e) => {
                    show_warning!("cannot touch '{}': {}", path, e);
                    continue;
                },
                _ => (),
            };

            // Minor optimization: if no reference time was specified, we're done.
            if !matches.opts_present(&["date".to_string(), "reference".to_string(), "t".to_string()]) {
                continue;
            }
        }

        // If changing "only" atime or mtime, grab the existing value of the other.
        // Note that "-a" and "-m" may be passed together; this is not an xor.
        if matches.opts_present(&["a".to_string(), "m".to_string(), "time".to_string()]) {
            let st = stat(path, !matches.opt_present("no-dereference"));
            let time = matches.opt_strs("time");

            if !(matches.opt_present("a") ||
                 time.contains(&"access".to_string()) ||
                 time.contains(&"atime".to_string()) ||
                 time.contains(&"use".to_string())) {
                atime = st.0;
            }

            if !(matches.opt_present("m") ||
                 time.contains(&"modify".to_string()) ||
                 time.contains(&"mtime".to_string())) {
                mtime = st.1;
            }
        }

        // this follows symlinks and thus does not work correctly for the -h flag
        // need to use lutimes() c function on supported platforms
        match filetime::set_file_times(path, atime, mtime) {
            Err(e) => show_warning!("cannot touch '{}': {}", path, e),
            _ => (),
        };
    }

    0
}

fn stat(path: &str, follow: bool) -> (FileTime, FileTime) {
    let metadata = if follow {
        fs::symlink_metadata(path)
    } else {
        fs::metadata(path)
    };

    match metadata {
        Ok(m) => (
            FileTime::from_last_access_time(&m),
            FileTime::from_last_modification_time(&m)
            ),
        Err(_) => crash!(1, "failed to get attributes of '{}': {}", path, Error::last_os_error())
    }
}

fn parse_date(str: &str) -> FileTime {
    // This isn't actually compatible with GNU touch, but there doesn't seem to
    // be any simple specification for what format this parameter allows and I'm
    // not about to implement GNU parse_datetime.
    // http://git.savannah.gnu.org/gitweb/?p=gnulib.git;a=blob_plain;f=lib/parse-datetime.y
    match time::strptime(str, "%c") {
        Ok(tm) => local_tm_to_filetime!(to_local!(tm)),
        Err(e) => panic!("Unable to parse date\n{}", e)
    }
}

fn parse_timestamp(s: &str) -> FileTime {
    let now = time::now();
    let (format, ts) = match s.chars().count() {
        15 => ("%Y%m%d%H%M.%S", s.to_string()),
        12 => ("%Y%m%d%H%M", s.to_string()),
        13 => ("%y%m%d%H%M.%S", s.to_string()),
        10 => ("%y%m%d%H%M", s.to_string()),
        11 => ("%Y%m%d%H%M.%S", format!("{}{}", now.tm_year + 1900, s)),
         8 => ("%Y%m%d%H%M", format!("{}{}", now.tm_year + 1900, s)),
         _ => panic!("Unknown timestamp format")
    };

    match time::strptime(&ts, format) {
        Ok(tm) => local_tm_to_filetime!(to_local!(tm)),
        Err(e) => panic!("Unable to parse timestamp\n{}", e)
    }
}


#[allow(dead_code)]
fn main() {
    std::process::exit(uumain(std::env::args().collect()));
}
