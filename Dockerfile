FROM lukemathwalker/cargo-chef:latest-rust-alpine as chef
WORKDIR /app

FROM chef as planner
COPY . .

# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json

# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

# Build our project
RUN cargo build --release --bin bios-checker

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/release/bios-checker bios-checker
ENTRYPOINT ["./bios-checker"]
