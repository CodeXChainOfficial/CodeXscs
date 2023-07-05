multiversx_sc::imports!();
multiversx_sc::derive_imports!();

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
    + crate::utils_module::UtilsModule
    + crate::nft_module::NftModule
{
    #[proxy]
    fn xexchange_pair_contract(
        &self,
        sc_address: ManagedAddress,
    ) -> xexchange_pair_proxy::Proxy<Self::Api>;

    fn xexchange_pair_get_equivalent(
        &self,
        sc_address: ManagedAddress,
        token_in: TokenIdentifier,
        amount_in: BigUint,
        callback: CallbackClosure<Self::Api>,
    ) {
        // let mut args = ManagedVec::new();
        // args.push(token_in.into_managed_buffer());
        // args.push(amount_in.to_bytes_be_buffer());

        // self.send()
        //     .contract_call::<()>(sc_address, ManagedBuffer::from("get_equivalent"))
        //     .with_raw_arguments(args.into())
        //     .async_call()
        //     .with_callback(callback)
        //     .call_and_exit()
        
        self.xexchange_pair_contract(sc_address)
            .get_equivalent(token_in, amount_in)
            .async_call()
            .with_callback(callback)
            .call_and_exit();
    }

    fn sync_get_equivalent(
        &self,
        sc_address: ManagedAddress,
        token_in: TokenIdentifier,
        amount_in: BigUint,
    ) -> BigUint {
        let amount_out: BigUint = self
            .xexchange_pair_contract(sc_address)
            .get_equivalent(token_in, amount_in)
            .execute_on_dest_context();
        amount_out
    }
}
