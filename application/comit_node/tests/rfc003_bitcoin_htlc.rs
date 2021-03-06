use bitcoin_rpc_client::*;
use bitcoin_rpc_test_helpers::RegtestHelperClient;
use bitcoin_support::{
    serialize::serialize_hex, Address, BitcoinQuantity, Network, OutPoint, PrivateKey, PubkeyHash,
};
use bitcoin_witness::{PrimedInput, PrimedTransaction};
use comit_node::swap_protocols::rfc003::{bitcoin::Htlc, Secret};
use secp256k1_support::KeyPair;
use std::str::FromStr;
use testcontainers::{clients::Cli, images::coblox_bitcoincore::BitcoinCore, Docker};

fn fund_htlc(
    client: &BitcoinCoreClient,
) -> (
    TransactionId,
    rpc::TransactionOutput,
    BitcoinQuantity,
    Htlc,
    u32,
    Secret,
    KeyPair,
    KeyPair,
) {
    let redeem_privkey =
        PrivateKey::from_str("cSrWvMrWE3biZinxPZc1hSwMMEdYgYsFpB6iEoh8KraLqYZUUCtt").unwrap();
    let redeem_keypair: KeyPair = redeem_privkey.secret_key().clone().into();
    let redeem_pubkey_hash: PubkeyHash = redeem_keypair.public_key().clone().into();
    let refund_privkey =
        PrivateKey::from_str("cNZUJxVXghSri4dUaNW8ES3KiFyDoWVffLYDz7KMcHmKhLdFyZPx").unwrap();
    let refund_keypair: KeyPair = refund_privkey.secret_key().clone().into();
    let secret = Secret::from(*b"hello world, you are beautiful!!");
    let refund_pubkey_hash: PubkeyHash = refund_keypair.public_key().clone().into();
    let sequence_lock = 10;

    let amount = BitcoinQuantity::from_satoshi(100_000_001);

    let htlc = Htlc::new(
        redeem_pubkey_hash,
        refund_pubkey_hash,
        secret.hash(),
        sequence_lock,
    );

    let htlc_address = htlc.compute_address(Network::Regtest);

    let txid = client
        .send_to_address(&htlc_address.clone().into(), amount.bitcoin())
        .unwrap()
        .unwrap();

    client.generate(1).unwrap().unwrap();

    let vout = client.find_vout_for_address(&txid, &htlc_address);

    (
        txid,
        vout.clone(),
        amount,
        htlc,
        sequence_lock,
        secret,
        redeem_keypair,
        refund_keypair,
    )
}

#[test]
fn redeem_htlc_with_secret() {
    let _ = pretty_env_logger::try_init();
    let docker = Cli::default();

    let container = docker.run(BitcoinCore::default());
    let client = tc_bitcoincore_client::new(&container);
    client.generate(432).unwrap().unwrap();

    let (txid, vout, input_amount, htlc, _, secret, keypair, _) = fund_htlc(&client);

    assert!(
        htlc.can_be_unlocked_with(secret, keypair).is_ok(),
        "Should be unlockable with the given secret and secret_key"
    );

    let alice_addr: Address = client.get_new_address().unwrap().unwrap().into();

    let fee = BitcoinQuantity::from_satoshi(1000);

    let redeem_tx = PrimedTransaction {
        inputs: vec![PrimedInput::new(
            OutPoint { txid, vout: vout.n },
            input_amount,
            htlc.unlock_with_secret(keypair, &secret),
        )],
        output_address: alice_addr.clone(),
        locktime: 0,
    }
    .sign_with_fee(fee);

    let redeem_tx_hex = serialize_hex(&redeem_tx).unwrap();

    let raw_redeem_tx = rpc::SerializedRawTransaction(redeem_tx_hex);

    let rpc_redeem_txid = client.send_raw_transaction(raw_redeem_tx).unwrap().unwrap();

    client.generate(1).unwrap().unwrap();

    assert!(
        client
            .find_utxo_at_tx_for_address(&rpc_redeem_txid, &alice_addr)
            .is_some(),
        "utxo should exist after redeeming htlc"
    );
}

#[test]
fn redeem_refund_htlc() {
    let _ = pretty_env_logger::try_init();
    let docker = Cli::default();

    let container = docker.run(BitcoinCore::default());
    let client = tc_bitcoincore_client::new(&container);
    client.generate(432).unwrap().unwrap();

    let (txid, vout, input_amount, htlc, nsequence, _, _, keypair) = fund_htlc(&client);

    let alice_addr: Address = client.get_new_address().unwrap().unwrap().into();
    let fee = BitcoinQuantity::from_satoshi(1000);

    let redeem_tx = PrimedTransaction {
        inputs: vec![PrimedInput::new(
            OutPoint { txid, vout: vout.n },
            input_amount,
            htlc.unlock_after_timeout(keypair),
        )],
        output_address: alice_addr.clone(),
        locktime: 0,
    }
    .sign_with_fee(fee);

    let redeem_tx_hex = serialize_hex(&redeem_tx).unwrap();

    let raw_redeem_tx = rpc::SerializedRawTransaction(redeem_tx_hex);

    let rpc_redeem_txid_error = client.send_raw_transaction(raw_redeem_tx.clone()).unwrap();

    // It should fail because it's too early
    assert!(rpc_redeem_txid_error.is_err());
    let error = rpc_redeem_txid_error.unwrap_err();

    assert_eq!(error.code, -26);
    /// RPC_VERIFY_REJECTED = -26, !< Transaction or block was rejected by
    /// network rules
    assert!(error.message.contains("non-BIP68-final"));

    client.generate(nsequence).unwrap().unwrap();

    let _txn = client.get_raw_transaction_verbose(&txid).unwrap().unwrap();

    let rpc_redeem_txid = client.send_raw_transaction(raw_redeem_tx).unwrap().unwrap();

    client.generate(1).unwrap().unwrap();

    assert!(
        client
            .find_utxo_at_tx_for_address(&rpc_redeem_txid, &alice_addr)
            .is_some(),
        "utxo should exist after refunding htlc"
    );
}
