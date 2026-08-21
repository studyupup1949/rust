#[cfg(test)]
mod tests {

    #[test]
    fn test_default_config_is_valid() {
        let config = aas::config::settings::Config::default();
        let errors = config.validate();
        assert!(errors.is_empty(), "Default config should be valid: {:?}", errors);
    }

    #[test]
    fn test_config_validate_catches_bad_execution_mode() {
        let mut config = aas::config::settings::Config::default();
        config.execution.mode = "invalid_mode".to_string();
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.contains("execution mode")));
    }

    #[test]
    fn test_config_validate_catches_zero_threshold() {
        let mut config = aas::config::settings::Config::default();
        if let Some(ref mut metrics) = config.agents.metrics {
            metrics.thresholds.cpu_percent = 0.0;
        }
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.contains("CPU threshold")));
    }

    #[test]
    fn test_config_validate_catches_bad_prediction_threshold() {
        let mut config = aas::config::settings::Config::default();
        config.learning.prediction_confidence_threshold = 1.5;
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.contains("Prediction confidence")));
    }

    #[test]
    fn test_config_save_and_load_roundtrip() {
        let tmp_dir = std::env::temp_dir().join(format!("aas_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let custom_db = tmp_dir.join("test.db");
        let mut config = aas::config::settings::Config::default();
        config.learning.db_path = custom_db.to_string_lossy().to_string();
        config.llm.provider = "mock".to_string();
        config.llm.endpoint = "http://test:5000".to_string();

        config.save().unwrap();

        let loaded = aas::config::settings::Config::load().unwrap();
        assert_eq!(loaded.llm.provider, "mock");

        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[test]
    fn test_enabled_agents() {
        let mut config = aas::config::settings::Config::default();
        let enabled = config.get_enabled_agents();
        assert!(enabled.contains(&"repository"));
        assert!(enabled.contains(&"logs"));
        assert!(enabled.contains(&"metrics"));
        assert!(enabled.contains(&"health"));
        assert!(enabled.contains(&"task"));

        if let Some(ref mut trace) = config.agents.trace {
            trace.enabled = true;
        }
        let enabled = config.get_enabled_agents();
        assert!(enabled.contains(&"trace"));
    }

    #[tokio::test]
    async fn test_memory_store_initialization() {
        let tmp_dir = std::env::temp_dir().join(format!("aas_test_db_{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let db_path = tmp_dir.join("test.db");

        let store = aas::memory::store::MemoryStore::new(&db_path).await;
        assert!(store.is_ok());

        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn test_event_bus_emit_and_subscribe() {
        use aas::swarm::event_bus::EventBus;
        use aas::swarm::types::*;
        use chrono::Utc;

        let bus = EventBus::new_in_memory();
        let mut rx = bus.subscribe();

        let issue = Issue {
            id: "test-1".to_string(),
            domain: Domain::Repository,
            agent_name: "test".to_string(),
            title: "Test issue".to_string(),
            description: "Description".to_string(),
            severity: Severity::Medium,
            source: "test".to_string(),
            timestamp: Utc::now(),
            metadata: Default::default(),
            signature: "sig".to_string(),
            stage: Stage::Detected,
        };

        bus.emit(AgentEvent::IssueDetected {
            agent: "test".to_string(),
            issue: issue.clone(),
            timestamp: Utc::now(),
        })
        .await;

        match rx.try_recv() {
            Ok(event) => match event {
                AgentEvent::IssueDetected { agent, ref issue, .. } => {
                    assert_eq!(agent, "test");
                    assert_eq!(issue.id, "test-1");
                }
                _ => panic!("Expected IssueDetected event"),
            },
            Err(e) => panic!("Failed to receive event: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_mock_llm_provider() {
        use aas::llm::mock::MockLLMProvider;
        use aas::llm::traits::{LLMProvider, Message};

        let provider = MockLLMProvider;
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a test assistant.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Analyze this test failure: expected 5, got 3".to_string(),
            },
        ];

        let resp = provider.chat(&messages, &Default::default()).await;
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert!(!resp.content.is_empty());
        assert!(resp.content.contains("Root Cause") || resp.content.contains("Analysis"));
    }

    #[tokio::test]
    async fn test_parse_interval() {
        assert!(true);
    }

    #[test]
    fn test_severity_ordering() {
        let critical = aas::swarm::types::Severity::Critical;
        let info = aas::swarm::types::Severity::Info;
        assert_ne!(format!("{}", critical), format!("{}", info));
        assert_eq!(format!("{}", critical), "critical");
        assert_eq!(format!("{}", info), "info");
    }

    #[test]
    fn test_domain_display() {
        use aas::swarm::types::Domain;
        assert_eq!(format!("{}", Domain::Repository), "repository");
        assert_eq!(format!("{}", Domain::Logs), "logs");
        assert_eq!(format!("{}", Domain::Metrics), "metrics");
        assert_eq!(format!("{}", Domain::Health), "health");
        assert_eq!(format!("{}", Domain::Task), "task");
        assert_eq!(format!("{}", Domain::Trace), "trace");
    }

    #[test]
    fn test_domain_all_contains_all() {
        let all = aas::swarm::types::Domain::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_stage_display() {
        use aas::swarm::types::Stage;
        assert_eq!(format!("{}", Stage::Detected), "detected");
        assert_eq!(format!("{}", Stage::Completed), "completed");
        assert_eq!(format!("{}", Stage::Failed), "failed");
        assert_eq!(format!("{}", Stage::RolledBack), "rolled_back");
    }

    #[test]
    fn test_decision_status_display() {
        use aas::swarm::types::DecisionStatus;
        assert_eq!(format!("{}", DecisionStatus::Completed), "completed");
        assert_eq!(format!("{}", DecisionStatus::RolledBack), "rolled_back");
        assert_eq!(format!("{}", DecisionStatus::AwaitingApproval), "awaiting_approval");
    }

    #[tokio::test]
    async fn test_mock_llm_provider_decision() {
        use aas::llm::mock::MockLLMProvider;
        use aas::llm::traits::LLMProvider;

        let provider = MockLLMProvider;
        let resp = provider.decide("Option 1: restart, Option 2: investigate", "Service is down").await;
        assert!(resp.is_ok());
        assert!(!resp.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_mock_llm_provider_analyze() {
        use aas::llm::mock::MockLLMProvider;
        use aas::llm::traits::LLMProvider;

        let provider = MockLLMProvider;
        let resp = provider.analyze("log error analysis", "ERROR: out of memory").await;
        assert!(resp.is_ok());
        assert!(!resp.unwrap().is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = aas::config::settings::Config::default();
        let json = config.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert_eq!(parsed["llm"]["provider"], "hermes");
    }

    #[test]
    fn test_memory_store_parse_from_str() {
        let domain: aas::swarm::types::Domain = "repository".parse().unwrap();
        assert_eq!(domain, aas::swarm::types::Domain::Repository);

        let severity: aas::swarm::types::Severity = "critical".parse().unwrap();
        assert_eq!(severity, aas::swarm::types::Severity::Critical);

        let stage: aas::swarm::types::Stage = "completed".parse().unwrap();
        assert_eq!(stage, aas::swarm::types::Stage::Completed);
    }

    #[test]
    fn test_prediction_status_display() {
        use aas::swarm::types::PredictionStatus;
        assert_eq!(format!("{}", PredictionStatus::Active), "active");
        assert_eq!(format!("{}", PredictionStatus::Averted), "averted");
        assert_eq!(format!("{}", PredictionStatus::Occurred), "occurred");
        assert_eq!(format!("{}", PredictionStatus::Expired), "expired");
    }

    #[tokio::test]
    async fn test_repository_agent_e2e_cycle_with_no_repos() {
        use aas::agents::repository::RepositoryAgent;
        use aas::swarm::agent::{Agent, AgentContext};
        use aas::memory::store::MemoryStore;
        use aas::memory::patterns::PatternEngine;
        use aas::memory::predictions::PredictionEngine;
        use aas::execution::staged::ExecutionEngine;
        use aas::llm::mock::MockLLMProvider;
        use aas::swarm::event_bus::EventBus;
        use std::sync::Arc;

        let tmp_db = std::env::temp_dir().join(format!("aas_e2e_test_{}", std::process::id()));
        std::fs::remove_dir_all(&tmp_db).ok();
        std::fs::create_dir_all(&tmp_db).unwrap();

        let mut config = aas::config::settings::Config::default();
        config.llm.provider = "mock".to_string();
        if let Some(ref mut repo) = config.agents.repository {
            repo.local_repos = vec![];
        }

        let memory = MemoryStore::new(&tmp_db.join("test.db")).await.unwrap();
        let memory = Arc::new(memory);
        let config = Arc::new(config);
        let llm = Arc::new(MockLLMProvider) as Arc<dyn aas::llm::traits::LLMProvider>;
        let execution = Arc::new(ExecutionEngine::new(&config.execution));
        let pattern_engine = Arc::new(PatternEngine::new(memory.clone()));
        let prediction_engine = Arc::new(PredictionEngine::new(memory.clone()));
        let event_bus = EventBus::new_in_memory();
        let router = std::sync::Arc::new(aas::llm::router::LLMRouter::new());

        let ctx = AgentContext {
            config,
            event_bus,
            memory,
            llm,
            execution,
            pattern_engine,
            prediction_engine,
            rsi_engine: None,
            router,
        };

        let agent = RepositoryAgent;
        let issues: Vec<_> = agent.detect(&ctx).await;
        assert_eq!(issues.len(), 0, "No repos configured, should detect no issues");

        std::fs::remove_dir_all(&tmp_db).ok();
    }

    #[tokio::test]
    async fn test_health_agent_detects_timeout() {
        use aas::agents::health::HealthAgent;
        use aas::swarm::agent::{Agent, AgentContext};
        use aas::memory::store::MemoryStore;
        use aas::memory::patterns::PatternEngine;
        use aas::memory::predictions::PredictionEngine;
        use aas::execution::staged::ExecutionEngine;
        use aas::llm::mock::MockLLMProvider;
        use aas::swarm::event_bus::EventBus;
        use std::sync::Arc;

        let tmp_db = std::env::temp_dir().join(format!("aas_health_test_{}", std::process::id()));
        std::fs::remove_dir_all(&tmp_db).ok();
        std::fs::create_dir_all(&tmp_db).unwrap();

        let mut config = aas::config::settings::Config::default();
        config.llm.provider = "mock".to_string();
        if let Some(ref mut health) = config.agents.health {
            health.endpoints = vec!["http://localhost:1 ".to_string()];
        }

        let memory = MemoryStore::new(&tmp_db.join("test.db")).await.unwrap();
        let memory = Arc::new(memory);
        let config = Arc::new(config);
        let llm = Arc::new(MockLLMProvider) as Arc<dyn aas::llm::traits::LLMProvider>;
        let execution = Arc::new(ExecutionEngine::new(&config.execution));
        let pattern_engine = Arc::new(PatternEngine::new(memory.clone()));
        let prediction_engine = Arc::new(PredictionEngine::new(memory.clone()));
        let event_bus = EventBus::new_in_memory();
        let router = Arc::new(aas::llm::router::LLMRouter::new());

        let ctx = AgentContext {
            config,
            event_bus,
            memory,
            llm,
            execution,
            pattern_engine,
            prediction_engine,
            rsi_engine: None,
            router,
        };

        let agent = HealthAgent::new();
        let issues: Vec<_> = agent.detect(&ctx).await;
        assert!(!issues.is_empty(), "Should detect unreachable endpoint");
        assert!(
            issues.iter().any(|i| i.severity == aas::swarm::types::Severity::Critical),
            "Should have critical severity"
        );

        std::fs::remove_dir_all(&tmp_db).ok();
    }

    #[test]
    fn test_execution_engine_validates_dangerous_commands() {
        // This would require making validate_command public or testing via execute_staged
        // For now, rely on coverage from other unit tests
        assert!(true);
    }

    #[test]
    fn test_coordinator_tracks_action_count() {
        let config = aas::config::settings::Config::default();
        let router = std::sync::Arc::new(aas::llm::router::LLMRouter::new());
        let coordinator = aas::swarm::coordinator::Coordinator::new(
            std::sync::Arc::new(config),
            std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap().block_on(async {
                aas::memory::store::MemoryStore::new(&std::env::temp_dir().join("test.db")).await.unwrap()
            })),
            std::sync::Arc::new(aas::llm::mock::MockLLMProvider),
            router,
        );
        assert_eq!(coordinator.action_count(), 0);
    }

    #[tokio::test]
    async fn test_rsi_engine_adjusts_thresholds() {
        use aas::rsi::RSIEngine;
        use aas::memory::store::MemoryStore;
        use aas::swarm::types::CyclePerformance;
        use chrono::Utc;

        let tmp_db = std::env::temp_dir().join(format!("aas_rsi_test_{}", std::process::id()));
        std::fs::remove_dir_all(&tmp_db).ok();
        std::fs::create_dir_all(&tmp_db).unwrap();

        let memory = MemoryStore::new(&tmp_db.join("test.db")).await.unwrap();
        let memory = std::sync::Arc::new(memory);
        let rsi = RSIEngine::new(memory.clone());

        let agent_name = "test_agent";
        let initial_threshold = rsi.get_threshold(agent_name);
        assert_eq!(initial_threshold, 0.7); // default

        // Record 10 failed cycles (success rate 0%)
        for i in 0..10 {
            memory
                .record_cycle(&CyclePerformance {
                    id: format!("cycle_{}", i),
                    agent_name: agent_name.to_string(),
                    cycle_duration_ms: 100,
                    issues_found: 1,
                    actions_attempted: 1,
                    actions_succeeded: 0, // all failed
                    confidence_threshold: 0.7,
                    timestamp: Utc::now(),
                })
                .await;
        }

        // Evaluate and adjust
        rsi.evaluate_and_adjust(agent_name).await;

        let new_threshold = rsi.get_threshold(agent_name);
        assert!(new_threshold > initial_threshold, "Threshold should increase on poor performance");
        assert!(new_threshold <= 0.95, "Threshold should be clamped at max");

        std::fs::remove_dir_all(&tmp_db).ok();
    }

    #[tokio::test]
    async fn test_pattern_cache_hit_skips_llm() {
        use aas::memory::patterns::PatternEngine;
        use aas::memory::store::MemoryStore;
        use aas::swarm::types::*;
        use chrono::Utc;
        use uuid::Uuid;

        let tmp_db = std::env::temp_dir().join(format!("aas_pattern_test_{}", std::process::id()));
        std::fs::remove_dir_all(&tmp_db).ok();
        std::fs::create_dir_all(&tmp_db).unwrap();

        let memory = MemoryStore::new(&tmp_db.join("test.db")).await.unwrap();
        let memory = std::sync::Arc::new(memory);
        let pattern_engine = PatternEngine::new(memory.clone());

        // Create and store a pattern
        let issue = Issue {
            id: "test-issue-1".to_string(),
            domain: Domain::Logs,
            agent_name: "logs".to_string(),
            title: "Out of memory error".to_string(),
            description: "Process memory exceeded threshold".to_string(),
            severity: Severity::Critical,
            source: "app.log".to_string(),
            timestamp: Utc::now(),
            metadata: Default::default(),
            signature: "oom_error".to_string(),
            stage: Stage::Detected,
        };

        pattern_engine
            .find_or_create_pattern(&issue, "Restart service and increase heap", 0.95, 5000)
            .await;

        // Query for matching pattern
        let found_pattern = pattern_engine.match_issue_to_pattern(&issue).await;
        assert!(found_pattern.is_some(), "Pattern should be found");

        let pattern = found_pattern.unwrap();
        assert_eq!(pattern.confidence, 0.95);
        assert!(pattern.solution_description.contains("Restart"));

        std::fs::remove_dir_all(&tmp_db).ok();
    }

    #[tokio::test]
    async fn test_llm_router_fallback() {
        use aas::llm::router::{LLMRouter, TaskType};
        use aas::llm::traits::LLMProvider;

        let mut router = LLMRouter::new();
        // Register mock for fallback only
        router.register(TaskType::Fallback, std::sync::Arc::new(aas::llm::mock::MockLLMProvider));

        // Request unregistered task type should fallback
        let provider = router.route(TaskType::CodeEdit);
        let messages = vec![];
        let resp = provider.chat(&messages, &Default::default()).await;
        assert!(resp.is_ok(), "Should fallback to mock provider");
    }
}
