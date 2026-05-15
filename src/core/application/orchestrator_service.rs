use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::core::application::audit_trail::AuditTrail;
use crate::core::application::health_aggregator::HealthAggregator;
use crate::core::application::kill_switch_controller::KillSwitchController;
use crate::core::application::mode_controller::ModeController;
use crate::core::application::policy_engine::PolicyEngine;
use crate::core::application::query_handler::QueryHandler;
use crate::core::application::saga_coordinator::SagaCoordinator;
use crate::core::application::service_registry::ServiceRegistryService;
use crate::core::application::workflow_engine::WorkflowEngine;
use crate::core::domain::commands::{CommandAck, OrchestratorCommand};
use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::health::HealthSnapshot;
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::health_port::IHealthPort;
use crate::core::ports::journal_port::IJournalPort;
use crate::core::ports::metrics_port::IMetricsPort;
use crate::core::ports::service_command_port::IServiceCommandPort;

pub struct OrchestratorService {
    kill_switch: Arc<KillSwitchController>,
    mode_controller: Arc<ModeController>,
    policy_engine: Arc<PolicyEngine>,
    health_aggregator: Arc<HealthAggregator>,
    query_handler: Arc<QueryHandler>,
    audit_trail: Arc<AuditTrail>,
    service_registry: Arc<ServiceRegistryService>,
    workflow_engine: Arc<WorkflowEngine>,
    saga_coordinator: Arc<SagaCoordinator>,
    command_tx: mpsc::Sender<(OrchestratorCommand, tokio::sync::oneshot::Sender<Result<CommandAck, OrchestratorError>>)>,
}

impl OrchestratorService {
    pub fn new(
        journal: Arc<dyn IJournalPort>,
        event_bus: Arc<dyn IEventBusPort>,
        health_port: Arc<dyn IHealthPort>,
        metrics: Arc<dyn IMetricsPort>,
        service_command: Arc<dyn IServiceCommandPort>,
    ) -> Self {
        let health_snapshot = Arc::new(RwLock::new(HealthSnapshot::new()));
        let kill_switch_active = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let kill_switch = Arc::new(KillSwitchController::new(
            journal.clone(),
            event_bus.clone(),
            service_command.clone(),
        ));

        let mode_controller = Arc::new(ModeController::new(
            journal.clone(),
            event_bus.clone(),
        ));

        let policy_engine = Arc::new(PolicyEngine::new(
            journal.clone(),
            event_bus.clone(),
        ));

        let health_aggregator = Arc::new(HealthAggregator::new(
            health_port.clone(),
            health_snapshot.clone(),
            metrics.clone(),
        ));

        let query_handler = Arc::new(QueryHandler::new(
            health_snapshot.clone(),
            kill_switch_active.clone(),
        ));

        let audit_trail = Arc::new(AuditTrail::new(journal.clone()));

        let service_registry = Arc::new(ServiceRegistryService::new());

        let workflow_engine = Arc::new(WorkflowEngine::new(
            journal.clone(),
            event_bus.clone(),
        ));

        let mut saga_coordinator = SagaCoordinator::new(
            journal.clone(),
            event_bus.clone(),
        );
        crate::core::application::named_sagas::register_all_named_sagas(&mut saga_coordinator);
        let saga_coordinator = Arc::new(saga_coordinator);

        let (command_tx, _command_rx) = mpsc::channel(1000);

        Self {
            kill_switch,
            mode_controller,
            policy_engine,
            health_aggregator,
            query_handler,
            audit_trail,
            service_registry,
            workflow_engine,
            saga_coordinator,
            command_tx,
        }
    }

    pub async fn handle_command(
        &self,
        cmd: OrchestratorCommand,
    ) -> Result<CommandAck, OrchestratorError> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.command_tx
            .send((cmd, reply_tx))
            .await
            .map_err(|_| OrchestratorError::CommandQueueFull)?;
        reply_rx
            .await
            .map_err(|_| OrchestratorError::Internal("Command reply channel closed".to_string()))?
    }

    pub async fn start(&self) -> Result<(), OrchestratorError> {
        self.kill_switch.restore_from_journal().await?;
        self.mode_controller.restore_from_journal().await?;
        self.workflow_engine.restore_from_journal().await?;
        self.saga_coordinator.restore_from_journal().await?;

        self.health_aggregator.start_polling().await;

        tracing::info!("Orchestrator service started");
        Ok(())
    }

    pub fn kill_switch(&self) -> &Arc<KillSwitchController> {
        &self.kill_switch
    }

    pub fn mode_controller(&self) -> &Arc<ModeController> {
        &self.mode_controller
    }

    pub fn policy_engine(&self) -> &Arc<PolicyEngine> {
        &self.policy_engine
    }

    pub fn query_handler(&self) -> &Arc<QueryHandler> {
        &self.query_handler
    }

    pub fn audit_trail(&self) -> &Arc<AuditTrail> {
        &self.audit_trail
    }

    pub fn service_registry(&self) -> &Arc<ServiceRegistryService> {
        &self.service_registry
    }

    pub fn workflow_engine(&self) -> &Arc<WorkflowEngine> {
        &self.workflow_engine
    }

    pub fn saga_coordinator(&self) -> &Arc<SagaCoordinator> {
        &self.saga_coordinator
    }
}
