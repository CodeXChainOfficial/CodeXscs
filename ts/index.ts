import { ApiNetworkProvider } from "@multiversx/sdk-network-providers"
import { AbiRegistry, Address, Account, AddressValue, SmartContract, U64Value, BinaryCodec, ResultsParser, StringValue, ArrayVec, ArrayVecType, StructType, FieldDefinition, Field, StringType, AddressType, U64Type, Struct, ContractFunction, EnumValue, EnumType, U8Value, BigUIntType, TokenTransfer, TokenIdentifierType, OptionalValue } from "@multiversx/sdk-core";
import { UserSigner } from "@multiversx/sdk-wallet"; // md-ignore
import { TransactionWatcher } from "@multiversx/sdk-core";
import { promises } from "fs";
import { getReservations } from "./reservation";
import { domainType, profileStruct, socialType, textRecordsType, walletsType } from "./domain";

const networkProvider = new ApiNetworkProvider("https://devnet-api.multiversx.com", { timeout: 1_000_000_000 });

const address = "erd1qqqqqqqqqqqqqpgqev7w2j8e54tvnzc2rtj6v7mxqdy5lam0vycseduvnh";
const abi_path = "./xn-main.abi.json";

let signer: UserSigner;
let contract: SmartContract;

const setEnv = async () => {
  const pemText = await promises.readFile("./walletKey.pem", { encoding: "utf8" });
  signer = UserSigner.fromPem(pemText);

  let abiJson = await promises.readFile(abi_path, { encoding: "utf8" });
  let abiObj = JSON.parse(abiJson);
  const abiRegistry = AbiRegistry.create(abiObj)

  contract = new SmartContract({
    address: new Address(address),
    abi: abiRegistry,
  })
};

const getDomain = async (domain: string) => {
  let query = contract.createQuery({
    func: new ContractFunction("get_domain_name"),
    args: [new StringValue(domain)]
  });

  let queryResponse = await networkProvider.queryContract(query);
  let bundle = new ResultsParser().parseUntypedQueryResponse(queryResponse);
  let firstValue = bundle.values[0];
  let decodedValue = new BinaryCodec().decodeTopLevel(firstValue, domainType);

  console.log(bundle.returnCode);
  console.log(bundle.returnMessage);
  console.log(bundle.values);
  console.log(decodedValue.valueOf());
  return decodedValue.valueOf()
}

const getEgldPrice = async () => {
  let query = contract.createQuery({
    func: new ContractFunction("get_egld_usd_price"),
    args: []
  });

  let queryResponse = await networkProvider.queryContract(query);
  let bundle = new ResultsParser().parseUntypedQueryResponse(queryResponse);
  let firstValue = bundle.values[0];
  let decodedValue = new BinaryCodec().decodeTopLevel(firstValue, new BigUIntType());

  console.log(decodedValue.valueOf().toFixed());
}
const getDomainNftId = async () => {
  let query = contract.createQuery({
    func: new ContractFunction("get_domain_nft_id"),
    args: []
  });

  let queryResponse = await networkProvider.queryContract(query);
  let bundle = new ResultsParser().parseUntypedQueryResponse(queryResponse);
  let firstValue = bundle.values[0];
  let decodedValue = new BinaryCodec().decodeTopLevel(firstValue, new TokenIdentifierType());

  console.log(firstValue)
  console.log(decodedValue.valueOf());
  return decodedValue.valueOf();
}

const register = async () => {
  let transaction = contract.call({
    caller: signer.getAddress(),
    func: new ContractFunction("register_or_renew"),
    gasLimit: 50_000_000,
    args: [
      new StringValue("marko1.mvx"),
      new U64Value(1),
      new U8Value(4)
    ],
    chainID: "D",
    value: 100_000
  });

  const account = new Account(signer.getAddress());
  const accountOnNetwork = await networkProvider.getAccount(signer.getAddress());
  account.update(accountOnNetwork);
  transaction.setNonce(account.getNonceThenIncrement());

  const serializedTransaction = transaction.serializeForSigning();
  const transactionSignature = await signer.sign(serializedTransaction);
  transaction.applySignature(transactionSignature);

  await networkProvider.sendTransaction(transaction);
  let transactionOnNetwork = await new TransactionWatcher(networkProvider).awaitCompleted(transaction);

  console.log(JSON.stringify(transactionOnNetwork))
}

