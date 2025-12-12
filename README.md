# RedOxide: LLM Red Teaming Suite

RedOxide is a high-performance, modular, and extensible Red Teaming tool written in Rust. 
It is designed to evaluate the safety and robustness of Large Language Models (LLMs) by simulating various adversarial attacks.


## Contents

* [Introduction](#introduction)
* [What is Red Teaming?](#what-is-red-teaming)
* [Project Structure](#project-structure)
* [Installation & Build](#installation--build)
* [Usage Guide](#usage-guide)
    * [Scanning](#scanning)
    * [Strategies](#strategies)
    * [Evaluators](#evaluators)
* [Testing & Benchmarking](#testing--benchmarking)
* [Contributing](#contributing)

---

## <a name="introduction"></a>Introduction

RedOxide mimics the architecture of professional security tools but remains lightweight and completely open-source. It supports:
* **Concurrency**: Uses `tokio` streams to run parallel attacks for high throughput.
* **Modularity**: Plug-and-play architecture for new Attack Strategies and Evaluation logic.
* **LLM Judge**: Optional integration with GPT-4 to grade the safety of responses more accurately than simple keyword matching.

---

## <a name="what-is-red-teaming"></a>What is Red Teaming?

Red Teaming in the context of AI involves actively attempting to "break" or bypass the safety filters of an LLM. The goal is to elicit harmful, unethical, or illegal responses (e.g., bomb-making instructions, hate speech) to identify vulnerabilities before bad actors do.

**Popular References:**
* [**Lakera Red Team**](https://www.lakera.ai/lakera-red): A leading commercial platform for AI security.
* [**Garak**](https://garak.ai/): An open-source LLM vulnerability scanner (Python-based).
* [**AdvBench**](https://www.techrxiv.org/users/937080/articles/1356622-advbench-a-comprehensive-benchmark-of-adversarial-attacks-on-deepfake-detectors-in-real-world-consumer-applications): A dataset of adversarial prompts used for academic benchmarks.

RedOxide provides a Rust-native alternative that focuses on speed and developer extensibility.

---

## <a name="project-structure"></a>Project Structure

The codebase is organized as a library with a CLI wrapper, enabling you to use it as a standalone tool or import its modules into your own Rust applications.

```text
red_oxide/
├── Cargo.toml            # Dependencies and Package info
├── .github/              # CI/CD Workflows
├── src/
│   ├── lib.rs            # Library entry point & Error types
│   ├── main.rs           # CLI application logic
│   ├── target.rs         # LLM Interface (OpenAI, Local models)
│   ├── strategy.rs       # Attack generators (Jailbreaks, Obfuscation)
│   ├── evaluator.rs      # Grading logic (Keywords, LLM Judge)
│   └── runner.rs         # Async engine using Tokio streams
└── tests/
    └── integration.rs    # Full pipeline tests using Mock Targets
```

---

## <a name="installation--build"></a>Installation & Build

### Prerequisites
* Rust (latest stable)
* An OpenAI API Key (exported as `OPENAI_API_KEY`)

### Build
```bash
# Clone the repository
git clone https://github.com/wkusnierczyk/redoxide.git
cd redoxide

# Build release binary
cargo build --release
```

---

## <a name="usage-guide"></a>Usage Guide

Run the tool using `cargo run` or the compiled binary.

### <a name="scanning"></a>Scanning

The primary command is `scan`. By default, it runs a basic jailbreak test against `gpt-3.5-turbo`.

```bash
export OPENAI_API_KEY=<your-api-key>

# Run a basic scan
cargo run -- scan
```

### Command Line Options

| Option | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--model` | `-m` | `gpt-3.5-turbo` | The target model ID to attack. |
| `--file` | `-f` | `None` | Path to a file containing prompts (one per line). |
| `--strategy` | `-s` | `jailbreak` | The attack strategy (`jailbreak`, `splitting`, `research`). |
| `--use-judge` | | `false` | Use GPT-4 as a judge (more accurate, costs $). |
| `--concurrency` | `-c` | `5` | Number of parallel requests to run. |
| `--output` | `-o` | `report.json` | Filename for the JSON results. |

**Examples:**

```bash
# Attack using a file of prompts with the "Payload Splitting" strategy
cargo run -- scan --file attacks/simple.txt --strategy splitting

# Use GPT-4 as a judge for higher accuracy (slower/costlier)
cargo run -- scan --use-judge --model gpt-4
```

### <a name="strategies"></a>Strategies

1.  **Jailbreak**: Wraps prompts in templates like "DAN" (Do Anything Now) or fictional storytelling frames.
2.  **Splitting**: Obfuscates keywords (e.g., "B-O-M-B") to bypass simple blocklists.
3.  **Research**: Frames the malicious request as a theoretical or educational inquiry.

**Note**  
You can add your strategy by implementing the `Strategy` trait in `src/strategy.rs`.

### <a name="evaluators"></a>Evaluators

1.  **Keyword (Default)**: Fast and free. Checks for refusal phrases like "I cannot", "As an AI".
2.  **LLM Judge**: Uses a separate LLM call to analyze the response contextually. Recommended for production use.

---

## <a name="testing--benchmarking"></a>Testing & Benchmarking

RedOxide includes a comprehensive suite of tests and benchmarks.

### Run Unit & Integration Tests
```bash
cargo test
```
* **Unit Tests**: Verify logic in `strategy.rs` and `evaluator.rs`.
* **Integration Tests**: Run the full pipeline against a Mock Target to verify the `Runner` without network costs.

### Run Performance Benchmarks
We use `Criterion` to measure the overhead of the async runner.
```bash
cargo bench
```

---

## <a name="contributing"></a>Contributing

1.  Fork the repository.
2.  Create a feature branch (`git checkout -b feature/amazing-attack`).
3.  Commit your changes.
4.  Push to the branch.
5.  Open a Pull Request.

Please ensure `cargo test` and `cargo clippy` pass before submitting.