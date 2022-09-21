use anyhow::Ok;
use near_sdk::{json_types::U128, AccountId};
use workspaces::{Contract, prelude::DevAccountDeployer, Worker, network::Sandbox, Account};
use near_units::parse_near;
use serde_json::{json, Value};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, Timestamp};

pub type CommandId = String;
pub type NameProduct = String;

const CONTRACT_PATH: &str = "./wasm/contract.wasm";

#[derive( Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]

pub struct Quality {
    pub certificate: Vec<AccountId>,
    pub stage: Vec<AccountId>,
}

#[derive( Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommandDetail {
    pub command_id: CommandId,
    pub name_product: NameProduct,
    pub is_sell: bool,
    pub amount_product: U128,
    pub price_per_product: U128,
    pub quality: Option<Quality>,
    pub command_owner_id: AccountId,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Tao worker -> start worker
    let worker = workspaces::sandbox().await?;

    // Dev-Deploy ecommerce payment contract len worker
    let contract_wasm = std::fs::read(CONTRACT_PATH)?;
    let contract: Contract = worker.dev_deploy(&contract_wasm).await?;

    // Create account
    // mainnet -> root account = near ex abc.near, xyz.near
    // testnet -> root account = testnet, ex: vbidev.testnet
    let owner = worker.root_account().unwrap();

    // tao account alice, vbidev.testnet, uit-payment-contract.vbidev.testnet
    let commander = owner.create_subaccount(&worker, "commander")
                                            .initial_balance(parse_near!("30 N"))
                                            .transact()
                                            .await?
                                            .into_result()?;

    // Init contract
    contract
        .call(&worker, "new")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
        }))?
        .transact()
        .await?;

    // Begin test
    test_add_buy_command(&commander, &contract, &worker).await?;
    Ok(())
}

async fn test_add_buy_command(user: &Account, contract: &Contract, worker: &Worker<Sandbox>) -> anyhow::Result<()> {
    let deposit = parse_near!("21 N");

    user.
        call(&worker, contract.id(), "add_command")
        .args_json(json!({
            "command_id": "command_1",
            "name_product": "Iphone_14",
            "is_sell": false,
            "amount_product": U128(1),
            "price_per_product": U128(20),
            "quality": null,
        }))?
        .deposit(deposit)
        .transact()
        .await?;

    println!("      Passed ✅  add_buy_command");

    let res_command: CommandDetail = user.call(worker, contract.id(), "get_command")
                                        .args_json(json!({
                                            "command_id": "conmmand_1"
                                        }))?
                                        .transact()
                                        .await?
                                        .json()?;

    assert_eq!(res_command.name_product.to_string(), "Iphone_14");
    assert_eq!(res_command.is_sell, false);

    println!("      Passed ✅  get_command");

    Ok(())
}