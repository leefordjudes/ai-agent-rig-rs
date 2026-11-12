use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    // Reads the OPENAI_API_KEY environment variable.
    // let model = OpenAI::from_env()?.completion(openai::GPT_5_5);
    let model = OpenAI::from_env()?.completion("apinex");

    // Build an agent from a model and a system prompt.
    let agent = AgentBuilder::new(model)
        .preamble("You are a helpful assistant.")
        .build();

    // Send a prompt and await the response.
    let answer = agent.prompt("Who are you?").await?;
    println!("{}", answer.output());

    Ok(())
}
