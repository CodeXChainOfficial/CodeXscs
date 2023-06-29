multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct PriceTag<M: ManagedTypeApi> {
    pub token: EgldOrEsdtTokenIdentifier<M>,
    pub nonce: u64,
    pub amount: BigUint<M>,
}

#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct Profile<M: ManagedTypeApi> {
  pub name: ManagedBuffer<M>,
  pub avatar: ManagedBuffer<M>,
  pub location: ManagedBuffer<M>,
  pub website: ManagedBuffer<M>,
  pub shortbio: ManagedBuffer<M>,
}

#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct SocialMedia<M: ManagedTypeApi> {
  pub telegram: ManagedBuffer<M>,
  pub discord: ManagedBuffer<M>,
  pub twitter: ManagedBuffer<M>,
  pub medium: ManagedBuffer<M>,
  pub facebook: ManagedBuffer<M>,
  pub other_link: ManagedBuffer<M>,
}

#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct TextRecord<M: ManagedTypeApi> {
  pub name_value: ManagedBuffer<M>,
  pub link: ManagedBuffer<M>,
}

#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct Wallets<M: ManagedTypeApi> {
  pub egld: ManagedBuffer<M>,
  pub btc: ManagedBuffer<M>,
  pub eth: ManagedBuffer<M>,
}

#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct DomainName<M: ManagedTypeApi> {
  pub name: ManagedBuffer<M>,
  pub expires_at: u64,
  pub nft_nonce: u64,
  pub profile: Option<Profile<M>>,
  pub social_media: Option<SocialMedia<M>>,
  pub text_record: Option<ManagedVec<M, TextRecord<M>>>,
  pub wallets: Option<Wallets<M>>
}

#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct DomainNameAttributes {
  pub expires_at: u64,
}


#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct AcceptRequest<M: ManagedTypeApi> {
  pub domain_name: DomainName<M>,
  pub until: u64,
}

#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct Reservation<M: ManagedTypeApi> {
  pub domain_name: ManagedBuffer<M>,
  pub until: u64,
  pub reserved_for: ManagedAddress<M>,
}

#[derive(
  ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, TypeAbi, Default,
)]
pub struct SubDomain<M: ManagedTypeApi> {
  pub name: ManagedBuffer<M>,
  pub address: ManagedAddress<M>
}
