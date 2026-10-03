use crate::{Snapshot, Store, StoreError};
use evobase_appspec::{
    MAX_BYTES, Scope,
    commands::{RawCommandRequest, TransitionReceipt, decide_command, prepare_command},
    policy::{Grant, HostAuthority, RequestChannel, SessionVerifier, TrustedContext},
};
use libsql::{Connection, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

/// A committed output DTO. Deserializing a receipt grants no authority to mutate or replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitReceipt {
    transition: TransitionReceipt,
    request_key: String,
    replayed: bool,
    revision: u64,
    expected_revision: u64,
    spec_identity: String,
}
impl CommitReceipt {
    pub fn transition(&self) -> &TransitionReceipt {
        &self.transition
    }
    pub fn request_key(&self) -> &str {
        &self.request_key
    }
    pub fn replayed(&self) -> bool {
        self.replayed
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    pub fn spec_identity(&self) -> &str {
        &self.spec_identity
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestEnvelope {
    expected_revision: u64,
    request: RawCommandRequest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommitStage {
    Facts,
    Audit,
    Events,
    Receipt,
}

impl Store {
    /// Loads current facts and authority inside a single immediate transaction. An exact retry
    /// returns its original receipt; a changed intent, release or expected revision conflicts.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute<V: SessionVerifier>(
        &self,
        host: &HostAuthority<V>,
        credential: &str,
        scope: &Scope,
        channel: &RequestChannel,
        now: u64,
        expected_revision: u64,
        request: &RawCommandRequest,
    ) -> Result<CommitReceipt, StoreError> {
        self.execute_at(
            host,
            credential,
            scope,
            channel,
            now,
            expected_revision,
            request,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_at<V: SessionVerifier>(
        &self,
        host: &HostAuthority<V>,
        credential: &str,
        scope: &Scope,
        channel: &RequestChannel,
        now: u64,
        expected_revision: u64,
        request: &RawCommandRequest,
        fault: Option<CommitStage>,
    ) -> Result<CommitReceipt, StoreError> {
        self.check_scope(scope)?;
        let connection = self.connection()?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;
        let result = self
            .execute_in(
                &tx,
                host,
                credential,
                scope,
                channel,
                now,
                expected_revision,
                request,
                fault,
            )
            .await;
        match result {
            Ok(receipt) => {
                tx.commit().await?;
                Ok(receipt)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_in<V: SessionVerifier>(
        &self,
        connection: &Connection,
        host: &HostAuthority<V>,
        credential: &str,
        scope: &Scope,
        channel: &RequestChannel,
        now: u64,
        expected_revision: u64,
        request: &RawCommandRequest,
        fault: Option<CommitStage>,
    ) -> Result<CommitReceipt, StoreError> {
        // Scope is a selector. The configured store and the host registry must agree before load.
        let context = host.resolve(credential, scope, channel, now, Grant::Write)?;
        self.check_binding(&context)?;
        let snapshot = self.load(connection, scope).await?;
        let prepared = prepare_command(
            host,
            credential,
            scope,
            channel,
            now,
            snapshot.spec(),
            snapshot.facts(),
            request,
        )?;
        if prepared.authority() != &context {
            return Err(StoreError::AuthorityChanged);
        }
        let intent = normalized_identity(&snapshot, expected_revision, prepared.intent_bytes())?;
        if let Some((previous, mut receipt, _)) = self
            .load_receipt(
                connection,
                scope,
                context.actor().as_str(),
                prepared.idempotency_key(),
            )
            .await?
        {
            if previous != intent {
                return Err(StoreError::IntentConflict);
            }
            self.validate_receipt(&snapshot, request, expected_revision, &receipt)?;
            self.revalidate(host, credential, scope, channel, now, &context)?;
            receipt.replayed = true;
            return Ok(receipt);
        }
        if expected_revision != snapshot.revision() {
            return Err(StoreError::Conflict {
                expected: expected_revision,
                actual: snapshot.revision(),
            });
        }
        let batch = decide_command(prepared, snapshot.revision())?;
        let mut final_rows = snapshot.facts().to_raw();
        for write in batch.writes() {
            let target = final_rows
                .iter_mut()
                .find(|row| row.id == write.id && row.table_id == write.table_id)
                .ok_or(StoreError::CorruptSnapshot)?;
            *target = write.clone();
        }
        let final_facts = snapshot.spec().validate_records(scope, &final_rows)?;
        let revision = snapshot
            .revision()
            .checked_add(1)
            .ok_or(StoreError::RevisionOverflow)?;
        let sql_revision = i64::try_from(revision).map_err(|_| StoreError::RevisionOverflow)?;
        let changed = connection.execute(
            "UPDATE applications SET facts_json = ?1, revision = ?2 WHERE app_id = ?3 AND revision = ?4 AND spec_identity = ?5",
            params![final_facts.encode()?, sql_revision, scope.app_id().as_str(), i64::try_from(expected_revision).map_err(|_| StoreError::RevisionOverflow)?, snapshot.spec_identity()],
        ).await?;
        if changed != 1 {
            return Err(StoreError::Conflict {
                expected: expected_revision,
                actual: snapshot.revision(),
            });
        }
        fail_at(fault, CommitStage::Facts)?;
        let audit = encode(&serde_json::json!({
            "actor_id": batch.actor().as_str(), "scope": scope, "binding_id": batch.binding_id(),
            "membership_revision": batch.membership_revision(), "policy_revision": batch.policy_revision(),
            "command_id": request.command_id, "record_id": request.record_id,
            "expected_revision": expected_revision, "revision": revision,
            "spec_identity": snapshot.spec_identity(), "host_time": now,
        }))?;
        connection
            .execute(
                "INSERT INTO command_audit VALUES (?1, ?2, ?3)",
                params![scope.app_id().as_str(), sql_revision, audit],
            )
            .await?;
        fail_at(fault, CommitStage::Audit)?;
        for (index, event) in batch.events().iter().enumerate() {
            let event_json = encode(&serde_json::json!({
                "event_id": event.event_id(), "payload": event.payload(),
                "revision": revision, "spec_identity": snapshot.spec_identity(),
            }))?;
            connection
                .execute(
                    "INSERT INTO committed_events VALUES (?1, ?2, ?3, ?4)",
                    params![
                        scope.app_id().as_str(),
                        sql_revision,
                        i64::try_from(index).map_err(|_| StoreError::CorruptSnapshot)?,
                        event_json
                    ],
                )
                .await?;
        }
        fail_at(fault, CommitStage::Events)?;
        let receipt = CommitReceipt {
            transition: batch.receipt().clone(),
            request_key: request.idempotency_key.clone(),
            replayed: false,
            revision,
            expected_revision,
            spec_identity: snapshot.spec_identity().to_owned(),
        };
        let request_json = encode(&RequestEnvelope {
            expected_revision,
            request: request.clone(),
        })?;
        connection
            .execute(
                "INSERT INTO command_receipts VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    scope.app_id().as_str(),
                    batch.actor().as_str(),
                    request.idempotency_key.as_str(),
                    intent,
                    request_json,
                    encode(&receipt)?
                ],
            )
            .await?;
        fail_at(fault, CommitStage::Receipt)?;
        // External membership is re-read and compared immediately before this transaction commits.
        // This detects observed changes; it does not claim a distributed transaction with a host.
        self.revalidate(host, credential, scope, channel, now, &context)?;
        Ok(receipt)
    }

    /// Receipt recovery requires current host grants and current row policy, including after restart.
    #[allow(clippy::too_many_arguments)]
    pub async fn receipt<V: SessionVerifier>(
        &self,
        host: &HostAuthority<V>,
        credential: &str,
        scope: &Scope,
        channel: &RequestChannel,
        now: u64,
        request_key: &str,
    ) -> Result<Option<CommitReceipt>, StoreError> {
        self.check_scope(scope)?;
        let context = host.resolve(credential, scope, channel, now, Grant::Write)?;
        self.check_binding(&context)?;
        if request_key.is_empty()
            || request_key.len() > 96
            || !request_key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err(StoreError::InvalidReceiptKey);
        }
        let connection = self.connection()?;
        let tx = connection.transaction().await?;
        let result = async {
            let Some((intent, mut receipt, request)) = self
                .load_receipt(&tx, scope, context.actor().as_str(), request_key)
                .await?
            else {
                return Ok(None);
            };
            let snapshot = self.load(&tx, scope).await?;
            let prepared = prepare_command(
                host,
                credential,
                scope,
                channel,
                now,
                snapshot.spec(),
                snapshot.facts(),
                &request.request,
            )?;
            if prepared.authority() != &context {
                return Err(StoreError::AuthorityChanged);
            }
            if normalized_identity(
                &snapshot,
                request.expected_revision,
                prepared.intent_bytes(),
            )? != intent
            {
                return Err(StoreError::IntentConflict);
            }
            self.validate_receipt(
                &snapshot,
                &request.request,
                request.expected_revision,
                &receipt,
            )?;
            self.revalidate(host, credential, scope, channel, now, &context)?;
            receipt.replayed = true;
            Ok(Some(receipt))
        }
        .await;
        // A read transaction never commits writes and is explicitly closed for remote connections.
        tx.rollback().await?;
        result
    }

    async fn load_receipt(
        &self,
        connection: &Connection,
        scope: &Scope,
        actor: &str,
        key: &str,
    ) -> Result<Option<(Vec<u8>, CommitReceipt, RequestEnvelope)>, StoreError> {
        let mut rows = connection.query("SELECT intent_json, receipt_json, request_json FROM command_receipts WHERE app_id = ?1 AND actor_id = ?2 AND request_key = ?3", params![scope.app_id().as_str(), actor, key]).await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let intent = row.get::<Vec<u8>>(0)?;
        let receipt_json = row.get::<Vec<u8>>(1)?;
        let request_json = row.get::<Vec<u8>>(2)?;
        if intent.len() > MAX_BYTES
            || receipt_json.len() > MAX_BYTES
            || request_json.len() > MAX_BYTES
        {
            return Err(StoreError::PayloadLimit);
        }
        let receipt =
            serde_json::from_slice(&receipt_json).map_err(|_| StoreError::CorruptSnapshot)?;
        let request: RequestEnvelope =
            serde_json::from_slice(&request_json).map_err(|_| StoreError::CorruptSnapshot)?;
        if request.request.idempotency_key != key {
            return Err(StoreError::CorruptSnapshot);
        }
        Ok(Some((intent, receipt, request)))
    }

    fn validate_receipt(
        &self,
        snapshot: &Snapshot,
        request: &RawCommandRequest,
        expected_revision: u64,
        receipt: &CommitReceipt,
    ) -> Result<(), StoreError> {
        if receipt.expected_revision != expected_revision
            || receipt.request_key != request.idempotency_key
            || receipt.replayed
            || receipt.revision
                != expected_revision
                    .checked_add(1)
                    .ok_or(StoreError::CorruptSnapshot)?
            || receipt.revision > snapshot.revision()
            || receipt.spec_identity != snapshot.spec_identity()
            || receipt.transition.command_id() != &request.command_id
            || receipt.transition.record_id() != &request.record_id
            || !snapshot.spec().definition().commands.iter().any(|command| {
                command.command_id == request.command_id
                    && command.to_state == receipt.transition.state()
            })
        {
            return Err(StoreError::CorruptSnapshot);
        }
        Ok(())
    }

    fn check_binding(&self, context: &TrustedContext) -> Result<(), StoreError> {
        if context.binding_id() != self.binding_id() {
            return Err(StoreError::BindingMismatch);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn revalidate<V: SessionVerifier>(
        &self,
        host: &HostAuthority<V>,
        credential: &str,
        scope: &Scope,
        channel: &RequestChannel,
        now: u64,
        previous: &TrustedContext,
    ) -> Result<(), StoreError> {
        let current = host.resolve(credential, scope, channel, now, Grant::Write)?;
        self.check_binding(&current)?;
        if &current != previous {
            return Err(StoreError::AuthorityChanged);
        }
        Ok(())
    }
}

fn normalized_identity(
    snapshot: &Snapshot,
    expected_revision: u64,
    semantic_intent: &[u8],
) -> Result<Vec<u8>, StoreError> {
    let semantic: serde_json::Value =
        serde_json::from_slice(semantic_intent).map_err(|_| StoreError::CorruptSnapshot)?;
    encode(
        &serde_json::json!({ "spec_identity": snapshot.spec_identity(), "expected_revision": expected_revision, "semantic_intent": semantic }),
    )
}

fn encode(value: &impl Serialize) -> Result<Vec<u8>, StoreError> {
    let bytes = serde_json::to_vec(value).map_err(|_| StoreError::CorruptSnapshot)?;
    if bytes.len() > MAX_BYTES {
        return Err(StoreError::PayloadLimit);
    }
    Ok(bytes)
}

fn fail_at(selected: Option<CommitStage>, stage: CommitStage) -> Result<(), StoreError> {
    if selected == Some(stage) {
        return Err(StoreError::InjectedFailure);
    }
    Ok(())
}

#[cfg(test)]
#[path = "atomic_tests.rs"]
mod tests;
