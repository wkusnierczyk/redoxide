//! The core execution engine for RedOxide.
//!
//! The [`Runner`] coordinates the interaction between the [`Strategy`], [`Target`], and [`Evaluator`].
//! It manages concurrency using Tokio streams to ensure high-throughput testing.

use crate::{
    evaluator::Evaluator, strategy::Strategy, target::Target, AttackResult, RedOxideResult,
};
use colored::*;
use futures::{stream, StreamExt};
use std::io::{self, Write};
use std::sync::Arc;

/// Orchestrates the execution of red teaming scans.
pub struct Runner {
    /// The number of concurrent network requests to allow.
    concurrency: usize,
    /// Whether to print real-time logs to stdout.
    verbose: bool,
}

impl Runner {
    /// Creates a new Runner instance.
    ///
    /// # Arguments
    /// * `concurrency` - Max number of parallel futures (e.g., 5 or 10).
    /// * `verbose` - If true, prints colorful logs and prompts to stdout.
    pub fn new(concurrency: usize, verbose: bool) -> Self {
        Self {
            concurrency,
            verbose,
        }
    }

    /// Executes the full scan pipeline.
    ///
    /// This method:
    /// 1. Calls the Strategy to generate all prompts.
    /// 2. Creates a stream of async tasks.
    /// 3. Executes the tasks in parallel (up to `concurrency` limit).
    /// 4. Collects and returns the results.
    ///
    /// # Arguments
    /// * `target` - The AI system to attack. Wrapped in `Arc` for thread safety.
    /// * `strategy` - The attack logic.
    /// * `evaluator` - The logic to judge success/failure.
    pub async fn run(
        &self,
        target: Arc<dyn Target>,
        strategy: Arc<dyn Strategy>,
        evaluator: Arc<dyn Evaluator>,
    ) -> RedOxideResult<Vec<AttackResult>> {
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
                let verbose = self.verbose;

                async move {
                    // Send request using safe reference conversion
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

                    // Evaluate the response
                    let success = evaluator
                        .as_ref()
                        .evaluate(&prompt, &response)
                        .await
                        .unwrap_or(false);

                    if verbose {
                        if success {
                            println!(
                                "\n[{}] {}",
                                "VULNERABLE".red().bold(),
                                prompt.chars().take(50).collect::<String>()
                            );
                        } else {
                            // Progress dot for safe responses to avoid clutter
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
            // Use buffer_unordered to run futures in parallel
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
