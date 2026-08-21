mod support;

use admixture::context::health::wait_until_healthy;
use admixture::service::ServiceSetup;
use std::time::Duration;
use support::mock_service::MockServiceSetup;

#[tokio::test]
async fn test_health_check_succeeds_immediately() {
    let service_setup = MockServiceSetup::healthy("immediate");
    let service = service_setup.start().await.expect("service should start");

    let result =
        wait_until_healthy(&service, Duration::from_secs(5), Duration::from_millis(500)).await;

    assert!(result.is_ok(), "Health check should succeed immediately");
}

#[tokio::test]
async fn test_health_check_retries_then_succeeds() {
    let service_setup = MockServiceSetup::delayed_health("delayed", Duration::from_secs(1));
    let service = service_setup.start().await.expect("service should start");

    let start = std::time::Instant::now();

    let result =
        wait_until_healthy(&service, Duration::from_secs(5), Duration::from_millis(500)).await;

    assert!(result.is_ok(), "Health check should eventually succeed");
    assert!(
        start.elapsed() >= Duration::from_secs(1),
        "Should have waited at least 1 second"
    );
}

#[tokio::test]
async fn test_health_check_timeout() {
    let service_setup = MockServiceSetup::fails_health("never_healthy");
    let service = service_setup.start().await.expect("service should start");

    let result =
        wait_until_healthy(&service, Duration::from_secs(2), Duration::from_millis(500)).await;

    assert!(result.is_err(), "Health check should timeout");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("failed to become healthy") || err_msg.contains("configured to fail"),
        "Error message should indicate health check failure: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_health_check_custom_interval() {
    let service_setup =
        MockServiceSetup::delayed_health("custom_interval", Duration::from_millis(300));
    let service = service_setup.start().await.expect("service should start");

    let start = std::time::Instant::now();

    let result =
        wait_until_healthy(&service, Duration::from_secs(5), Duration::from_millis(100)).await;

    assert!(
        result.is_ok(),
        "Health check should succeed with custom interval"
    );
    assert!(
        start.elapsed() >= Duration::from_millis(300),
        "Should have waited at least 300ms"
    );
}
