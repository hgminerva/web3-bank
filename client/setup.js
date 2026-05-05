import { ApiPromise, WsProvider } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";
import { Keyring } from "@polkadot/keyring";
import fs from "fs";
import 'dotenv/config';
import { decode } from "./decode.js";

const WS_ENDPOINT = process.env.WS_ENDPOINT;
const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;
const CONTRACT_ABI_PATH = process.env.CONTRACT_ABI_PATH;
const ALICE = process.env.ALICE;

console.log("Connecting to:", WS_ENDPOINT);
const wsProvider = new WsProvider(WS_ENDPOINT);
const api = await ApiPromise.create({ provider: wsProvider });
console.log("Connected successfully to:", (await api.rpc.system.chain()).toHuman());

const abiJSON = JSON.parse(fs.readFileSync(CONTRACT_ABI_PATH, "utf8"));
const contract = new ContractPromise(api, abiJSON, CONTRACT_ADDRESS);
const gasLimit = api.registry.createType('WeightV2', {
          refTime: 300000000000,
          proofSize: 500000,
});
const storageDepositLimit = null;

// Parameter
const assetId = "1000000003"
const loanAssetId = "1984"
const manager = "XqEqTNDXfTjWbSaCuio852eNsDRvCW73iuPSywizW7FNKeZ4L"
const maximumAccounts = 1000;
const threshold = 20;
const dailyBlocks = 14400;

// Caller
const keyring = new Keyring({ type: "sr25519" });
const alice = keyring.addFromUri(ALICE);

/// Execution
await new Promise(async (resolve, reject) => {
  const unsub = await contract.tx
    .setup({ storageDepositLimit, gasLimit }, 
      assetId,
      loanAssetId,
      manager,
      maximumAccounts,
      threshold,
      dailyBlocks
    ).signAndSend(alice, ({ status, events, dispatchError }) => {    
      console.log("Status:", status?.type);
      if(events?.length > 0) {
        events.forEach(({ event }) => {
          if (event.section === "contracts" && event.method === "ContractEmitted") {
            console.log(decode(event.data));
            unsub();
            resolve();
          }
        });
      }
  });
});


process.exit(0);