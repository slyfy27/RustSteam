//! Authentication example for rust-steam
//! 
//! This example demonstrates how to authenticate with Steam using the rust-steam library.
//! It shows both credential-based authentication and the callback system.

use rust_steam::prelude::*;
use std::env;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get credentials from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <username> <password>", args[0]);
        println!("Example: {} myusername mypassword", args[0]);
        return Ok(());
    }

    let username = &args[1];
    let password = &args[2];

    println!("Rust Steam Authentication Example");
    println!("==================================");
    println!("Username: {}", username);
    println!();

    // Create Steam client with default configuration
    let client = SteamClient::new().await?;
    
    // Get callback manager for subscribing to events
    let callback_manager = client.get_callback_manager();
    
    // Subscribe to connection events
    let _connected_sub = callback_manager.subscribe(|callback: ConnectedCallback| {
        println!("✓ Connected to Steam!");
    });
    
    let _disconnected_sub = callback_manager.subscribe(|callback: DisconnectedCallback| {
        if callback.is_user_initiated() {
            println!("✓ Disconnected from Steam (user initiated)");
        } else {
            println!("✗ Disconnected from Steam (unexpected): {:?}", callback.reason);
        }
    });
    
    let _logged_on_sub = callback_manager.subscribe(|callback: LoggedOnCallback| {
        match callback.get_result() {
            EResult::OK => {
                println!("✓ Successfully logged on to Steam!");
                println!("  Steam ID: {}", callback.steam_id);
                println!("  Account Name: {}", callback.account_name);
                println!("  Cell ID: {}", callback.cell_id);
                if let Some(email) = &callback.email_domain {
                    println!("  Email Domain: {}", email);
                }
                if callback.vac_banned {
                    println!("  ⚠️  Account is VAC banned");
                }
            }
            result => {
                println!("✗ Failed to log on: {:?}", result);
                if callback.get_extended_result() != EResult::OK {
                    println!("  Extended result: {:?}", callback.get_extended_result());
                }
            }
        }
    });
    
    let _logged_off_sub = callback_manager.subscribe(|callback: LoggedOffCallback| {
        println!("✓ Logged off from Steam: {:?}", callback.get_result());
    });

    // Connect to Steam
    println!("Connecting to Steam...");
    client.connect().await?;

    // Prepare authentication details
    let mut auth_details = AuthSessionDetails {
        username: username.to_string(),
        password: password.to_string(),
        persistent_session: false,
        guard_data: None,
        authenticator: Some(Box::new(ConsoleAuthenticator)),
        website_id: "Client".to_string(),
        device_friendly_name: "Rust Steam Client".to_string(),
        platform_type: EAuthTokenPlatformType::default(),
    };

    println!("Starting authentication...");
    
    // Authenticate with Steam
    match client.authenticate(auth_details).await {
        Ok(poll_result) => {
            println!("✓ Authentication successful!");
            println!("  Account: {}", poll_result.account_name);
            
            if poll_result.hadTwoFactorAuth {
                println!("  Two-factor authentication was required");
            }
            
            if let Some(guard_data) = &poll_result.new_guard_data {
                println!("  New guard data received (should be stored for future use)");
            }

            // Log on to Steam using the authentication tokens
            println!("Logging on to Steam...");
            let mut logon_details = LogOnDetails::new();
            logon_details.set_username(poll_result.account_name.clone());
            logon_details.set_access_token(poll_result.access_token);
            logon_details.set_refresh_token(poll_result.refresh_token);
            logon_details.set_login_id(149); // Unique login ID

            client.log_on(logon_details).await?;

            // Run the client for a short period to process callbacks
            println!("Processing events...");
            let start_time = std::time::Instant::now();
            while start_time.elapsed() < std::time::Duration::from_secs(5) {
                callback_manager.run_wait_callbacks(1000).await;
                
                if !client.is_running().await {
                    break;
                }
            }

            // Log off
            println!("Logging off...");
            client.log_off().await?;

            // Process final callbacks
            callback_manager.run_wait_callbacks(1000).await;
        }
        Err(e) => {
            println!("✗ Authentication failed: {}", e);
            return Err(e.into());
        }
    }

    // Disconnect
    println!("Disconnecting...");
    client.disconnect().await?;

    // Final callback processing
    callback_manager.run_wait_callbacks(500).await;

    println!();
    println!("Example completed successfully!");
    
    Ok(())
}

/// Helper function to handle two-factor authentication
async fn handle_two_factor_auth() -> Result<String, Box<dyn std::error::Error>> {
    println!("Two-factor authentication required.");
    println!("Please enter your authentication code:");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}

/// Helper function to handle email verification
async fn handle_email_verification() -> Result<String, Box<dyn std::error::Error>> {
    println!("Email verification required.");
    println!("Please check your email and enter the verification code:");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}