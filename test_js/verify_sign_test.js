// Required imports
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { stringToU8a, u8aToHex, hexToU8a } from '@polkadot/util';
import { CodePromise, ContractPromise } from '@polkadot/api-contract';
import { readFileSync, writeFileSync } from 'fs';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { time } from 'console';
import { v4 as uuidv4 } from 'uuid';
import { start } from 'repl';
import fetch from 'node-fetch';

const polkasign_abi = "../release/polkasign.contract";
// const polkasign_address = "5FxUzmXE8FTc2WUfEnjUh86mbR4ELEUd9fqTcaXwqhvBu3Fz";
// const ws_endpoint = "wss://alpha.subdao.org";

const polkasign_address = "5H1HERkVLkag99FXedGb9tYNKEAzadxvCk3W7CitD9wmkADF";
const ws_endpoint = "ws://127.0.0.1:9944";

console.log("contract ws: ", ws_endpoint);
async function main() {

    await cryptoWaitReady(); // wait for crypto initializing

    const keyring = new Keyring();
    const pair = keyring.addFromUri("model action demand click genius pizza pumpkin develop muffin acquire supreme expand",
        { name: 'know pair' }, 'ed25519');
    // the pair has been added to our keyring
    console.log(keyring.pairs.length, 'pairs available');

    // log the name & address (the latter encoded with the ss58Format)
    console.log(pair.meta.name, 'has address', pair.address);

    // create the message, actual signature and verify
    // const message = stringToU8a('12345678900987654321123456789009');
    const message = hexToU8a('0xa00f94828aebefb421b1180ffe372e0fd5fbdc90bc7348c1ad4a0819910f1dfe');
    const signature = pair.sign(message);
    const isValid = pair.verify(message, signature);

// output the result
    console.log(`message ${u8aToHex(message)}`);
    console.log(`signature ${u8aToHex(signature)} is ${isValid ? 'valid' : 'invalid'}`);

    const provider = new WsProvider(ws_endpoint);
    const api = await ApiPromise.create({
        provider: provider,
        types: {
            "Address": "MultiAddress",
            "LookupSource": "MultiAddress"
        }
    });

    // Retrieve the chain & node information information via rpc calls
    const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version()
    ]);

    console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);
    let wait = ms => new Promise(resolve => setTimeout(resolve, ms));

    const endowment = 1230000000000n;
    const gasLimit = 500000n * 10000000n;
    const polkasignAbi = JSON.parse(readFileSync(polkasign_abi).toString());
    const polkasignContract = new ContractPromise(api, polkasignAbi, polkasign_address);

    {
        console.log("========= begin to query checkSign");
        const { gasConsumed, result, output } = await polkasignContract.query.checkSign(pair.address,
            { value: 0, gasLimit: gasLimit },
            message,
            signature
        )
        console.log("gasConsumed", gasConsumed.toHuman());
        if (result.isOk) {
            console.log('checkSign Success', output.toHuman());
        } else {
            console.error('checkSign Error', result.toHuman());
        }
    }

    {
        // console.log("========= begin to checkSign");
        // let nonce = await api.rpc.system.accountNextIndex(pair.address);
        // const unsubCall1 = await polkasignContract.tx
        //     .checkSign({ value: 0, gasLimit: gasLimit }
        //         , message
        //         , signature)
        //     .signAndSend(pair, { nonce: nonce }, (result) => {
        //         if (result.status.isInBlock || result.status.isFinalized) {
        //             if (!!result.dispatchError) {
        //                 console.log('isBadOrigin is ', result.dispatchError.isBadOrigin);
        //                 console.log('isOther is ', result.dispatchError.isOther);
        //                 console.log('isModule is ', result.dispatchError.isModule);
        //             } else {
        //                 console.log('add service success for ', gw_service.id);
        //             }
        //             unsubCall1();
        //         }
        //     });
        await wait(10000); // 10s
    }

    console.log("The End!!!");
}

main().catch(console.error).finally(() => process.exit());
