use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// A single transaction: sender → receiver, some amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,   // wallet address (or "COINBASE" for mining reward)
    pub to: String,
    pub amount: f64,
    pub timestamp: i64,
    pub signature: String, // hex-encoded signature (or "COINBASE")
}

/// A block in the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub miner: String,
}

impl Block {
    /// Create and mine a new block
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        miner: String,
        difficulty: usize,
    ) -> Self {
        let timestamp = Utc::now().timestamp_millis();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            miner,
        };
        block.mine(difficulty);
        block
    }

    /// Compute the SHA-256 hash of the block
    pub fn compute_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.index,
            self.timestamp,
            serde_json::to_string(&self.transactions).unwrap_or_default(),
            self.previous_hash,
            self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Proof-of-Work: increment nonce until hash starts with `difficulty` zeros
    fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        loop {
            self.hash = self.compute_hash();
            if self.hash.starts_with(&target) {
                break;
            }
            self.nonce += 1;
        }
    }

    /// Verify this block's hash is valid
    pub fn is_valid_hash(&self, difficulty: usize) -> bool {
        let target = "0".repeat(difficulty);
        self.hash == self.compute_hash() && self.hash.starts_with(&target)
    }
}

/// The Blockchain: an ordered list of blocks + a mempool of pending transactions
#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    pub mining_reward: f64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut bc = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty: 3, // leading zeros required
            mining_reward: 50.0,
        };
        bc.add_genesis_block();
        bc
    }

    /// Create block #0 (genesis) with no real transactions
    fn add_genesis_block(&mut self) {
        let genesis = Block {
            index: 0,
            timestamp: 0,
            transactions: vec![],
            previous_hash: "0".repeat(64),
            hash: "GENESIS".to_string(),
            nonce: 0,
            miner: "GENESIS".to_string(),
        };
        self.chain.push(genesis);
    }

    /// The tip of the chain
    pub fn latest_block(&self) -> &Block {
        self.chain.last().expect("chain always has genesis block")
    }

    /// Add a transaction to the mempool
    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    /// Mine all pending transactions into a new block; reward the miner
    pub fn mine_pending_transactions(&mut self, miner_address: &str) -> Block {
        // Coinbase reward transaction
        let reward_tx = Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            from: "COINBASE".to_string(),
            to: miner_address.to_string(),
            amount: self.mining_reward,
            timestamp: Utc::now().timestamp_millis(),
            signature: "COINBASE".to_string(),
        };

        let mut txs = self.pending_transactions.clone();
        txs.push(reward_tx);

        let block = Block::new(
            self.chain.len() as u64,
            txs,
            self.latest_block().hash.clone(),
            miner_address.to_string(),
            self.difficulty,
        );

        self.chain.push(block.clone());
        self.pending_transactions.clear();
        block
    }

    /// Balance = sum of incoming amounts − sum of outgoing amounts across all confirmed blocks
    pub fn balance_of(&self, address: &str) -> f64 {
        let mut balance = 0.0_f64;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.to == address {
                    balance += tx.amount;
                }
                if tx.from == address {
                    balance -= tx.amount;
                }
            }
        }
        balance
    }

    /// Walk the chain and validate every block
    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.hash != current.compute_hash() {
                return false;
            }
            if current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }
}
