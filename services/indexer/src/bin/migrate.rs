#!/usr/bin/env cargo

//! Database migration utility for Cloak Indexer
//! 
//! Run with: cargo run --bin migrate

use cloak_indexer::{Config, IndexerError};
use cloak_indexer::database::Database;

#[tokio::main]
async fn main() -> Result<(), IndexerError> {
    // Initialize basic logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::from_env()?;
    
    tracing::info!("Starting database migrations");
    
    // Connect to database
    let database = Database::new(&config).await?;
    
    // Run migrations
    database.migrate().await?;
    
    tracing::info!("Database migrations completed successfully");
    
    Ok(())
}
