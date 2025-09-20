use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, Notify};
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, warn, error, debug};
use reqwest;
use serde_json;

/// Configuration for PythonManager
#[derive(Debug, Clone)]
pub struct PythonManagerConfig {
    pub script_path: String,
    pub max_restarts: u32,
    pub initial_restart_delay_secs: u64,
    pub health_check_url: String,
    pub health_check_timeout_secs: u64,
    pub health_check_max_retries: u32,
    pub health_check_retry_delay_secs: u64,
}

impl Default for PythonManagerConfig {
    fn default() -> Self {
        Self {
            script_path: "python_scripts/async_server.py".to_string(),
            max_restarts: 3,
            initial_restart_delay_secs: 2,
            health_check_url: "http://127.0.0.1:8001/health".to_string(),
            health_check_timeout_secs: 30,
            health_check_max_retries: 12,
            health_check_retry_delay_secs: 30,
        }
    }
}

/// Manages the Python sentiment analysis server as a subprocess
pub struct PythonManager {
    config: PythonManagerConfig,
    child_process: Arc<Mutex<Option<Child>>>,
    restart_count: Arc<Mutex<u32>>,
    shutdown_notify: Arc<Notify>,
    is_shutting_down: Arc<Mutex<bool>>,
    http_client: reqwest::Client,
}

impl PythonManager {
    pub fn new(config: Option<PythonManagerConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        Self {
            config,
            child_process: Arc::new(Mutex::new(None)),
            restart_count: Arc::new(Mutex::new(0)),
            shutdown_notify: Arc::new(Notify::new()),
            is_shutting_down: Arc::new(Mutex::new(false)),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Start the Python server subprocess with supervision
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // ARCHITECT GUIDANCE: Verify only one workflow instance runs at a time
        {
            let child_guard = self.child_process.lock().await;
            if child_guard.is_some() {
                info!("🚀 Python server subprocess manager already started, skipping duplicate start");
                return Ok(());
            }
        }
        
        info!("🚀 Starting Python server subprocess manager");
        
        // Start the subprocess
        self.spawn_subprocess().await?;
        
        // Start the supervision task
        let self_clone = self.clone_for_task();
        tokio::spawn(async move {
            self_clone.supervision_loop().await;
        });
        
        // Wait for the Python server to become healthy
        self.wait_for_health().await?;
        
        info!("✅ Python server subprocess started and healthy");
        Ok(())
    }

    /// Wait for the Python server to be healthy
    pub async fn wait_for_health(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🔄 STARTUP: Waiting for Python sentiment analysis server...");
        
        let start_time = Instant::now();
        let max_wait_time = Duration::from_secs(
            self.config.health_check_max_retries as u64 * self.config.health_check_retry_delay_secs
        );
        
        for attempt in 1..=self.config.health_check_max_retries {
            // Check if we're shutting down
            if *self.is_shutting_down.lock().await {
                return Err("Python server startup cancelled due to shutdown".into());
            }
            
            // Check if we've exceeded max wait time
            if start_time.elapsed() > max_wait_time {
                error!("❌ STARTUP: Python server health check timeout after {:?}", max_wait_time);
                break;
            }
            
            match self.check_health().await {
                Ok(health_data) => {
                    info!("✅ STARTUP: Python server is ready! Health check passed");
                    if let Some(libraries) = health_data.get("libraries") {
                        info!("   📚 Libraries: {:?}", libraries);
                    }
                    if let Some(primary_detector) = health_data.get("primary_detector") {
                        info!("   🎯 Primary detector: {:?}", primary_detector);
                    }
                    return Ok(());
                }
                Err(e) => {
                    warn!("⚠️ STARTUP: Attempt {}/{}: Python server not ready yet: {}", 
                         attempt, self.config.health_check_max_retries, e);
                }
            }
            
            if attempt < self.config.health_check_max_retries {
                info!("⏳ STARTUP: Retrying in {} seconds...", self.config.health_check_retry_delay_secs);
                sleep(Duration::from_secs(self.config.health_check_retry_delay_secs)).await;
            }
        }
        
        Err(format!("Python server failed to become healthy after {} attempts", 
                   self.config.health_check_max_retries).into())
    }

    /// Check Python server health
    async fn check_health(&self) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("🔍 PYTHON_HEALTH_DIAGNOSTIC: Making health check request");
        tracing::debug!("   🌐 URL: {}", self.config.health_check_url);
        tracing::debug!("   ⏰ Timeout: {}s", self.config.health_check_timeout_secs);
        
        let start_time = std::time::Instant::now();
        
        let response = self.http_client
            .get(&self.config.health_check_url)
            .timeout(Duration::from_secs(self.config.health_check_timeout_secs))
            .send()
            .await
            .map_err(|e| {
                let duration = start_time.elapsed();
                tracing::error!("   ❌ Health check request failed after {:?}: {}", duration, e);
                e
            })?;
        
        let request_duration = start_time.elapsed();
        tracing::debug!("   📊 Response received in {:?} - Status: {}", 
                       request_duration, response.status());
        
        if !response.status().is_success() {
            tracing::error!("   ❌ Health check failed with HTTP status: {}", response.status());
            return Err(format!("Health check failed with status: {}", response.status()).into());
        }
        
        let health_data: serde_json::Value = response.json().await
            .map_err(|e| {
                tracing::error!("   ❌ Failed to parse health check JSON: {}", e);
                e
            })?;
        
        tracing::debug!("   ✅ Health check successful - Response: {:?}", health_data);
        Ok(health_data)
    }

