import { ApiPromise, WsProvider } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";
import { Keyring } from "@polkadot/keyring";
import fs from "fs";
import 'dotenv/config';

/// Blockchain
const WS_ENDPOINT = process.env.WS_ENDPOINT;

console.log("Connecting to:", WS_ENDPOINT);
const wsProvider = new WsProvider(WS_ENDPOINT);
const api = await ApiPromise.create({ provider: wsProvider });
console.log("Connected successfully to:", (await api.rpc.system.chain()).toHuman());

process.exit(0);