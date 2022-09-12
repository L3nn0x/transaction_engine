use clap::Parser;
use log::error;

mod common_types;
mod transaction_engine;
mod account;
mod parser;
mod output;

#[derive(Parser)]
#[clap(author, version, about, long_about=None)]
struct Args {
    /// Path to input file
    #[clap(value_parser)]
    path: String,

    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8
}

fn main() {
    let args = Args::parse();

    stderrlog::new().module(module_path!()).verbosity(args.verbose as usize).init().unwrap();

    let mut transaction_engine = transaction_engine::TransactionEngine::new();

    if let Ok(transactions) = parser::parse_csv(args.path) {
        for transaction in transactions {
            transaction_engine.process_transaction(transaction);
        }

        output::output_accounts(transaction_engine.get_accounts());
    } else {
        error!("Failed to parse the input file");
        return;
    }
}

