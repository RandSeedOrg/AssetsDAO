import { writeFileSync } from 'fs';
import { resolve } from 'path';
import dotenv from 'dotenv';
dotenv.config();

const canisterConfig = {
  admin: {
    [process.env.DFX_NETWORK]: process.env.CANISTER_ID_ADMIN
  },
  staking: {
    [process.env.DFX_NETWORK]: process.env.CANISTER_ID_STAKING
  },
  assets_management: {
    [process.env.DFX_NETWORK]: process.env.CANISTER_ID_ASSETS_MANAGEMENT
  },
};

writeFileSync(resolve('./canister_ids.json'), JSON.stringify(canisterConfig, null, 2));

// 0. generate rpc scripts
const rpcScript = `
#!/bin/bash

# generate staking candid file
cargo build --target wasm32-unknown-unknown --release --package staking --locked
candid-extractor target/wasm32-unknown-unknown/release/staking.wasm > canisters/staking/staking.did
dfx generate staking

# generate assets_management candid file
cargo build --target wasm32-unknown-unknown --release --package assets_management --locked
candid-extractor target/wasm32-unknown-unknown/release/assets_management.wasm > canisters/assets_management/assets_management.did
dfx generate assets_management
`;

writeFileSync(resolve('./generate-rpc.sh'), rpcScript);

const generate_deploy_script = (canister_name, canister_args, build_script) => {
  return `
#!/bin/bash${build_script ? `\n${build_script}` : ''}
dfx deploy ${canister_name}${canister_args ? ` --argument '${canister_args}' ` : ' '}--network ${process.env.DFX_NETWORK} ${process.env.DFX_NETWORK === 'local' ? `--specified-id ${canisterConfig[canister_name][process.env.DFX_NETWORK]}` : ''}
  `;
}

const generate_deploy_script_with_subscribe = (canister_name, canister_args) => {
  return `${generate_deploy_script(canister_name, canister_args)}
dfx canister call ${canister_name} setup_subscribe '(principal "${process.env.CANISTER_ID_ADMIN}")' --network ${process.env.DFX_NETWORK}
  `;
}

// staking
const staking_shell = generate_deploy_script_with_subscribe('staking');
writeFileSync(resolve('./deploy-staking.sh'), staking_shell);

// staking
const assets_management_shell = generate_deploy_script_with_subscribe('assets_management');
writeFileSync(resolve('./deploy-assets-management.sh'), assets_management_shell);