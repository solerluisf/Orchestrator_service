use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaderState {
    Leader,
    Standby,
}

pub struct LeaderElection {
    state: Arc<RwLock<LeaderState>>,
    lock_file: PathBuf,
    identity: String,
}

impl LeaderElection {
    pub fn new(data_dir: &str, identity: &str) -> Self {
        let _ = std::fs::create_dir_all(data_dir);
        Self {
            state: Arc::new(RwLock::new(LeaderState::Standby)),
            lock_file: PathBuf::from(data_dir).join("leader.lock"),
            identity: identity.to_string(),
        }
    }

    pub async fn try_acquire_leadership(&self) -> bool {
        loop {
            let lock_content = format!("{}\n{}\n",
                self.identity,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );

            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.lock_file)
            {
                Ok(mut file) => {
                    use std::io::Write;
                    let _ = file.write_all(lock_content.as_bytes());
                    *self.state.write().await = LeaderState::Leader;
                    tracing::info!(identity = %self.identity, "Acquired leadership");
                    return true;
                }
                Err(_) => {
                    if let Ok(content) = std::fs::read_to_string(&self.lock_file) {
                        if let Some(ts_str) = content.lines().nth(1) {
                            if let Ok(ts) = ts_str.parse::<u64>() {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                if now - ts > 30 {
                                    let _ = std::fs::remove_file(&self.lock_file);
                                    continue;
                                }
                            }
                        }
                    }
                    *self.state.write().await = LeaderState::Standby;
                    return false;
                }
            }
        }
    }

    pub async fn release_leadership(&self) {
        let _ = std::fs::remove_file(&self.lock_file);
        *self.state.write().await = LeaderState::Standby;
        tracing::info!(identity = %self.identity, "Released leadership");
    }

    pub async fn is_leader(&self) -> bool {
        *self.state.read().await == LeaderState::Leader
    }

    pub async fn state(&self) -> LeaderState {
        self.state.read().await.clone()
    }

    pub async fn start_heartbeat(&self) {
        let state = self.state.clone();
        let lock_file = self.lock_file.clone();
        let identity = self.identity.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                if *state.read().await == LeaderState::Leader {
                    let content = format!("{}\n{}\n",
                        identity,
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    );
                    let _ = std::fs::write(&lock_file, &content);
                }
            }
        });
    }
}
