//! Advanced authorization and access control.

mod audit;
mod permissions;
mod policies;
mod roles;

pub use audit::{AuditEvent, AuditLevel, AuditLogger};
pub use permissions::{Permission, PermissionChecker, PermissionSet};
pub use policies::{AuthorizationResult, Policy, PolicyEvaluator};
pub use roles::{Role, RoleHierarchy, RoleManager};
