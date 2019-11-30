use crate::{Bar, Trade};


pub struct DollarBars {
    pub bars: Vec<Bar>,
    pub threshold: f64
}

impl DollarBars {
    pub fn new<I>(trades: I, threshold: f64) -> DollarBars
        where I: Iterator<Item = Trade>,
    {
        let all_trades: Vec<Trade> = trades.collect();
        let first_trade = &all_trades[0];
        let mut bars: Vec<Bar> = vec![Bar::new(first_trade)];
        let first_bar = bars.last_mut().unwrap();
        let window_size = 60 * 60 * 24;
        let mut current_bar = first_bar;
        let mut threshold = threshold;
        let mut t = 0;
        let slots = 100.0;

        for trade in &all_trades {
            current_bar.next(trade);
            if current_bar.dollar_volume > threshold {
                threshold = calculate_tail_sum(&all_trades[..t], window_size) / slots;
                bars.push(Bar::new(trade));
                current_bar = bars.last_mut().unwrap();
                println!("{} threshold = {}", trade.time(), threshold);
            }
            t += 1;
        }

        DollarBars {
            bars: bars,
            threshold
        }
    }
}

fn calculate_tail_sum(trades: &[Trade], window_size: u64) -> f64 {
    let t0 = trades[trades.len() - 1].timestamp;
    let mut sum = 0.0;
    let cutoff = t0 - window_size;
    for trade in trades.iter().rev() {
        if cutoff > trade.timestamp {
            break;
        }
        sum += trade.dollar_value();
    }
    sum
}

