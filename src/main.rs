use clap::Parser;

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
    path: String
}

fn main() {
    let args = Args::parse();

    let mut transaction_engine = transaction_engine::TransactionEngine::new();

    if let Ok(transactions) = parser::parse_csv(args.path) {
        for transaction in transactions {
            transaction_engine.process_transaction(transaction);
        }

        output::output_accounts(transaction_engine.get_accounts());
    } else {
        return;
    }
}

