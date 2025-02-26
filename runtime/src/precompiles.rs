use pallet_evm::{AddressMapping, Context, Precompile, PrecompileResult, PrecompileSet};
use sp_core::H160;
use sp_std::marker::PhantomData;

use pallet_evm_precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_octopus_appchain::OctopusAppchainWrapper;
use pallet_evm_precompile_octopus_session::OctopusSessionWrapper;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

pub struct NativeErc20Metadata;

/// ERC20 metadata for the native token.
impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"OCTAN token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"OCTAN"
	}

	/// Returns the decimals places of the token.
	fn decimals() -> u8 {
		18
	}

	/// Must return `true` only if it represents the main native currency of
	/// the network. It must be the currency used in `pallet_evm`.
	fn is_native_currency() -> bool {
		true
	}
}

/// The PrecompileSet installed in the Moonbeam runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
#[derive(Debug, Clone, Copy)]
pub struct FrontierPrecompiles<R>(PhantomData<R>);

impl<R> FrontierPrecompiles<R>
where
	R: pallet_evm::Config,
{
	pub fn new() -> Self {
		Self(Default::default())
	}
	/// Return all addresses that contain precompiles. This can be used to populate dummy code
	/// under the precompile.
	pub fn used_addresses() -> impl Iterator<Item = R::AccountId> {
		sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 2050]
			.into_iter()
			.map(|x| R::AddressMapping::into_account_id(hash(x)))
	}
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Moonbeam specific
/// 2048-4095 Moonbeam specific precompiles
impl<R> PrecompileSet for FrontierPrecompiles<R>
where
	Dispatch<R>: Precompile,
	Erc20BalancesPrecompile<R, NativeErc20Metadata>: Precompile,
	OctopusAppchainWrapper<R>: Precompile,
	OctopusSessionWrapper<R>: Precompile,
	R: pallet_evm::Config,
{
	fn execute(
		&self,
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<PrecompileResult> {
		match address {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(input, target_gas, context, is_static)),
			a if a == hash(2) => Some(Sha256::execute(input, target_gas, context, is_static)),
			a if a == hash(3) => Some(Ripemd160::execute(input, target_gas, context, is_static)),
			a if a == hash(4) => Some(Identity::execute(input, target_gas, context, is_static)),
			a if a == hash(5) => Some(Modexp::execute(input, target_gas, context, is_static)),
			a if a == hash(6) => Some(Bn128Add::execute(input, target_gas, context, is_static)),
			a if a == hash(7) => Some(Bn128Mul::execute(input, target_gas, context, is_static)),
			a if a == hash(8) => Some(Bn128Pairing::execute(input, target_gas, context, is_static)),
			a if a == hash(9) => Some(Blake2F::execute(input, target_gas, context, is_static)),
			// Non-Moonbeam specific nor Ethereum precompiles :
			a if a == hash(1024) =>
				Some(Sha3FIPS256::execute(input, target_gas, context, is_static)),
			a if a == hash(1025) =>
				Some(Dispatch::<R>::execute(input, target_gas, context, is_static)),
			a if a == hash(1026) =>
				Some(ECRecoverPublicKey::execute(input, target_gas, context, is_static)),
			a if a == hash(2050) =>
				Some(Erc20BalancesPrecompile::<R, NativeErc20Metadata>::execute(
					input, target_gas, context, is_static,
				)),
			a if a == hash(2051) =>
				Some(OctopusAppchainWrapper::<R>::execute(input, target_gas, context, is_static)),
			a if a == hash(2052) =>
				Some(OctopusSessionWrapper::<R>::execute(input, target_gas, context, is_static)),
			_ => None,
		}
	}
	fn is_precompile(&self, address: H160) -> bool {
		Self::used_addresses().any(|x| x == R::AddressMapping::into_account_id(address))
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
