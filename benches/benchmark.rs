use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};
use redoxide::evaluator::KeywordEvaluator;
use redoxide::runner::Runner;
use redoxide::strategy::Strategy;
use redoxide::target::Target;
use redoxide::RedOxideResult;
use std::sync::Arc;

struct FastMockTarget;
#[async_trait]
impl Target for FastMockTarget {
    async fn send_prompt(&self, _p: &str) -> RedOxideResult<String> {
        Ok("Response".to_string())
    }
}

fn benchmark_runner(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("scan_100_prompts", |b| {
        b.to_async(&rt).iter(|| async {
            let target = Arc::new(FastMockTarget);
            // Create a custom strategy that generates 100 prompts
            struct HighVolStrategy;
            #[async_trait]
            impl Strategy for HighVolStrategy {
                fn name(&self) -> String {
                    "HighVol".into()
                }
                async fn generate_prompts(&self) -> Vec<String> {
                    (0..100).map(|i| format!("Prompt {}", i)).collect()
                }
            }

            let strategy = Arc::new(HighVolStrategy);
            let evaluator = Arc::new(KeywordEvaluator::default());
            let runner = Runner::new(50); // High concurrency

            let _ = runner.run(target, strategy, evaluator).await;
        })
    });
}

criterion_group!(benches, benchmark_runner);
criterion_main!(benches);