const setReservation = async () => {
  const reservations = await getReservations();
  let transaction = contract.call({
    caller: signer.getAddress(),
    func: new ContractFunction("set_reservations"),
    gasLimit: 50_000_000,
    args: [reservations],
    chainID: "D"
  });

  const account = new Account(signer.getAddress());
  const accountOnNetwork = await networkProvider.getAccount(signer.getAddress());
  account.update(accountOnNetwork);
  transaction.setNonce(account.getNonceThenIncrement());

  const serializedTransaction = transaction.serializeForSigning();
  const transactionSignature = await signer.sign(serializedTransaction);
  transaction.applySignature(transactionSignature);

  await networkProvider.sendTransaction(transaction);
  let transactionOnNetwork = await new TransactionWatcher(networkProvider).awaitCompleted(transaction);

  console.log(JSON.stringify(transactionOnNetwork))
}

const setDomainProfile = async () => {
  const domain = await getDomain("marko1.mvx");
  const domain_nft_id = await getDomainNftId();

  let transaction = contract.methodsExplicit.update_domain_profile([
    new StringValue("marko1.mvx"),
    profileStruct,
    new OptionalValue(socialType, null),
    new OptionalValue(textRecordsType, null),
    new OptionalValue(walletsType, null)
  ])
  .withSender(signer.getAddress())
  .withSingleESDTNFTTransfer(TokenTransfer.nonFungible(domain_nft_id, domain.nft_nonce))
  .withGasLimit(50_000_000)
  .withChainID("D")
  .buildTransaction();

  const account = new Account(signer.getAddress());
  const accountOnNetwork = await networkProvider.getAccount(signer.getAddress());
  account.update(accountOnNetwork);
  transaction.setNonce(account.getNonceThenIncrement());

  const serializedTransaction = transaction.serializeForSigning();
  const transactionSignature = await signer.sign(serializedTransaction);
  transaction.applySignature(transactionSignature);

  await networkProvider.sendTransaction(transaction);
  let transactionOnNetwork = await new TransactionWatcher(networkProvider).awaitCompleted(transaction);

  console.log(JSON.stringify(transactionOnNetwork))
}

const registerSubdomain = async () => {
  const domain = await getDomain("marko1.mvx");
  const domain_nft_id = await getDomainNftId();

  let transaction = contract.methodsExplicit.register_sub_domain([
    new StringValue("www.marko1.mvx"),
    new AddressValue(signer.getAddress())
  ])
  .withSender(signer.getAddress())
  .withValue(10)
  .withSingleESDTNFTTransfer(TokenTransfer.nonFungible(domain_nft_id, domain.nft_nonce))
  .withGasLimit(50_000_000)
  .withChainID("D")
  .buildTransaction();

  const account = new Account(signer.getAddress());
  const accountOnNetwork = await networkProvider.getAccount(signer.getAddress());
  account.update(accountOnNetwork);
  transaction.setNonce(account.getNonceThenIncrement());

  const serializedTransaction = transaction.serializeForSigning();
  const transactionSignature = await signer.sign(serializedTransaction);
  transaction.applySignature(transactionSignature);

  await networkProvider.sendTransaction(transaction);
  let transactionOnNetwork = await new TransactionWatcher(networkProvider).awaitCompleted(transaction);

  console.log(JSON.stringify(transactionOnNetwork))
}

const main = async () => {
  await setEnv();

  // await getDomain();
  await getEgldPrice();
  // await getDomainNftId();
  // await register();
  // await setReservation();
  // await setDomainProfile();
  // await registerSubdomain();
}

main();