use nostr_sdk::prelude::*;
use otrta_wallet::nip60::{utils, Nip60Wallet};
use std::io::{self, Write};

const TEST_CASHU_TOKEN: &str = "cashuBo2FteCJodHRwczovL21pbnQubWluaWJpdHMuY2FzaC9CaXRjb2luYXVjc2F0YXSBomFpSABQBVDwSUFGYXCCpGFhAmFzeEBiNGNiMGFmZjIyOWFlMjA2MjAyOGI5NTFmOWU0MDdlZDY2NDdmZjc4NzNjMTdjNmVkODU2MTQxMjk5ODZkM2NlYWNYIQIesZwkmxDYwTcObiv9KlnD_hYAssHDc3scsqi0b1OSjmFko2FlWCB3VIzM-hk4v4v5A-vpeWXKup2etP3nxvAq5Bu6u0zgP2FzWCCBRP7Rv1wtdXsYh8qAjpVEmmQ1xLC6hV8tHwRLLgF-8mFyWCBYcEROJrzJQdZ-UinHuEXOBG6fIVHNHlRO_YDbb3tPNKRhYQFhc3hAMWVmNzY2Mzc5MWU2OGYwY2FkNzVlMTA4MTRkNThhMTNkNDYwYTkwY2U1YTgyN2QyNDk3NGI2MDYwYmNhZDIwYmFjWCED6iNPLgzAyft0CbOXQkhrN_1iITCa98GJTUoKyVOl7RZhZKNhZVggJ-4B2XimIkjhrgW3-dj4oqIULbd5rgk27N06g_iAeAJhc1ggwrmtHwNeQXucKNxTMUVwShU0rqIFAH2GLMdSE8s0QZ9hclggvylfw3OJ__VBMq8VrPTJlTdQpxv5s2uuQ1U9LX00f6c";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔥 NIP-60 Cashu Wallet State Manager Demo");
    println!("========================================");
    println!("This demo implements NIP-60: Cashu wallet state management on Nostr");
    println!("It only manages proofs as Nostr events - no actual cashu operations!");

    let keys = Keys::generate();
    println!("\nGenerated Nostr keys:");
    println!("  Public key: {}", keys.public_key());
    println!("  Secret key: {}", keys.secret_key().display_secret());

    let relays = vec!["wss://relay.damus.io", "wss://relay.snort.social"];

    // Generate wallet privkey for P2PK ecash
    let wallet_privkey = utils::generate_wallet_privkey();
    let mints = vec!["https://mint.minibits.cash/Bitcoin".to_string()];

    println!("\n🌐 Connecting to relays...");
    let wallet = Nip60Wallet::new(keys, relays, wallet_privkey, mints)
        .await
        .map_err(|e| format!("Failed to create wallet: {:?}", e))?;
    println!("✅ NIP-60 wallet state manager initialized successfully!");

    // Test cashu token parsing using CDK
    println!("\n🔍 Testing cashu token parsing with CDK nut00:");
    match wallet.parse_cashu_token(TEST_CASHU_TOKEN) {
        Ok(parsed_token) => {
            println!("✅ Token parsed successfully using CDK nut00!");
            let total_amount: u64 = parsed_token
                .proofs()
                .iter()
                .map(|proof| proof.amount.to_string().parse::<u64>().unwrap())
                .sum();
            println!("  Mint: {:?}", parsed_token.mint_url().unwrap());
            println!("  Total amount: {} sats", total_amount);
            println!("  Proofs: {}", parsed_token.proofs().len());
            if let Some(memo) = parsed_token.memo() {
                println!("  Memo: {}", memo);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse token: {:?}", e);
        }
    }

    println!("\n📊 Current wallet balance from Nostr events:");
    match wallet.calculate_balance().await {
        Ok(balance) => println!("  Balance: {} sats", balance),
        Err(e) => println!("  Error getting balance: {:?}", e),
    }

    println!("\n📋 Wallet configuration:");
    let config = wallet.get_config();
    println!("  Mints: {:?}", config.mints);
    println!("  P2PK privkey configured: {}", !config.privkey.is_empty());

    // Interactive operations
    loop {
        println!("\n🚀 What would you like to do?");
        println!("1. Record received tokens (from cashu token string)");
        println!("2. View token events from Nostr");
        println!("3. Check balance from Nostr events");
        println!("4. Get spending history from Nostr");
        println!("5. Parse custom token string");
        println!("6. Get wallet stats");
        println!("7. Show example NIP-60 operations");
        println!("8. Exit");
        print!("Enter your choice (1-8): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => {
                print!("Enter cashu token to record: ");
                io::stdout().flush()?;
                let mut token_input = String::new();
                io::stdin().read_line(&mut token_input)?;

                let token = token_input.trim();
                if !token.is_empty() {
                    println!("📥 Recording received token in Nostr events...");
                    match wallet.record_receive(token).await {
                        Ok(amount) => {
                            println!("✅ Recorded {} sats received!", amount);
                            println!("📝 Created token event (kind 7375) and spending history (kind 7376)");
                        }
                        Err(e) => println!("❌ Failed to record: {:?}", e),
                    }
                } else {
                    println!("❌ Empty token");
                }
            }
            "2" => match wallet.fetch_token_events().await {
                Ok(events) => {
                    if events.is_empty() {
                        println!("📭 No token events found");
                    } else {
                        println!("📄 Token events ({}) from Nostr:", events.len());
                        for (i, event) in events.iter().enumerate() {
                            let total: u64 = event.data.proofs.iter().map(|p| p.amount).sum();
                            println!(
                                "  {}. Event {} - {} sats from {} ({} proofs)",
                                i + 1,
                                event.id.to_hex()[..8].to_string() + "...",
                                total,
                                event.data.mint,
                                event.data.proofs.len()
                            );
                            if !event.data.del.is_empty() {
                                println!("     Replaced events: {:?}", event.data.del);
                            }
                        }
                    }
                }
                Err(e) => println!("❌ Error fetching events: {:?}", e),
            },
            "3" => match wallet.calculate_balance().await {
                Ok(balance) => println!("💰 Current balance from Nostr: {} sats", balance),
                Err(e) => println!("❌ Error getting balance: {:?}", e),
            },
            "4" => match wallet.get_spending_history().await {
                Ok(history) => {
                    if history.is_empty() {
                        println!("📜 No spending history");
                    } else {
                        println!(
                            "📜 Spending history ({} entries) from Nostr:",
                            history.len()
                        );
                        for (i, entry) in history.iter().enumerate() {
                            println!("  {}. {} {} sats", i + 1, entry.direction, entry.amount);
                            for event_ref in &entry.events {
                                println!("     Event: {} ({})", event_ref.1, event_ref.3);
                            }
                        }
                    }
                }
                Err(e) => println!("❌ Error getting history: {:?}", e),
            },
            "5" => {
                print!("Enter cashu token to parse: ");
                io::stdout().flush()?;
                let mut token_input = String::new();
                io::stdin().read_line(&mut token_input)?;

                let token = token_input.trim();
                if !token.is_empty() {
                    match wallet.parse_cashu_token(token) {
                        Ok(parsed) => {
                            println!("✅ Token parsed successfully!");
                            let total: u64 = parsed
                                .proofs()
                                .iter()
                                .map(|p| p.amount.to_string().parse::<u64>().unwrap())
                                .sum();
                            println!("  Total amount: {} sats", total);
                            println!("  Mint: {}", parsed.mint_url().unwrap());
                            println!("  Proofs: {}", parsed.proofs().len());
                        }
                        Err(e) => println!("❌ Failed to parse: {:?}", e),
                    }
                } else {
                    println!("❌ Empty token");
                }
            }
            "6" => match wallet.get_stats().await {
                Ok(stats) => {
                    println!("📊 Wallet Statistics:");
                    println!("  Balance: {} sats", stats.balance);
                    println!("  Token events: {}", stats.token_events);
                    println!("  Configured mints: {}", stats.mints.len());
                    for (i, mint) in stats.mints.iter().enumerate() {
                        println!("    {}. {}", i + 1, mint);
                    }
                }
                Err(e) => println!("❌ Error getting stats: {:?}", e),
            },
            "7" => {
                println!("\n📚 NIP-60 Implementation Features:");
                println!("==================================");
                println!("✅ Wallet Configuration Events (kind 17375):");
                println!("   - Encrypted wallet privkey for P2PK ecash");
                println!("   - List of configured mints");
                println!("");
                println!("✅ Token Events (kind 7375):");
                println!("   - Encrypted cashu proofs storage");
                println!("   - Rollover tracking with 'del' field");
                println!("   - NIP-09 deletion for spent tokens");
                println!("");
                println!("✅ Spending History Events (kind 7376):");
                println!("   - Encrypted transaction history");
                println!("   - Event references with markers");
                println!("   - Direction tracking (in/out)");
                println!("");
                println!("✅ CDK Integration:");
                println!("   - Official nut00 token parsing");
                println!("   - Proof validation");
                println!("   - Amount calculation");
                println!("");
                println!("💡 This implements the complete NIP-60 specification!");
                println!("🔗 Spec: https://nips.nostr.com/60");
            }
            "8" => {
                println!("👋 Goodbye! Your wallet state is preserved in Nostr events.");
                println!("🌐 You can restore it anytime with your Nostr keys.");
                break;
            }
            _ => println!("❌ Invalid choice. Please enter 1-8."),
        }
    }

    Ok(())
}
