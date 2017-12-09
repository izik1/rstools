// Copyright 2017 rstools Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate clap;

use std::fs::File;
use std::{io, process};
use std::io::{BufReader, BufRead};
use std::collections::{VecDeque, HashSet};
use std::error::Error;
use clap::{App, Arg, AppSettings, SubCommand, ArgMatches};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const NAME: &str = env!("CARGO_PKG_NAME");
const CMD_CMP: &str = "compare";
const CMD_UNQ: &str = "uniques";

fn cli<'a>() -> ArgMatches<'a> {
    App::new(NAME)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(VERSION)
        .author(AUTHORS)
        .about("Some misc tools to help with emu dev")
        .subcommand(SubCommand::with_name(CMD_CMP)
            .aliases(&["cmp", "cp"])
            .version("1.0.2")
            .author(AUTHORS)
            .about("Compares two files and prints the first line they are different (+context lines before it) on along with the line number")
            .args(&[
                Arg::with_name("file_1")
                    .help("The first file")
                    .required(true),
                Arg::with_name("file_2")
                    .help("the second file")
                    .required(true),
                Arg::with_name("context")
                    .help("the number of lines to show before the diff")
                    .short("c")
                    .long("context")
                    .takes_value(true)]))
        .subcommand(SubCommand::with_name(CMD_UNQ)
            .version("0.1.1")
            .author(AUTHORS)
            .about("Gets all the unique lines in a file")
            .arg(Arg::with_name("file")
                .help("The file to check")
                .required(true)))
        .get_matches()
}

fn uniques(path: &String) -> Result<(), Box<Error>> {
    let mut uniques = HashSet::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        if !uniques.contains(&line) {
            println!("{}", line);
            uniques.insert(line);
        }
    }

    Ok(())
}

fn compare(file1: &String, file2: &String, context_len: usize) -> Result<(), Box<Error>>{
    let mut buf = VecDeque::with_capacity(context_len);
    let f1 = File::open(file1)?;
    let f2 = File::open(file2)?;
    let mut i = 1;
    for (s1, s2) in BufReader::new(f1).lines().zip(BufReader::new(f2).lines()) {
        let s1 = s1?;
        let s2 = s2?;
        let sout = format!("{} {}", &s1, &s2);
        if &s1 != &s2 {
            println!("difference found on line {}", i);
            for s in buf {
                println!("{}", s);
            }

            println!("{}", sout);
            break;
        }

        if context_len > 0 {
            if buf.len() == context_len {
                buf.pop_front();
            }

            buf.push_back(sout);
        }

        i += 1;
    }

    Ok(())
}

fn run_command<'a> (matches: ArgMatches<'a>) -> Result<(), Box<Error>> {
    match matches.subcommand() {
        (CMD_CMP, Some(cmp_matches)) => {
            let context = match cmp_matches.value_of("context") {
                Some(num) => num.to_string().parse()?,
                None => 5
            };

            compare(&cmp_matches.value_of("file_1").unwrap().to_string(), &cmp_matches.value_of("file_2").unwrap().to_string(), context)
        }

        (CMD_UNQ, Some(uniques_matches)) => uniques(&uniques_matches.value_of("file").unwrap().to_string()),
        _ => unreachable!()
    }
}

fn main() {
    if let Err(e) = run_command(cli()) {
        eprintln!("{}", e);
        process::exit(1)
    }
}
