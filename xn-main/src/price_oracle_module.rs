multiversx_sc::imports!();

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
pub trait PriceOracleModule
{
    #[proxy]
    fn xexchange_pair_contract(&self, sc_address: ManagedAddress) -> xexchange_pair_proxy::Proxy<Self::Api>;

    fn xexchange_pair_get_equivalent(
        &self,
        sc_address: ManagedAddress,
        token_in: TokenIdentifier,
        amount_in: BigUint,
    ) -> BigUint {
        let amount_out: BigUint = self.xexchange_pair_contract(sc_address)
                .get_equivalent(token_in, amount_in)
                .execute_on_dest_context();
        
        amount_out
    }
}
