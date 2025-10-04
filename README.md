# goosekv

A simple thread per core key-value store implementation.

---

## Features

- **RESP2 Protocol** - makes it compatible with subset of REDIS commands.
- **Supported Commands**
  - `PING`
  - `GET`
  - simplified `SET` (without expiration time)
  - `DEL`
  - `EXISTS`
  - `INCR`
  - and more to come...

---

## Getting Started

### Prerequisites

- rust toolchain
- `redis-cli` or `valkey-cli` (for testing)

### Installation & Running

1.  **Clone the repository**

    ```bash
    git clone https://github.com/sobczal2/goosekv.git
    cd goosekv
    ```

2.  **Build and run the server**
    ```bash
    cargo run --release
    ```
    The server will start on `127.0.0.1:6379`.

---

## Usage

When the server is running, you can connect to it using `redis-cli` or `valkey-cli` and query it using supported commands.

---

## Architecture

The server is build in thread per core architecture using `glommio` async runtime. It creates one shard per cpu core.
Each shard consists of 3 components:

- **acceptor** - a simple actor responsible for accepting incoming tcp connections and delegating the work to next component.
- **processor** - an actor responsible for orchestrating work required to handle specified command. It parses the command, handles it and accesses storage as needed.
- **storage** - an actor responsible for storage of data in an ephemeral hashmap.

### Diagram showcasing tpc design

![architecture diagram](https://github.com/sobczal2/goosekv/blob/main/assets/arch_diagram.png?raw=true)
