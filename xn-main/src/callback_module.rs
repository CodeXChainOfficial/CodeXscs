multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait CallbackModule: crate::storage_module::StorageModule {
  #[callback]
  fn fetch_egld_usd_prices_callback(&self, #[call_result] result: ManagedAsyncCallResult<u64>) {
    match result {
      ManagedAsyncCallResult::Ok(price) => {
        self.egld_usd_price().set(price);
      }
      ManagedAsyncCallResult::Err(_) => {
        // this can only fail if the oracle contract address is invalid
        // nothing to revert in case of error
      }
    }
  }
  
  #[callback]
  fn set_user_name_callback(
    &self,
    domain_name: &ManagedBuffer,
    address: &ManagedAddress,
    #[call_result] result: ManagedAsyncCallResult<()>,
  ) {
    match result {
      ManagedAsyncCallResult::Ok(()) => {
        self.resolve_domain_name(&domain_name).set(address);
      }
      ManagedAsyncCallResult::Err(_) => {}
    }
  }
  
  #[callback]
  fn del_user_name_callback(
    &self,
    domain_name: &ManagedBuffer,
    #[call_result] result: ManagedAsyncCallResult<()>,
  ) {
    match result {
      ManagedAsyncCallResult::Ok(()) => {
        self.resolve_domain_name(&domain_name).clear();
      }
      ManagedAsyncCallResult::Err(_) => {}
    }
  }
}