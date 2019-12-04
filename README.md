# id-bars: Information-driven bar generator

id-bars is a tool for generating information driven bars from CVS trades files.

## What are information-driven bars?

Introduced by Marcos Lopez de Prado in Advances in Financial Machine Learning,
information-driven bars exhibit superiod statistical properties (namely
distribution closed to normal) when compared with standard time-driven bars.

Lopez de Prado introduces several types of bars:

* Standard bars:
  * Tick bars
  * Volume bars
  * Dollar bars
* Information-driven bars:
  * Tick-imbalance bars
  * Volume/Dollar imbalance bars
  * Tick run bars
  * Volume/dollar run bars

Standard bars are sampled based on predefined conditions, number of tick bars,
accumulated traded volume or accumulated traded dollar value.

Information-driven bars are sampled in accordance with the information arriving
to the market (information in the market microstructure). This means sampling
more frequently when new information is detected in the market.

## Usage

`id-bars` takes input in form of CSV file containing a list of trades with
following columns: timestamp, price and volume.

Ready-to-use CSV files can be downloaded from
(bitcoincharts)[http://api.bitcoincharts.com/v1/csv/].

```
cargo run -- -m volume coinbaseUSD.csv volume-bars.csv
```
