use crate::swap_protocols::{
    ledger::Ethereum,
    rfc003::{state_machine::HtlcParams, Ledger},
};
use ethereum_support::{web3::types::Address, Bytes, Erc20Quantity, EtherQuantity};
use std::time::Duration;

mod actions;
mod erc20_htlc;
mod ether_htlc;
mod extract_secret;
mod queries;
mod validation;

pub use self::{actions::*, erc20_htlc::*, ether_htlc::*, queries::*};

#[derive(Deserialize, Serialize, Debug)]
pub struct ByteCode(pub String);

impl Into<Bytes> for ByteCode {
    fn into(self) -> Bytes {
        Bytes(hex::decode(self.0).unwrap())
    }
}

pub trait Htlc {
    fn compile_to_hex(&self) -> ByteCode;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Seconds(pub u64);

impl From<Duration> for Seconds {
    fn from(duration: Duration) -> Self {
        Seconds(duration.as_secs())
    }
}

impl From<Seconds> for Duration {
    fn from(seconds: Seconds) -> Duration {
        Duration::from_secs(seconds.0)
    }
}

impl Ledger for Ethereum {
    type LockDuration = Seconds;
    type HtlcLocation = Address;
    type HtlcIdentity = Address;
}

impl From<HtlcParams<Ethereum, EtherQuantity>> for EtherHtlc {
    fn from(htlc_params: HtlcParams<Ethereum, EtherQuantity>) -> Self {
        EtherHtlc::new(
            htlc_params.lock_duration,
            htlc_params.refund_identity,
            htlc_params.redeem_identity,
            htlc_params.secret_hash,
        )
    }
}

impl HtlcParams<Ethereum, EtherQuantity> {
    pub fn bytecode(&self) -> Bytes {
        EtherHtlc::from(self.clone()).compile_to_hex().into()
    }
}

impl From<HtlcParams<Ethereum, Erc20Quantity>> for Erc20Htlc {
    fn from(htlc_params: HtlcParams<Ethereum, Erc20Quantity>) -> Self {
        Erc20Htlc::new(
            htlc_params.lock_duration,
            htlc_params.refund_identity,
            htlc_params.redeem_identity,
            htlc_params.secret_hash,
            htlc_params.asset.token_contract(),
            htlc_params.asset.quantity(),
        )
    }
}

impl HtlcParams<Ethereum, Erc20Quantity> {
    pub fn bytecode(&self) -> Bytes {
        Erc20Htlc::from(self.clone()).compile_to_hex().into()
    }
    pub fn funding_tx_payload(&self, htlc_location: Address) -> Bytes {
        Erc20Htlc::from(self.clone()).funding_tx_payload(htlc_location)
    }
}
