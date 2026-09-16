# EVM Bot

[![crates.io](https://img.shields.io/crates/v/evm-bot.svg)](https://crates.io/crates/evm-bot)

> ⚠️ PROJECT IS STILL IN EARLY DEVELOPMENT

## Quick Start

1. Open the terminal and clone the repo

    ```shell
    git clone https://github.com/Deezzir/evm-bot.git
    ```

2. Install Cargo/Rust and build.

    See the [Rust installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

    ```shell
    cd evm-bot && cargo build --release
    ```

3. Create a `.env` file in the root directory and add the following

    ```shell
    ETH_RPC_URL=
    RH_RPC_URL=
    BNB_RPC_URL=
    BASE_RPC_URL=
    INK_RPC_URL=
    HYPER_RPC_URL=
    ARC_RPC_URL=
    POLYGON_RPC_URL=
    ```

4. Run the project

    ```shell
    > alias bot="./target/release/evm-bot"
    > bot -h
    EVM Bot

    EVM Bot CLI

    Usage: evm-bot [OPTIONS] <COMMAND>
    
    Commands:
      balance         Get the balance of the wallets
      generate        Generate wallets and save them to a CSV file
      token-balance   Get the token balance of the wallets
      transfer        Transfer from the specified wallet to the receiver
      token-transfer  Transfer token from the specified wallet to the receiver
      buy             Buy a token through a DEX or launchpad
      create-token    Create a token on a launchpad
      snipe           Watch a launchpad and buy matching token launches
      help            Print this message or the help of the given subcommand(s)
    
    Options:
    -c, --chain <CHAIN>  Blockchain to use [default: ethereum] [possible values: ethereum, robinhood, bnb, base, ink, hyper, arc, polygon]
      -k, --keys <KEYS>    Path to the CSV file with the wallets [default: keys.csv]
          --no-colors      Disable colored output
      -h, --help           Print help
      -V, --version        Print version
    ```

> ⚠️ Help is available for each command. Use `bot <command> -h` to see the options for that command.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development and contribution guidelines.
