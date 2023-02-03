// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use parity_scale_codec::{Decode, Encode};
use sp_std::result::Result;
use xcm::latest::{Instruction, MultiLocation, Weight};

/// The reason why a given XCM is not permitted to execute.
#[derive(Clone, Debug, PartialEq, Eq, Decode, Encode)]
pub enum RejectReason {
	/// The supplied weight credit is not enough to pay for the weight required.
	InsufficientCredit,
	/// None of the barrier rules in a tuple matched successfully.
	NoRulesMatched,
	/// The series of `DescendOrigin` instructions would overflow the origin MultiLocation.
	OriginMultiLocationTooLong,
	/// Does not pass the barrier yet, but might pass in a later block.
	Suspended,
	/// Barrier does not recognize the incoming XCM format.
	UnexpectedMessageFormat,
	/// The receiving chain did not expect a response XCM from the origin.
	UnexpectedResponse,
	/// The origin is not trusted to pass this barrier.
	UntrustedOrigin,
	/// The incoming XCM contains a weight limit that is lower than the required weight, as weighed
	/// by the receiver.
	WeightLimitTooLow,
}

/// Trait to determine whether the execution engine should actually execute a given XCM.
///
/// Can be amalgamated into a tuple to have multiple trials. If any of the tuple elements returns `Ok()`, the
/// execution stops. Else, `Err(_)` is returned if all elements reject the message.
pub trait ShouldExecute {
	/// Returns `true` if the given `message` may be executed.
	///
	/// - `origin`: The origin (sender) of the message.
	/// - `instructions`: The message itself.
	/// - `max_weight`: The (possibly over-) estimation of the weight of execution of the message.
	/// - `weight_credit`: The pre-established amount of weight that the system has determined this
	///   message may utilize in its execution. Typically non-zero only because of prior fee
	///   payment, but could in principle be due to other factors.
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> Result<(), RejectReason>;
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl ShouldExecute for Tuple {
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> Result<(), RejectReason> {
		for_tuples!( #(
			match Tuple::should_execute(origin, instructions, max_weight, weight_credit) {
				Ok(()) => return Ok(()),
				Err(e) => log::trace!(
					target: "xcm::should_execute",
					"barrier rule rejected message with reason: {:?}", e
				),
			};
		)* );
		log::trace!(
			target: "xcm::should_execute",
			"did not pass any barrier rules: origin: {:?}, instructions: {:?}, max_weight: {:?}, weight_credit: {:?}",
			origin,
			instructions,
			max_weight,
			weight_credit,
		);
		Err(RejectReason::NoRulesMatched)
	}
}

/// Trait to determine whether the execution engine is suspended from executing a given XCM.
///
/// The trait method is given the same parameters as `ShouldExecute::should_execute`, so that the
/// implementer will have all the context necessary to determine whether or not to suspend the
/// XCM executor.
///
/// Can be chained together in tuples to have multiple rounds of checks. If all of the tuple
/// elements returns false, then execution is not suspended. Otherwise, execution is suspended
/// if any of the tuple elements returns true.
pub trait CheckSuspension {
	fn is_suspended<Call>(
		origin: &MultiLocation,
		instructions: &mut [Instruction<Call>],
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> bool;
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl CheckSuspension for Tuple {
	fn is_suspended<Call>(
		origin: &MultiLocation,
		instruction: &mut [Instruction<Call>],
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> bool {
		for_tuples!( #(
			if Tuple::is_suspended(origin, instruction, max_weight, weight_credit) {
				return true
			}
		)* );

		false
	}
}
