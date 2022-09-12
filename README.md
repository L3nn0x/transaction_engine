# Transaction Engine Toy
Attempt at writing a toy transaction engine, to better my rust understanding.

The program can be run with the following arguments: `cargo run -- file` and will output the result on stdout.
The program will output logs on stderr. It is possible to increase the verbosity with the -v (-vv) command line argument.

It uses clap, serde, csv, log and stderrlog as dependencies.