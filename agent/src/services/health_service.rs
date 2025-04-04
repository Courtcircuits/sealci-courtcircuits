use crate::proto::Health;
use std::sync::Arc;
use sysinfo::System;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone)]
pub struct HealthService {
    system: Arc<Mutex<System>>,
}

impl HealthService {
    pub fn new() -> Self {
        HealthService {
            system: Arc::new(Mutex::new(System::new_all())),
        }
    }

    pub fn get_health_stream(&mut self) -> (UnboundedReceiverStream<Health>, JoinHandle<()>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut previous_usage = Health {
            cpu_avail: 0,
            memory_avail: 0,
        };
        let mut service = self.clone();
        let handle_health_lifecycle = tokio::spawn(async move {
            loop {
                // Fetch current usage
                let current_health = service.get_health().await;

                // Check if the change is significant
                if HealthService::has_significant_change(previous_usage, current_health, 5.0) {
                    previous_usage = current_health;
                    let _ = tx.send(current_health);
                }

                // Delay before next check
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        (UnboundedReceiverStream::new(rx), handle_health_lifecycle)
    }

    fn has_significant_change(prev: Health, current: Health, threshold: f32) -> bool {
        let cpu_change = (current.cpu_avail as f32 - prev.cpu_avail as f32).abs();
        let memory_change = ((current.memory_avail as f32 - prev.memory_avail as f32)
            / prev.memory_avail as f32
            * 100.0)
            .abs();
        cpu_change >= threshold || memory_change >= threshold
    }

    pub async fn get_health(&mut self) -> Health {
        let mut sys = self.system.lock().await;
        sys.refresh_all();
        let cpu_avail = 100 - sys.global_cpu_info().cpu_usage() as u32;
        let memory_avail = (sys.total_memory() - sys.used_memory()) as u64;

        Health {
            cpu_avail,
            memory_avail,
        }
    }
}
