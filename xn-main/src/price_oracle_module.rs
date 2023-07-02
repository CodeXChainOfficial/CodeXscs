multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::callback_module::CallbackProxy;

mod xexchange_pair_proxy {
    use super::*;

    #[multiversx_sc::proxy]
    pub trait XexchangePairProxy {
        #[view(getEquivalent)]
        fn get_equivalent(&self, token_in: TokenIdentifier, amount_in: BigUint) -> BigUint;

        #[view(getFirstTokenId)]
        #[storage_mapper("first_token_id")]
        fn first_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

        #[view(getSecondTokenId)]
        #[storage_mapper("second_token_id")]
        fn second_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
    }
}

#[multiversx_sc::module]
pub trait PriceOracleModule: 
    crate::callback_module::CallbackModule
    + crate::storage_module::StorageModule
{
    #[proxy]
    fn xexchange_pair_contract(&self, sc_address: ManagedAddress) -> xexchange_pair_proxy::Proxy<Self::Api>;

    fn xexchange_pair_get_equivalent(
        &self,
        sc_address: ManagedAddress,
        token_in: TokenIdentifier,
        amount_in: BigUint,
    ) {
        let mut args = ManagedVec::new();
        args.push(token_in.into_managed_buffer());
        args.push(amount_in.to_bytes_be_buffer());

        self.send()
        .contract_call::<()>(sc_address, ManagedBuffer::from("get_equivalent"))
        .with_raw_arguments(args.into())
        .async_call()
        .with_callback(self.callbacks().xexchange_callback())
        .call_and_exit()
    }
}
