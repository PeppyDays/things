# Things

This is an example repository to implement to do application with Rust. It aims for excercising Axum framework and tries to follow domain-driven and test-driven development. I'm going to log all decisions made to implement this application.

## How to Run

```bash
# run account api
cd account
cargo run -p api

# should return 200 as a health check
# installing httpie is required, or use curl
http get :8080/account/user/check-health
```

## Architecture Decision

- [Account Boundary Context](https://github.com/PeppyDays/things/wiki/Account-Boundary-Context)
- [Command and Event Structure in Aggregate](https://github.com/PeppyDays/things/wiki/Command-and-Event-Structure-in-Aggregate)

## To Do Items

I've finished implementing basic functionalities designed for account bounded context. For further learning, these are actions items to do more.

[] Parse application configuration from environment variables (consider using `dotenvy`)
[] Reorganise error definition and split error messages for the clients and developers (consider using `anyhow`)
[] Add logging and tracing
[] Add database migration (consider using `sqlx::migrate!()`)
[] Add integration tests, running the application locally and test from APIs to the database
[] Implement caching in GitHub workflow to speed up `cargo build --release`
[] Update ADR docs
