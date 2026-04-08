//! Permission-based access control.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: Action,
    pub scope: Scope,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Read,
    Write,
    Delete,
    Manage,
    Execute,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    Own,
    Team,
    All,
    None,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: Action) -> Self {
        Self {
            resource: resource.into(),
            action,
            scope: Scope::Own,
        }
    }

    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    pub fn matches(&self, resource: &str, action: &Action) -> bool {
        self.resource == resource && (self.action == *action || self.action == Action::Manage)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn remove(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    pub fn has(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn can(&self, resource: &str, action: &Action) -> bool {
        self.permissions.iter().any(|p| p.matches(resource, action))
    }

    pub fn can_with_scope(
        &self,
        resource: &str,
        action: &Action,
        scope: &Scope,
        user_scope: &Scope,
    ) -> bool {
        if !self.can(resource, action) {
            return false;
        }
        matches!(
            (scope, user_scope),
            (Scope::All, _)
                | (Scope::Team, Scope::Team)
                | (Scope::Team, Scope::All)
                | (Scope::Own, Scope::Own)
                | (Scope::Own, Scope::Team)
                | (Scope::Own, Scope::All)
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = &Permission> {
        self.permissions.iter()
    }

    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }
}

pub struct PermissionChecker<P: PermissionProvider> {
    provider: P,
}

impl<P: PermissionProvider> PermissionChecker<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn check(&self, user_id: &str, resource: &str, action: &Action) -> bool {
        let permissions = self.provider.get_permissions(user_id);
        permissions.can(resource, action)
    }

    pub fn check_with_context(
        &self,
        user_id: &str,
        resource: &str,
        action: &Action,
        context: &AuthContext,
    ) -> bool {
        let permissions = self.provider.get_permissions(user_id);
        if !permissions.can(resource, action) {
            return false;
        }
        self.check_scope(&permissions, resource, context)
    }

    fn check_scope(
        &self,
        permissions: &PermissionSet,
        resource: &str,
        context: &AuthContext,
    ) -> bool {
        if let Some(p) = permissions.iter().find(|p| p.resource == resource) {
            match &p.scope {
                Scope::All => true,
                Scope::Own => context.owner_id.as_ref() == Some(&context.user_id),
                Scope::Team => context.team_id.is_some(),
                Scope::None => false,
            }
        } else {
            false
        }
    }
}

pub trait PermissionProvider: Send + Sync {
    fn get_permissions(&self, user_id: &str) -> PermissionSet;
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub owner_id: Option<String>,
    pub team_id: Option<String>,
    pub roles: Vec<String>,
}

impl AuthContext {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            owner_id: None,
            team_id: None,
            roles: Vec::new(),
        }
    }

    pub fn with_owner(mut self, owner_id: impl Into<String>) -> Self {
        self.owner_id = Some(owner_id.into());
        self
    }

    pub fn with_team(mut self, team_id: impl Into<String>) -> Self {
        self.team_id = Some(team_id.into());
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn is_owner(&self) -> bool {
        self.owner_id.as_ref() == Some(&self.user_id)
    }
}

#[allow(dead_code)]
pub struct InMemoryPermissionProvider {
    permissions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, PermissionSet>>>,
}

#[allow(dead_code)]
impl InMemoryPermissionProvider {
    pub fn new() -> Self {
        Self {
            permissions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_permissions(&self, user_id: &str, permissions: Vec<Permission>) {
        let mut perms = self.permissions.write().await;
        let set = perms
            .entry(user_id.to_string())
            .or_insert_with(PermissionSet::new);
        for p in permissions {
            set.add(p);
        }
    }
}

impl Default for InMemoryPermissionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionProvider for InMemoryPermissionProvider {
    fn get_permissions(&self, user_id: &str) -> PermissionSet {
        let perms = self.permissions.blocking_read();
        perms.get(user_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_matching() {
        let perm = Permission::new("posts", Action::Read);
        assert!(perm.matches("posts", &Action::Read));
        assert!(!perm.matches("posts", &Action::Manage));
        assert!(!perm.matches("posts", &Action::Delete));
        assert!(!perm.matches("users", &Action::Read));
    }

    #[test]
    fn permission_set_checks() {
        let mut set = PermissionSet::new();
        set.add(Permission::new("posts", Action::Read));
        set.add(Permission::new("posts", Action::Write));

        assert!(set.can("posts", &Action::Read));
        assert!(set.can("posts", &Action::Write));
        assert!(!set.can("posts", &Action::Delete));
        assert!(!set.can("users", &Action::Read));
    }

    #[test]
    fn auth_context() {
        let ctx = AuthContext::new("user-123")
            .with_owner("user-123")
            .with_team("team-456")
            .with_roles(vec!["admin".to_string()]);

        assert!(ctx.is_owner());
        assert!(ctx.team_id.is_some());
        assert_eq!(ctx.roles, vec!["admin"]);
    }
}
