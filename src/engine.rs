use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    r#type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<f32>,
    #[serde(skip_deserializing)]
    disputed: bool,
}

#[derive(Debug, Serialize)]
struct Account {
    client: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}

impl Account {
    fn new(client: u16) -> Self {
        Account {
            client,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    fn deposit(&mut self, amount: f32) {
        self.available += amount;
        self.compute_total();
    }

    fn withdraw(&mut self, amount: f32) {
        if self.available >= amount {
            self.available -= amount;
            self.compute_total();
        }
    }

    fn dispute(&mut self, amount: f32) {
        self.available -= amount;
        self.held += amount;
    }

    fn resolve(&mut self, amount: f32) {
        self.available += amount;
        self.held -= amount;
    }

    fn chargeback(&mut self, amount: f32) {
        self.held -= amount;
        self.locked = true;
        self.compute_total();
    }

    fn compute_total(&mut self) {
        self.total = self.available + self.held;
    }
}

#[derive(Debug)]
pub struct Engine<V>
where
    V: io::Read,
{
    clients: HashMap<u16, Account>,
    saved_transactions: HashMap<u32, Transaction>,
    reader: Reader<V>,
}

impl Engine<BufReader<File>> {
    pub fn from_buf_reader(input_path: &Path) -> Self {
        let file = File::open(input_path).unwrap();
        let reader = csv::ReaderBuilder::new().from_reader(BufReader::new(file));
        Engine {
            clients: HashMap::new(),
            saved_transactions: HashMap::new(),
            reader,
        }
    }
}

impl<V> Engine<V>
where
    V: io::Read,
{
    pub fn process(&mut self) {
        for result in self.reader.deserialize() {
            let transaction: Transaction = result.unwrap();
            if !self.clients.contains_key(&transaction.client) {
                self.clients
                    .insert(transaction.client, Account::new(transaction.client));
            }
            match transaction.r#type {
                TransactionType::Deposit => {
                    self.clients
                        .get_mut(&transaction.client)
                        .unwrap()
                        .deposit(transaction.amount.unwrap());
                    self.saved_transactions.insert(transaction.tx, transaction);
                }
                TransactionType::Withdrawal => {
                    self.clients
                        .get_mut(&transaction.client)
                        .unwrap()
                        .withdraw(transaction.amount.unwrap());
                    self.saved_transactions.insert(transaction.tx, transaction);
                }
                TransactionType::Dispute => {
                    match self.saved_transactions.get_mut(&transaction.tx) {
                        Some(disputed_transaction) => {
                            disputed_transaction.disputed = true;
                            self.clients
                                .get_mut(&transaction.client)
                                .unwrap()
                                .dispute(disputed_transaction.amount.unwrap());
                        }
                        None => {}
                    }
                }
                // I am assuming that a dispute has two possible resolutions,
                // a resolve or a chargeback, and such when a resolve or chargeback
                // occurs, the transaction is not longer being disputed.
                TransactionType::Resolve => {
                    match self.saved_transactions.get_mut(&transaction.tx) {
                        Some(resolved_transaction) => {
                            if resolved_transaction.disputed {
                                self.clients
                                    .get_mut(&transaction.client)
                                    .unwrap()
                                    .resolve(resolved_transaction.amount.unwrap());
                            }
                            resolved_transaction.disputed = false;
                        }
                        None => {}
                    }
                }
                TransactionType::Chargeback => {
                    match self.saved_transactions.get_mut(&transaction.tx) {
                        Some(charged_back_transaction) => {
                            if charged_back_transaction.disputed {
                                self.clients
                                    .get_mut(&transaction.client)
                                    .unwrap()
                                    .chargeback(charged_back_transaction.amount.unwrap());
                            }
                            charged_back_transaction.disputed = false;
                        }
                        None => {}
                    }
                }
            }
        }
    }

    pub fn output_clients(&mut self) {
        let mut writer = Writer::from_writer(io::stdout());
        self.clients
            .iter()
            .for_each(|(_, client)| writer.serialize(client).unwrap());
        writer.flush().unwrap();
    }
}
