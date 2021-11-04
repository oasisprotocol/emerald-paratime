//! The Emerald ParaTime.
use std::collections::BTreeMap;

use oasis_runtime_sdk::{
    self as sdk,
    module::InvariantHandler as _,
    modules,
    modules::accounts::API as _,
    types::token::{BaseUnits, Denomination},
    Module, Version,
};

#[cfg(test)]
mod test;

/// Configuration of the various modules.
pub struct Config;

impl module_evm::Config for Config {
    type Accounts = modules::accounts::Module;

    const CHAIN_ID: u64 = 0xa515;

    const TOKEN_DENOMINATION: Denomination = Denomination::NATIVE;
}

/// The EVM ParaTime.
pub struct Runtime;

impl sdk::Runtime for Runtime {
    /// Version of the runtime.
    const VERSION: Version = sdk::version_from_cargo!();
    /// Current version of the global state (e.g. parameters). Any parameter updates should bump
    /// this version in order for the migrations to be executed.
    const STATE_VERSION: u32 = 1;

    type Modules = (
        // Core.
        modules::core::Module,
        // Accounts.
        modules::accounts::Module,
        // Consensus layer interface.
        modules::consensus::Module,
        // Consensus layer accounts.
        modules::consensus_accounts::Module<modules::accounts::Module, modules::consensus::Module>,
        // Rewards.
        modules::rewards::Module<modules::accounts::Module>,
        // EVM.
        module_evm::Module<Config>,
    );

    fn genesis_state() -> <Self::Modules as sdk::module::MigrationHandler>::Genesis {
        (
            modules::core::Genesis {
                parameters: modules::core::Parameters {
                    min_gas_price: {
                        let mut mgp = BTreeMap::new();
                        mgp.insert(Denomination::NATIVE, 0);
                        mgp
                    },
                    max_batch_gas: 10_000_000,
                    max_tx_signers: 1,
                    max_multisig_signers: 8,
                    gas_costs: modules::core::GasCosts {
                        tx_byte: 1,
                        auth_signature: 1_000,
                        auth_multisig_signer: 1_000,
                        callformat_x25519_deoxysii: 10_000,
                    },
                },
            },
            modules::accounts::Genesis {
                parameters: modules::accounts::Parameters {
                    gas_costs: modules::accounts::GasCosts { tx_transfer: 1_000 },
                    denomination_infos: {
                        let mut denomination_infos = BTreeMap::new();
                        denomination_infos.insert(
                            Denomination::NATIVE,
                            modules::accounts::types::DenominationInfo {
                                // Consistent with EVM ecosystem.
                                decimals: 18,
                            },
                        );
                        denomination_infos
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            modules::consensus::Genesis {
                parameters: modules::consensus::Parameters {
                    // Consensus layer denomination is the native denomination of this runtime.
                    consensus_denomination: Denomination::NATIVE,
                    // Scale to 18 decimal places as this is what is expected in the EVM ecosystem.
                    consensus_scaling_factor: 1_000_000_000,
                },
            },
            modules::consensus_accounts::Genesis {
                parameters: modules::consensus_accounts::Parameters {
                    gas_costs: modules::consensus_accounts::GasCosts {
                        tx_deposit: 10_000,
                        tx_withdraw: 10_000,
                    },
                },
            },
            modules::rewards::Genesis {
                parameters: modules::rewards::Parameters {
                    schedule: modules::rewards::types::RewardSchedule {
                        // TODO: Define propoer reward schedule.
                        steps: vec![modules::rewards::types::RewardStep {
                            until: 26_700,
                            amount: BaseUnits::new(1_000_000_000_000_000_000, Denomination::NATIVE),
                        }],
                    },
                    participation_threshold_numerator: 3,
                    participation_threshold_denominator: 4,
                },
            },
            module_evm::Genesis {
                parameters: module_evm::Parameters {
                    gas_costs: module_evm::GasCosts {},
                },
            },
        )
    }

    fn migrate_state<C: sdk::Context>(ctx: &mut C) {
        // State migration from by copying over parameters from updated genesis state.
        let genesis = Self::genesis_state();

        // Determine configured scaling factor for migration below.
        let scaling_factor = genesis.2.parameters.consensus_scaling_factor.into();

        // Accounts.
        modules::accounts::Module::set_params(ctx.runtime_state(), genesis.1.parameters);
        // Consensus.
        modules::consensus::Module::set_params(ctx.runtime_state(), genesis.2.parameters);
        // Rewards.
        modules::rewards::Module::<modules::accounts::Module>::set_params(
            ctx.runtime_state(),
            genesis.4.parameters,
        );

        // Migrate accounts such that all base units for the native denomination are scaled.
        let mut new_total_supply = 0u128;
        for address in
            modules::accounts::Module::get_addresses(ctx.runtime_state(), Denomination::NATIVE)
                .unwrap()
        {
            // Fetch original balance for the account.
            let amount = modules::accounts::Module::get_balance(
                ctx.runtime_state(),
                address,
                Denomination::NATIVE,
            )
            .unwrap();
            // Multiply it by 10^9.
            let amount = amount.checked_mul(scaling_factor).unwrap();
            modules::accounts::Module::set_balance(
                ctx.runtime_state(),
                address,
                &BaseUnits::new(amount, Denomination::NATIVE),
            );
            // Compute new total supply as a sanity check.
            new_total_supply = new_total_supply.checked_add(amount).unwrap();
        }

        if new_total_supply > 0 {
            // Update total supply.
            let total_supplies =
                modules::accounts::Module::get_total_supplies(ctx.runtime_state()).unwrap();
            let total_supply = total_supplies.get(&Denomination::NATIVE).unwrap();
            let total_supply = total_supply.checked_mul(scaling_factor).unwrap();
            // Make sure that both supplies match.
            assert!(total_supply == new_total_supply);
            // Update total supply.
            modules::accounts::Module::set_total_supply(
                ctx.runtime_state(),
                &BaseUnits::new(total_supply, Denomination::NATIVE),
            );
        }

        // Run invariant check after migration.
        Self::Modules::check_invariants(ctx).unwrap();
    }
}
