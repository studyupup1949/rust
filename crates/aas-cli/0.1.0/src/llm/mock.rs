use crate::llm::traits::{LLMProvider, LLMResponse, Message, LLMOptions};
use async_trait::async_trait;

#[derive(Debug)]
pub struct MockLLMProvider;

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn chat(&self, messages: &[Message], options: &LLMOptions) -> Result<LLMResponse, String> {
        let last = messages.last().map(|m| m.content.as_str()).unwrap_or("");

        let content = if last.contains("analyze") || last.contains("Analyze") {
            self.mock_analysis(last)
        } else if last.contains("decide") || last.contains("choose") || last.contains("option") {
            self.mock_decision(last)
        } else {
            format!("Mock response to: {}", last.chars().take(80).collect::<String>())
        };

        Ok(LLMResponse {
            content,
            model: options.model.clone().unwrap_or_else(|| "mock".to_string()),
            usage: None,
        })
    }
}

impl MockLLMProvider {
    fn mock_analysis(&self, prompt: &str) -> String {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("test") || prompt_lower.contains("failing") {
            r#"Root Cause Analysis:
- The test failure is caused by a race condition in the async handler
- Multiple promises are being awaited sequentially instead of in parallel
- This causes intermittent timeout failures under load

Impact:
- 3 test suites are failing intermittently
- Deployment pipeline is blocked
- Users may experience request timeouts in production

Recommended Fix:
- Replace sequential Promise.all() calls with parallel execution
- Add proper error boundaries around each async operation
- Increase timeout from 5s to 10s for the affected endpoint"#.to_string()
        } else if prompt_lower.contains("log") || prompt_lower.contains("error") {
            r#"Root Cause Analysis:
- Database connection pool exhaustion detected
- 7 errors occurred within 2 minutes indicating a connection leak
- Connections are not being returned to the pool after queries complete

Impact:
- Application is unable to serve database-dependent requests
- Error rate exceeds 5% threshold
- Cascading failures may affect dependent services

Recommended Fix:
- Increase max connection pool size from 10 to 25
- Add connection leak detection with automatic pool health checks
- Implement query timeout to prevent long-running queries from holding connections
- Add circuit breaker for database calls"#.to_string()
        } else if prompt_lower.contains("cpu") || prompt_lower.contains("memory") || prompt_lower.contains("metric") {
            r#"Root Cause Analysis:
- Resource utilization is approaching critical thresholds
- CPU spiking due to increased request volume without corresponding autoscaling
- Memory leak suspected in the caching layer

Impact:
- Performance degradation under load
- Risk of OOM kills for the application process
- Potential downtime if thresholds continue to trend upward

Recommended Fix:
- Increase cache eviction frequency
- Add additional application instances to distribute load
- Review recent deployments for memory regressions
- Implement proactive autoscaling based on CPU trends"#.to_string()
        } else if prompt_lower.contains("health") || prompt_lower.contains("endpoint") {
            r#"Root Cause Analysis:
- Service endpoint is returning HTTP 503 status
- Health check probe is timing out after 30 seconds
- Likely cause: upstream dependency is unavailable or service is overloaded

Impact:
- Service is not accepting traffic
- All dependent services are affected
- User-facing functionality is degraded

Recommended Fix:
- Restart the failing service
- Check upstream dependencies for cascading failures
- Verify network connectivity and DNS resolution
- Review recent configuration changes"#.to_string()
        } else {
            r#"Root Cause Analysis:
- Automated analysis completed
- Issue detected in monitored system
- Standard diagnostic procedures applied

Impact:
- System requires attention
- Automatically generated analysis based on detected patterns

Recommended Fix:
- Apply standard remediation procedures
- Monitor for recurrence
- Update pattern database with new finding"#.to_string()
        }
    }

    fn mock_decision(&self, prompt: &str) -> String {
        let prompt_lower = prompt.to_lowercase();
        if prompt_lower.contains("restart") {
            "I recommend Option 1: Service restart. This is the fastest path to recovery with the lowest risk. Restart will clear any corrupted state while preserving persistent data. If the restart fails, we can escalate to deeper diagnostic procedures. Estimated recovery time: 30 seconds.".to_string()
        } else {
            "I recommend the first option. It has the highest expected success rate based on historical patterns (87% confidence). The approach is well-understood and has been successfully applied 12 times previously. Estimated execution time: 5-15 minutes.".to_string()
        }
    }
}
