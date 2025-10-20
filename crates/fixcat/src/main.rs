use crocofix::message::Message;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

const FIX_MESSAGE_PREFIX: &str = "8=FIX";

enum Input {
    Stdin(io::Stdin),
    File(File),
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Input::Stdin(stdin) => stdin.read(buf),
            Input::File(file) => file.read(buf),
        }
    }
}

/// Pretty print FIX protocol messages
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Options {

    /// Include administrative messages
    #[arg(long)]
    admin: bool,

    /// Print non FIX text in the output
    #[arg(long)]
    mix: bool,

    /// Optional input files, if not specifed input will be read from STDIN
    files: Vec<String>
}

impl Options {

    pub fn input_readers(&self) -> io::Result<Vec<BufReader<Input>>> {
        if self.files.is_empty() {
            Ok(vec![BufReader::new(Input::Stdin(io::stdin()))])
        } else {
            self.files
                .iter()
                .map(|path| File::open(path).map(|f| BufReader::new(Input::File(f))))
                .collect()
        }
    }
}

fn decode_and_print_line(line: &String, options: &Options) -> Result<(), crocofix::error::Error>
{
    if let Some(start_of_message) = line.find(FIX_MESSAGE_PREFIX) {
        let (_prefix, suffix) = line.split_at(start_of_message);
        let mut message = Message::default();
        let result = message.decode(suffix.as_bytes())?;
        if result.consumed > 0 {
            if !options.admin && message.is_admin() {
                return Ok(());
            }
            println!("{}\n", message);
        }
    }
    else {
        if options.mix {
            println!("{}", line);
        }
    }

    Ok(())
}

fn main() -> Result<(), crocofix::error::Error>
{
    let options = Options::parse();

    for reader in options.input_readers()? {
        for line in reader.lines() {
            decode_and_print_line(&line?, &options)?;
        }
    };

    Ok(())
}
