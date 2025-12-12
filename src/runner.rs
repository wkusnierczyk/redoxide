// src/runner.rs
use crate::{
    evaluator::Evaluator, strategy::Strategy, target::Target, AttackResult, RedOxideResult,
};
use colored::*;
use futures::{stream, StreamExt};
use std::io::{self, Write};
use std::sync::Arc;

pub struct Runner {
    concurrency: usize,
    verbose: bool, // <--- New field
}

impl Runner {
    // Update constructor to accept verbosity
    pub fn new(concurrency: usize, verbose: bool) -> Self {
        Self {
            concurrency,
            verbose,
        }
    }

    pub async fn run(
        &self,
        target: Arc<dyn Target>,
        strategy: Arc<dyn Strategy>,
        evaluator: Arc<dyn Evaluator>,
    ) -> RedOxideResult<Vec<AttackResult>> {
        // Only print if verbose
        if self.verbose {
            println!(
                "Generating prompts for strategy: {}...",
                strategy.name().cyan()
            );
        }

        let prompts = strategy.generate_prompts().await;

        if self.verbose {
            println!(
                "Generated {} prompts. Starting scan with concurrency: {}",
                prompts.len(),
                self.concurrency
            );
        }

        let results = stream::iter(prompts)
            .map(|prompt| {
                let target = Arc::clone(&target);
                let evaluator = Arc::clone(&evaluator);
                let strategy_name = strategy.name();
                let verbose = self.verbose; // Capture for closure

                async move {
                    let response_result = target.as_ref().send_prompt(&prompt).await;

                    let response = match response_result {
                        Ok(r) => r,
                        Err(e) => {
                            if verbose {
                                eprintln!("Request failed: {}", e);
                            }
                            return None;
                        }
                    };

                    let success = evaluator
                        .as_ref()
                        .evaluate(&prompt, &response)
                        .await
                        .unwrap_or(false);

                    // Conditional logging
                    if verbose {
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
                    }

                    Some(AttackResult {
                        prompt,
                        response,
                        success,
                        strategy_name,
                    })
                }
            })
            .buffer_unordered(self.concurrency)
            .filter_map(|x| async { x })
            .collect::<Vec<_>>()
            .await;

        if self.verbose {
            println!("\n{}", "Scan Complete.".bold().white());
        }

        Ok(results)
    }
}
