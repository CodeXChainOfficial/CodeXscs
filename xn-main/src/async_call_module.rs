multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::callback_module::*;
use crate::constant_module::WEGLD_ID;
use crate::data_module::PeriodType;

#[multiversx_sc::module]
pub trait AsyncCallModule:
    crate::storage_module::StorageModule
    + crate::callback_module::CallbackModule
    + crate::utils_module::UtilsModule
    + crate::price_oracle_module::PriceOracleModule
    + crate::nft_module::NftModule
{
    fn get_egld_price_for_register(
        &self,
        domain_name: ManagedBuffer,
        period: u8,
        unit: PeriodType,
        assign_to: OptionalValue<ManagedAddress>,
    ) {
        let payments = self.call_value().all_esdt_transfers();
        let caller = self.blockchain().get_caller();

        let _assign_to = match assign_to {
            OptionalValue::Some(to) => Some(to),
            _ => None,
        };

        self.xexchange_pair_get_equivalent(
            self.oracle_address().get(),
            TokenIdentifier::from_esdt_bytes(WEGLD_ID),
            BigUint::from(1_000_000_000_000_000_000u64),
            self.callbacks().register_or_renew_callback(
                caller,
                payments,
                domain_name,
                period,
                unit,
                _assign_to,
            ),
        )
    }
    fn get_egld_price_for_register_subdomain(
        &self,
        sub_domain: ManagedBuffer,
        address: ManagedAddress,
    ) {
        let payments = self.call_value().all_esdt_transfers();
        let caller = self.blockchain().get_caller();

        self.xexchange_pair_get_equivalent(
            self.oracle_address().get(),
            TokenIdentifier::from_esdt_bytes(WEGLD_ID),
            BigUint::from(1_000_000_000_000_000_000u64),
            self.callbacks()
                .register_subdomain_callback(caller, payments, sub_domain, address),
        )
    }

    fn internal_set_egld_price(&self) {
        self.xexchange_pair_get_equivalent(
            self.oracle_address().get(),
            TokenIdentifier::from_esdt_bytes(WEGLD_ID),
            BigUint::from(1_000_000_000_000_000_000u64),
            self.callbacks().xexchange_callback(),
        )
    }

    fn internal_fetch_egld_usd_prices(&self) {
        let oracle_address = self.oracle_address().get();

        let mut args = ManagedVec::new();
        args.push(ManagedBuffer::from("egld"));
        args.push(ManagedBuffer::from("usd"));

        self.send()
            .contract_call::<()>(oracle_address, ManagedBuffer::from("latestPriceFeed"))
            .with_raw_arguments(args.into())
            .async_call()
            .with_callback(self.callbacks().fetch_egld_usd_prices_callback())
            .call_and_exit()
    }
}
