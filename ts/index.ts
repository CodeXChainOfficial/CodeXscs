import { ApiNetworkProvider } from "@multiversx/sdk-network-providers"
import { AbiRegistry, Address, Account, AddressValue, SmartContract, U64Value, BinaryCodec, ResultsParser, StringValue, StringType, ContractFunction, U8Value, BigUIntType, TokenTransfer, TokenIdentifierType, TokenIdentifierValue, CompositeValue, CompositeType, VariadicType, VariadicValue } from "@multiversx/sdk-core";
import { UserSigner } from "@multiversx/sdk-wallet"; // md-ignore
import { TransactionWatcher } from "@multiversx/sdk-core";
import { promises } from "fs";
import { getReservations } from "./reservation";
import { domainType, textRecordsType, textRecord1, textRecordType, textRecord2 } from "./domain";

const networkProvider = new ApiNetworkProvider("https://devnet-api.multiversx.com", { timeout: 1_000_000_000 });

const address = "erd1qqqqqqqqqqqqqpgqm2y7angkuyew964jah77s20es9h3u55evycszs3hgn";
const abi_path = "./xn-main.abi.json";
const WEGLD = "WEGLD-d7c6bb";
const domain1 = "marko1.mvx";
const domain2 = "marko2.mvx";
const subdomain1 = "www.marko1.mvx";

let signer: UserSigner;
let other: UserSigner;
let contract: SmartContract;
let abiRegistry: AbiRegistry;

const setEnv = async () => {
  let pemText = await promises.readFile("./walletKey.pem", { encoding: "utf8" });
  signer = UserSigner.fromPem(pemText);

  pemText = await promises.readFile("./otherKey.pem", { encoding: "utf8" });
  other = UserSigner.fromPem(pemText);

  let abiJson = await promises.readFile(abi_path, { encoding: "utf8" });
  const abiObj = JSON.parse(abiJson);
  abiRegistry = AbiRegistry.create(abiObj)

  contract = new SmartContract({
    address: new Address(address),
    abi: abiRegistry,
  })
};

const getDomain = async (domain: string) => {
  let query = contract.createQuery({
    func: new ContractFunction("get_domain"),
    args: [new StringValue(domain)]
  });

  let queryResponse = await networkProvider.queryContract(query);

  //==============================
  //   const getDomainNameEndpoint = abiRegistry.getEndpoint("get_domain_name");
  //   let { values } = new ResultsParser().parseQueryResponse(queryResponse, getDomainNameEndpoint);
  //   console.log((values[0] as any).fields[6]);

  //=====================================
  let bundle = new ResultsParser().parseUntypedQueryResponse(queryResponse);
  let firstValue = bundle.values[0];

  // const codec = new BinaryCodec();
  // const domainType = abiRegistry.getStruct("Domain");
  // const data = Buffer.from(firstValue);
  // const [decoded, decodedLength] = codec.decodeNested(data, domainType);
  // const decodedDomain = decoded.valueOf();
  // console.log(decodedDomain);

  let decodedValue = new BinaryCodec().decodeTopLevel(firstValue, domainType);
  console.log(decodedValue.valueOf());
  return decodedValue.valueOf()
}


const getSubDomains = async (domain: string) => {
  let query = contract.createQuery({
    func: new ContractFunction("get_sub_domains"),
    args: [new StringValue(domain)]
  });

  let queryResponse = await networkProvider.queryContract(query);
  const getDomainNameEndpoint = abiRegistry.getEndpoint("get_sub_domains");
  let { values } = new ResultsParser().parseQueryResponse(queryResponse, getDomainNameEndpoint);
  console.log((values[0] as any).items[0].items[0].value.toString());
}

