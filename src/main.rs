// Copyright 2017 rstools Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate clap;

use std::fs::File;
use std::io::{BufReader, BufRead};
use std::collections::{VecDeque, HashSet};
use std::process;

use clap::{App, Arg, AppSettings, SubCommand, ArgMatches};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const NAME: &str = env!("CARGO_PKG_NAME");

fn cli<'a>() -> ArgMatches<'a> {
    App::new(NAME)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(VERSION)
        .author(AUTHORS)
        .about("Some misc tools to help with emu dev")
        .subcommand(SubCommand::with_name("compare")
            .aliases(&["cmp", "cp"])
            .version("1.0.1")
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
        .subcommand(SubCommand::with_name("uniques")
            .version("0.1.0")
            .author(AUTHORS)
            .about("Gets all the unique lines in a file")
            .arg(Arg::with_name("file")
                .help("The file to check")
                .required(true)))
        .get_matches()
}

fn uniques(path: &String) {
    let mut uniques = HashSet::new();
    let file = if let Ok(f) = File::open(path) {f} else {
        eprintln!("Failed to open file ({})", path); process::exit(1)
    };

    for line in BufReader::new(file).lines() {
        let line = line.unwrap_or_else(|x|{eprintln!("Failed to read line! ({})", x); process::exit(1)});
        if !uniques.contains(&line) {
            println!("{}", line);
            uniques.insert(line);
        }
    }
}

fn compare(file1: &String, file2: &String, context_len: usize) {
    let mut buf = VecDeque::new();
    let f1 = if let Ok(f) = File::open(file1){
        f
    } else {eprintln!("Failed to open file_1"); process::exit(1)};
    let f2 = if let Ok(f) = File::open(file2){
        f
    } else {eprintln!("Failed to open file_2"); process::exit(1)};
    let mut i = 0;
    for lp in BufReader::new(f1).lines().zip(BufReader::new(f2).lines()) {
        i += 1;
        let (s1, s2) = lp;
        let s1 = s1.unwrap();
        let s2 = s2.unwrap();
        let sout = format!("{} {}", &s1, &s2);
        if &s1 != &s2 {
            println!("difference found on line {}", i);
            for s in buf {
                println!("{}", s);
            }

            println!("{}", sout);
            break;
        }

        buf.push_back(sout);
        if buf.len() > context_len {
            buf.pop_front();
        }
    }
}

fn main() {
    let matches = cli();
    match matches.subcommand() {
        ("compare", Some(cmp_matches)) => {
            let context = match cmp_matches.value_of("context") {
                Some(num) => if let Ok(n) = num.to_string().parse() {n} else {
                    eprintln!("Couldn't convert context to number ({})", num);std::process::exit(1)
                },
                None => 5
            };

            compare(&cmp_matches.value_of("file_1").unwrap().to_string(), &cmp_matches.value_of("file_2").unwrap().to_string(), context);
        }

        ("uniques", Some(uniques_matches)) => uniques(&uniques_matches.value_of("file").unwrap().to_string()),
        _ => unreachable!()
    }
}
