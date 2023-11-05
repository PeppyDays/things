use sqlx::mysql::MySqlPoolOptions;

use domain::identity::{
    commands::CommandExecutor as IdentityCommandExecutor,
    queries::QueryReader as IdentityQueryReader,
};
use domain::user::{
    commands::CommandExecutor as UserCommandExecutor, queries::QueryReader as UserQueryReader,
};
use event_sourcing::repository::mysql::MySqlRepository as UserMySqlRepository;
use infrastructure::repositories::identity::MySqlRepository as IdentityMySqlRepository;

#[derive(Clone)]
pub struct Container {
    pub user_command_executor: UserCommandExecutor<UserMySqlRepository>,
    pub user_query_reader: UserQueryReader<UserMySqlRepository>,
    pub identity_command_executor: IdentityCommandExecutor<IdentityMySqlRepository>,
    pub identity_query_reader: IdentityQueryReader,
}

impl Container {
    pub async fn new() -> Self {
        let user_repository = UserMySqlRepository::new(
            MySqlPoolOptions::new()
                .max_connections(5)
                .connect("mysql://root:welcome@localhost:3306/account")
                .await
                .unwrap(),
        );
        let user_command_executor = UserCommandExecutor::new(user_repository.clone());
        let user_query_reader = UserQueryReader::new(user_repository.clone());

        let identity_repository = IdentityMySqlRepository::new(
            MySqlPoolOptions::new()
                .max_connections(5)
                .connect("mysql://root:welcome@localhost:3306/account")
                .await
                .unwrap(),
        );
        let identity_command_executor = IdentityCommandExecutor::new(identity_repository.clone());
        let identity_query_reader = IdentityQueryReader::new();

        Self {
            user_command_executor,
            user_query_reader,
            identity_command_executor,
            identity_query_reader,
        }
    }
}

pub async fn get_container() -> Container {
    Container::new().await
}
