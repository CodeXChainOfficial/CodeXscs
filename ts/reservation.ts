import { AbiRegistry, Address, Account, AddressValue, SmartContract, U64Value, BinaryCodec, ResultsParser, StringValue, ArrayVec, ArrayVecType, StructType, FieldDefinition, Field, StringType, AddressType, U64Type, Struct, ContractFunction, EnumValue, EnumType, U8Value } from "@multiversx/sdk-core";

import { promises } from "fs";

export let reservationType = new StructType(
    "reservation",
    [
        new FieldDefinition("domain_name", "", new StringType()),
        new FieldDefinition("reserved_for", "", new AddressType()),
        new FieldDefinition("until", "", new U64Type())
    ]
);

export const getReservations = async () => {
    const reservationsJson = JSON.parse(await promises.readFile("./reservations.json", { encoding: "utf8" }));

    const length = reservationsJson.hits.hits.length;
    let arrayType = new ArrayVecType(length, reservationType);
    const reservations: Struct[] = [];
  
    reservationsJson.hits.hits.forEach((element: any) => {
      let reservationStruct = new Struct(reservationType, [
        new Field(new StringValue(element._source.userName), "domain_name"),
        new Field(new AddressValue(new Address(element._source.address)), "reserved_for"),
        new Field(new U64Value(Date.now() + 365 * 24 * 60 * 60), "until")
      ]);
      reservations.push(reservationStruct)
    });
    let array = new ArrayVec(
      arrayType,
      reservations
    );
    return array;
}