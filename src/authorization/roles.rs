//! Role hierarchy and management.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub level: u32,
    pub parents: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Role {
    pub fn new(name: impl Into<String>, level: u32) -> Self {
        Self {
            name: name.into(),
            level,
            parents: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_parents(mut self, parents: Vec<String>) -> Self {
        self.parents = parents;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn is_ancestor_of(&self, other: &Role) -> bool {
        other.level > self.level
    }
}

#[derive(Debug, Clone, Default)]
pub struct RoleHierarchy {
    roles: HashMap<String, Role>,
}

impl RoleHierarchy {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }

    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    pub fn get_all_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    pub fn has_role(&self, user_roles: &[String], required: &str) -> bool {
        if user_roles.iter().any(|r| r == required) {
            return true;
        }
        for role in user_roles {
            if self.inherits(role, required) {
                return true;
            }
        }
        false
    }

    pub fn has_any_role(&self, user_roles: &[String], required: &[String]) -> bool {
        required.iter().any(|r| self.has_role(user_roles, r))
    }

    pub fn has_all_roles(&self, user_roles: &[String], required: &[String]) -> bool {
        required.iter().all(|r| self.has_role(user_roles, r))
    }

    pub fn inherits(&self, role: &str, target: &str) -> bool {
        if role == target {
            return true;
        }
        self.get_ancestors(role).contains(target)
    }

    pub fn get_ancestors(&self, role: &str) -> HashSet<String> {
        let mut ancestors = HashSet::new();
        let mut to_visit = vec![role.to_string()];

        while let Some(current) = to_visit.pop() {
            if let Some(r) = self.roles.get(&current) {
                for parent in &r.parents {
                    if ancestors.insert(parent.clone()) {
                        to_visit.push(parent.clone());
                    }
                }
            }
        }
        ancestors
    }

    pub fn get_descendants(&self, role: &str) -> HashSet<String> {
        let mut descendants = HashSet::new();
        let mut to_visit = vec![role.to_string()];

        while let Some(current) = to_visit.pop() {
            for (name, r) in &self.roles {
                if r.parents.contains(&current) && descendants.insert(name.clone()) {
                    to_visit.push(name.clone());
                }
            }
        }
        descendants
    }

    pub fn get_effective_roles(&self, user_roles: &[String]) -> HashSet<String> {
        let mut effective = HashSet::new();
        for role in user_roles {
            effective.insert(role.clone());
            effective.extend(self.get_ancestors(role));
        }
        effective
    }
}

pub struct RoleManager {
    hierarchy: RoleHierarchy,
}

impl RoleManager {
    pub fn new() -> Self {
        Self {
            hierarchy: RoleHierarchy::new(),
        }
    }

    pub fn with_default_roles(mut self) -> Self {
        self.hierarchy.add_role(Role::new("superadmin", 100));
        self.hierarchy
            .add_role(Role::new("admin", 50).with_parents(vec!["superadmin".to_string()]));
        self.hierarchy
            .add_role(Role::new("moderator", 30).with_parents(vec!["admin".to_string()]));
        self.hierarchy
            .add_role(Role::new("user", 10).with_parents(vec!["moderator".to_string()]));
        self.hierarchy.add_role(Role::new("guest", 0));
        self
    }

    pub fn add_role(&mut self, role: Role) {
        self.hierarchy.add_role(role)
    }

    pub fn check_role(&self, user_roles: &[String], required: &str) -> bool {
        self.hierarchy.has_role(user_roles, required)
    }

    pub fn check_any_role(&self, user_roles: &[String], required: &[String]) -> bool {
        self.hierarchy.has_any_role(user_roles, required)
    }

    pub fn check_all_roles(&self, user_roles: &[String], required: &[String]) -> bool {
        self.hierarchy.has_all_roles(user_roles, required)
    }

    pub fn get_effective_roles(&self, user_roles: &[String]) -> HashSet<String> {
        self.hierarchy.get_effective_roles(user_roles)
    }

    pub fn hierarchy(&self) -> &RoleHierarchy {
        &self.hierarchy
    }
}

impl Default for RoleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_creation() {
        let role = Role::new("admin", 10).with_metadata("dept", "engineering");
        assert_eq!(role.name, "admin");
        assert_eq!(role.level, 10);
        assert_eq!(role.metadata.get("dept"), Some(&"engineering".to_string()));
    }

    #[test]
    fn role_hierarchy_inheritance() {
        let mut hierarchy = RoleHierarchy::new();
        hierarchy.add_role(Role::new("admin", 50).with_parents(vec!["superadmin".to_string()]));
        hierarchy.add_role(Role::new("superadmin", 100));
        hierarchy.add_role(Role::new("user", 10));

        assert!(hierarchy.inherits("admin", "superadmin"));
        assert!(hierarchy.inherits("superadmin", "superadmin"));
        assert!(!hierarchy.inherits("user", "admin"));
    }

    #[test]
    fn has_role_with_inheritance() {
        let mut hierarchy = RoleHierarchy::new();
        hierarchy.add_role(Role::new("superadmin", 100));
        hierarchy.add_role(Role::new("admin", 50).with_parents(vec!["superadmin".to_string()]));
        hierarchy.add_role(Role::new("user", 10));

        assert!(hierarchy.has_role(&["admin".to_string()], "superadmin"));
        assert!(hierarchy.has_role(&["admin".to_string()], "admin"));
        assert!(hierarchy.has_role(&["superadmin".to_string()], "superadmin"));
        assert!(!hierarchy.has_role(&["user".to_string()], "admin"));
    }

    #[test]
    fn effective_roles() {
        let mut hierarchy = RoleHierarchy::new();
        hierarchy.add_role(Role::new("superadmin", 100));
        hierarchy.add_role(Role::new("admin", 50).with_parents(vec!["superadmin".to_string()]));

        let effective = hierarchy.get_effective_roles(&["admin".to_string()]);
        assert!(effective.contains("admin"));
        assert!(effective.contains("superadmin"));
    }

    #[test]
    fn role_manager_default() {
        let manager = RoleManager::new().with_default_roles();
        assert!(manager.check_role(&["admin".to_string()], "superadmin"));
        assert!(manager.check_role(&["user".to_string()], "moderator"));
        assert!(!manager.check_role(&["guest".to_string()], "user"));
    }
}
