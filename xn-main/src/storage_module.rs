multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::data_module::*;

#[multiversx_sc::module]
pub trait StorageModule {
  // storage
  #[view(get_reservation)]
  #[storage_mapper("reservations")]
    fn reservations(
      &self,
      domain_name: &ManagedBuffer,
  ) -> SingleValueMapper<Reservation<Self::Api>>;

  #[view(getNftTokenId)]
  #[storage_mapper("nftTokenId")]
  fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

  #[view(get_accept_request)]
  #[storage_mapper("accept_request")]
  fn accept_request(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

  #[view(get_domain_name)]
  #[storage_mapper("domain_name")]
  fn domain_name(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<DomainName<Self::Api>>;

  #[view(get_sub_domains)]
  #[storage_mapper("sub_domains")]
  fn sub_domains(&self, domain_name: &ManagedBuffer) -> VecMapper<SubDomain<Self::Api>>;

  #[view(get_owner_domain_name)]
  #[storage_mapper("owner_domain_name")]
  fn owner_domain_name(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

  #[view(resolve)]
  #[storage_mapper("resolve_domain_name")]
  fn resolve_domain_name(&self, domain_name: &ManagedBuffer)
      -> SingleValueMapper<ManagedAddress>;

  #[view(resolve_domain_name_key)]
  #[storage_mapper("resolve_key")]
  fn resolve_domain_name_key(
      &self,
      domain_name: &ManagedBuffer,
      key: &ManagedBuffer,
  ) -> SingleValueMapper<ManagedBuffer>;

  #[view(get_prices_usd)]
  #[storage_mapper("prices_usd")]
  fn domain_length_to_yearly_rent_usd(&self) -> VecMapper<u64>;

  #[view(get_egld_usd_price)]
  #[storage_mapper("egld_usd_price")]
  fn egld_usd_price(&self) -> SingleValueMapper<u64>;

  #[storage_mapper("oracle_address")]
  fn oracle_address(&self) -> SingleValueMapper<ManagedAddress>;

  #[view(get_allowed_top_level_domains)]
  #[storage_mapper("allowed_top_level_domains")]
  fn allowed_top_level_domains(&self) -> VecMapper<ManagedBuffer>;
}