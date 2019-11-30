#[macro_use]
extern crate clap;

use std::process;
use id_bars::Config;

fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Jiri Z. <jzajpt@gmail.com>")
        (about: "Does awesome things")
        (@arg INPUT: +required "Sets the input file to use")
        (@arg output: +takes_value "Sets the output file to use")
        (@arg method: -m +takes_value "Sets the sampling method to produce bars")
        (@arg debug: -d ... "Sets the level of debugging information")
    ).get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("output").unwrap_or("output-bars.csv");
    let method = matches.value_of("method").unwrap_or("dollar");
    let config = Config { input, output, method };
    if let Err(e) = id_bars::run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
