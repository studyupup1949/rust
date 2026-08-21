use std::sync::Arc;

use tokio::runtime::{Builder, Runtime};

use crate::{Engine, Result};

pub struct EngineBuilder {
    async_worker_thread_number: u16,
    rt: Option<Arc<Runtime>>,
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self {
            async_worker_thread_number: 16,
            rt: None,
        }
    }
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn async_worker_thread_number(
        mut self,
        n: u16,
    ) -> Self {
        self.async_worker_thread_number = n;
        self
    }

    pub fn runtime(
        mut self,
        runtime: Arc<Runtime>,
    ) -> Self {
        self.rt = Some(runtime);
        self
    }

    pub fn build(&self) -> Result<Engine> {
        let runtime = if self.rt.is_some() {
            self.rt.as_ref().unwrap().clone()
        } else {
            Arc::new(Builder::new_multi_thread().worker_threads(self.async_worker_thread_number.into()).enable_all().build().unwrap())
        };
        let engine = Engine::new(runtime);

        Ok(engine)
    }
}
