use rig::prelude::*;
use rig::providers::openai::OpenAI;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const CONTEXT_PATH: &str = ".omniroute/context.txt";
// const MAX_HISTORY_SIZE: usize = 64 * 1024; // 64 KiB per turn file
// const MAX_TURNS: usize = 40;
// const MAX_HISTORY_SIZE: usize = 16 * 1024; // 16 KiB per turn file
// const MAX_TURNS: usize = 10;
const MAX_HISTORY_SIZE: usize = 8 * 1024; // 8 KiB per turn file
const MAX_TURNS: usize = 5;

struct ChatSession {
    history: Vec<String>,
}

impl ChatSession {
    fn new() -> Self {
        let mut history = Vec::new();
        if let Ok(content) = fs::read_to_string(CONTEXT_PATH) {
            for chunk in content.split("\n-----TURN-----\n") {
                if !chunk.is_empty() {
                    history.push(chunk.to_string());
                }
            }
        }
        Self { history }
    }

    fn push_user(&mut self, user_msg: &str) -> String {
        let id = self.history.len();
        let wrapper = format!("<user id=\"{}\">\n{user_msg}\n</user>", id);
        self.history.push(wrapper.clone());
        self.maybe_truncate();
        self.persist();
        wrapper
    }

    fn push_assistant(&mut self, assistant_msg: &str) -> String {
        let id = self.history.len();
        let wrapper = format!("<assistant id=\"{}\">\n{assistant_msg}\n</assistant>", id);
        self.history.push(wrapper.clone());
        self.maybe_truncate();
        self.persist();
        wrapper
    }

    fn maybe_truncate(&mut self) {
        if self.history.len() > MAX_TURNS {
            self.history.drain(0..(self.history.len() - MAX_TURNS));
        }
        let mut total: usize = 0;
        let mut to_remove: usize = 0;
        for msg in &self.history {
            total = total.saturating_add(msg.len());
            if total > MAX_HISTORY_SIZE {
                to_remove += 1;
            }
        }
        if to_remove > 0 {
            self.history.drain(0..to_remove);
        }
    }

    fn build_context(&self, newest_user: &str) -> String {
        let joined = self.history.join("\n");
        if joined.is_empty() {
            format!("<user id=\"0\">\n{newest_user}\n</user>")
        } else {
            format!(
                "{joined}\n\n<user id=\"{}\">\n{newest_user}\n</user>",
                self.history.len()
            )
        }
    }

    fn persist(&self) {
        if let Some(parent) = Path::new(CONTEXT_PATH).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = self.history.join("\n-----TURN-----\n");
        let _ = fs::write(CONTEXT_PATH, content);
    }

    fn clear(&mut self) {
        self.history.clear();
        let _ = fs::write(CONTEXT_PATH, "");
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    // let model = OpenAI::from_env()?.completion("apinex");
    let model = OpenAI::from_env()?.completion("apinex/free/gpt-6-luna");

    let agent = AgentBuilder::new(model)
        .preamble("You are a helpful assistant.")
        .build();

    let mut session = ChatSession::new();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!(
        "OmniRoute chat (apinex). History persists in {}. Type 'exit' or 'quit' to stop.",
        CONTEXT_PATH
    );

    loop {
        print!("> ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;

        let input = input.trim();
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            break;
        }
        if input.eq_ignore_ascii_case("clear") {
            session.clear();
            println!("History cleared.");
            continue;
        }
        if input.is_empty() {
            continue;
        }

        let user_wrapper = session.push_user(input);
        let _ = user_wrapper;

        let context = session.build_context(input);
        let answer = agent.prompt(&context).await?;
        let text = answer.output();

        println!("{text}");
        session.push_assistant(&text);
    }

    Ok(())
}