    /// Spawn the Python subprocess
    async fn spawn_subprocess(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // ARCHITECT GUIDANCE: Probe health before spawning to avoid duplicates
        info!("[PY] Checking if Python server is already running on port 8001...");
        match self.check_health().await {
            Ok(_) => {
                info!("[PY] ✅ Python server already running and healthy, skipping subprocess spawn");
                return Ok(());
            }
            Err(_) => {
                info!("[PY] No existing Python server found, proceeding to spawn new subprocess");
            }
        }
        
        info!("[PY] Spawning Python server subprocess: {}", self.config.script_path);
        
        let mut child = Command::new("python3")
            .arg(&self.config.script_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                error!("[PY] Failed to spawn Python subprocess: {}", e);
                format!("Failed to spawn Python subprocess: {}", e)
            })?;
        
        // Start log forwarding tasks
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            tokio::spawn(async move {
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    info!("[PY] {}", line);
                }
            });
        }
        
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            tokio::spawn(async move {
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    warn!("[PY] {}", line);
                }
            });
        }
        
        // Store the child process
        *self.child_process.lock().await = Some(child);
        
        debug!("[PY] Python subprocess spawned successfully");
        Ok(())
    }

    /// Supervision loop that monitors and restarts the subprocess if needed
    #[allow(dead_code)]
    async fn supervision_loop(&self) {
        info!("[PY] Starting supervision loop");
        
        loop {
            // Check if we're shutting down
            if *self.is_shutting_down.lock().await {
                info!("[PY] Supervision loop stopping due to shutdown");
                break;
            }
            
            // Check if child process is still running
            let needs_restart = {
                let mut child_guard = self.child_process.lock().await;
                if let Some(child) = child_guard.as_mut() {
                    match child.try_wait() {
                        Ok(Some(exit_status)) => {
                            error!("[PY] Python subprocess exited with status: {:?}", exit_status);
                            *child_guard = None; // Clear the dead process
                            true
                        }
                        Ok(None) => {
                            // Process is still running
                            false
                        }
                        Err(e) => {
                            error!("[PY] Error checking subprocess status: {}", e);
                            *child_guard = None; // Clear the potentially corrupted process
                            true
                        }
                    }
                } else {
                    // No child process, need to start one
                    true
                }
            };
            
            if needs_restart && !*self.is_shutting_down.lock().await {
                let mut restart_count = self.restart_count.lock().await;
                if *restart_count < self.config.max_restarts {
                    *restart_count += 1;
                    warn!("[PY] Attempting restart {}/{}", *restart_count, self.config.max_restarts);
                    
                    // Exponential backoff: 2^(restart_count-1) * initial_delay
                    let delay = self.config.initial_restart_delay_secs * (2_u64.pow((*restart_count - 1) as u32));
                    info!("[PY] Exponential backoff delay: {} seconds", delay);
                    sleep(Duration::from_secs(delay)).await;
                    
                    match self.spawn_subprocess().await {
                        Ok(()) => {
                            info!("[PY] Python subprocess restarted successfully");
                        }
                        Err(e) => {
                            error!("[PY] Failed to restart Python subprocess: {}", e);
                        }
                    }
                } else {
                    error!("[PY] Maximum restart attempts ({}) reached. No longer attempting restarts.", 
                          self.config.max_restarts);
                    break;
                }
            }
            
            // Wait before checking again
            tokio::select! {
                _ = sleep(Duration::from_secs(5)) => {}
                _ = self.shutdown_notify.notified() => {
                    info!("[PY] Supervision loop received shutdown notification");
                    break;
                }
            }
        }
        
        info!("[PY] Supervision loop ended");
    }

    /// Gracefully shutdown the Python server
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("[PY] Starting graceful shutdown of Python server");
        
        // Set shutdown flag
        *self.is_shutting_down.lock().await = true;
        
        // Notify supervision loop to stop
        self.shutdown_notify.notify_waiters();
        
        // ARCHITECT GUIDANCE: Ensure proper Child handle management and termination on shutdown
        // Terminate the child process
        let mut child_guard = self.child_process.lock().await;
        if let Some(mut child) = child_guard.take() {
            info!("[PY] Terminating Python subprocess");
            
            // First try graceful termination with SIGTERM
            match child.kill().await {
                Ok(()) => {
                    info!("[PY] Sent termination signal to Python subprocess");
                    
                    // Wait for graceful exit with timeout
                    match tokio::time::timeout(Duration::from_secs(10), child.wait()).await {
                        Ok(Ok(exit_status)) => {
                            info!("[PY] Python subprocess exited gracefully with status: {:?}", exit_status);
                        }
                        Ok(Err(e)) => {
                            warn!("[PY] Error waiting for Python subprocess to exit: {}", e);
                        }
                        Err(_) => {
                            warn!("[PY] Python subprocess didn't exit within timeout, force killing");
                            // Force kill if still running
                            let _ = child.kill().await;
                            let _ = child.wait().await;
                        }
                    }
                }
                Err(e) => {
                    warn!("[PY] Error terminating Python subprocess: {}", e);
                    // Try to wait anyway in case it's already exiting
                    let _ = child.wait().await;
                }
            }
        } else {
            info!("[PY] No child process to terminate");
        }
        
        info!("[PY] Python server shutdown completed");
        Ok(())
    }

    /// Check if the Python server is running and healthy with comprehensive diagnostics
    pub async fn is_healthy(&self) -> bool {
        tracing::debug!("🔍 PYTHON_HEALTH_CHECK: Starting health check");
        let start_time = std::time::Instant::now();
        
        let is_healthy = self.check_health().await.is_ok();
        let duration = start_time.elapsed();
        
        if is_healthy {
            tracing::debug!("   ✅ Python server is healthy (check completed in {:?})", duration);
        } else {
            tracing::warn!("   ❌ Python server health check failed after {:?}", duration);
        }
        
        is_healthy
    }

    /// Clone for use in async tasks
    fn clone_for_task(&self) -> PythonManagerForTask {
        PythonManagerForTask {
            config: self.config.clone(),
            child_process: self.child_process.clone(),
            restart_count: self.restart_count.clone(),
            shutdown_notify: self.shutdown_notify.clone(),
            is_shutting_down: self.is_shutting_down.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

// Helper struct for tasks that need to clone the manager
#[derive(Clone)]
struct PythonManagerForTask {
    config: PythonManagerConfig,
    child_process: Arc<Mutex<Option<Child>>>,
    restart_count: Arc<Mutex<u32>>,
    shutdown_notify: Arc<Notify>,
    is_shutting_down: Arc<Mutex<bool>>,
    http_client: reqwest::Client,
}

impl PythonManagerForTask {
    /// Check Python server health (same as main implementation)
    async fn check_health(&self) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("🔍 PYTHON_HEALTH_DIAGNOSTIC: Making health check request");
        tracing::debug!("   🌐 URL: {}", self.config.health_check_url);
        tracing::debug!("   ⏰ Timeout: {}s", self.config.health_check_timeout_secs);
        
        let start_time = std::time::Instant::now();
        
        let response = self.http_client
            .get(&self.config.health_check_url)
            .timeout(Duration::from_secs(self.config.health_check_timeout_secs))
            .send()
            .await
            .map_err(|e| {
                let duration = start_time.elapsed();
                tracing::error!("   ❌ Health check request failed after {:?}: {}", duration, e);
                e
            })?;
        
        let request_duration = start_time.elapsed();
        tracing::debug!("   📊 Response received in {:?} - Status: {}", 
                       request_duration, response.status());
        
        if !response.status().is_success() {
            tracing::error!("   ❌ Health check failed with HTTP status: {}", response.status());
            return Err(format!("Health check failed with status: {}", response.status()).into());
        }
        
        let health_data: serde_json::Value = response.json().await
            .map_err(|e| {
                tracing::error!("   ❌ Failed to parse health check JSON: {}", e);
                e
            })?;
        
        tracing::debug!("   ✅ Health check successful - Response: {:?}", health_data);
        Ok(health_data)
    }

    async fn supervision_loop(&self) {
        info!("[PY] Starting supervision loop");
        
        loop {
            // Check if we're shutting down
            if *self.is_shutting_down.lock().await {
                info!("[PY] Supervision loop stopping due to shutdown");
                break;
            }
            
            // Check if child process is still running
            let needs_restart = {
                let mut child_guard = self.child_process.lock().await;
                if let Some(child) = child_guard.as_mut() {
                    match child.try_wait() {
                        Ok(Some(exit_status)) => {
                            error!("[PY] Python subprocess exited with status: {:?}", exit_status);
                            *child_guard = None; // Clear the dead process
                            true
                        }
                        Ok(None) => {
                            // Process is still running
                            false
                        }
                        Err(e) => {
                            error!("[PY] Error checking subprocess status: {}", e);
                            *child_guard = None; // Clear the potentially corrupted process
                            true
                        }
                    }
                } else {
                    // No child process, need to start one
                    true
                }
            };
            
            if needs_restart && !*self.is_shutting_down.lock().await {
                let mut restart_count = self.restart_count.lock().await;
                if *restart_count < self.config.max_restarts {
                    *restart_count += 1;
                    warn!("[PY] Attempting restart {}/{}", *restart_count, self.config.max_restarts);
                    
                    // Exponential backoff: 2^(restart_count-1) * initial_delay
                    let delay = self.config.initial_restart_delay_secs * (2_u64.pow((*restart_count - 1) as u32));
                    info!("[PY] Exponential backoff delay: {} seconds", delay);
                    sleep(Duration::from_secs(delay)).await;
                    
                    match self.spawn_subprocess().await {
                        Ok(()) => {
                            info!("[PY] Python subprocess restarted successfully");
                        }
                        Err(e) => {
                            error!("[PY] Failed to restart Python subprocess: {}", e);
                        }
                    }
                } else {
                    error!("[PY] Maximum restart attempts ({}) reached. No longer attempting restarts.", 
                          self.config.max_restarts);
                    break;
                }
            }
            
            // Wait before checking again
            tokio::select! {
                _ = sleep(Duration::from_secs(5)) => {}
                _ = self.shutdown_notify.notified() => {
                    info!("[PY] Supervision loop received shutdown notification");
                    break;
                }
            }
        }
        
        info!("[PY] Supervision loop ended");
    }

    /// Spawn the Python subprocess
    async fn spawn_subprocess(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // ARCHITECT GUIDANCE: Probe health before spawning to avoid duplicates
        info!("[PY] Checking if Python server is already running on port 8001...");
        match self.check_health().await {
            Ok(_) => {
                info!("[PY] ✅ Python server already running and healthy, skipping subprocess spawn");
                return Ok(());
            }
            Err(_) => {
                info!("[PY] No existing Python server found, proceeding to spawn new subprocess");
            }
        }
        
        info!("[PY] Spawning Python server subprocess: {}", self.config.script_path);
        
        let mut child = Command::new("python3")
            .arg(&self.config.script_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                error!("[PY] Failed to spawn Python subprocess: {}", e);
                format!("Failed to spawn Python subprocess: {}", e)
            })?;
        
        // Start log forwarding tasks
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            tokio::spawn(async move {
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    info!("[PY] {}", line);
                }
            });
        }
        
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            tokio::spawn(async move {
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    warn!("[PY] {}", line);
                }
            });
        }
        
        // Store the child process
        *self.child_process.lock().await = Some(child);
        
        debug!("[PY] Python subprocess spawned successfully");
        Ok(())
    }
}