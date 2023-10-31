use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Serialization(Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("{0}")]
    Deserialization(Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("{0}")]
    Connection(Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("{0}")]
    Transaction(Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("{0}")]
    Execution(Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("No entity found with ID {0}")]
    NotFound(Uuid),

    #[error("Unknown repository error")]
    Unknown,
}
