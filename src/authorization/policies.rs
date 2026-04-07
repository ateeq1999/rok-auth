//! Policy-based authorization evaluation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub effect: PolicyEffect,
    pub resources: Vec<String>,
    pub actions: Vec<String>,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    In,
    NotIn,
    RegexMatch,
}

impl Condition {
    pub fn evaluate(&self, context: &PolicyContext) -> bool {
        let Some(value) = context.get(&self.field) else {
            return false;
        };
        match self.operator {
            ConditionOperator::Equals => value == &self.value,
            ConditionOperator::NotEquals => value != &self.value,
            ConditionOperator::Contains => {
                if let (Some(a), Some(b)) = (value.as_str(), self.value.as_str()) {
                    a.contains(b)
                } else {
                    false
                }
            }
            ConditionOperator::StartsWith => {
                if let (Some(a), Some(b)) = (value.as_str(), self.value.as_str()) {
                    a.starts_with(b)
                } else {
                    false
                }
            }
            ConditionOperator::EndsWith => {
                if let (Some(a), Some(b)) = (value.as_str(), self.value.as_str()) {
                    a.ends_with(b)
                } else {
                    false
                }
            }
            ConditionOperator::In => {
                if let Some(arr) = self.value.as_array() {
                    arr.contains(value)
                } else {
                    false
                }
            }
            ConditionOperator::NotIn => {
                if let Some(arr) = self.value.as_array() {
                    !arr.contains(value)
                } else {
                    true
                }
            }
            ConditionOperator::GreaterThan => {
                if let (Some(a), Some(b)) = (value.as_i64(), self.value.as_i64()) {
                    a > b
                } else {
                    false
                }
            }
            ConditionOperator::LessThan => {
                if let (Some(a), Some(b)) = (value.as_i64(), self.value.as_i64()) {
                    a < b
                } else {
                    false
                }
            }
            ConditionOperator::RegexMatch => {
                if let (Some(a), Some(b)) = (value.as_str(), self.value.as_str()) {
                    regex::Regex::new(b).map(|r| r.is_match(a)).unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolicyContext {
    values: HashMap<String, serde_json::Value>,
}

impl PolicyContext {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn with_value(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.values.get(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.values.insert(key.into(), value.into());
    }
}

impl Default for PolicyContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AuthorizationResult {
    pub allowed: bool,
    pub reason: String,
    pub matched_policy: Option<String>,
}

impl AuthorizationResult {
    pub fn allow(policy: &str) -> Self {
        Self {
            allowed: true,
            reason: format!("Allowed by policy '{}'", policy),
            matched_policy: Some(policy.to_string()),
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
            matched_policy: None,
        }
    }

    pub fn deny_no_matching_policy() -> Self {
        Self {
            allowed: false,
            reason: "No matching policy found".to_string(),
            matched_policy: None,
        }
    }
}

pub struct PolicyEvaluator {
    policies: Vec<Policy>,
}

impl PolicyEvaluator {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    pub fn evaluate(
        &self,
        resource: &str,
        action: &str,
        context: &PolicyContext,
    ) -> AuthorizationResult {
        let matching_policies: Vec<&Policy> = self
            .policies
            .iter()
            .filter(|p| {
                p.resources.iter().any(|r| glob_match(r, resource))
                    && p.actions.iter().any(|a| glob_match(a, action))
            })
            .collect();

        if matching_policies.is_empty() {
            return AuthorizationResult::deny_no_matching_policy();
        }

        for policy in &matching_policies {
            let conditions_match = policy.conditions.is_empty()
                || policy.conditions.iter().all(|c| c.evaluate(context));

            if conditions_match {
                match policy.effect {
                    PolicyEffect::Allow => return AuthorizationResult::allow(&policy.name),
                    PolicyEffect::Deny => {
                        return AuthorizationResult::deny(format!(
                            "Denied by policy '{}': {}",
                            policy.name,
                            policy
                                .conditions
                                .iter()
                                .map(|c| format!("{} {:?} {}", c.field, c.operator, c.value))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                    }
                }
            }
        }

        AuthorizationResult::deny("No matching policy conditions satisfied")
    }

    pub fn can(&self, resource: &str, action: &str, context: &PolicyContext) -> bool {
        self.evaluate(resource, action, context).allowed
    }
}

impl Default for PolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

fn glob_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        return value.starts_with(prefix);
    }
    if pattern.starts_with("*") {
        let suffix = &pattern[1..];
        return value.ends_with(suffix);
    }
    pattern == value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_evaluation() {
        let ctx = PolicyContext::new()
            .with_value("user.role", "admin")
            .with_value("user.age", 25);

        let cond = Condition {
            field: "user.role".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::json!("admin"),
        };
        assert!(cond.evaluate(&ctx));

        let cond2 = Condition {
            field: "user.role".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::json!("user"),
        };
        assert!(!cond2.evaluate(&ctx));
    }

    #[test]
    fn condition_in_operator() {
        let ctx = PolicyContext::new().with_value("user.role", "admin");

        let cond = Condition {
            field: "user.role".to_string(),
            operator: ConditionOperator::In,
            value: serde_json::json!(["admin", "moderator"]),
        };
        assert!(cond.evaluate(&ctx));
    }

    #[test]
    fn policy_evaluator_allow() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_policy(Policy {
            id: "1".to_string(),
            name: "AdminAllAccess".to_string(),
            effect: PolicyEffect::Allow,
            resources: vec!["*".to_string()],
            actions: vec!["*".to_string()],
            conditions: vec![Condition {
                field: "user.role".to_string(),
                operator: ConditionOperator::Equals,
                value: serde_json::json!("admin"),
            }],
        });

        let ctx = PolicyContext::new().with_value("user.role", "admin");
        let result = evaluator.evaluate("any-resource", "any-action", &ctx);
        assert!(result.allowed);
    }

    #[test]
    fn policy_evaluator_deny() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_policy(Policy {
            id: "1".to_string(),
            name: "DenyDelete".to_string(),
            effect: PolicyEffect::Deny,
            resources: vec!["sensitive".to_string()],
            actions: vec!["delete".to_string()],
            conditions: vec![],
        });

        let ctx = PolicyContext::new();
        let result = evaluator.evaluate("sensitive", "delete", &ctx);
        assert!(!result.allowed);
    }

    #[test]
    fn glob_matching() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("admin*", "admin-users"));
        assert!(glob_match("*-api", "v1-api"));
        assert!(glob_match("exact", "exact"));
        assert!(!glob_match("admin*", "user-admin"));
    }
}
