use std::env;

use sqlx::mysql::MySqlPoolOptions;

use domain::identity::services::Service as IdentityService;
use domain::user::{
    commands::CommandExecutor as UserCommandExecutor, queries::QueryReader as UserQueryReader,
};
use event_sourcing::repository::mysql::MySqlRepository as UserMySqlRepository;
use infrastructure::repositories::identity::MySqlRepository as IdentityMySqlRepository;

#[derive(Clone)]
pub struct Container {
    pub user_command_executor: UserCommandExecutor<UserMySqlRepository>,
    pub user_query_reader: UserQueryReader<UserMySqlRepository>,
    pub identity_service: IdentityService<IdentityMySqlRepository>,
}

impl Container {
    pub async fn new() -> Self {
        let database_url = get_database_url();

        let user_repository = UserMySqlRepository::new(
            MySqlPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .unwrap(),
        );
        let user_command_executor = UserCommandExecutor::new(user_repository.clone());
        let user_query_reader = UserQueryReader::new(user_repository.clone());

        let identity_repository = IdentityMySqlRepository::new(
            MySqlPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .unwrap(),
        );
        let identity_service = IdentityService::new(identity_repository.clone());

        Self {
            user_command_executor,
            user_query_reader,
            identity_service,
        }
    }
}

pub async fn get_container() -> Container {
    Container::new().await
}

fn get_database_url() -> String {
    let host = env::var("ACCOUNT_DATABASE_HOST").unwrap_or(String::from("127.0.0.1"));
    let port = env::var("ACCOUNT_DATABASE_PORT")
        .unwrap_or(String::from("3306"))
        .parse::<u16>()
        .unwrap();
    let username = env::var("ACCOUNT_DATABASE_USERNAME").unwrap_or(String::from("root"));
    let password = env::var("ACCOUNT_DATABASE_PASSWORD").unwrap_or(String::from("welcome"));
    let schema = env::var("ACCOUNT_DATABASE_SCHEMA").unwrap_or(String::from("account"));

    format!(
        "mysql://{}:{}@{}:{}/{}",
        username, password, host, port, schema
    )
}
