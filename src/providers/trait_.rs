//! Trait for bridging rok-auth to your user model.

use std::future::Future;

pub trait UserProvider: Sized + Send + Unpin {
    type Id: ToString + Send;

    fn user_id(&self) -> Self::Id;
    fn password_hash(&self) -> &str;
    fn roles(&self) -> Vec<String>;

    fn find_by_email(
        pool: &sqlx::PgPool,
        email: &str,
    ) -> impl Future<Output = Result<Option<Self>, crate::AuthError>> + Send;
}
