# tx_engine

Consumes transactions csv file, processes different kinds of transactions and produces accounts for each client.

Ex:

- transactions.csv

```
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
dispute, 1, 1,
resolve, 1, 1,
chargeback, 1, 2,
```

output:

- accounts.csv

```
client,available,held,total,locked
2,2.0000,0.0000,2.0000,false
1,1.5000,0.0000,1.5000,false
```

## Assumptions

1. Only dipustable transaction is Deposit.

## Build

```
cargo build
```

## Test

```
cargo test
```

## Run

```
cargo run -- transactions.csv > accounts.csv
```

## The core idea

Each transaction implements the `Tx` trait which contains the process method. Process method is invoked on each transaction and internal app state is computed, eventually this state is flushed out to stdout.

This app uses visitor pattern to execute code on each transaction. `TxProcessor` trait declares the contract for processing each transaction, `TxProcessorImpl` which is the implementation of `TxProcessor` contains the transaction handling logic for each kind of transaction.
