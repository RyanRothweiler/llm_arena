#![allow(dead_code)]

use kalosm::language::*;
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

const DEBUG: bool = false;

pub mod ai_error;

pub use ai_error::AIError;

// First, derive an efficient parser for your structured data
#[derive(Parse, Clone, Debug, Schema, Deserialize)]
enum Shape {
    None,
    Circle,
    Square,
    Triangle,
}

#[derive(Parse, Clone, Debug, Schema, Deserialize)]
pub struct LevelGenResponse {
    valid: bool,
    error: String,
    shape: Shape,
    count: i32,
}

pub async fn test_gen(prompt: &str) -> Result<LevelGenResponse, AIError> {
    println!("Start classification");

    let mut llm = OpenAICompatibleChatModel::builder()
        .with_gpt_4o_mini()
        .build();

    println!("Model started");

    let schema: String = LevelGenResponse::schema().to_string();
    let task =
        llm.task(&format!("You classify the user's description of a shape. Only include the properties field. Respond in formatted json following this schema {}. ", schema));

    println!("Running classification");
    let response_text = task(prompt).await?;

    let trimmed = response_text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if DEBUG {
        println!("llm response {:?}", trimmed)
    }

    let response: LevelGenResponse = serde_json::from_str(&trimmed)?;
    println!("Successful classification");

    Ok(response)
}
