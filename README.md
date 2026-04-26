# Sugarscape

A Rust version of the simulation described here: https://jasss.soc.surrey.ac.uk/12/1/6/appendixB/EpsteinAxtell1996.html 


## How to run

First you need to run the simulation:

```
cargo run --release --bin sugarscape-sim -- --config config.yaml --output output.parquet
```

Then you can run the visualization:

```
cargo run --release --bin sugarscape-viz -- --input output.parquet
```