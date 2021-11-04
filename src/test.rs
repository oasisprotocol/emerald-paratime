use std::collections::BTreeMap;

use io_context::Context as IoContext;
use oasis_runtime_sdk::{
    context::{Context, Mode},
    core::{
        consensus::{
            address::Address,
            staking::{Account, GeneralAccount},
            state::ConsensusState,
        },
        storage::mkvs,
    },
    modules,
    modules::accounts::{Module as Accounts, API as _},
    testing::{keys, mock},
    types::token::Denomination,
    Runtime as _,
};

use super::Runtime;

#[test]
fn test_migration() {
    let mut mock = mock::Mock::default();

    // Populate consensus layer state.
    // TODO: Add some wrappers to make mocking consensus state easier.
    let mut consensus_tree = mkvs::Tree::make()
        .with_root_type(mkvs::RootType::State)
        .new(Box::new(mkvs::sync::NoopReadSyncer));

    // Insert account balance for the runtime account. This is required for invariant checks.
    let runtime_address = Address::from_runtime_id(&Default::default());
    consensus_tree
        .insert(
            IoContext::background(),
            &[&[0x50], runtime_address.as_ref()].concat(),
            &cbor::to_vec(Account {
                general: GeneralAccount {
                    // All tokens inside the runtime come from the consensus layer.
                    balance: 1_234_567u128.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .unwrap();

    mock.consensus_state = ConsensusState::new(consensus_tree);

    let mut ctx = mock.create_ctx_for_runtime::<Runtime>(Mode::ExecuteTx);

    // Prepare some dummy initial state.
    Accounts::init(
        &mut ctx,
        modules::accounts::Genesis {
            balances: {
                let mut balances = BTreeMap::new();
                // Alice.
                balances.insert(keys::alice::address(), {
                    let mut denominations = BTreeMap::new();
                    denominations.insert(Denomination::NATIVE, 1_234_567);
                    denominations
                });
                balances
            },
            total_supplies: {
                let mut total_supplies = BTreeMap::new();
                total_supplies.insert(Denomination::NATIVE, 1_234_567);
                total_supplies
            },
            parameters: modules::accounts::Parameters {
                gas_costs: modules::accounts::GasCosts { tx_transfer: 1_000 },
                ..Default::default()
            },
            ..Default::default()
        },
    );

    // Run the state migration.
    Runtime::migrate_state(&mut ctx);

    // Check Alice account balances.
    let bals = Accounts::get_balances(ctx.runtime_state(), keys::alice::address())
        .expect("get_balances should succeed");
    assert_eq!(
        bals.balances[&Denomination::NATIVE],
        1_234_567_000_000_000,
        "balance in account should be correctly scaled"
    );
    assert_eq!(
        bals.balances.len(),
        1,
        "there should only be one denomination"
    );

    // Check total supply.
    let total_supplies = Accounts::get_total_supplies(ctx.runtime_state()).unwrap();
    let total_supply = total_supplies.get(&Denomination::NATIVE).unwrap();
    assert_eq!(
        *total_supply, 1_234_567_000_000_000,
        "total supply should be correct"
    );
}
