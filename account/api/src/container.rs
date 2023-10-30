use domain::identity::commands::CommandExecutor as IdentityCommandExecutor;
use domain::user::{
    commands::CommandExecutor as UserCommandExecutor, queries::QueryReader as UserQueryReader,
};
use event_sourcing::repository::mysql::MySqlRepository;
use sqlx::mysql::MySqlPoolOptions;

#[derive(Clone)]
pub struct Container {
    pub user_command_executor: UserCommandExecutor<MySqlRepository>,
    pub user_query_reader: UserQueryReader<MySqlRepository>,
    pub identity_command_executor: IdentityCommandExecutor<MySqlRepository>,
}

impl Container {
    pub async fn new() -> Self {
        let command_repository = MySqlRepository::new(
            MySqlPoolOptions::new()
                .max_connections(5)
                .connect("mysql://root:welcome@localhost:3306/account")
                .await
                .unwrap(),
        );
        let user_command_executor = UserCommandExecutor::new(command_repository.clone());
        let identity_command_executor = IdentityCommandExecutor::new(command_repository.clone());

        let query_repository = MySqlRepository::new(
            MySqlPoolOptions::new()
                .max_connections(5)
                .connect("mysql://root:welcome@localhost:3306/account")
                .await
                .unwrap(),
        );
        let user_query_reader = UserQueryReader::new(query_repository.clone());

        Self {
            user_command_executor,
            user_query_reader,
            identity_command_executor,
        }
    }
}

pub async fn get_container() -> Container {
    Container::new().await
}
