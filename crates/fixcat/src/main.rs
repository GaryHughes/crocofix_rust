use crocofix::message::Message;
use crocofix::order_book::OrderBook;
use crocofix::order_report::OrderReport;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, stdout};

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

fn validate_field(tag_or_name: &str) -> Result<u32, String> 
{
    let trimmed = tag_or_name.trim();

    let tag = match trimmed.parse::<u32>() {
        Ok(tag) => tag,
        Err(_) => {         
            let Some(field) = crocofix::FIX_5_0SP2::fields().field_with_name(trimmed) else {
                return Err(format!("Unable to find a FIX field with name or tag = '{}'", tag_or_name));
            };
            return Ok(field.tag())
        }
    };

    if crocofix::FIX_5_0SP2::fields().is_tag_valid(tag as usize) {
        return Ok(tag);
    }

    Err(format!("Unable to find a FIX field with tag = '{}'", tag))
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

    /// Track order state
    #[arg(long)]
    orders: bool,

    /// Comma separated list of field names or tags to display when tracking order state
    #[arg(long, value_delimiter = ',', value_parser = validate_field)]
    fields: Option<Vec<u32>>,

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

fn decode_and_print_line(line: &String, options: &Options, order_book: &mut OrderBook, order_report: &mut OrderReport)
{
    if let Some(start_of_message) = line.find(FIX_MESSAGE_PREFIX) {
        let (_prefix, suffix) = line.split_at(start_of_message);
        let mut message = Message::default();
        let result = match message.decode(suffix.as_bytes()) {
            Ok(result) => result,
            Err(error) => {
                eprintln!("{:?}", error);
                return;
            }
        };
        if result.consumed > 0 {
            if !options.admin && message.is_admin() {
                return;
            }
            println!("{}\n", message);
            if options.orders {
                match order_book.process(&message) {
                    Ok(()) => {
                        let _ = match order_report.print(&mut stdout(), order_book) {
                            Ok(bytes) => bytes,
                            Err(error) => eprintln!("{:?}", error)
                        };
                    },
                    Err(error) => eprintln!("{:?}", error)
                }
            }
        }
    }
    else {
        if options.mix {
            println!("{}", line);
        }
    }
}

fn main() -> Result<(), crocofix::error::Error>
{
    let options = Options::parse();

    for reader in options.input_readers()? {
        let mut order_book = OrderBook::default();
        let mut order_report = OrderReport::with_fields(options.fields.clone());
        for line in reader.lines() {
            decode_and_print_line(&line?, &options, &mut order_book, &mut order_report);
        }
    };

    Ok(())
}
