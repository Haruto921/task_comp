# syntax=docker/dockerfile:1
FROM rust:1.75.0@sha256:87f3b2f93b82995443a1a558c234212dafe79cfdc3af956539610560369ddcd0

# Install system dependencies required for agent execution and build
RUN apt-get update && apt-get install -y --no-install-recommends \
    tmux \
    asciinema \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency definitions first for layer caching
# (When build context is root, we need to specify environment/ path)
COPY Cargo.toml ./
COPY src/ ./src/

# Build main binary (src/ includes lib.rs)
RUN cargo build --release

# Create dummy src to fetch dependencies
RUN mkdir -p src/bin && echo "fn main() {}" > src/bin/dummy.rs
RUN cargo fetch
RUN rm -rf src

# Copy source code from environment directory
COPY src/ ./src/

# Copy tests from repository root (build context must be set to repository root)
COPY tests/ ./tests/

# Build tests if tests directory exists
RUN if [ -d "tests" ]; then cargo build --tests --release; fi

CMD ["/bin/bash"]