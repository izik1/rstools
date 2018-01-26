// Copyright 2017-2018 rstools Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate structopt;

#[macro_use]
extern crate structopt_derive;

use std::fs::File;
use std::process;
use std::io::{BufRead, BufReader};
use std::collections::{HashSet, VecDeque};
use std::error::Error;

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Some misc tools to help with emu dev")]
enum Cli {
    #[structopt(name = "compare",
    aliases_raw = "&[\"cmp, cp\"]",
    version = "1.1.0",
    about = "Compares two files and prints the first line they are different (+context lines before it) on along with the line number")]
    Compare {
        #[structopt(help = "The first file.")]
        file_1: String,
        #[structopt(help = "The second file.")]
        file_2: String,
        #[structopt(short = "c", long = "context", help = "The number of lines to show before the diff.", default_value = "5")]
        context: usize,
    },

    #[structopt(name = "uniques", version = "0.2.0",
    about = "Gets all the unique lines in a file")]
    Uniques {
        #[structopt(help = "The file to check")]
        file: String
    }
}

fn uniques(path: &str) -> Result<(), Box<Error>> {
    let mut uniques = HashSet::new();
    for line in BufReader::new(File::open(&path)?).lines() {
        let line = line?;
        if !uniques.contains(&line) {
            println!("{}", line);
            uniques.insert(line);
        }
    }

    Ok(())
}

fn compare(file1: &str, file2: &str, context_len: usize) -> Result<(), Box<Error>> {
    let mut buf = VecDeque::with_capacity(context_len);
    let f1 = File::open(file1)?;
    let f2 = File::open(file2)?;
    let mut i = 1;
    for (s1, s2) in BufReader::new(f1).lines().zip(BufReader::new(f2).lines()) {
        let s1 = s1?;
        let s2 = s2?;
        let sout = format!("{} {}", &s1, &s2);
        if s1 != s2 {
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

fn run_command(cli: Cli) -> Result<(), Box<Error>> {
    match cli {
        Cli::Compare {
            file_1,
            file_2,
            context,
        } => compare(&file_1, &file_2, context),
        Cli::Uniques { file } => uniques(&file),
    }
}

fn main() {
    if let Err(e) = run_command(Cli::from_args()) {
        eprintln!("{}", e);
        process::exit(1)
    }
}
