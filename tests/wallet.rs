use alloy::signers::local::PrivateKeySigner;
use evm_bot::wallet::{Wallet, read_wallets_from_csv, write_wallets_to_csv};
use tempfile::tempdir;

const PRIVATE_KEY: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";

fn signer() -> PrivateKeySigner {
    PRIVATE_KEY.parse().expect("test private key is valid")
}

fn wallet(name: &str, is_reserve: bool) -> Wallet {
    Wallet::from_keypair(name.to_owned(), signer(), is_reserve)
}

#[test]
fn new_uses_default_name_and_reserve_status() {
    let wallet = Wallet::new(None::<String>, None::<bool>);

    assert_eq!(wallet.name, "wallet");
    assert!(!wallet.is_reserve);
}

#[test]
fn csv_row_contains_stable_wallet_fields() {
    let wallet = wallet("primary", true);

    assert_eq!(
        wallet.csv_row(),
        [
            "primary".to_owned(),
            PRIVATE_KEY.to_owned(),
            "true".to_owned(),
            wallet.keypair.address().to_string(),
        ]
    );
}

#[test]
fn csv_round_trip_orders_reserve_wallets_first() {
    let directory = tempdir().expect("temporary directory is created");
    let path = directory.path().join("nested/wallets.csv");
    let wallets = [
        wallet("primary", false),
        wallet("reserve-one", true),
        wallet("secondary", false),
        wallet("reserve-two", true),
    ];

    write_wallets_to_csv(&path, &wallets).expect("wallets are written");
    let loaded = read_wallets_from_csv(&path).expect("wallets are read");

    assert_eq!(
        loaded
            .iter()
            .map(|wallet| wallet.name.as_str())
            .collect::<Vec<_>>(),
        ["reserve-one", "reserve-two", "primary", "secondary"]
    );
    assert_eq!(
        loaded
            .iter()
            .map(|wallet| wallet.is_reserve)
            .collect::<Vec<_>>(),
        [true, true, false, false]
    );
    assert!(
        loaded
            .iter()
            .all(|wallet| wallet.keypair.to_bytes() == signer().to_bytes())
    );
}

#[cfg(unix)]
#[test]
fn wallet_csv_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempdir().expect("temporary directory is created");
    let path = directory.path().join("wallets.csv");

    write_wallets_to_csv(&path, &[wallet("primary", false)]).expect("wallet is written");

    let mode = std::fs::metadata(path)
        .expect("wallet file metadata is available")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn missing_csv_returns_no_wallets() {
    let directory = tempdir().expect("temporary directory is created");

    let wallets = read_wallets_from_csv(directory.path().join("missing.csv"))
        .expect("a missing wallet file is allowed");

    assert!(wallets.is_empty());
}

#[test]
fn unexpected_csv_header_is_rejected() {
    let directory = tempdir().expect("temporary directory is created");
    let path = directory.path().join("wallets.csv");
    std::fs::write(&path, "name,private_key\n").expect("fixture is written");

    let error = read_wallets_from_csv(&path).expect_err("invalid header must fail");

    assert!(error.to_string().contains("unexpected CSV header"));
}

#[test]
fn invalid_wallet_fields_are_rejected() {
    let directory = tempdir().expect("temporary directory is created");
    let path = directory.path().join("wallets.csv");
    let header = "name,private_key,is_reserve,public_key,created_at\n";

    std::fs::write(
        &path,
        format!("{header}primary,invalid,false,unused,2026-09-16\n"),
    )
    .expect("fixture is written");
    let key_error = read_wallets_from_csv(&path).expect_err("invalid key must fail");
    assert!(
        key_error
            .to_string()
            .contains("invalid private key at line 2")
    );

    std::fs::write(
        &path,
        format!("{header}primary,{PRIVATE_KEY},sometimes,unused,2026-09-16\n"),
    )
    .expect("fixture is written");
    let reserve_error = read_wallets_from_csv(&path).expect_err("invalid reserve flag must fail");
    assert!(
        reserve_error
            .to_string()
            .contains("invalid reserve flag at line 2")
    );
}
