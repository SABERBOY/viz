use crate::{async_trait, Request};

/// A handler with extractors.
#[async_trait]
pub trait FnExt<E>: Clone + Send + Sync + 'static {
    /// The returned type after the call operator is used.
    type Output;

    /// Performs the call operation.
    #[must_use]
    async fn call(&self, req: Request) -> Self::Output;
}
