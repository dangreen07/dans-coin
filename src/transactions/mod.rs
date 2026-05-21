use ed25519_dalek::{Signer, SigningKey};
use sha3::Digest;

#[derive(Clone)]
pub struct Recipient {
    address: [u8; 32],
    amount: f64,
}

impl Recipient {
    pub fn new(address: [u8; 32], amount: f64) -> Self {
        Recipient { address, amount }
    }
}

#[derive(Clone)]
pub struct TransactionData {
    sender_address: [u8; 32],
    input_amount: f64,
    fee: f64,
    recipients: Vec<Recipient>,
}

impl TransactionData {
    pub fn format(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.sender_address);
        result.extend(self.input_amount.to_le_bytes());
        result.extend(self.fee.to_le_bytes());
        result.extend(self.recipients.len().to_le_bytes());
        for recipient in self.recipients.iter() {
            result.extend(recipient.address);
            result.extend(recipient.amount.to_le_bytes());
        }
        return result;
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let sender_address: [u8; 32] = bytes[0..32].try_into().unwrap();
        let input_amount: f64 = f64::from_le_bytes(bytes[32..40].try_into().unwrap());
        let fee: f64 = f64::from_le_bytes(bytes[40..48].try_into().unwrap());
        let recipients_length: usize = usize::from_le_bytes(bytes[48..56].try_into().unwrap());
        let mut recipients = Vec::new();
        for i in 0..recipients_length {
            let recipient_address: [u8; 32] =
                bytes[56 + i * 32..56 + (i + 1) * 32].try_into().unwrap();
            let recipient_amount: f64 = f64::from_le_bytes(
                bytes[56 + (i + 1) * 32..56 + (i + 2) * 32]
                    .try_into()
                    .unwrap(),
            );
            recipients.push(Recipient {
                address: recipient_address,
                amount: recipient_amount,
            });
        }
        return TransactionData {
            sender_address: sender_address,
            input_amount: input_amount,
            fee: fee,
            recipients: recipients,
        };
    }
}

#[derive(Clone)]
pub struct Transaction {
    pub transaction_data: TransactionData,
    pub hash: [u8; 64],
    pub signature: [u8; 64],
}

pub fn create_transaction(
    sender_address: [u8; 32],
    input_amount: f64,
    fee: f64,
    recipients: Vec<Recipient>,
    secret_key: SigningKey,
) -> Transaction {
    let transaction = TransactionData {
        sender_address: sender_address,
        input_amount: input_amount,
        fee: fee,
        recipients: recipients,
    };
    let mut hasher = sha3::Keccak512::new();
    hasher.update(transaction.format());
    let hashed_transaction: [u8; 64] = hasher.finalize().into();
    let signature = secret_key.sign(&hashed_transaction).to_bytes();
    Transaction {
        transaction_data: transaction,
        hash: hashed_transaction,
        signature: signature,
    }
}
