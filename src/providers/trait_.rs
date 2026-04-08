//! Trait for bridging rok-auth to your user model.
//!
//! Implement [`UserProvider`] on your user struct to expose the fields
//! rok-auth needs for password verification and token issuance.
//! Querying users from your database is left to your application code.
//!
//! # Example
//!
//! ```rust,no_run
//! use rok_auth::UserProvider;
//!
//! struct User {
//!     id: String,
//!     password_hash: String,
//!     roles: Vec<String>,
//! }
//!
//! impl UserProvider for User {
//!     fn user_id(&self) -> String { self.id.clone() }
//!     fn password_hash(&self) -> &str { &self.password_hash }
//!     fn roles(&self) -> Vec<String> { self.roles.clone() }
//! }
//! ```

/// Describes the shape of a user object that rok-auth can work with.
///
/// Implement this on your application's user struct. rok-auth does not
/// dictate how users are persisted — bring your own database layer.
pub trait UserProvider: Send + Sync {
    /// Returns the user's unique identifier (used as the JWT `sub` claim).
    fn user_id(&self) -> String;

    /// Returns the Argon2id password hash stored for this user.
    fn password_hash(&self) -> &str;

    /// Returns the list of role names assigned to this user.
    fn roles(&self) -> Vec<String>;
}