/*!
 * AssetsDAO
 * https://github.com/RandSeedOrg/AssetsDAO
 * Copyright (C) 2025 RandSeedOrg
 * https://github.com/RandSeedOrg/AssetsDAO/blob/master/LICENSE
 */

#[ic_cdk::update]
fn greet() -> String {
  "Hello, AssetsDAO!".to_string()
}

ic_cdk::export_candid!();