const getMainDomain = async (domain: string) => {
  let query = contract.createQuery({
    func: new ContractFunction("get_main_domain"),
    args: [new AddressValue(signer.getAddress())]
  });

  let queryResponse = await networkProvider.queryContract(query);
  let bundle = new ResultsParser().parseUntypedQueryResponse(queryResponse);
  let firstValue = bundle.values[0];
  let decodedValue = new BinaryCodec().decodeTopLevel(firstValue, new StringType());

  console.log(decodedValue.valueOf());
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

const setEgldPrice = async () => {
  let transaction = contract.methodsExplicit.fetch_egld_usd_prices([
  ])
    .withSender(signer.getAddress())
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

const register = async () => {
  let transaction = contract.methodsExplicit.register_or_renew([
    new StringValue(domain1),
    new U64Value(1),
    new U8Value(4)
  ])
    .withSender(signer.getAddress())
    .withSingleESDTTransfer(
      TokenTransfer.fungibleFromAmount(WEGLD, 0.000_000_000_1, 18),
    )
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


const setDomainProfileOverview = async () => {
  const domain = await getDomain(domain1);
  const domain_nft_id = await getDomainNftId();

  const compositeType = new CompositeType(new StringType, new StringType, new StringType, new StringType, new StringType);
  let transaction = contract.methodsExplicit.update_domain_overview([
    new StringValue(domain1),
    new CompositeValue(compositeType, [
      new StringValue("marko"),
      new StringValue("avatar"),
      new StringValue("location"),
      new StringValue("website"),
      new StringValue("shortbio"),
    ])
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

const setDomainProfileSocial = async () => {
  const domain = await getDomain(domain1);
  const domain_nft_id = await getDomainNftId();

  const compositeType = new CompositeType(new StringType, new StringType, new StringType, new StringType, new StringType, new StringType);

  let transaction = contract.methodsExplicit.update_domain_socials([
    new StringValue(domain1),
    new CompositeValue(compositeType, [
      new StringValue("telegram"),
      new StringValue("discord"),
      new StringValue("twitter"),
      new StringValue("medium"),
      new StringValue("facebook"),
      new StringValue("other_link")
    ]),
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

const setDomainProfileWallets = async () => {
  const domain = await getDomain(domain1);
  const domain_nft_id = await getDomainNftId();

  const compositeType = new CompositeType(new StringType, new StringType, new StringType);

  let transaction = contract.methodsExplicit.update_domain_wallets([
    new StringValue(domain1),
    new CompositeValue(compositeType, [
      new StringValue("egld"),
      new StringValue("btc"),
      new StringValue("eth"),
    ])
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

const setDomainProfileTextRecords = async () => {
  const domain = await getDomain(domain1);
  const domain_nft_id = await getDomainNftId();

  console.log(textRecord1.valueOf());

  let variadicType = new VariadicType(textRecordType);
  let transaction = contract.methodsExplicit.update_domain_textrecord([
    new StringValue(domain1),
    new VariadicValue(variadicType, [textRecord1, textRecord2]),
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
  const domain = await getDomain(domain1);
  const domain_nft_id = await getDomainNftId();

  console.log(new TokenIdentifierValue(WEGLD).toString());

  let transaction = contract.methodsExplicit.register_sub_domain([
    new StringValue(subdomain1),
    new AddressValue(signer.getAddress())
  ])
    .withSender(signer.getAddress())
    .withMultiESDTNFTTransfer([
      TokenTransfer.fungibleFromAmount(WEGLD, 0.000_000_000_1, 18),
      TokenTransfer.nonFungible(domain_nft_id, domain.nft_nonce)
    ])
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

const transferDomain = async () => {
  const domain = await getDomain(domain1);
  const domain_nft_id = await getDomainNftId();

  let transaction = contract.methodsExplicit.transfer_domain([
    new StringValue(domain1),
    new AddressValue(other.getAddress()),
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


const updatePrimaryDomain = async () => {
  const domain = await getDomain(domain1);
  const domain_nft_id = await getDomainNftId();

  let transaction = contract.methodsExplicit.update_primary_address([
    new StringValue(domain1),
  ])
    .withSender(other.getAddress())
    .withSingleESDTNFTTransfer(TokenTransfer.nonFungible(domain_nft_id, domain.nft_nonce))
    .withGasLimit(50_000_000)
    .withChainID("D")
    .buildTransaction();

  const account = new Account(other.getAddress());
  const accountOnNetwork = await networkProvider.getAccount(other.getAddress());
  account.update(accountOnNetwork);
  transaction.setNonce(account.getNonceThenIncrement());

  const serializedTransaction = transaction.serializeForSigning();
  const transactionSignature = await other.sign(serializedTransaction);
  transaction.applySignature(transactionSignature);

  await networkProvider.sendTransaction(transaction);
  let transactionOnNetwork = await new TransactionWatcher(networkProvider).awaitCompleted(transaction);

  console.log(JSON.stringify(transactionOnNetwork))
}

const main = async () => {
  await setEnv();
  // await getDomain(domain1);
  await getEgldPrice();
  await getDomainNftId();
  // await getSubDomains(domain1);
  
  // await setEgldPrice();
  // await register();
  // await setReservation();
  // await setDomainProfileOverview();
  // await setDomainProfileSocial();
  // await setDomainProfileWallets();
  // await setDomainProfileTextRecords();
  // await registerSubdomain();
  // await transferDomain();
  // await updatePrimaryDomain();
}

main();