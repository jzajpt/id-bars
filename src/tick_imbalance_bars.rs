use crate::{BarGenerator, Bar, Trade};
use num_traits::ToPrimitive;


const EWMA_ALPHA: f64 = 1.0 / 2.71828;

enum GeneratorState {
    Empty,
    InitialBar,
    Working
}

pub struct TickImbalanceBars {
    pub bars: Vec<Bar>,
    initial_t: i32,
    b_ts: Vec<i8>,
    t_vals: Vec<i32>,
    theta: i32,
    theta_expectation: i32,
    last_trade: Option<Trade>,
    current_bar_idx: usize,
    count: i32,
    count_expectation: i32,
    state: GeneratorState,
}

impl TickImbalanceBars {
    pub fn new(initial_t: i32) -> TickImbalanceBars
    {
        let b_ts: Vec<i8> = vec![];
        let t_vals: Vec<i32> = vec![];
        let theta_expectation = initial_t;

        TickImbalanceBars {
            bars: vec![],
            initial_t,
            b_ts,
            t_vals,
            theta: 0,
            theta_expectation,
            last_trade: None,
            current_bar_idx: 0,
            count_expectation: initial_t,
            count: 0,
            state: GeneratorState::Empty
        }
    }

    // [bt = 1] is the unconditional probability that a tick is classified as a buy,
    fn implied_imbalance(&self) -> f64 {
        let b_ts = self.b_ts.iter();
        let alpha = 2.0 / (1.0 + self.count_expectation as f64);
        ewma(b_ts, alpha).abs()
    }

    fn expected_t_value(&self) -> i32 {
        let alpha = 2.0 / 21.0;
        return ewma(self.t_vals.iter(), alpha) as i32;
    }

    fn update_metrics(&mut self, trade: &Trade) {
        let last_trade = self.last_trade.as_ref().unwrap();
        let price_diff = trade.price - last_trade.price;
        let b_t: i8;
            if price_diff == 0.0 {
                b_t = *self.b_ts.last().unwrap_or(&1);
            } else {
                b_t = (price_diff.abs() / price_diff) as i8;
            }
        self.b_ts.push(b_t);
        self.theta += b_t as i32;
    }

    fn new_bar(&mut self, trade: &Trade) -> &Bar {
        self.bars.push(Bar::new(&trade));
        self.t_vals.push(self.count);
        self.current_bar_idx = self.bars.len() - 1;
        self.theta = 0;
        self.count = 0;
        self.count_expectation = self.expected_t_value();
        println!("count_expectation = {}", self.count_expectation);
        let implied_imbalance = self.implied_imbalance();
        println!("expected_imbalance = {}", implied_imbalance);

        self.theta_expectation = (self.count_expectation as f64 *
            implied_imbalance) as i32;
        println!("expected_theta = {}", self.theta_expectation);
        self.bars.last().unwrap()
    }

}

impl BarGenerator for TickImbalanceBars {
    fn process_trade(&mut self, trade: Trade) -> Option<&Bar> {
        self.state = match self.state {
            GeneratorState::Empty => {
                self.bars.push(Bar::new(&trade));
                GeneratorState::InitialBar
            },
            GeneratorState::InitialBar => {
                self.update_metrics(&trade);
                if self.count >= self.initial_t {
                    self.new_bar(&trade);
                    GeneratorState::Working
                } else {
                    GeneratorState::InitialBar
                }
            },
            GeneratorState::Working => {
                self.update_metrics(&trade);
                if self.theta.abs() >= self.theta_expectation {
                    println!("! reached expectation theta = {} @ t={}", self.theta_expectation,
                             self.count);
                    println!("- Bar found: {:?}", self.bars.last().unwrap());
                    self.new_bar(&trade);
                }
                GeneratorState::Working
            },
        };
        let bar = self.bars.last_mut().unwrap();
        bar.next(&trade);
        self.last_trade = Some(trade);
        self.count += 1;
        None
    }
}

/// Calculate the exponential weighted moving average for a vector of numbers, with a smoothing
/// factor `alpha` between 0 and 1. A higher `alpha` discounts older observations faster.
pub fn ewma<'a, T, I>(mut samples: I, alpha: f64) -> f64
    where T: ToPrimitive + 'a,
          I: Iterator<Item = &'a T>
{
    let first = samples.next().map_or(0.0, |v| v.to_f64().unwrap());
    samples.map(|v| v.to_f64().unwrap())
           .fold(first, |avg, sample| alpha * sample + (1.0 - alpha) * avg)
}

