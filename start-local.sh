#!/bin/bash
rm -rf canister_ids.json
rm -rf deploy-assets-management.sh
rm -rf deploy-staking.sh
rm -rf generate-rpc.sh

dfx wallet upgrade
dfx canister create --all

# 生成本地环境相关的配置文件
npm run ic-env:local
chmod +x generate-rpc.sh
chmod +x deploy-assets-management.sh
chmod +x deploy-staking.sh
./generate-rpc.sh
./deploy-staking.sh
./deploy-assets-management.sh