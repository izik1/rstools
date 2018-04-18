// Copyright 2017-2018 rstools Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[macro_use]
extern crate structopt;

use std::{fmt, process, collections::{HashSet, VecDeque}, fs::File,
          io::{self, BufRead, BufReader, Write}};

use structopt::StructOpt;

enum Error {
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => writeln!(f, "{}", e),
        }
    }
}

type Result<T> = ::std::result::Result<T, Error>;

struct ReadBuffer<'a, T: BufRead + 'a> {
    reader: &'a mut T,
    buffer: String,
}

impl<'a, T: BufRead + 'a> ReadBuffer<'a, T> {
    fn new(reader: &'a mut T) -> Self {
        ReadBuffer {
            reader,
            buffer: String::new(),
        }
    }

    fn get_line(&mut self) -> Result<Option<&str>> {
        self.buffer.clear();
        if self.reader.read_line(&mut self.buffer)? > 0 {
            Ok(Some(self.buffer.trim()))
        } else {
            Ok(None)
        }
    }
}

#[derive(StructOpt)]
#[structopt(about = "Some misc tools to help with emu dev")]
enum Cli {
    #[structopt(name = "compare", raw(aliases = r#"&["cmp, cp"]"#), version = "1.1.0",
                about = "Compares two files and prints the first line they are different (+context lines before it) on along with the line number")]
    Compare {
        #[structopt(help = "The first file.")]
        file_1: String,
        #[structopt(help = "The second file.")]
        file_2: String,
        #[structopt(short = "c", long = "context",
                    help = "The number of lines to show before the diff.", default_value = "5")]
        context: usize,
    },

    #[structopt(name = "uniques", version = "0.2.1",
                about = "Gets all the unique lines in a file")]
    Uniques {
        #[structopt(help = "The file to check")]
        file: String,
    },
}

fn uniques<T: BufRead>(reader: &mut T) -> Result<()> {
    let mut reader = ReadBuffer::new(reader);
    let mut uniques = HashSet::new();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    while let Ok(Some(line)) = reader.get_line() {
        if uniques.insert(line.to_string()) {
            writeln!(stdout, "{}", line)?;
        }
    }

    Ok(())
}

fn compare<T: BufRead>(reader1: &mut T, reader2: &mut T, context_len: usize) -> Result<()> {
    struct Iter<'a, 'b, T: BufRead + 'a, Q: BufRead + 'b> {
        reader1: ReadBuffer<'a, T>,
        reader2: ReadBuffer<'b, Q>,
        line_num: usize,
    }

    impl<'a, 'b, T: BufRead + 'a, Q: BufRead + 'b> Iter<'a, 'b, T, Q> {
        fn new(reader1: ReadBuffer<'a, T>, reader2: ReadBuffer<'b, Q>) -> Self {
            Iter {
                reader1,
                reader2,
                line_num: 0,
            }
        }

        fn get_line(&mut self) -> Result<Option<(usize, &str, &str)>> {
            // The temporary is to prevent the line number from overflowing when a reader returns `Err` or `None`.
            let res = Ok(Some((
                self.line_num,
                match self.reader1.get_line()? {
                    Some(l) => l,
                    None => return Ok(None),
                },
                match self.reader2.get_line()? {
                    Some(l) => l,
                    None => return Ok(None),
                },
            )));

            self.line_num += 1;
            res
        }
    }

    let mut buf = VecDeque::with_capacity(context_len);

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut iter = Iter::new(ReadBuffer::new(reader1), ReadBuffer::new(reader2));

    while let Some((i, s1, s2)) = iter.get_line()? {
        if s1 != s2 {
            writeln!(stdout, "difference found on line {}", i + 1)?;
            for s in buf {
                writeln!(stdout, "{}", s)?;
            }

            writeln!(stdout, "{} {}", &s1, &s2)?;
            break;
        }

        if context_len > 0 {
            if buf.len() == context_len {
                buf.pop_front();
            }

            buf.push_back([s1, s2].join(" "));
        }
    }

    Ok(())
}

fn run_command(cli: Cli) -> Result<()> {
    match cli {
        Cli::Compare {
            file_1,
            file_2,
            context,
        } => compare(
            &mut BufReader::new(File::open(file_1)?),
            &mut BufReader::new(File::open(file_2)?),
            context,
        ),
        Cli::Uniques { file } => uniques(&mut BufReader::new(File::open(&file)?)),
    }
}

fn main() {
    if let Err(e) = run_command(Cli::from_args()) {
        eprintln!("{}", e);
        process::exit(1)
    }
}
