# Transaction Engine Toy
Attempt at writing a toy transaction engine, to better my rust understanding.

The program can be run with the following arguments: `cargo run -- file` and will output the result on stdout.
The program will output logs on stderr. It is possible to increase the verbosity with the -v (-vv) command line argument.

It uses clap, serde, csv, log and stderrlog as dependencies.

# Assumptions
We assume that the disputes/resolves/chargebacks `ClientID` value refer to the initial client ID from the initial transaction.
If the `ClientID` differs from the original's transaction ID, we discard the transaction as being faulty.

We assume that all the input amounts are positive. If a transaction amount is negative, we automatically convert it to zero.

We assume that all disputes are on deposits only. We skip any dispute that is not on a deposit transaction.
While it would be possible to support disputes on withdrawals, the current output format doesn't work for it since it only
has one "held" column. We would need two columns "deposit_held" and "withdrawal_held" to properly represent withdrawal disputes.