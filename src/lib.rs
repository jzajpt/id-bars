extern crate csv;
#[macro_use]
extern crate serde_derive;

pub mod volume_bars;
pub mod dollar_bars;
pub mod tick_imbalance_bars;

use chrono::prelude::*;
use std::fmt;
use std::io;
use std::fs::File;
use volume_bars::VolumeBars;
use dollar_bars::DollarBars;
use tick_imbalance_bars::TickImbalanceBars;


pub struct Config<'a> {
    pub input: &'a str,
    pub output: &'a str,
    pub method: &'a str,
}

pub struct CsvTradesFile {
    file: File
}

#[derive(Deserialize, Debug)]
pub struct Trade {
    pub timestamp: u64,
    pub price: f64,
    pub amount: f64
}

#[derive(Serialize, Debug)]
pub struct Bar {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub dollar_volume: f64,
    pub count: u32,
    pub last_timestamp: u64,
}

trait BarGenerator {
    fn load_trades<I>(& mut self, trades: I)
        where I: Iterator<Item = Trade>
    {
        for trade in trades {
            self.process_trade(trade);
        }
    }

    fn process_trade(&mut self, trade: Trade) -> Option<&Bar>;
}

impl Bar {
    /// Returns a new bar based on the first trade
    fn new(trade: &Trade) -> Bar {
        Bar {
            timestamp: trade.timestamp,
            open: trade.price,
            high: trade.price,
            low: trade.price,
            close: trade.price,
            volume: trade.amount,
            dollar_volume: trade.amount * trade.price,
            count: 1,
            last_timestamp: trade.timestamp,
        }
    }

    /// Registers a tick (trade) into the bar
    fn next(&mut self, trade: &Trade) {
        if self.high < trade.price {
            self.high = trade.price;
        }
        if self.low > trade.price {
            self.low = trade.price;
        }
        self.close = trade.price;
        self.volume += trade.amount;
        self.dollar_volume += trade.amount * trade.price;
        self.count = self.count + 1;
        self.last_timestamp = trade.timestamp
    }
}

impl fmt::Display for Bar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let time = Utc.timestamp(self.timestamp as i64, 0).to_string();
        let last_time = Utc.timestamp(self.last_timestamp as i64, 0).to_string();
        write!(f, "{}-{}: {}-{}, {}", time, last_time, self.open, self.close, self.count)
    }
}

impl Trade {
    pub fn dollar_value(&self) -> f64 {
        self.price * self.amount
    }

    pub fn time(&self) -> String {
         Utc.timestamp(self.timestamp as i64, 0).to_string()
    }
}

impl CsvTradesFile {
    /// Returns new CsvTradesFile given a filename
    fn new(filename: &str) -> io::Result<CsvTradesFile> {
        let file = File::open(filename)?;
        Ok(CsvTradesFile { file })
    }

    /// Reads contents of CSV and returns Iterator
    fn read(self) -> impl Iterator<Item = Trade> {
        let buf_reader = io::BufReader::new(self.file);
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(buf_reader);
        let iter = reader.into_deserialize();

        iter.map(|result| result.unwrap())
    }
}


fn save(bars: Vec<Bar>, filename: &str) -> io::Result<()> {
    let mut csv_writer = csv::Writer::from_path(filename)?;
    for bar in bars {
        csv_writer.serialize(bar)?;
    }
    Ok(())
}

pub fn run(config: Config) -> Result<(), String> {
    let trades_file = CsvTradesFile::new(config.input);
    if let Err(_) = trades_file {
        return Err(format!("Unable to open file {}", config.input))
    }
    let trades = trades_file.unwrap().read();
    let bars = match config.method {
        "dollar" => DollarBars::new(trades, 500000.0).bars,
        "volume" => VolumeBars::new(trades, 500.0).bars,
        "tib" => {
            let mut tib = TickImbalanceBars::new(50);
            tib.load_trades(trades);
            tib.bars
        },
        _ => {
            return Err("invalid sampling method".to_string());
        }
    };

    save(bars, config.output).expect("unable to save file");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_initializes_properly() {
        let trade = Trade{
            timestamp: 1546016874,
            price: 3800.0,
            amount: 0.25
        };
        let bar = Bar::new(&trade);

        assert_eq!(trade.timestamp, bar.timestamp);
        assert_eq!(trade.timestamp, bar.last_timestamp);
        assert_eq!(trade.price, bar.open);
        assert_eq!(trade.price, bar.high);
        assert_eq!(trade.price, bar.low);
        assert_eq!(trade.price, bar.close);
        assert_eq!(trade.amount, bar.volume);
        assert_eq!(950.0, bar.dollar_volume);
        assert_eq!(1, bar.count);
    }


    #[test]
    fn bar_updates_new_high() {
        let trade1 = Trade{
            timestamp: 1546016874,
            price: 3800.0,
            amount: 0.25
        };
        let trade2 = Trade{
            timestamp: 1546016875,
            price: 3805.0,
            amount: 0.50
        };
        let mut bar = Bar::new(&trade1);
        bar.next(&trade2);

        assert_eq!(trade1.timestamp, bar.timestamp);
        assert_eq!(trade2.timestamp, bar.last_timestamp);
        assert_eq!(trade1.price, bar.open);
        assert_eq!(trade2.price, bar.high);
        assert_eq!(trade1.price, bar.low);
        assert_eq!(trade2.price, bar.close);
        assert_eq!(trade1.amount + trade2.amount, bar.volume);
        assert_eq!(2, bar.count);
    }
}
