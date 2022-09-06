# transaction-engine

## Completeness

The transaction engine handles all 5 times of transactions:
* Deposit
* Withdrawal
* Dispute
* Resolve
* Chargeback

## Correctness

The correctness of the engine was tested against sample data (transactions.csv). And we count on the Rust type system and ownership to ensure correctness as well.

We could furthermore make sure that the behaviour is correct by writing some unit tests for all the transactions types.

## Safety and Robustness

Safety is ensured by the Rust ownership and type system, and there is no use of unsafe.

Errors like a transaction being under dispute not existing are not specifically handled. If this was a production application, those errors could be logged and the results returned to the clients.

## Efficiency

The engine is efficient in terms of memory since it uses a `BufReader` to stream the dataset rather the loading it all up front. We could even improve this by only allocating the size of the CSV line once.

There are also improvements that could be made in terms on time, by using a producer-consumer async pattern:
* For this however, all the transactions need unique UUIDs, so they can be stored in DB (either in memory or not), since now only deposits and withdrawals have unique UUIDs.
* If the condition above is satisfied, we could have a process that continuously reads transactions, stores them, and puts their UUIDs in a queue.
* The queue can then be processed by the engine in order
* These two processes can run asynchronously, so the engine could be used in a server as well
