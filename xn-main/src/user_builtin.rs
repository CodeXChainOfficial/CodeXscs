multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::proxy]
pub trait UserBuiltin {
    #[endpoint(SetUserName)]
    fn set_user_name(&self, name: ManagedBuffer);

    #[endpoint(DelUserName)]
    fn del_user_name(&self);
}
