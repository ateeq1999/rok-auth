# Phase 6: Advanced RBAC & Authorization

Role-based access control with role hierarchies, permissions, policy evaluation, and audit logging. Implemented in [src/authorization/](../src/authorization/).

---

## Roles

Defined in [src/authorization/roles.rs](../src/authorization/roles.rs).

### Role

```rust
pub struct Role {
    pub name: String,
    pub level: u32,                       // higher = more privileged
    pub parents: Vec<String>,             // inherited roles
    pub metadata: HashMap<String, String>,
}
```

```rust
use rok_auth::authorization::roles::Role;

let role = Role::new("editor", 20)
    .with_parents(vec!["viewer".to_string()])
    .with_metadata("department", "content");
```

### RoleHierarchy

Manages a graph of roles and resolves inheritance:

```rust
use rok_auth::authorization::roles::RoleHierarchy;

let mut hierarchy = RoleHierarchy::new();
hierarchy.add_role(Role::new("superadmin", 100));
hierarchy.add_role(Role::new("admin", 50).with_parents(vec!["superadmin".to_string()]));
hierarchy.add_role(Role::new("user", 10));

// Inheritance check: admin inherits superadmin
assert!(hierarchy.inherits("admin", "superadmin")); // true
assert!(hierarchy.inherits("user", "admin"));       // false

// User has "admin" role — does it satisfy "superadmin" requirement?
let user_roles = vec!["admin".to_string()];
assert!(hierarchy.has_role(&user_roles, "superadmin")); // true (via inheritance)

// Get all roles a user effectively has (direct + inherited)
let effective = hierarchy.get_effective_roles(&["admin".to_string()]);
// {"admin", "superadmin"}
```

### RoleManager

Convenience wrapper around `RoleHierarchy`:

```rust
use rok_auth::authorization::roles::RoleManager;

// With built-in role hierarchy:
// superadmin (100) <- admin (50) <- moderator (30) <- user (10)
// guest (0) — no inheritance
let manager = RoleManager::new().with_default_roles();

assert!(manager.check_role(&["admin".to_string()], "superadmin")); // true
assert!(manager.check_role(&["user".to_string()], "moderator"));   // true
assert!(!manager.check_role(&["guest".to_string()], "user"));      // false

// Check multiple
manager.check_any_role(&user_roles, &required);
manager.check_all_roles(&user_roles, &required);
```

---

## Permissions

Defined in [src/authorization/permissions.rs](../src/authorization/permissions.rs).

```rust
pub enum Permission {
    Read,
    Write,
    Delete,
    Manage,
    Execute,
    Custom(String),
}

pub enum PermissionScope {
    Own,   // only resources the user owns
    Team,  // resources belonging to the user's team
    All,   // all resources
}
```

```rust
use rok_auth::authorization::permissions::{Permission, PermissionScope, PermissionSet};

let perms = PermissionSet::new()
    .allow(Permission::Read, PermissionScope::All)
    .allow(Permission::Write, PermissionScope::Own);

assert!(perms.can(Permission::Read, PermissionScope::All));
assert!(perms.can(Permission::Write, PermissionScope::Own));
assert!(!perms.can(Permission::Delete, PermissionScope::All));
```

---

## Policies

Defined in [src/authorization/policies.rs](../src/authorization/policies.rs).

A `Policy` is a named rule that evaluates a `Claims` object and returns allow/deny:

```rust
use rok_auth::authorization::policies::{Policy, PolicyEvaluator};

let evaluator = PolicyEvaluator::new()
    .add_policy(Policy::require_role("admin"))
    .add_policy(Policy::require_any_role(vec!["editor", "publisher"]));

let allowed = evaluator.evaluate(&claims)?;
```

---

## Audit Logging

Defined in [src/authorization/audit.rs](../src/authorization/audit.rs).

```rust
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub details: serde_json::Value,
}
```

The `AuditLogger` trait lets you plug in any backend:

```rust
use rok_auth::authorization::audit::AuditLogger;

struct DatabaseAuditLogger { pool: PgPool }

impl AuditLogger for DatabaseAuditLogger {
    async fn log(&self, event: AuditEvent) {
        // insert into audit_log table
    }

    async fn get_events(&self, filter: &AuditFilter) -> Vec<AuditEvent> {
        // query with filter
    }
}
```

### AuditFilter

```rust
let filter = AuditFilter {
    user_id: Some("user-123".to_string()),
    event_type: Some(AuditEventType::Login),
    from: Some(Utc::now() - Duration::days(7)),
    to: None,
    success: Some(false), // only failed events
};
```

---

## Axum Integration with Macros

Use the procedural macros from `rok-auth-macros` to protect handlers:

```rust
use rok_auth_macros::{require_role, require_any_role};

#[require_role("admin")]
async fn admin_dashboard(claims: Claims) -> impl IntoResponse {
    "welcome, admin"
}

#[require_any_role("editor", "publisher")]
async fn publish_post(claims: Claims) -> impl IntoResponse {
    "post published"
}
```
