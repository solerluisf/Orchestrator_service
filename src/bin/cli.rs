use clap::Parser;
use orchestrator_service::infra::cli::commands::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let base = &cli.base_url;

    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(ref token) = cli.token {
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
    }

    match cli.command {
        Commands::Health { command } => {
            match command.unwrap_or(HealthCommands::System) {
                HealthCommands::System => {
                    let resp = client.get(format!("{}/health", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                HealthCommands::Services => {
                    let resp = client.get(format!("{}/health/services", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                HealthCommands::Service { service_id } => {
                    let resp = client.get(format!("{}/health/services/{}", base, service_id)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                HealthCommands::Workflows => {
                    let resp = client.get(format!("{}/health/workflows", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                HealthCommands::Sagas => {
                    let resp = client.get(format!("{}/health/sagas", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
            }
        }
        Commands::KillSwitch { command } => {
            match command {
                KillSwitchCommands::Activate { reason } => {
                    let body = serde_json::json!({"reason": reason, "actor": "cli"});
                    let resp = client.post(format!("{}/control/kill-switch/activate", base)).headers(headers.clone()).json(&body).send().await?;
                    println!("{}", resp.text().await?);
                }
                KillSwitchCommands::Clear { reason } => {
                    let body = serde_json::json!({"reason": reason, "actor": "cli"});
                    let resp = client.post(format!("{}/control/kill-switch/clear", base)).headers(headers.clone()).json(&body).send().await?;
                    println!("{}", resp.text().await?);
                }
                KillSwitchCommands::Status => {
                    let resp = client.get(format!("{}/control/kill-switch/status", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
            }
        }
        Commands::Mode { command } => {
            match command {
                ModeCommands::Get => {
                    let resp = client.get(format!("{}/control/mode", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                ModeCommands::Transition { to, reason, dry_run } => {
                    let body = serde_json::json!({
                        "target_mode": to,
                        "reason": reason.unwrap_or_else(|| "CLI transition".to_string()),
                        "dry_run": dry_run,
                    });
                    let resp = client.post(format!("{}/control/mode/transition", base)).headers(headers.clone()).json(&body).send().await?;
                    println!("{}", resp.text().await?);
                }
                ModeCommands::History => {
                    let resp = client.get(format!("{}/control/mode/history", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
            }
        }
        Commands::Policy { command } => {
            match command {
                PolicyCommands::List => {
                    let resp = client.get(format!("{}/control/policies", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                PolicyCommands::Update { policy_id, key, value } => {
                    let body: serde_json::Value = serde_json::from_str(&format!("{{\"{}\":{}}}", key, value))
                        .unwrap_or(serde_json::json!({key: value}));
                    let resp = client.post(format!("{}/control/policies/{}/update", base, policy_id)).headers(headers.clone()).json(&body).send().await?;
                    println!("{}", resp.text().await?);
                }
                PolicyCommands::Reload => {
                    let resp = client.post(format!("{}/control/policies/reload", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
            }
        }
        Commands::Service { command } => {
            match command {
                ServiceCommands::List => {
                    let resp = client.get(format!("{}/control/services", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                ServiceCommands::Pause { service_id, reason } => {
                    let body = serde_json::json!({"reason": reason.unwrap_or_else(|| "CLI pause".to_string())});
                    let resp = client.post(format!("{}/control/services/{}/pause", base, service_id)).headers(headers.clone()).json(&body).send().await?;
                    println!("{}", resp.text().await?);
                }
                ServiceCommands::Resume { service_id, reason } => {
                    let body = serde_json::json!({"reason": reason.unwrap_or_else(|| "CLI resume".to_string())});
                    let resp = client.post(format!("{}/control/services/{}/resume", base, service_id)).headers(headers.clone()).json(&body).send().await?;
                    println!("{}", resp.text().await?);
                }
            }
        }
        Commands::Audit { command } => {
            match command {
                AuditCommands::Tail { limit } => {
                    let resp = client.get(format!("{}/audit?limit={}", base, limit)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                AuditCommands::Query { from, to, action } => {
                    let mut params = Vec::new();
                    if let Some(f) = from { params.push(format!("from={}", f)); }
                    if let Some(t) = to { params.push(format!("to={}", t)); }
                    if let Some(a) = action { params.push(format!("action={}", a)); }
                    let query = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
                    let resp = client.get(format!("{}/audit{}", base, query)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
            }
        }
        Commands::Workflow { command } => {
            match command {
                WorkflowCommands::List => {
                    let resp = client.get(format!("{}/workflows", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                WorkflowCommands::Trigger { workflow_id } => {
                    let body = serde_json::json!({});
                    let resp = client.post(format!("{}/workflows/{}/trigger", base, workflow_id)).headers(headers.clone()).json(&body).send().await?;
                    println!("{}", resp.text().await?);
                }
                WorkflowCommands::Status { instance_id } => {
                    let resp = client.get(format!("{}/workflows/instances/{}", base, instance_id)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                WorkflowCommands::Cancel { instance_id } => {
                    let resp = client.post(format!("{}/workflows/instances/{}/cancel", base, instance_id)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
            }
        }
        Commands::Config { command } => {
            match command.unwrap_or(ConfigCommands::Show) {
                ConfigCommands::Show => {
                    let resp = client.get(format!("{}/config", base)).headers(headers.clone()).send().await?;
                    println!("{}", resp.text().await?);
                }
                ConfigCommands::Reload => {
                    println!("Config reload: use POST /config/reload via API");
                }
            }
        }
        Commands::Auth { command } => {
            match command {
                AuthCommands::Login { user_id, role } => {
                    let body = serde_json::json!({"user_id": user_id, "role": role});
                    let resp = client.post(format!("{}/auth/token", base)).json(&body).send().await?;
                    println!("{}", resp.text().await?);
                }
            }
        }
    }

    Ok(())
}
