import { AbiRegistry, Address, Account, AddressValue, SmartContract, U64Value, BinaryCodec, ResultsParser, StringValue, ArrayVec, ArrayVecType, StructType, FieldDefinition, Field, StringType, AddressType, U64Type, Struct, ContractFunction, EnumValue, EnumType, U8Value, OptionType } from "@multiversx/sdk-core";

export let profileType = new StructType(
    "profile",
    [
        new FieldDefinition("name", "", new StringType()),
        new FieldDefinition("avatar", "", new StringType()),
        new FieldDefinition("location", "", new StringType()),
        new FieldDefinition("website", "", new StringType()),
        new FieldDefinition("shortbio", "", new StringType()),

    ]
);

export let socialType = new StructType(
    "social",
    [
        new FieldDefinition("telegram", "", new StringType()),
        new FieldDefinition("discord", "", new StringType()),
        new FieldDefinition("twitter", "", new StringType()),
        new FieldDefinition("medium", "", new StringType()),
        new FieldDefinition("facebook", "", new StringType()),
        new FieldDefinition("other_link", "", new StringType()),
    ]
);

export let textRecordType = new StructType(
    "textrecord",
    [
        new FieldDefinition("name_value", "", new StringType()),
        new FieldDefinition("link", "", new StringType()),
    ]
);
export let textRecordsType = new ArrayVecType(1, textRecordType);

export let walletsType = new StructType(
    "wallets",
    [
        new FieldDefinition("egld", "", new StringType()),
        new FieldDefinition("btc", "", new StringType()),
        new FieldDefinition("eth", "", new StringType()),
    ]
);

export let domainType = new StructType(
    "domain",
    [
        new FieldDefinition("name", "", new StringType()),
        new FieldDefinition("expires_at", "", new U64Type()),
        new FieldDefinition("nft_nonce", "", new U64Type()),
        new FieldDefinition("profile", "", new OptionType(profileType)),
        new FieldDefinition("social_media", "", new OptionType(socialType)),
        new FieldDefinition("wallets", "", new OptionType(walletsType)),
        new FieldDefinition("text_record", "", new OptionType(textRecordsType)),
    ]
);

export const profileStruct = new Struct(profileType, [
    new Field(new StringValue("Marko"), "name"),
    new Field(new StringValue("avatar"), "avatar"),
    new Field(new StringValue("Serbia"), "location"),
    new Field(new StringValue("https://facebook.io"), "website"),
    new Field(new StringValue("Dev"), "shortbio"),
]);

export const socialStruct = new Struct(socialType, [
    new Field(new StringValue("https://telegram.com/marko"), "telegram"),
    new Field(new StringValue("https://discord.com/marko"), "discord"),
    new Field(new StringValue("https://twitter.com/marko"), "twitter"),
    new Field(new StringValue("https://medium.io/marko"), "medium"),
    new Field(new StringValue("https://facebook.io"), "facebook"),
    new Field(new StringValue("https://sample.dev.io/marko"), "other_link"),
]);

export const textRecord1 = new Struct(textRecordType, [
    new Field(new StringValue("name_value"), "name_value"),
    new Field(new StringValue("https://discord.com/marko"), "link"),
]);
export const textRecords = new ArrayVec(textRecordsType, [textRecord1]);

export const walletStruct = new Struct(walletsType, [
    new Field(new StringValue("erd1jk8tetypqufjwkydyvk0gcta9wnqjxh05krnedhv4yf52pwgvycs5k6lsr"), "egld"),
    new Field(new StringValue("erd1jk8tetypqufjwkydyvk0gcta9wnqjxh05krnedhv4yf52pwgvycs5k6lsr"), "btc"),
    new Field(new StringValue("erd1jk8tetypqufjwkydyvk0gcta9wnqjxh05krnedhv4yf52pwgvycs5k6lsr"), "eth")
]);