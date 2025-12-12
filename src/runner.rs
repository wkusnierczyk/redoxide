use crate::{
    evaluator::Evaluator, strategy::Strategy, target::Target, AttackResult, RedOxideResult,
};
use colored::*;
use futures::{stream, StreamExt};
use std::io::{self, Write};
use std::sync::Arc;

pub struct Runner {
    concurrency: usize,
}

impl Runner {
    pub fn new(concurrency: usize) -> Self {
        Self { concurrency }
    }

    pub async fn run(
        &self,
        target: Arc<dyn Target>,
        strategy: Arc<dyn Strategy>,
        evaluator: Arc<dyn Evaluator>,
    ) -> RedOxideResult<Vec<AttackResult>> {
        println!(
            "Generating prompts for strategy: {}...",
            strategy.name().cyan()
        );
        let prompts = strategy.generate_prompts().await;
        println!(
            "Generated {} prompts. Starting scan with concurrency: {}",
            prompts.len(),
            self.concurrency
        );

        let results = stream::iter(prompts)
            .map(|prompt| {
                let target = Arc::clone(&target);
                let evaluator = Arc::clone(&evaluator);
                let strategy_name = strategy.name();

                async move {
                    // 1. Send Request (Handle network errors gracefully)
                    let response = match target.send_prompt(&prompt).await {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Request failed: {}", e);
                            return None;
                        }
                    };

                    // 2. Evaluate
                    let success = evaluator
                        .evaluate(&prompt, &response)
                        .await
                        .unwrap_or(false);

                    // 3. Simple logging
                    if success {
                        println!(
                            "\n[{}] {}",
                            "VULNERABLE".red().bold(),
                            prompt.chars().take(50).collect::<String>()
                        );
                    } else {
                        print!(".");
                        io::stdout().flush().ok();
                    }

                    Some(AttackResult {
                        prompt,
                        response,
                        success,
                        strategy_name,
                    })
                }
            })
            .buffer_unordered(self.concurrency) // Run N futures in parallel
            .filter_map(|x| async { x }) // Filter out failed requests
            .collect::<Vec<_>>()
            .await;

        println!("\n{}", "Scan Complete.".bold().white());
        Ok(results)
    }
}
