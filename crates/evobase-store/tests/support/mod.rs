#![allow(dead_code)]
use evobase_appspec::{
    CheckedAppSpec, CheckedRecords, CommandId, FieldId, RecordId, Scope, Value,
    commands::RawCommandRequest,
    fixtures,
    policy::{
        ActorId, AuthorityError, BindingFacts, Grant, MembershipFacts, SessionFacts,
        SessionVerifier,
    },
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};

#[derive(Clone)]
pub struct FixtureHost {
    pub scope: Scope,
    pub binding_id: String,
    pub active: Arc<AtomicBool>,
    pub revision: Arc<AtomicU64>,
    pub reads: Arc<AtomicUsize>,
    pub race_on_read: usize,
    pub remove_roles_on_read: usize,
    pub expires_on_clock_read: usize,
    pub clock_reads: Arc<AtomicUsize>,
}
impl FixtureHost {
    pub fn new(scope: &Scope, binding_id: &str) -> Self {
        Self {
            scope: scope.clone(),
            binding_id: binding_id.to_owned(),
            active: Arc::new(AtomicBool::new(true)),
            revision: Arc::new(AtomicU64::new(1)),
            reads: Arc::new(AtomicUsize::new(0)),
            race_on_read: usize::MAX,
            remove_roles_on_read: usize::MAX,
            expires_on_clock_read: usize::MAX,
            clock_reads: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn revoke(&self) {
        self.active.store(false, Ordering::SeqCst);
    }
}
impl SessionVerifier for FixtureHost {
    fn current_time(&self, provided_now: u64) -> u64 {
        if self.clock_reads.fetch_add(1, Ordering::SeqCst) + 1 >= self.expires_on_clock_read {
            1000
        } else {
            provided_now
        }
    }
    fn verify_session(&self, credential: &str) -> Result<SessionFacts, AuthorityError> {
        if credential != "fixture-verified" {
            return Err(AuthorityError::Unverified);
        }
        Ok(SessionFacts {
            actor: ActorId::new("actor_alice")?,
            expires_at: 1000,
            revoked: false,
        })
    }
    fn current_membership(
        &self,
        actor: &ActorId,
        scope: &Scope,
    ) -> Result<MembershipFacts, AuthorityError> {
        if scope != &self.scope {
            return Err(AuthorityError::WrongScope);
        }
        let call = self.reads.fetch_add(1, Ordering::SeqCst) + 1;
        if call == self.race_on_read {
            self.revision.fetch_add(1, Ordering::SeqCst);
        }
        Ok(MembershipFacts {
            actor: actor.clone(),
            scope: scope.clone(),
            active: self.active.load(Ordering::SeqCst),
            revision: self.revision.load(Ordering::SeqCst),
            grants: [Grant::Read, Grant::Write].into_iter().collect(),
            roles: if self.remove_roles_on_read != usize::MAX && call < self.remove_roles_on_read {
                [evobase_appspec::policy::RoleId::new("role_editor").unwrap()]
                    .into_iter()
                    .collect()
            } else {
                Default::default()
            },
        })
    }
    fn registry_binding(&self, scope: &Scope) -> Result<BindingFacts, AuthorityError> {
        Ok(BindingFacts {
            scope: scope.clone(),
            binding_id: self.binding_id.clone(),
            allowed_mutation_origins: Default::default(),
        })
    }
}

pub fn fixture() -> (CheckedAppSpec, Scope, CheckedRecords, RawCommandRequest) {
    let spec = CheckedAppSpec::compile(fixtures::support_workflow_spec()).unwrap();
    let scope = Scope::new("tenant_local", spec.app_id().clone()).unwrap();
    let facts = spec
        .validate_records(&scope, &fixtures::support_workflow_records(&scope))
        .unwrap();
    let request = RawCommandRequest {
        command_id: CommandId::new("cmd_resolve_request").unwrap(),
        record_id: RecordId::new("rec_request_1").unwrap(),
        idempotency_key: "finish_once".to_owned(),
        inputs: BTreeMap::from([(
            FieldId::new("fld_request_note").unwrap(),
            Value::Text("Resolved after inspection".to_owned()),
        )]),
    };
    (spec, scope, facts, request)
}
