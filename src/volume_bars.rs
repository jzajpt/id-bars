use crate::{Bar, Trade};

pub struct VolumeBars {
    pub bars: Vec<Bar>,
    pub threshold: f64
}

impl VolumeBars {
    pub fn new<I>(mut trades: I, threshold: f64) -> VolumeBars
        where I: Iterator<Item = Trade>,
    {
        let first_trade = trades.next().unwrap();
        let mut bars: Vec<Bar> = vec![Bar::new(&first_trade)];

        for trade in trades {
            let bar = bars.last_mut().unwrap();
            bar.next(&trade);
            if bar.volume > threshold {
                bars.push(Bar::new(&trade));
            }
        }

        VolumeBars {
            bars: bars,
            threshold
        }
    }

}

