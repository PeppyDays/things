# Things

This is an example repository to implement to do application with Rust. It aims for excercising Axum framework and tries to follow domain-driven and test-driven development. I'm going to log all decisions made to implement this application.

## How to Run

```bash
# run account api
cd account
cargo run -p api

# should return 200 as a health check
# installing httpie is required
http get :8080/
```

## Architecture Decision

- [Account Boundary Context](https://github.com/PeppyDays/things/wiki/Account-Boundary-Context)
- [Command and Event Structure in Aggregate](https://github.com/PeppyDays/things/wiki/Command-and-Event-Structure-in-Aggregate)
