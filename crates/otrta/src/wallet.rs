use ecash_402_wallet::wallet::CashuWalletClient;

use crate::db::{
    transaction::{add_transaction, TransactionDirection},
    Pool,
};

#[derive(Debug)]
pub enum SendAmoundResponse {
    Error(String),
}

pub async fn send_with_retry(
    wallet: &CashuWalletClient,
    amount: i64,
    retries: Option<i32>,
) -> Result<String, SendAmoundResponse> {
    for _ in 1..if let Some(retry_count) = retries {
        retry_count
    } else {
        3
    } {
        if let Ok(token_result) = wallet.send(amount as u64).await {
            return Ok(token_result);
        }
    }

    Err(SendAmoundResponse::Error(
        "wallet: failed to generate token".to_string(),
    ))
}

pub async fn finalize_request(
    db: &Pool,
    wallet: &CashuWalletClient,
    token_send: &str,
    sats_send: i64,
    token_received: &str,
) {
    if let Ok(res) = wallet.receive(token_received).await {
        add_transaction(
            &db,
            token_send,
            &sats_send.to_string(),
            TransactionDirection::Outgoing,
        )
        .await
        .unwrap();

        add_transaction(
            &db,
            token_received,
            &res.to_string(),
            TransactionDirection::Incoming,
        )
        .await
        .unwrap();
    }
}
