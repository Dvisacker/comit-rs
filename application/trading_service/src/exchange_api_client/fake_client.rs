use bitcoin_support;
use common_types::{
    ledger::{bitcoin::Bitcoin, ethereum::Ethereum},
    TradingSymbol,
};
use ethereum_support;
use exchange_api_client::{
    client::OrderResponseBody, ApiClient, OfferResponseBody, OrderRequestBody,
};
use reqwest;
use swaps::TradeId;
use uuid::Uuid;

#[allow(dead_code)]
pub struct FakeApiClient;

impl FakeApiClient {
    pub fn new() -> Self {
        FakeApiClient {}
    }
}

impl ApiClient for FakeApiClient {
    fn create_buy_offer(
        &self,
        symbol: TradingSymbol,
        amount: f64,
    ) -> Result<OfferResponseBody<Ethereum, Bitcoin>, reqwest::Error> {
        let rate = 0.1;
        let sell_amount = amount * rate;
        let offer = OfferResponseBody {
            uid: TradeId::from(Uuid::new_v4()),
            symbol,
            rate,
            sell_amount: bitcoin_support::BitcoinQuantity::from_bitcoin(sell_amount),
            buy_amount: ethereum_support::EthereumQuantity::from_eth(amount),
        };
        Ok(offer)
    }

    fn create_buy_order(
        &self,
        _symbol: TradingSymbol,
        _uid: TradeId,
        _trade_request: &OrderRequestBody,
    ) -> Result<OrderResponseBody, reqwest::Error> {
        let accept = OrderResponseBody {
            exchange_refund_address: String::from("34b19d15e793883d840c563d7dbc8a6723465146"),
            exchange_contract_time_lock: 43200,
            exchange_success_address: String::from("bcrt1qcqslz7lfn34dl096t5uwurff9spen5h4v2pmap"),
        };

        Ok(accept)
    }

    fn create_sell_offer(
        &self,
        symbol: TradingSymbol,
        amount: f64,
    ) -> Result<OfferResponseBody<Bitcoin, Ethereum>, reqwest::Error> {
        let rate = 0.1;
        let buy_amount = amount * rate;
        let offer = OfferResponseBody {
            uid: TradeId::from(Uuid::new_v4()),
            symbol,
            rate,
            sell_amount: ethereum_support::EthereumQuantity::from_eth(amount),
            buy_amount: bitcoin_support::BitcoinQuantity::from_bitcoin(buy_amount),
        };
        Ok(offer)
    }

    fn create_sell_order(
        &self,
        _symbol: TradingSymbol,
        _uid: TradeId,
        _trade_request: &OrderRequestBody,
    ) -> Result<OrderResponseBody, reqwest::Error> {
        let accept = OrderResponseBody {
            exchange_refund_address: String::from("bcrt1qcqslz7lfn34dl096t5uwurff9spen5h4v2pmap"),
            exchange_contract_time_lock: 43200,
            exchange_success_address: String::from("34b19d15e793883d840c563d7dbc8a6723465146"),
        };

        Ok(accept)
    }
}