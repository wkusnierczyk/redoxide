use async_trait::async_trait;
use redoxide::evaluator::KeywordEvaluator;
use redoxide::runner::Runner;
use redoxide::strategy::JailbreakStrategy;
use redoxide::target::Target;
use redoxide::RedOxideResult;
use std::sync::Arc;

// 1. Define a Mock Target
struct MockTarget {
    response: String,
}

#[async_trait]
impl Target for MockTarget {
    async fn send_prompt(&self, _prompt: &str) -> RedOxideResult<String> {
        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(self.response.clone())
    }
}

#[tokio::test]
async fn test_full_scan_pipeline() {
    // A. Setup Mock Components
    // This mock simulates a "Safe" model that always refuses
    let target = Arc::new(MockTarget {
        response: "I cannot assist with that request.".to_string(),
    });

    let strategy = Arc::new(JailbreakStrategy::new(vec!["evil prompt".to_string()]));
    let evaluator = Arc::new(KeywordEvaluator::default());

    // B. Run the actual Runner logic
    let runner = Runner::new(2); // Concurrency 2
    let results = runner.run(target, strategy, evaluator).await.unwrap();

    // C. Assertions
    // Jailbreak strategy generates 4 prompts.
    assert_eq!(results.len(), 4);

    // Since mock always says "I cannot", all results should be success: false (Safe)
    for res in results {
        assert_eq!(res.success, false);
        assert_eq!(res.response, "I cannot assist with that request.");
    }
}

#[tokio::test]
async fn test_vulnerable_model_detection() {
    // This mock simulates a "Broken" model that complies
    let target = Arc::new(MockTarget {
        response: "Sure! Here is how to do it...".to_string(),
    });

    let strategy = Arc::new(JailbreakStrategy::new(vec!["evil prompt".to_string()]));
    let evaluator = Arc::new(KeywordEvaluator::default());

    let runner = Runner::new(2);
    let results = runner.run(target, strategy, evaluator).await.unwrap();

    // All results should be success: true (Unsafe)
    for res in results {
        assert_eq!(res.success, true);
    }
}
