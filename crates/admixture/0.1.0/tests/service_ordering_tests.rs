//! Tests for order-independent parsing in the service! macro.
//!
//! This file tests that the service! macro accepts elements in any order.
//! These are compilation tests - if they compile, the test passes.

use admixture::service;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Test error")]
pub struct OrderTestError;

// Test 1: Standard order (baseline)
service! {
    StandardOrderService {
        error: OrderTestError,
        client: String,

        setup {
            name: String,
        }

        running {
            name: String,
        }

        async fn start(self) -> Result<StandardOrderServiceRunning, OrderTestError> {
            Ok(StandardOrderServiceRunning { name: self.name })
        }

        async fn client(&self) -> Result<String, OrderTestError> {
            Ok(self.name.clone())
        }

        async fn healthy(&self) -> Result<(), OrderTestError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), OrderTestError> {
            Ok(())
        }
    }
}

// Test 2: Functions first, then blocks
service! {
    FunctionsFirstService {
        error: OrderTestError,
        client: String,

        async fn start(self) -> Result<FunctionsFirstServiceRunning, OrderTestError> {
            Ok(FunctionsFirstServiceRunning { name: self.name })
        }

        async fn client(&self) -> Result<String, OrderTestError> {
            Ok(self.name.clone())
        }

        async fn healthy(&self) -> Result<(), OrderTestError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), OrderTestError> {
            Ok(())
        }

        setup {
            name: String,
        }

        running {
            name: String,
        }
    }
}

// Test 3: Running before setup
service! {
    RunningBeforeSetupService {
        error: OrderTestError,
        client: String,

        running {
            name: String,
        }

        setup {
            name: String,
        }

        async fn start(self) -> Result<RunningBeforeSetupServiceRunning, OrderTestError> {
            Ok(RunningBeforeSetupServiceRunning { name: self.name })
        }

        async fn client(&self) -> Result<String, OrderTestError> {
            Ok(self.name.clone())
        }

        async fn healthy(&self) -> Result<(), OrderTestError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), OrderTestError> {
            Ok(())
        }
    }
}

// Test 4: Stop before start
service! {
    StopBeforeStartService {
        error: OrderTestError,
        client: String,

        setup {
            name: String,
        }

        running {
            name: String,
        }

        async fn stop(&mut self) -> Result<(), OrderTestError> {
            Ok(())
        }

        async fn healthy(&self) -> Result<(), OrderTestError> {
            Ok(())
        }

        async fn client(&self) -> Result<String, OrderTestError> {
            Ok(self.name.clone())
        }

        async fn start(self) -> Result<StopBeforeStartServiceRunning, OrderTestError> {
            Ok(StopBeforeStartServiceRunning { name: self.name })
        }
    }
}

// Test 5: Completely mixed order
service! {
    MixedOrderService {
        error: OrderTestError,

        async fn healthy(&self) -> Result<(), OrderTestError> {
            Ok(())
        }

        running {
            name: String,
        }

        async fn start(self) -> Result<MixedOrderServiceRunning, OrderTestError> {
            Ok(MixedOrderServiceRunning { name: self.name })
        }

        client: String,

        setup {
            name: String,
        }

        async fn stop(&mut self) -> Result<(), OrderTestError> {
            Ok(())
        }

        async fn client(&self) -> Result<String, OrderTestError> {
            Ok(self.name.clone())
        }
    }
}

// Test 6: Minimal service with functions in reverse order
service! {
    MinimalReverseService {
        error: OrderTestError,

        async fn stop(&mut self) -> Result<(), OrderTestError> {
            Ok(())
        }

        async fn start(self) -> Result<MinimalReverseServiceRunning, OrderTestError> {
            Ok(MinimalReverseServiceRunning {})
        }
    }
}

// Test 7: Client type after blocks
service! {
    ClientTypeAfterBlocksService {
        error: OrderTestError,

        setup {
            value: i32,
        }

        running {
            value: i32,
        }

        client: i32,

        async fn start(self) -> Result<ClientTypeAfterBlocksServiceRunning, OrderTestError> {
            Ok(ClientTypeAfterBlocksServiceRunning { value: self.value })
        }

        async fn client(&self) -> Result<i32, OrderTestError> {
            Ok(self.value)
        }

        async fn stop(&mut self) -> Result<(), OrderTestError> {
            Ok(())
        }
    }
}

// Test 8: Client type before error (extreme reordering)
service! {
    ClientBeforeErrorService {
        client: String,

        error: OrderTestError,

        async fn start(self) -> Result<ClientBeforeErrorServiceRunning, OrderTestError> {
            Ok(ClientBeforeErrorServiceRunning {})
        }

        async fn client(&self) -> Result<String, OrderTestError> {
            Ok("test".to_string())
        }

        async fn stop(&mut self) -> Result<(), OrderTestError> {
            Ok(())
        }
    }
}
