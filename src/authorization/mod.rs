//! Advanced authorization and access control.

mod roles;
mod permissions;
mod policies;
mod audit;

pub use roles::{Role, RoleHierarchy, RoleManager};
pub use permissions::{Permission, PermissionSet, PermissionChecker};
pub use policies::{Policy, PolicyEvaluator, AuthorizationResult};
pub use audit::{AuditLogger, AuditEvent, AuditLevel};
