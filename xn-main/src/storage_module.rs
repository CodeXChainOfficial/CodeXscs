multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::data_module::*;

#[multiversx_sc::module]
pub trait StorageModule {
  #[view(get_reservation)]
  #[storage_mapper("reservations")]
    fn reservations(
      &self,
      domain_name: &ManagedBuffer,
  ) -> SingleValueMapper<Reservation<Self::Api>>;

  #[view(get_domain_nft)]
  #[storage_mapper("domainNFT")]
  fn domain_nft(&self) -> NonFungibleTokenMapper;

  #[view(get_domain)]
  #[storage_mapper("domain")]
  fn domain(&self, domain_name: &ManagedBuffer) -> SingleValueMapper<Domain<Self::Api>>;

  #[view(get_sub_domains)]
  #[storage_mapper("sub_domains")]
  fn sub_domains(&self, domain_name: &ManagedBuffer) -> MapMapper<ManagedBuffer, SubDomain<Self::Api>>;

  #[view(get_main_domain)]
  #[storage_mapper("main_domain")]
  fn main_domain(&self, addr: &ManagedAddress) -> SingleValueMapper<ManagedBuffer>;

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
  fn rental_fee(&self) -> SingleValueMapper<RentalFee>; 

  #[view(get_egld_usd_price)]
  #[storage_mapper("egld_usd_price")]
  fn egld_usd_price(&self) -> SingleValueMapper<u64>;

  #[storage_mapper("oracle_address")]
  fn oracle_address(&self) -> SingleValueMapper<ManagedAddress>;

  #[view(get_allowed_top_level_domains)]
  #[storage_mapper("allowed_top_level_domains")]
  fn allowed_top_level_domains(&self) -> VecMapper<ManagedBuffer>;

  #[view(get_migration_start_time)]
  #[storage_mapper("migration_start_time")]
  fn migration_start_time(&self) -> SingleValueMapper<u64>;

  #[view(get_royalties)]
  #[storage_mapper("get_royalties")]
  fn royalties(&self) -> SingleValueMapper<u64>;

}