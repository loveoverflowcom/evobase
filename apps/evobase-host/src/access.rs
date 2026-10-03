//! Operator-issued pilot credentials, separate from portable AppSpec and legacy JWT auth.
//! The host reads current access facts for every resolution, including receipt recovery.
use evobase_appspec::{
    Scope,
    policy::{
        ActorId, AuthorityError, BindingFacts, Grant, MembershipFacts, RoleId, SessionFacts,
        SessionVerifier,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use subtle::ConstantTimeEq;

const MAX_ACCESS_BYTES: u64 = 16_384;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PilotGrant {
    Design,
    Publish,
    Manage,
    Read,
    Write,
    Submit,
}
impl From<PilotGrant> for Grant {
    fn from(grant: PilotGrant) -> Self {
        match grant {
            PilotGrant::Design => Self::Design,
            PilotGrant::Publish => Self::Publish,
            PilotGrant::Manage => Self::Manage,
            PilotGrant::Read => Self::Read,
            PilotGrant::Write => Self::Write,
            PilotGrant::Submit => Self::Submit,
        }
    }
}

/// This file is server-only operator configuration, never accepted in an HTTP request.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PilotAccess {
    pub token_sha256: String,
    pub actor_id: String,
    pub expires_at: u64,
    pub revoked: bool,
    pub active: bool,
    pub membership_revision: u64,
    pub roles: Vec<String>,
    pub grants: Vec<PilotGrant>,
}
impl PilotAccess {
    fn validate(&self) -> Result<(), AuthorityError> {
        hex_digest(&self.token_sha256)?;
        ActorId::new(&self.actor_id)?;
        if self.roles.len() > 64 || self.grants.len() > 6 || self.expires_at == 0 {
            return Err(AuthorityError::InvalidIdentity);
        }
        for role in &self.roles {
            RoleId::new(role)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum AccessSource {
    File(PathBuf),
    /// Trusted in-process operator fixture; HTTP callers cannot mutate this source.
    Memory(Arc<RwLock<PilotAccess>>),
}
impl AccessSource {
    fn current(&self) -> Result<PilotAccess, AuthorityError> {
        let access = match self {
            Self::Memory(facts) => facts
                .read()
                .map_err(|_| AuthorityError::BindingUnavailable)?
                .clone(),
            Self::File(path) => {
                use std::io::Read;
                let file =
                    std::fs::File::open(path).map_err(|_| AuthorityError::BindingUnavailable)?;
                let mut bytes = Vec::new();
                file.take(MAX_ACCESS_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| AuthorityError::BindingUnavailable)?;
                if bytes.len() as u64 > MAX_ACCESS_BYTES {
                    return Err(AuthorityError::BindingUnavailable);
                }
                serde_json::from_slice(&bytes).map_err(|_| AuthorityError::BindingUnavailable)?
            }
        };
        access.validate()?;
        Ok(access)
    }
}

#[derive(Clone)]
pub struct PilotVerifier {
    source: AccessSource,
    scope: Scope,
    binding_id: String,
    origins: BTreeSet<String>,
}
impl PilotVerifier {
    pub fn new(
        source: AccessSource,
        scope: Scope,
        binding_id: String,
        origins: BTreeSet<String>,
    ) -> Result<Self, AuthorityError> {
        source.current()?;
        if binding_id.is_empty()
            || binding_id.len() > 96
            || origins.len() > 8
            || origins.iter().any(|o| {
                o.len() > 256
                    || !(o.starts_with("http://") || o.starts_with("https://"))
                    || o.contains(['\r', '\n', '*'])
            })
        {
            return Err(AuthorityError::BindingUnavailable);
        }
        Ok(Self {
            source,
            scope,
            binding_id,
            origins,
        })
    }
    pub fn allows_origin(&self, origin: &str) -> bool {
        self.origins.contains(origin)
    }
    pub(crate) fn snapshot_for_operation(&self) -> Result<Self, AuthorityError> {
        let mut snapshot = self.clone();
        snapshot.source = AccessSource::Memory(Arc::new(RwLock::new(self.source.current()?)));
        Ok(snapshot)
    }
}
impl SessionVerifier for PilotVerifier {
    fn current_time(&self, _fallback_now: u64) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(u64::MAX, |duration| duration.as_secs())
    }
    fn access_snapshot(
        &self,
        credential: &str,
        scope: &Scope,
    ) -> Result<(SessionFacts, MembershipFacts, BindingFacts), AuthorityError> {
        // One file read binds the verified token and current membership to the same generation.
        // In particular, token rotation cannot combine an old token with elevated new grants.
        let current = self.source.current()?;
        let session = session_from(&current, credential)?;
        if scope != &self.scope {
            return Err(AuthorityError::WrongScope);
        }
        let membership = membership_from(&current, &self.scope)?;
        let binding = self.registry_binding(scope)?;
        Ok((session, membership, binding))
    }
    fn verify_session(&self, credential: &str) -> Result<SessionFacts, AuthorityError> {
        let current = self.source.current()?;
        session_from(&current, credential)
    }
    fn current_membership(
        &self,
        actor: &ActorId,
        scope: &Scope,
    ) -> Result<MembershipFacts, AuthorityError> {
        if scope != &self.scope {
            return Err(AuthorityError::WrongScope);
        }
        let current = self.source.current()?;
        let canonical = ActorId::new(&current.actor_id)?;
        if actor != &canonical || current.revoked {
            return Err(AuthorityError::Revoked);
        }
        membership_from(&current, &self.scope)
    }
    fn registry_binding(&self, scope: &Scope) -> Result<BindingFacts, AuthorityError> {
        if scope != &self.scope {
            return Err(AuthorityError::WrongScope);
        }
        Ok(BindingFacts {
            scope: self.scope.clone(),
            binding_id: self.binding_id.clone(),
            allowed_mutation_origins: self.origins.clone(),
        })
    }
}
fn session_from(current: &PilotAccess, credential: &str) -> Result<SessionFacts, AuthorityError> {
    // Operator contract: 32 random bytes encoded by `openssl rand -hex 32`.
    if credential.len() != 64
        || !credential
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(AuthorityError::Unverified);
    }
    let expected = hex_digest(&current.token_sha256)?;
    let actual: [u8; 32] = Sha256::digest(credential.as_bytes()).into();
    if !bool::from(actual.ct_eq(&expected)) {
        return Err(AuthorityError::Unverified);
    }
    Ok(SessionFacts {
        actor: ActorId::new(&current.actor_id)?,
        expires_at: current.expires_at,
        revoked: current.revoked,
    })
}
fn membership_from(
    current: &PilotAccess,
    scope: &Scope,
) -> Result<MembershipFacts, AuthorityError> {
    Ok(MembershipFacts {
        actor: ActorId::new(&current.actor_id)?,
        scope: scope.clone(),
        active: current.active,
        revision: current.membership_revision,
        grants: current.grants.iter().copied().map(Into::into).collect(),
        roles: current
            .roles
            .iter()
            .map(|r| RoleId::new(r))
            .collect::<Result<_, _>>()?,
    })
}
fn hex_digest(value: &str) -> Result<[u8; 32], AuthorityError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(AuthorityError::InvalidIdentity);
    }
    let mut out = [0; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16)
            .map_err(|_| AuthorityError::InvalidIdentity)?;
    }
    Ok(out)
}
