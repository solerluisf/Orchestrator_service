use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "orchestrator-cli")]
#[command(about = "Orchestrator Service CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Health status commands
    Health {
        #[command(subcommand)]
        command: Option<HealthCommands>,
    },
    /// Kill switch commands
    KillSwitch {
        #[command(subcommand)]
        command: KillSwitchCommands,
    },
    /// Operation mode commands
    Mode {
        #[command(subcommand)]
        command: ModeCommands,
    },
    /// Policy commands
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },
    /// Service control commands
    Service {
        #[command(subcommand)]
        command: ServiceCommands,
    },
    /// Audit commands
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },
    /// Config commands
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
}

#[derive(Subcommand)]
pub enum HealthCommands {
    /// Show system health
    System,
    /// Show all service health
    Services,
    /// Show specific service health
    Service {
        #[arg(short, long)]
        service_id: String,
    },
}

#[derive(Subcommand)]
pub enum KillSwitchCommands {
    /// Activate kill switch
    Activate {
        #[arg(short, long)]
        reason: String,
    },
    /// Clear kill switch
    Clear {
        #[arg(short, long)]
        reason: String,
    },
    /// Show kill switch status
    Status,
}

#[derive(Subcommand)]
pub enum ModeCommands {
    /// Get current operation mode
    Get,
    /// Transition to a new mode
    Transition {
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        reason: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Show mode transition history
    History,
}

#[derive(Subcommand)]
pub enum PolicyCommands {
    /// List all policies
    List,
    /// Update a policy
    Update {
        #[arg(short, long)]
        policy_id: String,
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        value: String,
    },
    /// Reload all policies from config
    Reload,
}

#[derive(Subcommand)]
pub enum ServiceCommands {
    /// Pause a service
    Pause {
        #[arg(short, long)]
        service_id: String,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Resume a service
    Resume {
        #[arg(short, long)]
        service_id: String,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Reset circuit breaker for a service
    CircuitBreakerReset {
        #[arg(short, long)]
        service_id: String,
    },
}

#[derive(Subcommand)]
pub enum AuditCommands {
    /// Show recent audit entries
    Tail {
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Query audit entries
    Query {
        #[arg(short, long)]
        from: Option<String>,
        #[arg(short, long)]
        to: Option<String>,
        #[arg(short, long)]
        action: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current config
    Show,
    /// Reload config from manifest
    Reload,
    /// Show config diff
    Diff,
}
