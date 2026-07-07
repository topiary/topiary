(*****************************************************************************)
(*                                                                           *)
(* Open Source License                                                       *)
(* Copyright (c) 2018 Dynamic Ledger Solutions, Inc. <contact@tezos.com>     *)
(* Copyright (c) 2019-2022 Nomadic Labs <contact@nomadic-labs.com>           *)
(* Copyright (c) 2022 TriliTech <contact@trili.tech>                         *)
(* Copyright (c) 2022 DaiLambda, Inc. <contact@dailambda,jp>                 *)
(*                                                                           *)
(* Permission is hereby granted, free of charge, to any person obtaining a   *)
(* copy of this software and associated documentation files (the "Software"),*)
(* to deal in the Software without restriction, including without limitation *)
(* the rights to use, copy, modify, merge, publish, distribute, sublicense,  *)
(* and/or sell copies of the Software, and to permit persons to whom the     *)
(* Software is furnished to do so, subject to the following conditions:      *)
(*                                                                           *)
(* The above copyright notice and this permission notice shall be included   *)
(* in all copies or substantial portions of the Software.                    *)
(*                                                                           *)
(* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR*)
(* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,  *)
(* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL   *)
(* THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER*)
(* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING   *)
(* FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER       *)
(* DEALINGS IN THE SOFTWARE.                                                 *)
(*                                                                           *)
(*****************************************************************************)

(** An [Alpha_context.t] is an immutable snapshot of the ledger state at some block
    height, preserving
    {{:https://tezos.gitlab.io/developer/entering_alpha.html#the-big-abstraction-barrier-alpha-context}
    type-safety and invariants} of the ledger state.

    {2 Implementation}

    [Alpha_context.t] is a wrapper over [Raw_context.t], which in turn is a
    wrapper around [Context.t] from the Protocol Environment.

    {2 Lifetime of an Alpha_context}

    - Creation, using [prepare] or [prepare_first_block]

    - Modification, using the operations defined in this signature

    - Finalization, using [finalize]
 *)

module type BASIC_DATA = sig
  type t

  include Compare.S with type t := t

  val encoding : t Data_encoding.t

  val pp : Format.formatter -> t -> unit
end

type t

type context = t

type public_key = Signature.Public_key.t

type public_key_hash = Signature.Public_key_hash.t

type signature = Signature.t

(** This module re-exports definitions from {!Slot_repr}. *)
module Slot : sig
  type t

  type slot = t

  include Compare.S with type t := t

  val pp : Format.formatter -> t -> unit

  val zero : t

  val succ : t -> t tzresult

  val of_int_do_not_use_except_for_parameters : int -> t

  val encoding : t Data_encoding.encoding

  module Range : sig
    type t

    val create : min: int -> count: int -> t tzresult

    val fold : ('a -> slot -> 'a) -> 'a -> t -> 'a

    val fold_es :
      ('a -> slot -> 'a tzresult Lwt.t) -> 'a -> t -> 'a tzresult Lwt.t

    val rev_fold_es :
      ('a -> slot -> 'a tzresult Lwt.t) -> 'a -> t -> 'a tzresult Lwt.t
  end

  module Map : Map.S with type key = t

  module Set : Set.S with type elt = t

  module Internal_for_tests : sig
    val of_int : int -> t tzresult
  end
end

(** This module re-exports definitions from {!Tez_repr}. *)
module Tez : sig
  type repr

  type t = Tez_tag of repr [@@ocaml.unboxed]

  include BASIC_DATA with type t := t

  type tez = t

  val zero : tez

  val one_mutez : tez

  val one_cent : tez

  val fifty_cents : tez

  val one : tez

  val (-?): tez -> tez -> tez tzresult

  val sub_opt : tez -> tez -> tez option

  val (+?): tez -> tez -> tez tzresult

  val ( *? ): tez -> int64 -> tez tzresult

  val ( /? ): tez -> int64 -> tez tzresult

  val of_string : string -> tez option

  val to_string : tez -> string

  val of_mutez : int64 -> tez option

  val to_mutez : tez -> int64

  val of_mutez_exn : int64 -> t

  val mul_exn : t -> int -> t

  val div_exn : t -> int -> t
end

(** This module re-exports definitions from {!Period_repr}. *)
module Period : sig
  include BASIC_DATA

  type period = t

  val rpc_arg : period RPC_arg.arg

  val of_seconds : int64 -> period tzresult

  val of_seconds_exn : int64 -> period

  val to_seconds : period -> int64

  val add : period -> period -> period tzresult

  val mult : int32 -> period -> period tzresult

  val zero : period

  val one_second : period

  val one_minute : period

  val one_hour : period

  val compare : period -> period -> int
end

(** This module re-exports definitions from {!Time_repr}. *)
module Timestamp : sig
  include BASIC_DATA with type t = Time.t

  type time = t

  val (+?): time -> Period.t -> time tzresult

  val (-?): time -> time -> Period.t tzresult

  val (-): time -> Period.t -> time

  val of_notation : string -> time option

  val to_notation : time -> string

  val of_seconds : int64 -> time

  val to_seconds : time -> int64

  val of_seconds_string : string -> time option

  val to_seconds_string : time -> string

  (** See {!Raw_context.current_timestamp}. *)
  val current : context -> time

  (** See {!Raw_context.predecessor_timestamp}. *)
  val predecessor : context -> time
end

(** This module re-exports definitions from {!Ratio_repr}. *)
module Ratio : sig
  type t = {numerator: int; denominator: int}

  val encoding : t Data_encoding.t

  val pp : Format.formatter -> t -> unit
end

(** This module re-exports definitions from {!Raw_level_repr}. *)
module Raw_level : sig
  include BASIC_DATA

  type raw_level = t

  val rpc_arg : raw_level RPC_arg.arg

  val diff : raw_level -> raw_level -> int32

  val root : raw_level

  val succ : raw_level -> raw_level

  val pred : raw_level -> raw_level option

  val to_int32 : raw_level -> int32

  val of_int32 : int32 -> raw_level tzresult

  val of_int32_exn : int32 -> raw_level

  module Set : Set.S with type elt = raw_level

  module Map : Map.S with type key = raw_level

  module Internal_for_tests : sig
    val add : raw_level -> int -> raw_level

    val sub : raw_level -> int -> raw_level option
  end
end

(** This module re-exports definitions from {!Cycle_repr}. *)
module Cycle : sig
  include BASIC_DATA

  type cycle = t

  val rpc_arg : cycle RPC_arg.arg

  val root : cycle

  val succ : cycle -> cycle

  val pred : cycle -> cycle option

  val add : cycle -> int -> cycle

  val sub : cycle -> int -> cycle option

  val to_int32 : cycle -> int32

  module Map : Map.S with type key = cycle
end

(** This module re-exports definitions from {!Round_repr}. *)
module Round : sig
  (* A round represents an iteration of the single-shot consensus algorithm.
     This mostly simply re-exports [Round_repr]. See [Round_repr] for
     additional documentation of this module *)

  type t

  val zero : t

  val succ : t -> t

  val pred : t -> t tzresult

  val to_int32 : t -> int32

  val of_int32 : int32 -> t tzresult

  val of_int : int -> t tzresult

  val to_int : t -> int tzresult

  val to_slot : t -> committee_size: int -> Slot.t tzresult

  val pp : Format.formatter -> t -> unit

  val encoding : t Data_encoding.t

  include Compare.S with type t := t

  module Map : Map.S with type key = t

  (** See {!Round_repr.Durations.t}. *)
  type round_durations

  (** See {!Round_repr.Durations.pp}. *)
  val pp_round_durations : Format.formatter -> round_durations -> unit

  (** See {!Round_repr.Durations.encoding}. *)
  val round_durations_encoding : round_durations Data_encoding.t

  (** See {!Round_repr.Durations.round_duration}. *)
  val round_duration : round_durations -> t -> Period.t

  module Durations : sig
    val create :
      first_round_duration: Period.t ->
      delay_increment_per_round: Period.t ->
      round_durations tzresult

    val create_opt :
      first_round_duration: Period.t ->
      delay_increment_per_round: Period.t ->
      round_durations option
  end

  val level_offset_of_round : round_durations -> round: t -> Period.t tzresult

  val timestamp_of_round :
    round_durations ->
    predecessor_timestamp: Time.t ->
    predecessor_round: t ->
    round: t ->
    Time.t tzresult

  val timestamp_of_another_round_same_level :
    round_durations ->
    current_timestamp: Time.t ->
    current_round: t ->
    considered_round: t ->
    Time.t tzresult

  val round_of_timestamp :
    round_durations ->
    predecessor_timestamp: Time.t ->
    predecessor_round: t ->
    timestamp: Time.t ->
    t tzresult

  (* retrieve a round from the context *)
  val get : context -> t tzresult Lwt.t

  (* store a round in context *)
  val update : context -> t -> context tzresult Lwt.t
end

module Gas : sig
  (** This module implements the gas subsystem of the context.

     Gas reflects the computational cost of each operation to limit
     the cost of operations and, by extension, the cost of blocks.

     There are two gas quotas: one for operation and one for
     block. For this reason, we maintain two gas levels -- one for
     operations and another one for blocks -- that correspond to the
     remaining amounts of gas, initialized with the quota
     limits and decreased each time gas is consumed.

  *)

  module Arith :
    Fixed_point_repr.Safe with
    type 'a t = private Saturation_repr.may_saturate Saturation_repr.t

  (** For maintenance operations or for testing, gas can be
     [Unaccounted]. Otherwise, the computation is [Limited] by the
     [remaining] gas in the context. *)
  type t = private Unaccounted | Limited of {remaining: Arith.fp}

  val encoding : t Data_encoding.encoding

  val pp : Format.formatter -> t -> unit

  (** [set_limit ctxt limit] returns a context with a given
     [limit] level of gas allocated for an operation. *)
  val set_limit : context -> 'a Arith.t -> context

  (** [set_unlimited] allows unlimited gas consumption. *)
  val set_unlimited : context -> context

  (** [remaining_operation_gas ctxt] returns the current gas level in
     the context [ctxt] for the current operation. If gas is
     [Unaccounted], an arbitrary value will be returned. *)
  val remaining_operation_gas : context -> Arith.fp

  (** [reset_block_gas ctxt] returns a context where the remaining gas
     in the block is reset to the constant [hard_gas_limit_per_block],
     i.e., as if no operations have been included in the block.

     /!\ Do not call this function unless you want to validate
     operations on their own (like in the mempool). *)
  val reset_block_gas : context -> context

  (** [level ctxt] is the current gas level in [ctxt] for the current
     operation. *)
  val level : context -> t

  (** [update_remaining_operation_gas ctxt remaining] sets the current
     gas level for operations to [remaining]. *)
  val update_remaining_operation_gas : context -> Arith.fp -> context

  (** [consumed since until] is the operation gas level difference
     between context [since] and context [until]. This function
     returns [Arith.zero] if any of the two contexts allows for an
     unlimited gas consumption. This function also returns
     [Arith.zero] if [since] has less gas than [until]. *)
  val consumed : since: context -> until: context -> Arith.fp

  (** [block_level ctxt] returns the block gas level in context [ctxt]. *)
  val block_level : context -> Arith.fp

  (** Costs are computed using a saturating arithmetic. See
     {!Saturation_repr}. *)
  type cost = Saturation_repr.may_saturate Saturation_repr.t

  val cost_encoding : cost Data_encoding.encoding

  val pp_cost : Format.formatter -> cost -> unit

  val pp_cost_as_gas : Format.formatter -> cost -> unit

  type error += Operation_quota_exceeded
  (* `Temporary *)

  (** [consume ctxt cost] subtracts [cost] to the current operation
     gas level in [ctxt]. This operation may fail with
     [Operation_quota_exceeded] if the operation gas level would
     go below zero. *)
  val consume : context -> cost -> context tzresult

  (** [consume_from available_gas cost] subtracts [cost] from
      [available_gas] and returns the remaining gas.

      @return [Error Operation_quota_exceeded] if the remaining gas
      would fall below [0]. *)
  val consume_from : Arith.fp -> cost -> Arith.fp tzresult

  type error += Block_quota_exceeded
  (* `Temporary *)

  type error += Gas_limit_too_high
  (* `Permanent *)

  (** See {!Raw_context.consume_gas_limit_in_block}. *)
  val consume_limit_in_block : context -> 'a Arith.t -> context tzresult

  (** Check that [gas_limit] is a valid operation gas limit: at most
      [hard_gas_limit_per_operation] and nonnegative.

      @return [Error Gas_limit_too_high] if [gas_limit] is greater
      than [hard_gas_limit_per_operation] or negative. *)
  val check_gas_limit :
    hard_gas_limit_per_operation: Arith.integral ->
    gas_limit: Arith.integral ->
    unit tzresult

  (** The cost of free operation is [0]. *)
  val free : cost

  (** Convert a fixed-point amount of gas to a cost. *)
  val cost_of_gas : 'a Arith.t -> cost

  (** Convert an amount of milligas expressed as an int to Arith.fp.  *)
  val fp_of_milligas_int : int -> Arith.fp

  (** [atomic_step_cost x] corresponds to [x] milliunit of gas. *)
  val atomic_step_cost : _ Saturation_repr.t -> cost

  (** [step_cost x] corresponds to [x] units of gas. *)
  val step_cost : _ Saturation_repr.t -> cost

  (** Cost of allocating qwords of storage.
    [alloc_cost n] estimates the cost of allocating [n] qwords of storage. *)
  val alloc_cost : _ Saturation_repr.t -> cost

  (** Cost of allocating bytes in the storage.
    [alloc_bytes_cost b] estimates the cost of allocating [b] bytes of
    storage. *)
  val alloc_bytes_cost : int -> cost

  (** Cost of allocating bytes in the storage.

      [alloc_mbytes_cost b] estimates the cost of allocating [b] bytes of
      storage and the cost of an header to describe these bytes. *)
  val alloc_mbytes_cost : int -> cost

  (** Cost of reading the storage.
    [read_bytes_cost n] estimates the cost of reading [n] bytes of storage. *)
  val read_bytes_cost : int -> cost

  (** Cost of writing to storage.
    [write_bytes_const n] estimates the cost of writing [n] bytes to the
    storage. *)
  val write_bytes_cost : int -> cost

  (** Multiply a cost by a factor. Both arguments are saturated arithmetic values,
    so no negative numbers are involved. *)
  val ( *@ ): _ Saturation_repr.t -> cost -> cost

  (** Add two costs together. *)
  val (+@): cost -> cost -> cost

  (** [cost_of_repr] is an internal operation needed to inject costs
     for Storage_costs into Gas.cost. *)
  val cost_of_repr : Gas_limit_repr.cost -> cost
end

module Entrypoint : module type of Entrypoint_repr

(** This module re-exports definitions from {!Script_repr} and
    {!Michelson_v1_primitives}. *)
module Script : sig
  type error += Lazy_script_decode

  type prim = Michelson_v1_primitives.prim =
    | K_parameter
    | K_storage
    | K_code
    | K_view
    | D_False
    | D_Elt
    | D_Left
    | D_None
    | D_Pair
    | D_Right
    | D_Some
    | D_True
    | D_Unit
    | D_Lambda_rec
    | I_PACK
    | I_UNPACK
    | I_BLAKE2B
    | I_SHA256
    | I_SHA512
    | I_ABS
    | I_ADD
    | I_AMOUNT
    | I_AND
    | I_BALANCE
    | I_CAR
    | I_CDR
    | I_CHAIN_ID
    | I_CHECK_SIGNATURE
    | I_COMPARE
    | I_CONCAT
    | I_CONS
    | I_CREATE_ACCOUNT
    | I_CREATE_CONTRACT
    | I_IMPLICIT_ACCOUNT
    | I_DIP
    | I_DROP
    | I_DUP
    | I_VIEW
    | I_EDIV
    | I_EMPTY_BIG_MAP
    | I_EMPTY_MAP
    | I_EMPTY_SET
    | I_EQ
    | I_EXEC
    | I_APPLY
    | I_FAILWITH
    | I_GE
    | I_GET
    | I_GET_AND_UPDATE
    | I_GT
    | I_HASH_KEY
    | I_IF
    | I_IF_CONS
    | I_IF_LEFT
    | I_IF_NONE
    | I_INT
    | I_LAMBDA
    | I_LAMBDA_REC
    | I_LE
    | I_LEFT
    | I_LEVEL
    | I_LOOP
    | I_LSL
    | I_LSR
    | I_LT
    | I_MAP
    | I_MEM
    | I_MUL
    | I_NEG
    | I_NEQ
    | I_NIL
    | I_NONE
    | I_NOT
    | I_NOW
    | I_MIN_BLOCK_TIME
    | I_OR
    | I_PAIR
    | I_UNPAIR
    | I_PUSH
    | I_RIGHT
    | I_SIZE
    | I_SOME
    | I_SOURCE
    | I_SENDER
    | I_SELF
    | I_SELF_ADDRESS
    | I_SLICE
    | I_STEPS_TO_QUOTA
    | I_SUB
    | I_SUB_MUTEZ
    | I_SWAP
    | I_TRANSFER_TOKENS
    | I_SET_DELEGATE
    | I_UNIT
    | I_UPDATE
    | I_XOR
    | I_ITER
    | I_LOOP_LEFT
    | I_ADDRESS
    | I_CONTRACT
    | I_ISNAT
    | I_CAST
    | I_RENAME
    | I_SAPLING_EMPTY_STATE
    | I_SAPLING_VERIFY_UPDATE
    | I_DIG
    | I_DUG
    | I_NEVER
    | I_VOTING_POWER
    | I_TOTAL_VOTING_POWER
    | I_KECCAK
    | I_SHA3
    | I_PAIRING_CHECK
    | I_TICKET
    | I_TICKET_DEPRECATED
    | I_READ_TICKET
    | I_SPLIT_TICKET
    | I_JOIN_TICKETS
    | I_OPEN_CHEST
    | I_EMIT
    | I_BYTES
    | I_NAT
    | T_bool
    | T_contract
    | T_int
    | T_key
    | T_key_hash
    | T_lambda
    | T_list
    | T_map
    | T_big_map
    | T_nat
    | T_option
    | T_or
    | T_pair
    | T_set
    | T_signature
    | T_string
    | T_bytes
    | T_mutez
    | T_timestamp
    | T_unit
    | T_operation
    | T_address
    | T_tx_rollup_l2_address
    | T_sapling_transaction
    | T_sapling_transaction_deprecated
    | T_sapling_state
    | T_chain_id
    | T_never
    | T_bls12_381_g1
    | T_bls12_381_g2
    | T_bls12_381_fr
    | T_ticket
    | T_chest_key
    | T_chest
    | H_constant

  type location = Micheline.canonical_location

  type annot = Micheline.annot

  type expr = prim Micheline.canonical

  type lazy_expr = expr Data_encoding.lazy_t

  val lazy_expr : expr -> lazy_expr

  type 'location michelson_node = ('location, prim) Micheline.node

  type node = location michelson_node

  type t = {code: lazy_expr; storage: lazy_expr}

  val location_encoding : location Data_encoding.t

  val expr_encoding : expr Data_encoding.t

  val prim_encoding : prim Data_encoding.t

  val encoding : t Data_encoding.t

  val lazy_expr_encoding : lazy_expr Data_encoding.t

  val deserialization_cost_estimated_from_bytes : int -> Gas.cost

  val deserialized_cost : expr -> Gas.cost

  val micheline_serialization_cost : expr -> Gas.cost

  val bytes_node_cost : bytes -> Gas.cost

  (** Mode of deserialization gas consumption in {!force_decode}:

      - {!Always}: the gas is taken independently of the internal state of the
        [lazy_expr]
      - {!When_needed}: the gas is consumed only if the [lazy_expr] has never
        been deserialized before. *)
  type consume_deserialization_gas = Always | When_needed

  (** Decode an expression in the context after consuming the deserialization
      gas cost (see {!consume_deserialization_gas}). *)
  val force_decode_in_context :
    consume_deserialization_gas: consume_deserialization_gas ->
    context ->
    lazy_expr ->
    (expr * context) tzresult

  (** Decode an expression in the context after consuming the deserialization
      gas cost. *)
  val force_bytes_in_context :
    context -> lazy_expr -> (bytes * context) tzresult

  (** [consume_decoding_gas available_gas lexpr] subtracts (a lower
      bound on) the cost to deserialize [lexpr] from [available_gas].
      The cost does not depend on the internal state of the lazy_expr.

      @return [Error Operation_quota_exceeded] if the remaining gas
      would fall below [0].

      This mimics the gas consuming part of {!force_decode_in_context}
      called with [consume_deserialization_gas:Always]. *)
  val consume_decoding_gas : Gas.Arith.fp -> lazy_expr -> Gas.Arith.fp tzresult

  val unit_parameter : lazy_expr

  val strip_locations_cost : _ michelson_node -> Gas.cost

  val strip_annotations_cost : node -> Gas.cost

  val strip_annotations : node -> node
end

(** This module re-exports definitions from {!Constants_repr} and
    {!Constants_storage}. *)
module Constants : sig
  (** Fixed constants *)
  type fixed

  val fixed_encoding : fixed Data_encoding.t

  val mainnet_id : Chain_id.t

  val proof_of_work_nonce_size : int

  val nonce_length : int

  val max_anon_ops_per_block : int

  val max_operation_data_length : int

  val max_proposals_per_delegate : int

  val michelson_maximum_type_size : int

  val sc_rollup_message_size_limit : int

  val sc_rollup_max_number_of_messages_per_level : Z.t

  (** Constants parameterized by context. See {!Constants_parametric_repr}. *)
  module Parametric : sig
    type dal = {
      feature_enable: bool;
      number_of_slots: int;
      attestation_lag: int;
      availability_threshold: int;
      blocks_per_epoch: int32;
      cryptobox_parameters: Dal.parameters;
    }

    val dal_encoding : dal Data_encoding.t

    type tx_rollup = {
      enable: bool;
      origination_size: int;
      hard_size_limit_per_inbox: int;
      hard_size_limit_per_message: int;
      commitment_bond: Tez.t;
      finality_period: int;
      withdraw_period: int;
      max_inboxes_count: int;
      max_messages_per_inbox: int;
      max_commitments_count: int;
      cost_per_byte_ema_factor: int;
      max_ticket_payload_size: int;
      max_withdrawals_per_batch: int;
      rejection_max_proof_size: int;
      sunset_level: int32;
    }

    type sc_rollup = {
      enable: bool;
      arith_pvm_enable: bool;
      origination_size: int;
      challenge_window_in_blocks: int;
      stake_amount: Tez.t;
      commitment_period_in_blocks: int;
      max_lookahead_in_blocks: int32;
      max_active_outbox_levels: int32;
      max_outbox_messages_per_level: int;
      number_of_sections_in_dissection: int;
      timeout_period_in_blocks: int;
      max_number_of_stored_cemented_commitments: int;
      max_number_of_parallel_games: int;
    }

    type zk_rollup = {
      enable: bool;
      origination_size: int;
      min_pending_to_process: int;
    }

    type t = {
      preserved_cycles: int;
      blocks_per_cycle: int32;
      blocks_per_commitment: int32;
      nonce_revelation_threshold: int32;
      blocks_per_stake_snapshot: int32;
      cycles_per_voting_period: int32;
      hard_gas_limit_per_operation: Gas.Arith.integral;
      hard_gas_limit_per_block: Gas.Arith.integral;
      proof_of_work_threshold: int64;
      minimal_stake: Tez.t;
      vdf_difficulty: int64;
      seed_nonce_revelation_tip: Tez.t;
      origination_size: int;
      baking_reward_fixed_portion: Tez.t;
      baking_reward_bonus_per_slot: Tez.t;
      endorsing_reward_per_slot: Tez.t;
      cost_per_byte: Tez.t;
      hard_storage_limit_per_operation: Z.t;
      quorum_min: int32;
      quorum_max: int32;
      min_proposal_quorum: int32;
      liquidity_baking_subsidy: Tez.t;
      liquidity_baking_toggle_ema_threshold: int32;
      max_operations_time_to_live: int;
      minimal_block_delay: Period.t;
      delay_increment_per_round: Period.t;
      minimal_participation_ratio: Ratio.t;
      consensus_committee_size: int;
      consensus_threshold: int;
      max_slashing_period: int;
      frozen_deposits_percentage: int;
      double_baking_punishment: Tez.t;
      ratio_of_frozen_deposits_slashed_per_double_endorsement: Ratio.t;
      testnet_dictator: public_key_hash option;
      initial_seed: State_hash.t option;
      cache_script_size: int;
      cache_stake_distribution_cycles: int;
      cache_sampler_state_cycles: int;
      tx_rollup: tx_rollup;
      dal: dal;
      sc_rollup: sc_rollup;
      zk_rollup: zk_rollup;
    }

    val encoding : t Data_encoding.t
  end

  module Generated : sig
    type t = {
      consensus_threshold: int;
      baking_reward_fixed_portion: Tez.t;
      baking_reward_bonus_per_slot: Tez.t;
      endorsing_reward_per_slot: Tez.t;
      liquidity_baking_subsidy: Tez.t;
    }

    val generate :
      consensus_committee_size: int -> blocks_per_minute: Ratio.t -> t
  end

  val parametric : context -> Parametric.t

  val tx_rollup : context -> Parametric.tx_rollup

  val sc_rollup : context -> Parametric.sc_rollup

  val preserved_cycles : context -> int

  val blocks_per_cycle : context -> int32

  val blocks_per_commitment : context -> int32

  val nonce_revelation_threshold : context -> int32

  val blocks_per_stake_snapshot : context -> int32

  val cycles_per_voting_period : context -> int32

  val hard_gas_limit_per_operation : context -> Gas.Arith.integral

  val hard_gas_limit_per_block : context -> Gas.Arith.integral

  val cost_per_byte : context -> Tez.t

  val hard_storage_limit_per_operation : context -> Z.t

  val proof_of_work_threshold : context -> int64

  val minimal_stake : context -> Tez.t

  val vdf_difficulty : context -> int64

  val seed_nonce_revelation_tip : context -> Tez.t

  val origination_size : context -> int

  val baking_reward_fixed_portion : context -> Tez.t

  val baking_reward_bonus_per_slot : context -> Tez.t

  val endorsing_reward_per_slot : context -> Tez.t

  val quorum_min : context -> int32

  val quorum_max : context -> int32

  val min_proposal_quorum : context -> int32

  val liquidity_baking_subsidy : context -> Tez.t

  val liquidity_baking_toggle_ema_threshold : context -> int32

  val minimal_block_delay : context -> Period.t

  val delay_increment_per_round : context -> Period.t

  (** See {!Raw_context.round_durations}. *)
  val round_durations : context -> Round.round_durations

  val consensus_committee_size : context -> int

  val consensus_threshold : context -> int

  val minimal_participation_ratio : context -> Ratio.t

  val max_slashing_period : context -> int

  val frozen_deposits_percentage : context -> int

  val double_baking_punishment : context -> Tez.t

  val ratio_of_frozen_deposits_slashed_per_double_endorsement :
    context -> Ratio.t

  val testnet_dictator : context -> public_key_hash option

  val tx_rollup_enable : context -> bool

  val tx_rollup_origination_size : context -> int

  val tx_rollup_hard_size_limit_per_inbox : context -> int

  val tx_rollup_hard_size_limit_per_message : context -> int

  val tx_rollup_max_withdrawals_per_batch : context -> int

  val tx_rollup_commitment_bond : context -> Tez.t

  val tx_rollup_finality_period : context -> int

  val tx_rollup_max_inboxes_count : context -> int

  val tx_rollup_max_messages_per_inbox : context -> int

  val tx_rollup_max_commitments_count : context -> int

  val tx_rollup_max_ticket_payload_size : context -> int

  val tx_rollup_rejection_max_proof_size : context -> int

  val tx_rollup_sunset_level : context -> int32

  val sc_rollup_enable : context -> bool

  val sc_rollup_arith_pvm_enable : context -> bool

  val dal_enable : context -> bool

  val sc_rollup_origination_size : context -> int

  val sc_rollup_stake_amount : t -> Tez.t

  val sc_rollup_commitment_period_in_blocks : t -> int

  val sc_rollup_max_lookahead_in_blocks : t -> int32

  val sc_rollup_max_active_outbox_levels : context -> int32

  val sc_rollup_max_outbox_messages_per_level : context -> int

  val sc_rollup_number_of_sections_in_dissection : context -> int

  val max_number_of_stored_cemented_commitments : context -> int

  val zk_rollup_enable : context -> bool

  val zk_rollup_min_pending_to_process : context -> int

  (** All constants: fixed and parametric *)
  type t = private {fixed: fixed; parametric: Parametric.t}

  val all : context -> t

  val encoding : t Data_encoding.t
end

(** See the definitions inside the module. *)
module Global_constants_storage : sig
  type error += Expression_too_deep

  type error += Expression_already_registered

  (** A constant is the prim of the literal characters "constant".
    A constant must have a single argument, being a string with a
    well formed hash of a Micheline expression (i.e generated by
    [Script_expr_hash.to_b58check]). *)
  type error += Badly_formed_constant_expression

  type error += Nonexistent_global

  (** [get context hash] retrieves the Micheline value with the given hash.

    Fails with [Nonexistent_global] if no value is found at the given hash.

    Fails with [Storage_error Corrupted_data] if the deserialisation fails.

    Consumes [Gas_repr.read_bytes_cost <size of the value>]. *)
  val get : t -> Script_expr_hash.t -> (t * Script.expr) tzresult Lwt.t

  (** [register context value] Register a constant in the global table of constants,
    returning the hash and storage bytes consumed.

    Does not type-check the Micheline code being registered, allow potentially
    ill-typed Michelson values (see note at top of module in global_constants_storage.mli).

    The constant is stored unexpanded, but it is temporarily expanded at registration
    time only to check the expanded version respects the following limits.

    Fails with [Expression_too_deep] if, after fully, expanding all constants,
    the expression would contain too many nested levels, that is more than
    [Constants_repr.max_allowed_global_constant_depth].

    Fails with [Badly_formed_constant_expression] if constants are not
    well-formed (see declaration of [Badly_formed_constant_expression]) or with
    [Nonexistent_global] if a referenced constant does not exist in the table.

    Consumes serialization cost.
    Consumes [Gas_repr.write_bytes_cost <size>] where size is the number
    of bytes in the binary serialization provided by [Script.expr_encoding].*)
  val register :
    t -> Script.expr -> (t * Script_expr_hash.t * Z.t) tzresult Lwt.t

  (** [expand context expr] Replaces every constant in the
    given Michelson expression with its value stored in the global table.

    The expansion is applied recursively so that the returned expression
    contains no constant.

    Fails with [Badly_formed_constant_expression] if constants are not
    well-formed (see declaration of [Badly_formed_constant_expression]) or
    with [Nonexistent_global] if a referenced constant does not exist in
    the table. *)
  val expand : t -> Script.expr -> (t * Script.expr) tzresult Lwt.t

  (** This module discloses definitions that are only useful for tests and must
      not be used otherwise. *)
  module Internal_for_tests : sig
    (** [node_too_large node] returns true if:
      - The number of sub-nodes in the [node]
        exceeds [Global_constants_storage.node_size_limit].
      - The sum of the bytes in String, Int,
        and Bytes sub-nodes of [node] exceeds
        [Global_constants_storage.bytes_size_limit].

      Otherwise returns false.  *)
    val node_too_large : Script.node -> bool

    (** [bottom_up_fold_cps initial_accumulator node initial_k f]
        folds [node] and all its sub-nodes if any, starting from
        [initial_accumulator], using an initial continuation [initial_k].
        At each node, [f] is called to transform the continuation [k] into
        the next one. This explicit manipulation of the continuation
        is typically useful to short-circuit.

        Notice that a common source of bug is to forget to properly call the
        continuation in `f`. *)
    val bottom_up_fold_cps :
      'accumulator ->
      'loc Script.michelson_node ->
      ('accumulator -> 'loc Script.michelson_node -> 'return) ->
      ('accumulator ->
      'loc Script.michelson_node ->
      ('accumulator -> 'loc Script.michelson_node -> 'return) ->
      'return) ->
      'return

    (** [expr_to_address_in_context context expr] converts [expr]
       into a unique hash represented by a [Script_expr_hash.t].

       Consumes gas corresponding to the cost of converting [expr]
       to bytes and hashing the bytes. *)
    val expr_to_address_in_context :
      t -> Script.expr -> (t * Script_expr_hash.t) tzresult
  end
end

(** This module discloses definitions that are only useful for tests and must
    not be used otherwise. *)
module Internal_for_tests : sig
  val to_raw : context -> Raw_context.t
end

(** This module re-exports definitions from {!Level_repr} and
    {!Level_storage}. *)
module Level : sig
  type t = private {
    level: Raw_level.t;
    level_position: int32;
    cycle: Cycle.t;
    cycle_position: int32;
    expected_commitment: bool;
  }

  include BASIC_DATA with type t := t

  val pp_full : Format.formatter -> t -> unit

  type level = t

  val root : context -> level

  val succ : context -> level -> level

  val pred : context -> level -> level option

  val from_raw : context -> Raw_level.t -> level

  (** Fails with [Negative_level_and_offset_sum] if the sum of the raw_level and the offset is negative. *)
  val from_raw_with_offset :
    context -> offset: int32 -> Raw_level.t -> level tzresult

  (** [add c level i] i must be positive *)
  val add : context -> level -> int -> level

  (** [sub c level i] i must be positive *)
  val sub : context -> level -> int -> level option

  val diff : level -> level -> int32

  val current : context -> level

  val last_level_in_cycle : context -> Cycle.t -> level

  val levels_in_cycle : context -> Cycle.t -> level list

  val levels_in_current_cycle : context -> ?offset: int32 -> unit -> level list

  val last_allowed_fork_level : context -> Raw_level.t

  val dawn_of_a_new_cycle : context -> Cycle.t option

  val may_snapshot_stake_distribution : context -> bool

  val may_compute_randao : context -> bool
end

(** This module re-exports definitions from {!Fitness_repr}. *)
module Fitness : sig
  type error += Invalid_fitness | Wrong_fitness | Outdated_fitness

  type raw = Fitness.t

  type t

  val encoding : t Data_encoding.t

  val pp : Format.formatter -> t -> unit

  val create :
    level: Raw_level.t ->
    locked_round: Round.t option ->
    predecessor_round: Round.t ->
    round: Round.t ->
    t tzresult

  val create_without_locked_round :
    level: Raw_level.t -> predecessor_round: Round.t -> round: Round.t -> t

  val to_raw : t -> raw

  val from_raw : raw -> t tzresult

  val round_from_raw : raw -> Round.t tzresult

  val predecessor_round_from_raw : raw -> Round.t tzresult

  val level : t -> Raw_level.t

  val round : t -> Round.t

  val locked_round : t -> Round.t option

  val predecessor_round : t -> Round.t
end

(** This module re-exports definitions from {!Nonce_storage}. *)
module Nonce : sig
  type t

  type nonce = t

  val encoding : nonce Data_encoding.t

  type unrevealed = {nonce_hash: Nonce_hash.t; delegate: public_key_hash}

  val record_hash : context -> unrevealed -> context tzresult Lwt.t

  (** See {!Nonce_storage.check_unrevealed}. *)
  val check_unrevealed : context -> Level.t -> nonce -> unit tzresult Lwt.t

  val reveal : context -> Level.t -> nonce -> context tzresult Lwt.t

  type status = Unrevealed of unrevealed | Revealed of nonce

  val get : context -> Level.t -> status tzresult Lwt.t

  val of_bytes : bytes -> nonce tzresult

  val hash : nonce -> Nonce_hash.t

  val check_hash : nonce -> Nonce_hash.t -> bool
end

(** This module re-exports definitions from {!Seed_repr} and {!Seed_storage}. *)
module Seed : sig
  type seed

  val seed_encoding : seed Data_encoding.t

  type vdf_solution = Vdf.result * Vdf.proof

  val vdf_solution_encoding : vdf_solution Data_encoding.t

  val pp_solution : Format.formatter -> vdf_solution -> unit

  type vdf_setup = Vdf.discriminant * Vdf.challenge

  type error +=
    | Unknown of {oldest: Cycle.t; cycle: Cycle.t; latest: Cycle.t}
    | Already_accepted
    | Unverified_vdf
    | Too_early_revelation

  val generate_vdf_setup :
    seed_discriminant: seed -> seed_challenge: seed -> vdf_setup

  (** See {!Seed_storage.check_vdf}. *)
  val check_vdf : context -> vdf_solution -> unit tzresult Lwt.t

  (** See {!Seed_storage.update_seed}. *)
  val update_seed : context -> vdf_solution -> context tzresult Lwt.t

  (** See {!Seed_repr.compare_vdf_solution}. *)
  val compare_vdf_solution : vdf_solution -> vdf_solution -> int

  val compute_randao : context -> context tzresult Lwt.t

  (* RPC *)
  type seed_computation_status =
    | Nonce_revelation_stage
    | Vdf_revelation_stage of {seed_discriminant: seed; seed_challenge: seed}
    | Computation_finished

  val for_cycle : context -> Cycle.t -> seed tzresult Lwt.t

  val get_seed_computation_status :
    context -> seed_computation_status tzresult Lwt.t
end

(** Big maps are a data structure storing key-value associations, just like
    regular maps, but here the whole content of the structure is not loaded in
    memory when interacting with it.
    They are thus suitable for a Michelson contract, for instance, when there are a
    lot of bindings, but only a few items are accessed at each contract call. *)
module Big_map : sig
  (** A big map is referenced in the storage by its identifier. *)
  module Id : sig
    type t = Lazy_storage_kind.Big_map.Id.t

    val encoding : t Data_encoding.t

    (** Big map argument for a RPC call. *)
    val rpc_arg : t RPC_arg.arg

    (** In the protocol, to be used in parse_data only *)
    val parse_z : Z.t -> t

    (** In the protocol, to be used in unparse_data only *)
    val unparse_to_z : t -> Z.t
  end

  (** Create a fresh big map in the context. *)
  val fresh : temporary: bool -> context -> (context * Id.t) tzresult Lwt.t

  (** Carbonated membership of a key (from its hash) in a big map. *)
  val mem :
    context -> Id.t -> Script_expr_hash.t -> (context * bool) tzresult Lwt.t

  (** Carbonated retrieval of the value associated to a key (from its hash) in
      a big map, if any. *)
  val get_opt :
    context ->
    Id.t ->
    Script_expr_hash.t ->
    (context * Script.expr option) tzresult Lwt.t

  (** Carbonated retrieval of the key and value types of the bindings in a big
      map referenced by its identifier, if this identifier is actually bound to a big map in the context. *)
  val exists :
    context ->
    Id.t ->
    (context * (Script.expr * Script.expr) option) tzresult Lwt.t

  (** [list_key_values ?offset ?length ctxt id] lists the key hash and value for
      each entry in big map [id]. The first [offset] values are ignored (if
      passed). Negative offsets are treated as [0]. There will be no more than
      [length] values in the result list (if passed). Negative values are
      treated as [0].

      The returned {!context} takes into account gas consumption of traversing
      the keys and loading values. *)
  val list_key_values :
    ?offset: int ->
    ?length: int ->
    context ->
    Id.t ->
    (context * (Script_expr_hash.t * Script.expr) list) tzresult Lwt.t

  (** The type of big map updates. When [value = None], the potential binding
      associated to the [key] will be removed. *)
  type update = {
    key: Script_repr.expr;
    (** The key is ignored by an update but is shown in the receipt. *)
    key_hash: Script_expr_hash.t;
    value: Script_repr.expr option;
  }

  type updates = update list

  (** The types of keys and values in a big map. *)
  type alloc = {key_type: Script_repr.expr; value_type: Script_repr.expr}
end

(** This module re-exports definitions from {!Sapling_repr}, {!Sapling_storage}
    and {!Sapling_validator}. *)
module Sapling : sig
  (** See {!Sapling_state.Id}. *)
  module Id : sig
    type t

    val encoding : t Data_encoding.t

    val rpc_arg : t RPC_arg.arg

    val parse_z : Z.t -> t
    (* To be used in parse_data only *)

    val unparse_to_z : t -> Z.t
    (* To be used in unparse_data only *)
  end

  (** Create a fresh sapling state in the context. *)
  val fresh : temporary: bool -> context -> (context * Id.t) tzresult Lwt.t

  type diff = private {
    commitments_and_ciphertexts:
    (Sapling.Commitment.t * Sapling.Ciphertext.t) list;
    nullifiers: Sapling.Nullifier.t list;
  }

  val diff_encoding : diff Data_encoding.t

  module Memo_size : sig
    type t

    val encoding : t Data_encoding.t

    val equal : t -> t -> bool

    val parse_z : Z.t -> (t, string) result

    val unparse_to_z : t -> Z.t

    val in_memory_size : t -> Cache_memory_helpers.sint
  end

  type state = private {id: Id.t option; diff: diff; memo_size: Memo_size.t}

  (**
    Returns a [state] with fields filled accordingly.
    [id] should only be used by [extract_lazy_storage_updates].
   *)
  val empty_state : ?id: Id.t -> memo_size: Memo_size.t -> unit -> state

  type transaction = Sapling.UTXO.transaction

  val transaction_encoding : transaction Data_encoding.t

  val transaction_get_memo_size : transaction -> Memo_size.t option

  (**
    Tries to fetch a state from the storage.
   *)
  val state_from_id : context -> Id.t -> (state * context) tzresult Lwt.t

  val rpc_arg : Id.t RPC_arg.t

  type root = Sapling.Hash.t

  val root_encoding : root Data_encoding.t

  (* Function exposed as RPC. Returns the root and a diff of a state starting
     from an optional offset which is zero by default. *)
  val get_diff :
    context ->
    Id.t ->
    ?offset_commitment: Int64.t ->
    ?offset_nullifier: Int64.t ->
    unit ->
    (root * diff) tzresult Lwt.t

  val verify_update :
    context ->
    state ->
    transaction ->
    string ->
    (context * (Int64.t * state) option) tzresult Lwt.t

  (** See {!Lazy_storage_kind.Sapling_state.alloc}. *)
  type alloc = {memo_size: Memo_size.t}

  type updates = diff

  val transaction_in_memory_size : transaction -> Cache_memory_helpers.sint

  val diff_in_memory_size : diff -> Cache_memory_helpers.sint

  module Legacy : sig
    type transaction = Sapling.UTXO.Legacy.transaction

    val transaction_encoding : transaction Data_encoding.t

    val transaction_get_memo_size : transaction -> Memo_size.t option

    val transaction_in_memory_size :
      transaction -> Saturation_repr.may_saturate Saturation_repr.t

    val verify_update :
      context ->
      state ->
      transaction ->
      string ->
      (context * (Int64.t * state) option) tzresult Lwt.t
  end
end

(** This module re-exports definitions from {!Lazy_storage_diff}. *)
module Lazy_storage : sig
  (** This module re-exports definitions from {!Lazy_storage_kind}. *)
  module Kind : sig
    type ('id, 'alloc, 'updates) t =
      | Big_map : (Big_map.Id.t, Big_map.alloc, Big_map.updates) t
      | Sapling_state : (Sapling.Id.t, Sapling.alloc, Sapling.updates) t
  end

  (** This module re-exports definitions from {!Lazy_storage_kind.IdSet}. *)
  module IdSet : sig
    type t

    type 'acc fold_f = {f: 'i 'a 'u. ('i, 'a, 'u) Kind.t -> 'i -> 'acc -> 'acc}

    val empty : t

    val mem : ('i, 'a, 'u) Kind.t -> 'i -> t -> bool

    val add : ('i, 'a, 'u) Kind.t -> 'i -> t -> t

    val diff : t -> t -> t

    val fold : ('i, 'a, 'u) Kind.t -> ('i -> 'acc -> 'acc) -> t -> 'acc -> 'acc

    val fold_all : 'acc fold_f -> t -> 'acc -> 'acc
  end

  type ('id, 'alloc) init = Existing | Copy of {src: 'id} | Alloc of 'alloc

  type ('id, 'alloc, 'updates) diff =
    | Remove
    | Update of {init: ('id, 'alloc) init; updates: 'updates}

  type diffs_item =
    private
    | Item :
      ('i, 'a, 'u) Lazy_storage_kind.t
      * 'i
      * ('i, 'a, 'u) diff ->
        diffs_item

  val make : ('i, 'a, 'u) Kind.t -> 'i -> ('i, 'a, 'u) diff -> diffs_item

  type diffs = diffs_item list

  val encoding : diffs Data_encoding.t

  val diffs_in_memory_size : diffs -> Cache_memory_helpers.nodes_and_size

  val cleanup_temporaries : context -> context Lwt.t

  val apply : t -> diffs -> (t * Z.t) tzresult Lwt.t
end

(** See the definitions inside the module. *)
module Origination_nonce : sig
  (** See {!Raw_context.init_origination_nonce}. *)
  val init : context -> Operation_hash.t -> context

  (** See {!Raw_context.unset_origination_nonce}. *)
  val unset : context -> context

  (** This module discloses definitions that are only useful for tests and must
      not be used otherwise. See {!Origination_nonce}. *)
  module Internal_for_tests : sig
    type t

    val initial : Operation_hash.t -> t

    val incr : t -> t
  end
end

(** This module re-exports definitions from {!Ticket_hash_repr}. *)
module Ticket_hash : sig
  type t

  val encoding : t Data_encoding.t

  val pp : Format.formatter -> t -> unit

  val zero : t

  val of_script_expr_hash : Script_expr_hash.t -> t

  val to_b58check : t -> string

  val of_b58check_opt : string -> t option

  val of_b58check_exn : string -> t

  val of_bytes_exn : bytes -> t

  val of_bytes_opt : bytes -> t option

  val equal : t -> t -> bool

  val compare : t -> t -> int

  val make :
    context ->
    ticketer: Script.node ->
    ty: Script.node ->
    contents: Script.node ->
    owner: Script.node ->
    (t * context) tzresult

  (** This module discloses definitions that are only useful for tests and must
      not be used otherwise. *)
  module Internal_for_tests : sig
    val make_uncarbonated :
      ticketer: Script.node ->
      ty: Script.node ->
      contents: Script.node ->
      owner: Script.node ->
      t tzresult
  end
end

(** This module re-exports definitions from {!Manager_counter_repr}. *)
module Manager_counter : sig
  include Compare.S

  val succ : t -> t

  val pp : Format.formatter -> t -> unit

  val encoding_for_RPCs : t Data_encoding.t

  module Internal_for_injection : sig
    val of_string : string -> t option
  end

  module Internal_for_tests : sig
    val of_int : int -> t

    val to_int : t -> int

    val add : t -> int -> t
  end
end

(** This module re-exports definitions from {!Contract_repr} and
    {!Contract_storage}. *)
module Contract : sig
  type t = Implicit of public_key_hash | Originated of Contract_hash.t

  (** Functions related to contracts address. *)

  type error += Non_existing_contract of t

  include BASIC_DATA with type t := t

  val implicit_encoding : public_key_hash Data_encoding.t

  val originated_encoding : Contract_hash.t Data_encoding.t

  val in_memory_size : t -> Cache_memory_helpers.sint

  val rpc_arg : t RPC_arg.arg

  val to_b58check : t -> string

  val of_b58check : string -> t tzresult

  (** Functions related to contracts existence. *)

  val exists : context -> t -> bool Lwt.t

  val must_exist : context -> t -> unit tzresult Lwt.t

  val allocated : context -> t -> bool Lwt.t

  val must_be_allocated : context -> t -> unit tzresult Lwt.t

  val list : context -> t list Lwt.t

  (** Functions related to both implicit accounts and originated contracts. *)

  (** See {!Contract_storage.get_balance}. *)
  val get_balance : context -> t -> Tez.t tzresult Lwt.t

  val get_balance_carbonated : context -> t -> (context * Tez.t) tzresult Lwt.t

  val get_frozen_bonds : context -> t -> Tez.t tzresult Lwt.t

  val get_balance_and_frozen_bonds : context -> t -> Tez.t tzresult Lwt.t

  (** Functions related to implicit accounts. *)

  (** See {!Contract_manager_storage.get_manager_key}. *)
  val get_manager_key :
    ?error: error -> context -> public_key_hash -> public_key tzresult Lwt.t

  (** See {!Contract_manager_storage.is_manager_key_revealed}. *)
  val is_manager_key_revealed :
    context -> public_key_hash -> bool tzresult Lwt.t

  (** See {!Contract_manager_storage.check_public_key}. *)
  val check_public_key : public_key -> public_key_hash -> unit tzresult

  (** See {!Contract_manager_storage.reveal_manager_key}. *)
  val reveal_manager_key :
    ?check_consistency: bool ->
    context ->
    public_key_hash ->
    public_key ->
    context tzresult Lwt.t

  val get_counter :
    context -> public_key_hash -> Manager_counter.t tzresult Lwt.t

  val increment_counter : context -> public_key_hash -> context tzresult Lwt.t

  val check_counter_increment :
    context -> public_key_hash -> Manager_counter.t -> unit tzresult Lwt.t

  (** See {!Contract_storage.check_allocated_and_get_balance}. *)
  val check_allocated_and_get_balance :
    context -> public_key_hash -> Tez.t tzresult Lwt.t

  (** See {!Contract_storage.simulate_spending}. *)
  val simulate_spending :
    context ->
    balance: Tez.t ->
    amount: Tez.t ->
    public_key_hash ->
    (Tez.t * bool) tzresult Lwt.t

  (** Functions related to smart contracts. *)

  val get_script_code :
    context ->
    Contract_hash.t ->
    (context * Script.lazy_expr option) tzresult Lwt.t

  val get_script :
    context -> Contract_hash.t -> (context * Script.t option) tzresult Lwt.t

  val get_storage :
    context -> Contract_hash.t -> (context * Script.expr option) tzresult Lwt.t

  val used_storage_space : context -> t -> Z.t tzresult Lwt.t

  val paid_storage_space : context -> t -> Z.t tzresult Lwt.t

  val increase_paid_storage :
    context -> Contract_hash.t -> amount_in_bytes: Z.t -> context tzresult Lwt.t

  val fresh_contract_from_current_nonce :
    context -> (context * Contract_hash.t) tzresult

  val originated_from_current_nonce :
    since: context -> until: context -> Contract_hash.t list tzresult Lwt.t

  val update_script_storage :
    context ->
    Contract_hash.t ->
    Script.expr ->
    Lazy_storage.diffs option ->
    context tzresult Lwt.t

  val raw_originate :
    context ->
    prepaid_bootstrap_storage: bool ->
    Contract_hash.t ->
    script: Script.t * Lazy_storage.diffs option ->
    context tzresult Lwt.t

  module Legacy_big_map_diff : sig
    type item =
      private
      | Update of {
          big_map: Z.t;
          diff_key: Script.expr;
          diff_key_hash: Script_expr_hash.t;
          diff_value: Script.expr option;
        }
      | Clear of Z.t
      | Copy of {src: Z.t; dst: Z.t}
      | Alloc of {
          big_map: Z.t;
          key_type: Script.expr;
          value_type: Script.expr;
        }

    type t = private item list

    val of_lazy_storage_diff : Lazy_storage.diffs -> t
  end

  (** Functions for handling the delegate of a contract.*)
  module Delegate : sig
    (** See {!Contract_delegate_storage.find}. *)
    val find : context -> t -> public_key_hash option tzresult Lwt.t

    (** See {!Delegate_storage.Contract.init}. *)
    val init : context -> t -> public_key_hash -> context tzresult Lwt.t

    (** See {!Delegate_storage.Contract.set}. *)
    val set : context -> t -> public_key_hash option -> context tzresult Lwt.t
  end

  (** This module discloses definitions that are only useful for tests and must
      not be used otherwise. *)
  module Internal_for_tests : sig
    (** See {!Contract_repr.originated_contract}. *)
    val originated_contract : Origination_nonce.Internal_for_tests.t -> t

    val paid_storage_space : context -> t -> Z.t tzresult Lwt.t
  end
end

(** This module re-exports definitions from {!Tx_rollup_level_repr}. *)
module Tx_rollup_level : sig
  include BASIC_DATA

  type level = t

  val rpc_arg : level RPC_arg.arg

  val diff : level -> level -> int32

  val root : level

  val succ : level -> level

  val pred : level -> level option

  val to_int32 : level -> int32

  val of_int32 : int32 -> level tzresult
end

(** This module re-exports definitions from {!Tx_rollup_repr} and
    {!Tx_rollup_storage}. *)
module Tx_rollup : sig
  include BASIC_DATA

  val in_memory_size : t -> Cache_memory_helpers.sint

  val rpc_arg : t RPC_arg.arg

  val to_b58check : t -> string

  val of_b58check : string -> t tzresult

  val of_b58check_opt : string -> t option

  val pp : Format.formatter -> t -> unit

  val encoding : t Data_encoding.t

  val originate : context -> (context * t) tzresult Lwt.t

  module Set : Set.S with type elt = t

  (** This module discloses definitions that are only useful for tests and must
      not be used otherwise. *)
  module Internal_for_tests : sig
    (** See {!Tx_rollup_repr.originated_tx_rollup}. *)
    val originated_tx_rollup : Origination_nonce.Internal_for_tests.t -> t
  end
end

(** This module re-exports definitions from {!Tx_rollup_withdraw_repr}. *)
module Tx_rollup_withdraw : sig
  type order = {
    claimer: public_key_hash;
    ticket_hash: Ticket_hash.t;
    amount: Tx_rollup_l2_qty.t;
  }

  type t = order

  val encoding : t Data_encoding.t
end

(** This module re-exports definitions from
    {!Tx_rollup_withdraw_list_hash_repr}. *)
module Tx_rollup_withdraw_list_hash : sig
  include S.HASH

  val hash_uncarbonated : Tx_rollup_withdraw.t list -> t

  val empty : t
end

(** This module re-exports definitions from {!Tx_rollup_message_result_repr}. *)
module Tx_rollup_message_result : sig
  type t = {
    context_hash: Context_hash.t;
    withdraw_list_hash: Tx_rollup_withdraw_list_hash.t;
  }

  val encoding : t Data_encoding.t

  val empty_l2_context_hash : Context_hash.t

  val init : t
end

(** This module re-exports definitions from
    {!Tx_rollup_message_result_hash_repr}. *)
module Tx_rollup_message_result_hash : sig
  include S.HASH

  val hash_uncarbonated : Tx_rollup_message_result.t -> t

  val init : t
end

(** This module re-exports definitions from {!Tx_rollup_commitment_repr.Hash}.
*)
module Tx_rollup_commitment_hash : sig
  val commitment_hash : string

  include S.HASH
end

(** This module re-exports definitions from {!Tx_rollup_state_repr}
    and {!Tx_rollup_state_storage}. *)
module Tx_rollup_state : sig
  type t

  val initial_state : pre_allocated_storage: Z.t -> t

  val encoding : t Data_encoding.t

  val pp : Format.formatter -> t -> unit

  val find : context -> Tx_rollup.t -> (context * t option) tzresult Lwt.t

  val get : context -> Tx_rollup.t -> (context * t) tzresult Lwt.t

  val update : context -> Tx_rollup.t -> t -> context tzresult Lwt.t

  val burn_cost : limit: Tez.t option -> t -> int -> Tez.t tzresult

  val assert_exist : context -> Tx_rollup.t -> context tzresult Lwt.t

  val head_levels : t -> (Tx_rollup_level.t * Raw_level.t) option

  val check_level_can_be_rejected : t -> Tx_rollup_level.t -> unit tzresult

  val last_removed_commitment_hashes :
    t -> (Tx_rollup_message_result_hash.t * Tx_rollup_commitment_hash.t) option

  val adjust_storage_allocation : t -> delta: Z.t -> (t * Z.t) tzresult

  (** This module discloses definitions that are only useful for tests and must
      not be used otherwise. *)
  module Internal_for_tests : sig
    val make :
      ?burn_per_byte: Tez.t ->
      ?inbox_ema: int ->
      ?last_removed_commitment_hashes:
      Tx_rollup_message_result_hash.t * Tx_rollup_commitment_hash.t ->
      ?finalized_commitments: Tx_rollup_level.t * Tx_rollup_level.t ->
      ?unfinalized_commitments: Tx_rollup_level.t * Tx_rollup_level.t ->
      ?uncommitted_inboxes: Tx_rollup_level.t * Tx_rollup_level.t ->
      ?commitment_newest_hash: Tx_rollup_commitment_hash.t ->
      ?tezos_head_level: Raw_level.t ->
      ?occupied_storage: Z.t ->
      ?commitments_watermark: Tx_rollup_level.t ->
      allocated_storage: Z.t ->
      unit ->
      t

    val update_burn_per_byte :
      t -> elapsed: int -> factor: int -> final_size: int -> hard_limit: int -> t

    val get_inbox_ema : t -> int

    val record_inbox_deletion : t -> Tx_rollup_level.t -> t tzresult

    val get_occupied_storage : t -> Z.t

    val set_occupied_storage : Z.t -> t -> t

    val get_allocated_storage : t -> Z.t

    val set_allocated_storage : Z.t -> t -> t

    val next_commitment_level : t -> Raw_level.t -> Tx_rollup_level.t tzresult

    val uncommitted_inboxes_count : t -> int

    val reset_commitments_watermark : t -> t

    val get_commitments_watermark : t -> Tx_rollup_level.t option
  end
end

(** This module re-exports definitions from {!Tx_rollup_reveal_repr} and
    {!Tx_rollup_reveal_storage}. *)
module Tx_rollup_reveal : sig
  type t = {
    contents: Script.lazy_expr;
    ty: Script.lazy_expr;
    ticketer: Contract.t;
    amount: Tx_rollup_l2_qty.t;
    claimer: public_key_hash;
  }

  val encoding : t Data_encoding.t

  val record :
    context ->
    Tx_rollup.t ->
    Tx_rollup_level.t ->
    message_position: int ->
    context tzresult Lwt.t

  val mem :
    context ->
    Tx_rollup.t ->
    Tx_rollup_level.t ->
    message_position: int ->
    (context * bool) tzresult Lwt.t

  val remove :
    context -> Tx_rollup.t -> Tx_rollup_level.t -> context tzresult Lwt.t
end

(** This module re-exports definitions from {!Tx_rollup_message_repr}. *)
module Tx_rollup_message : sig
  type deposit = {
    sender: public_key_hash;
    destination: Tx_rollup_l2_address.Indexable.value;
    ticket_hash: Ticket_hash.t;
    amount: Tx_rollup_l2_qty.t;
  }

  type t = private Batch of string | Deposit of deposit

  (** [make_batch batch] creates a new [Batch] message to be added that can be
      added to an inbox, along with its size in bytes. See
      {!Tx_rollup_message_repr.size}. *)
  val make_batch : string -> t * int

  (** [make_deposit destination ticket_hash qty] creates a new
      [Deposit] message to be added that can be added to an inbox,
      along with its size in bytes. See
      {!Tx_rollup_message_repr.size}. *)
  val make_deposit :
    public_key_hash ->
    Tx_rollup_l2_address.t Indexable.value ->
    Ticket_hash.t ->
    Tx_rollup_l2_qty.t ->
    t * int

  val encoding : t Data_encoding.t

  val pp : Format.formatter -> t -> unit
end

(** This module re-exports definitions from {!Tx_rollup_message_hash_repr}. *)
module Tx_rollup_message_hash : sig
  include S.HASH

  val hash_uncarbonated : Tx_rollup_message.t -> t
end

(** This module re-exports definitions from {!Tx_rollup_inbox_repr} and
    {!Tx_rollup_inbox_storage}. *)
module Tx_rollup_inbox : sig
  module Merkle : sig
    type root

    type path

    val path_encoding : path Data_encoding.t

    val root_encoding : root Data_encoding.t

    val root_of_b58check_opt : string -> root option

    val compute_path : Tx_rollup_message_hash.t list -> int -> path tzresult

    val merklize_list : Tx_rollup_message_hash.t list -> root

    val path_depth : path -> int
  end

  type t = {inbox_length: int; cumulated_size: int; merkle_root: Merkle.root}

  val size : Z.t

  val (=): t -> t -> bool

  val pp : Format.formatter -> t -> unit

  val encoding : t Data_encoding.t

  val append_message :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    Tx_rollup_message.t ->
    (context * Tx_rollup_state.t * Z.t) tzresult Lwt.t

  val get :
    context -> Tx_rollup_level.t -> Tx_rollup.t -> (context * t) tzresult Lwt.t

  val find :
    context ->
    Tx_rollup_level.t ->
    Tx_rollup.t ->
    (context * t option) tzresult Lwt.t

  val check_message_hash :
    context ->
    Tx_rollup_level.t ->
    Tx_rollup.t ->
    position: int ->
    Tx_rollup_message.t ->
    Merkle.path ->
    context tzresult Lwt.t
end

(** This module re-exports definitions from {!Tx_rollup_commitment_repr}. *)
module Tx_rollup_commitment : sig
  module Merkle_hash : S.HASH

  module Merkle :
    Merkle_list.T with
    type elt = Tx_rollup_message_result_hash.t
    and type h = Merkle_hash.t

  type 'a template = {
    level: Tx_rollup_level.t;
    messages: 'a;
    predecessor: Tx_rollup_commitment_hash.t option;
    inbox_merkle_root: Tx_rollup_inbox.Merkle.root;
  }

  module Compact : sig
    type excerpt = {
      count: int;
      root: Merkle.h;
      last_result_message_hash: Tx_rollup_message_result_hash.t;
    }

    type t = excerpt template

    val pp : Format.formatter -> t -> unit

    val encoding : t Data_encoding.t

    val hash : t -> Tx_rollup_commitment_hash.t
  end

  module Submitted_commitment : sig
    type nonrec t = {
      commitment: Compact.t;
      commitment_hash: Tx_rollup_commitment_hash.t;
      committer: public_key_hash;
      submitted_at: Raw_level.t;
      finalized_at: Raw_level.t option;
    }

    val encoding : t Data_encoding.t
  end

  module Full : sig
    type t = Tx_rollup_message_result_hash.t list template

    val encoding : t Data_encoding.t

    val pp : Format.formatter -> t -> unit

    val compact : t -> Compact.t
  end

  val check_message_result :
    context ->
    Compact.t ->
    [
      | `Hash of Tx_rollup_message_result_hash.t
      | `Result of Tx_rollup_message_result.t
    ] ->
    path: Merkle.path ->
    index: int ->
    context tzresult

  val add_commitment :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    public_key_hash ->
    Full.t ->
    (context * Tx_rollup_state.t * public_key_hash option) tzresult Lwt.t

  val find :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    Tx_rollup_level.t ->
    (context * Submitted_commitment.t option) tzresult Lwt.t

  val get :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    Tx_rollup_level.t ->
    (context * Submitted_commitment.t) tzresult Lwt.t

  val check_agreed_and_disputed_results :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    Submitted_commitment.t ->
    agreed_result: Tx_rollup_message_result.t ->
    agreed_result_path: Merkle.path ->
    disputed_result: Tx_rollup_message_result_hash.t ->
    disputed_position: int ->
    disputed_result_path: Merkle.path ->
    context tzresult Lwt.t

  val get_finalized :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    Tx_rollup_level.t ->
    (context * Submitted_commitment.t) tzresult Lwt.t

  val pending_bonded_commitments :
    context -> Tx_rollup.t -> public_key_hash -> (context * int) tzresult Lwt.t

  val has_bond :
    context -> Tx_rollup.t -> public_key_hash -> (context * bool) tzresult Lwt.t

  val finalize_commitment :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    (context * Tx_rollup_state.t * Tx_rollup_level.t) tzresult Lwt.t

  val remove_commitment :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    (context * Tx_rollup_state.t * Tx_rollup_level.t) tzresult Lwt.t

  val remove_bond :
    context -> Tx_rollup.t -> public_key_hash -> context tzresult Lwt.t

  val slash_bond :
    context -> Tx_rollup.t -> public_key_hash -> (context * bool) tzresult Lwt.t

  val reject_commitment :
    context ->
    Tx_rollup.t ->
    Tx_rollup_state.t ->
    Tx_rollup_level.t ->
    (context * Tx_rollup_state.t) tzresult Lwt.t
end

(** This module re-exports definitions from {!Tx_rollup_hash_builder}. *)
module Tx_rollup_hash : sig
  val message_result :
    context ->
    Tx_rollup_message_result.t ->
    (context * Tx_rollup_message_result_hash.t) tzresult

  val compact_commitment :
    context ->
    Tx_rollup_commitment.Compact.t ->
    (context * Tx_rollup_commitment_hash.t) tzresult

  val withdraw_list :
    context ->
    Tx_rollup_withdraw.t list ->
    (context * Tx_rollup_withdraw_list_hash.t) tzresult
end

(** This module re-exports definitions from {!Tx_rollup_errors_repr}. *)
module Tx_rollup_errors : sig
  type error +=
    | Tx_rollup_already_exists of Tx_rollup.t
    | Tx_rollup_does_not_exist of Tx_rollup.t
    | Submit_batch_burn_exceeded of {burn: Tez.t; limit: Tez.t}
    | Inbox_does_not_exist of Tx_rollup.t * Tx_rollup_level.t
    | Inbox_size_would_exceed_limit of Tx_rollup.t
    | Inbox_count_would_exceed_limit of Tx_rollup.t
    | Message_size_exceeds_limit
    | Too_many_inboxes
    | Too_many_commitments
    | Too_many_withdrawals
    | Wrong_batch_count
    | Commitment_too_early of {
        provided: Tx_rollup_level.t;
        expected: Tx_rollup_level.t;
      }
    | Level_already_has_commitment of Tx_rollup_level.t
    | Wrong_inbox_hash
    | Bond_does_not_exist of public_key_hash
    | Bond_in_use of public_key_hash
    | No_uncommitted_inbox
    | No_commitment_to_finalize
    | No_commitment_to_remove
    | Invalid_committer
    | Remove_commitment_too_early
    | Commitment_does_not_exist of Tx_rollup_level.t
    | Wrong_predecessor_hash of {
        provided: Tx_rollup_commitment_hash.t option;
        expected: Tx_rollup_commitment_hash.t option;
      }
    | Internal_error of string
    | Wrong_message_position of {
        level: Tx_rollup_level.t;
        position: int;
        length: int;
      }
    | Wrong_path_depth of {
        kind: [`Inbox | `Commitment];
        provided: int;
        limit: int;
      }
    | Wrong_message_path of {expected: Tx_rollup_inbox.Merkle.root}
    | No_finalized_commitment_for_level of {
        level: Tx_rollup_level.t;
        window: (Tx_rollup_level.t * Tx_rollup_level.t) option;
      }
    | Withdraw_invalid_path
    | Withdraw_already_consumed
    | Withdrawals_invalid_path
    | Withdrawals_already_dispatched
    | Cannot_reject_level of {
        provided: Tx_rollup_level.t;
        accepted_range: (Tx_rollup_level.t * Tx_rollup_level.t) option;
      }
    | Wrong_rejection_hash of {
        provided: Tx_rollup_message_result_hash.t;
        expected: [
          | `Valid_path of Tx_rollup_commitment.Merkle.h * int
          | `Hash of Tx_rollup_message_result_hash.t
        ];
      }
    | Proof_undecodable
    | Proof_failed_to_reject
    | Proof_produced_rejected_state
    | Proof_invalid_before of {
        agreed: Context_hash.t;
        provided: Context_hash.t;
      }
    | No_withdrawals_to_dispatch

  val check_path_depth :
    [`Inbox | `Commitment] -> int -> count_limit: int -> unit tzresult
end

(** This is a forward declaration to avoid circular dependencies.
    Use module [Sc_rollup] instead whenever possible.
    TODO : find a better way to resolve the circular dependency
           https://gitlab.com/tezos/tezos/-/issues/3147 *)
module Sc_rollup_repr : sig
  module Address : S.HASH

  type t = Address.t
end

(** This module re-exports definitions from {!Bond_id_repr}. *)
module Bond_id : sig
  type t =
    | Tx_rollup_bond_id of Tx_rollup.t
    | Sc_rollup_bond_id of Sc_rollup_repr.t

  val pp : Format.formatter -> t -> unit

  val compare : t -> t -> int

  (** This module discloses definitions that are only useful for tests and must
      not be used otherwise. *)
  module Internal_for_tests : sig
    val fold_on_bond_ids :
      context ->
      Contract.t ->
      order: [`Sorted | `Undefined] ->
      init: 'a ->
      f: (t -> 'a -> 'a Lwt.t) ->
      'a Lwt.t
  end
end

(** This module re-exports definitions from {!Zk_rollup_repr} and
    {!Zk_rollup_storage}. *)
module Zk_rollup : sig
  module Address : S.HASH

  type t = Address.t

  type scalar := Bls.Primitive.Fr.t

  val to_scalar : t -> scalar

  (** This module re-exports definitions from {!Zk_rollup_state_repr}. *)
  module State : sig
    type t = scalar array

    val encoding : t Data_encoding.t
  end

  (** This module re-exports definitions from {!Zk_rollup_account_repr}. *)
  module Account : sig
    module SMap : Map.S with type key = string

    type static = {
      public_parameters: Plonk.public_parameters;
      state_length: int;
      circuits_info: [`Public | `Private | `Fee] SMap.t;
      nb_ops: int;
    }

    type dynamic = {
      state: State.t;
      paid_l2_operations_storage_space: Z.t;
      used_l2_operations_storage_space: Z.t;
    }

    type t = {static: static; dynamic: dynamic}

    val encoding : t Data_encoding.t

    val circuits_info_encoding :
      [`Public | `Private | `Fee] SMap.t Data_encoding.t
  end

  (** This module re-exports definitions from {!Zk_rollup_operation_repr}. *)
  module Operation : sig
    type price = {id: Ticket_hash.t; amount: Z.t}

    type t = {
      op_code: int;
      price: price;
      l1_dst: Signature.Public_key_hash.t;
      rollup_id: Address.t;
      payload: scalar array;
    }

    val encoding : t Data_encoding.t

    val to_scalar_array : t -> scalar array
  end

  module Ticket : sig
    type t = {contents: Script.expr; ty: Script.expr; ticketer: Contract.t}

    val encoding : t Data_encoding.t
  end

  module Circuit_public_inputs : sig
    type pending_op_public_inputs = {
      old_state: State.t;
      new_state: State.t;
      fee: scalar;
      exit_validity: bool;
      zk_rollup: t;
      l2_op: Operation.t;
    }

    type private_batch_public_inputs = {
      old_state: State.t;
      new_state: State.t;
      fees: scalar;
      zk_rollup: t;
    }

    type fee_public_inputs = {
      old_state: State.t;
      new_state: State.t;
      fees: scalar;
    }

    type t =
      | Pending_op of pending_op_public_inputs
      | Private_batch of private_batch_public_inputs
      | Fee of fee_public_inputs

    val to_scalar_array : t -> scalar array
  end

  module Update : sig
    type op_pi = {new_state: State.t; fee: scalar; exit_validity: bool}

    type private_inner_pi = {new_state: State.t; fees: scalar}

    type fee_pi = {new_state: State.t}

    type t = {
      pending_pis: (string * op_pi) list;
      private_pis: (string * private_inner_pi) list;
      fee_pi: fee_pi;
      proof: Plonk.proof;
    }

    val encoding : t Data_encoding.t
  end

  type pending_list =
    | Empty of {next_index: int64}
    | Pending of {next_index: int64; length: int}

  val pending_list_encoding : pending_list Data_encoding.t

  val in_memory_size : t -> Cache_memory_helpers.sint

  val originate :
    context ->
    Account.static ->
    init_state: State.t ->
    (context * Address.t * Z.t) tzresult Lwt.t

  val add_to_pending :
    context ->
    Address.t ->
    (Operation.t * Ticket_hash.t option) list ->
    (context * Z.t) tzresult Lwt.t

  val get_pending_length :
    context -> Address.t -> (context * int) tzresult Lwt.t

  val get_prefix :
    context ->
    Address.t ->
    int ->
    (context * (Operation.t * Ticket_hash.t option) list) tzresult Lwt.t

  val update :
    context ->
    Address.t ->
    pending_to_drop: int ->
    new_account: Account.t ->
    context tzresult Lwt.t

  val account : context -> t -> (context * Account.t) tzresult Lwt.t

  val pending_list : context -> t -> (context * pending_list) tzresult Lwt.t

  val pending_op :
    context ->
    t ->
    Int64.t ->
    (context * (Operation.t * Ticket_hash.t option)) tzresult Lwt.t

  val assert_exist : context -> t -> context tzresult Lwt.t

  val exists : context -> t -> (context * bool) tzresult Lwt.t

  module Errors : sig
    type error +=
      | Deposit_as_external
      | Invalid_deposit_amount
      | Invalid_deposit_ticket
      | Wrong_deposit_parameters
      | Ticket_payload_size_limit_exceeded of {
          payload_size: Saturation_repr.may_saturate Saturation_repr.t;
          limit: int;
        }
      | Invalid_verification
      | Invalid_circuit
      | Inconsistent_state_update
      | Pending_bound
  end

  module Internal_for_tests : sig
    val originated_zk_rollup : Origination_nonce.Internal_for_tests.t -> t
  end
end

(** This module re-exports definitions from {!Receipt_repr}. *)
module Receipt : sig
  type balance =
    | Contract of Contract.t
    | Block_fees
    | Deposits of public_key_hash
    | Nonce_revelation_rewards
    | Double_signing_evidence_rewards
    | Endorsing_rewards
    | Baking_rewards
    | Baking_bonuses
    | Storage_fees
    | Double_signing_punishments
    | Lost_endorsing_rewards of public_key_hash * bool * bool
    | Liquidity_baking_subsidies
    | Burned
    | Commitments of Blinded_public_key_hash.t
    | Bootstrap
    | Invoice
    | Initial_commitments
    | Minted
    | Frozen_bonds of Contract.t * Bond_id.t
    | Tx_rollup_rejection_punishments
    | Tx_rollup_rejection_rewards
    | Sc_rollup_refutation_punishments
    | Sc_rollup_refutation_rewards

  val compare_balance : balance -> balance -> int

  type balance_update = Debited of Tez.t | Credited of Tez.t

  type update_origin =
    | Block_application
    | Protocol_migration
    | Subsidy
    | Simulation

  val compare_update_origin : update_origin -> update_origin -> int

  type balance_updates = (balance * balance_update * update_origin) list

  val balance_updates_encoding : balance_updates Data_encoding.t

  val group_balance_updates : balance_updates -> balance_updates tzresult
end

(** This module re-exports definitions from {!Delegate_consensus_key}. *)
module Consensus_key : sig
  type pk = {
    delegate: Signature.Public_key_hash.t;
    consensus_pk: Signature.Public_key.t;
    consensus_pkh: Signature.Public_key_hash.t;
  }

  type t = {
    delegate: Signature.Public_key_hash.t;
    consensus_pkh: Signature.Public_key_hash.t;
  }

  val zero : t

  val pp : Format.formatter -> t -> unit

  val pkh : pk -> t
end

(** This module re-exports definitions from {!Delegate_storage},
   {!Delegate_consensus_key}, {!Delegate_missed_endorsements_storage},
   {!Delegate_slashed_deposits_storage}, {!Delegate_cycles}. *)
module Delegate : sig
  val check_not_tz4 : Signature.public_key_hash -> unit tzresult

  val frozen_deposits_limit :
    context -> public_key_hash -> Tez.t option tzresult Lwt.t

  val set_frozen_deposits_limit :
    context -> public_key_hash -> Tez.t option -> context Lwt.t

  val fold :
    context ->
    order: [`Sorted | `Undefined] ->
    init: 'a ->
    f: (public_key_hash -> 'a -> 'a Lwt.t) ->
    'a Lwt.t

  val list : context -> public_key_hash list Lwt.t

  val drain :
    context ->
    delegate: public_key_hash ->
    destination: public_key_hash ->
    (context * bool * Tez.t * Receipt.balance_updates) tzresult Lwt.t

  type participation_info = {
    expected_cycle_activity: int;
    minimal_cycle_activity: int;
    missed_slots: int;
    missed_levels: int;
    remaining_allowed_missed_slots: int;
    expected_endorsing_rewards: Tez.t;
  }

  val participation_info :
    context -> public_key_hash -> participation_info tzresult Lwt.t

  val cycle_end :
    context ->
    Cycle.t ->
    (context * Receipt.balance_updates * public_key_hash list) tzresult Lwt.t

  val already_slashed_for_double_endorsing :
    context -> public_key_hash -> Level.t -> bool tzresult Lwt.t

  val already_slashed_for_double_baking :
    context -> public_key_hash -> Level.t -> bool tzresult Lwt.t

  val punish_double_endorsing :
    context ->
    public_key_hash ->
    Level.t ->
    (context * Tez.t * Receipt.balance_updates) tzresult Lwt.t

  val punish_double_baking :
    context ->
    public_key_hash ->
    Level.t ->
    (context * Tez.t * Receipt.balance_updates) tzresult Lwt.t

  val full_balance : context -> public_key_hash -> Tez.t tzresult Lwt.t

  type level_participation = Participated | Didn't_participate

  val record_baking_activity_and_pay_rewards_and_fees :
    context ->
    payload_producer: public_key_hash ->
    block_producer: public_key_hash ->
    baking_reward: Tez.t ->
    reward_bonus: Tez.t option ->
    (context * Receipt.balance_updates) tzresult Lwt.t

  val record_endorsing_participation :
    context ->
    delegate: public_key_hash ->
    participation: level_participation ->
    endorsing_power: int ->
    context tzresult Lwt.t

  type deposits = {initial_amount: Tez.t; current_amount: Tez.t}

  val frozen_deposits : context -> public_key_hash -> deposits tzresult Lwt.t

  val staking_balance : context -> public_key_hash -> Tez.t tzresult Lwt.t

  (** See {!Contract_delegate_storage.delegated_contracts}. *)
  val delegated_contracts : context -> public_key_hash -> Contract.t list Lwt.t

  val delegated_balance : context -> public_key_hash -> Tez.t tzresult Lwt.t

  val registered : context -> public_key_hash -> bool Lwt.t

  val deactivated : context -> public_key_hash -> bool tzresult Lwt.t

  (** See {!Delegate_activation_storage.last_cycle_before_deactivation}. *)
  val last_cycle_before_deactivation :
    context -> public_key_hash -> Cycle.t tzresult Lwt.t

  module Consensus_key : sig
    val check_not_tz4 : Signature.public_key -> unit tzresult

    val active_pubkey :
      context -> public_key_hash -> Consensus_key.pk tzresult Lwt.t

    val pending_updates :
      context ->
      public_key_hash ->
      (Cycle.t * public_key_hash) list tzresult Lwt.t

    val register_update :
      context -> public_key_hash -> public_key -> context tzresult Lwt.t
  end

  (** See {!Stake_storage.prepare_stake_distribution}. *)
  val prepare_stake_distribution : context -> context tzresult Lwt.t
end

(** This module re-exports definitions from {!Voting_period_repr} and
    {!Voting_period_storage}. *)
module Voting_period : sig
  type kind = Proposal | Exploration | Cooldown | Promotion | Adoption

  val kind_encoding : kind Data_encoding.encoding

  val pp_kind : Format.formatter -> kind -> unit

  (* This type should be abstract *)
  type voting_period = private {
    index: int32;
    kind: kind;
    start_position: int32;
  }

  type t = voting_period

  include BASIC_DATA with type t := t

  val encoding : voting_period Data_encoding.t

  val pp : Format.formatter -> voting_period -> unit

  val reset : context -> context tzresult Lwt.t

  val succ : context -> context tzresult Lwt.t

  val get_current : context -> voting_period tzresult Lwt.t

  val get_current_kind : context -> kind tzresult Lwt.t

  val is_last_block : context -> bool tzresult Lwt.t

  type info = {voting_period: t; position: int32; remaining: int32}

  val info_encoding : info Data_encoding.t

  val pp_info : Format.formatter -> info -> unit

  val get_rpc_current_info : context -> info tzresult Lwt.t

  val get_rpc_succ_info : context -> info tzresult Lwt.t

  module Testnet_dictator : sig
    (** See {!Voting_period_storage.Testnet_dictator.overwrite_current_kind}. *)
    val overwrite_current_kind :
      context -> Chain_id.t -> Voting_period_repr.kind -> context tzresult Lwt.t
  end
end

(** This module re-exports definitions from {!Vote_repr} and {!Vote_storage}. *)
module Vote : sig
  type proposal = Protocol_hash.t

  (** See {!Vote_storage.get_delegate_proposal_count}. *)
  val get_delegate_proposal_count :
    context -> public_key_hash -> int tzresult Lwt.t

  (** See {!Vote_storage.set_delegate_proposal_count}. *)
  val set_delegate_proposal_count :
    context -> public_key_hash -> int -> context Lwt.t

  (** See {!Vote_storage.has_proposed}. *)
  val has_proposed : context -> public_key_hash -> proposal -> bool Lwt.t

  (** See {!Vote_storage.add_proposal}. *)
  val add_proposal : context -> public_key_hash -> proposal -> context Lwt.t

  val get_proposals : context -> int64 Protocol_hash.Map.t tzresult Lwt.t

  val clear_proposals : context -> context Lwt.t

  val listings_encoding : (public_key_hash * int64) list Data_encoding.t

  val update_listings : context -> context tzresult Lwt.t

  val in_listings : context -> public_key_hash -> bool Lwt.t

  val get_listings : context -> (public_key_hash * int64) list Lwt.t

  type ballot = Yay | Nay | Pass

  val equal_ballot : ballot -> ballot -> bool

  val pp_ballot : Format.formatter -> ballot -> unit

  type delegate_info = {
    voting_power: Int64.t option;
    current_ballot: ballot option;
    current_proposals: Protocol_hash.t list;
    remaining_proposals: int;
  }

  val pp_delegate_info : Format.formatter -> delegate_info -> unit

  val delegate_info_encoding : delegate_info Data_encoding.t

  val get_delegate_info :
    context -> public_key_hash -> delegate_info tzresult Lwt.t

  val get_voting_power_free : context -> public_key_hash -> int64 tzresult Lwt.t

  val get_voting_power :
    context -> public_key_hash -> (context * int64) tzresult Lwt.t

  val get_total_voting_power_free : context -> int64 tzresult Lwt.t

  val get_total_voting_power : context -> (context * int64) tzresult Lwt.t

  val ballot_encoding : ballot Data_encoding.t

  type ballots = {yay: int64; nay: int64; pass: int64}

  (** See {!Vote_storage.ballots_zero}. *)
  val ballots_zero : ballots

  (** See {!Vote_storage.ballots_encoding} *)
  val ballots_encoding : ballots Data_encoding.t

  (** See {!Vote_storage.equal_ballots}. *)
  val equal_ballots : ballots -> ballots -> bool

  (** See {!Vote_storage.pp_ballots}. *)
  val pp_ballots : Format.formatter -> ballots -> unit

  val has_recorded_ballot : context -> public_key_hash -> bool Lwt.t

  val record_ballot :
    context -> public_key_hash -> ballot -> context tzresult Lwt.t

  val get_ballots : context -> ballots tzresult Lwt.t

  val get_ballot_list : context -> (public_key_hash * ballot) list Lwt.t

  val clear_ballots : context -> context Lwt.t

  val get_current_quorum : context -> int32 tzresult Lwt.t

  val get_participation_ema : context -> int32 tzresult Lwt.t

  val set_participation_ema : context -> int32 -> context tzresult Lwt.t

  (** See {!Vote_storage.current_proposal_exists}. *)
  val current_proposal_exists : context -> bool Lwt.t

  (** See {!Vote_storage.get_current_proposal}. *)
  val get_current_proposal : context -> proposal tzresult Lwt.t

  (** See {!Vote_storage.find_current_proposal}. *)
  val find_current_proposal : context -> proposal option tzresult Lwt.t

  (** See {!Vote_storage.init_current_proposal}. *)
  val init_current_proposal : context -> proposal -> context tzresult Lwt.t

  (** See {!Vote_storage.clear_current_proposal}. *)
  val clear_current_proposal : context -> context Lwt.t
end

(** This module exposes definitions for the data-availability layer. *)
module Dal : sig
  type parameters = Dal.parameters = {
    redundancy_factor: int;
    page_size: int;
    slot_size: int;
    number_of_shards: int;
  }

  type cryptobox

  val make : context -> cryptobox tzresult

  val number_of_slots : context -> int

  (** This module re-exports definitions from {!Dal_slot_index_repr}. *)
  module Slot_index : sig
    type t

    val pp : Format.formatter -> t -> unit

    val zero : t

    val encoding : t Data_encoding.t

    val of_int_opt : int -> t option

    val of_int : int -> t tzresult

    val to_int : t -> int

    val to_int_list : t list -> int list

    val compare : t -> t -> int

    val equal : t -> t -> bool

    val slots_range : lower: int -> upper: int -> t list tzresult

    val slots_range_opt : lower: int -> upper: int -> t list option
  end

  (** This module re-exports definitions from {!Dal_attestation_repr} and
      {!Raw_context.Dal}. *)
  module Attestation : sig
    type t

    type operation = {
      attestor: public_key_hash;
      attestation: t;
      level: Raw_level.t;
    }

    type shard_index = int

    module Shard_map : Map.S with type key = shard_index

    val encoding : t Data_encoding.t

    val empty : t

    val is_attested : t -> Slot_index.t -> bool

    val occupied_size_in_bits : t -> int

    val expected_size_in_bits : max_index: Slot_index.t -> int

    val shards_of_attestor :
      context -> attestor: public_key_hash -> shard_index list option

    val record_attested_shards : context -> t -> int list -> context

    type committee = {
      pkh_to_shards: (shard_index * int) Signature.Public_key_hash.Map.t;
      shard_to_pkh: Signature.Public_key_hash.t Shard_map.t;
    }

    val compute_committee :
      context ->
      (Slot.t -> (context * Signature.Public_key_hash.t) tzresult Lwt.t) ->
      committee tzresult Lwt.t

    val init_committee : context -> committee -> context
  end

  type slot_id = {published_level: Raw_level.t; index: Slot_index.t}

  module Page : sig
    type content = bytes

    val pages_per_slot : parameters -> int

    module Index : sig
      type t = int

      val encoding : int Data_encoding.t

      val pp : Format.formatter -> int -> unit

      val compare : int -> int -> int

      val equal : int -> int -> bool
    end

    type t = {slot_id: slot_id; page_index: Index.t}

    val content_encoding : content Data_encoding.t

    type proof = Dal.page_proof

    val encoding : t Data_encoding.t

    val pp : Format.formatter -> t -> unit

    val equal : t -> t -> bool
  end

  (** This module re-exports definitions from {!Dal_slot_repr},
      {!Dal_slot_storage} and {!Raw_context.Dal}. *)
  module Slot : sig
    (** This module re-exports definitions from {!Dal_slot_repr.Header}. *)
    module Commitment : sig
      type t = Dal.commitment

      val encoding : t Data_encoding.t

      val zero : t
    end

    module Commitment_proof : sig
      type t = Dal.commitment_proof

      val encoding : t Data_encoding.t

      val zero : t
    end

    module Header : sig
      type id = slot_id = {published_level: Raw_level.t; index: Slot_index.t}

      type t = {id: id; commitment: Commitment.t}

      val id_encoding : id Data_encoding.t

      val encoding : t Data_encoding.t

      val pp_id : Format.formatter -> id -> unit

      val pp : Format.formatter -> t -> unit

      val equal : t -> t -> bool
    end

    val register_slot_header : context -> Header.t -> context tzresult

    val find_slot_headers :
      context -> Raw_level.t -> Header.t list option tzresult Lwt.t

    val finalize_current_slot_headers : context -> context Lwt.t

    val finalize_pending_slot_headers :
      context -> (context * Attestation.t) tzresult Lwt.t
  end

  module Operations : sig
    module Publish_slot_header : sig
      type t = {
        published_level: Raw_level.t;
        slot_index: Slot_index.t;
        commitment: Slot.Commitment.t;
        commitment_proof: Slot.Commitment_proof.t;
      }

      val encoding : t Data_encoding.t

      val pp : Format.formatter -> t -> unit

      val check_level : current_level: Raw_level.t -> t -> unit tzresult

      val slot_header :
        cryptobox: cryptobox ->
        number_of_slots: int ->
        current_level: Raw_level.t ->
        t ->
        Slot.Header.t tzresult
    end
  end

  module Slots_history : sig
    type t

    type hash

    (* FIXME/DAL: https://gitlab.com/tezos/tezos/-/issues/3766
       Do we need to export this? *)
    val genesis : t

    val equal : t -> t -> bool

    val encoding : t Data_encoding.t

    val hash : t -> hash

    module History_cache :
      Bounded_history_repr.S with type key = hash and type value = t

    val add_confirmed_slot_headers_no_cache :
      t -> Slot.Header.t list -> t tzresult

    val add_confirmed_slot_headers :
      t ->
      History_cache.t ->
      Slot.Header.t list ->
      (t * History_cache.t) tzresult

    type proof
  end

  module Slots_storage : sig
    val get_slot_headers_history : context -> Slots_history.t tzresult Lwt.t
  end
end

(** This module re-exports definitions from {!Dal_errors_repr}. *)
module Dal_errors : sig
  (* DAL/FIXME: https://gitlab.com/tezos/tezos/-/issues/3168
     do not expose these errors and return them in functions
     from Dal_slot_repr or Dal_attestation_repr. *)
  type error +=
    | Dal_feature_disabled
    | Dal_slot_index_above_hard_limit
    | Dal_attestation_unexpected_size of {expected: int; got: int}
    | Dal_publish_slot_header_future_level of {
        provided: Raw_level.t;
        expected: Raw_level.t;
      }
    | Dal_publish_slot_header_past_level of {
        provided: Raw_level.t;
        expected: Raw_level.t;
      }
    | Dal_publish_slot_header_invalid_index of {
        given: Dal.Slot_index.t;
        maximum: Dal.Slot_index.t;
      }
    | Dal_publish_slot_header_candidate_with_low_fees of {proposed_fees: Tez.t}
    | Dal_attestation_size_limit_exceeded of {maximum_size: int; got: int}
    | Dal_publish_slot_header_duplicate of {slot_header: Dal.Slot.Header.t}
    | Dal_publish_slot_header_invalid_proof of {
        commitment: Dal.Slot.Commitment.t;
        commitment_proof: Dal.Slot.Commitment_proof.t;
      }
    | Dal_data_availability_attestor_not_in_committee of {
        attestor: Signature.Public_key_hash.t;
        level: Level.t;
      }
    | Dal_operation_for_old_level of {
        current: Raw_level.t;
        given: Raw_level.t;
      }
    | Dal_operation_for_future_level of {
        current: Raw_level.t;
        given: Raw_level.t;
      }
    | Dal_cryptobox_error of {explanation: string}
end

(** This module re-exports definitions from {!Sc_rollup_storage} and
    {!Sc_rollup_repr}. *)
module Sc_rollup : sig
  (** See {!Sc_rollup_tick_repr}. *)
  module Tick : sig
    type t

    val initial : t

    val next : t -> t

    val jump : t -> Z.t -> t

    val distance : t -> t -> Z.t

    val of_int : int -> t option

    val to_int : t -> int option

    val of_z : Z.t -> t

    val to_z : t -> Z.t

    val encoding : t Data_encoding.t

    val pp : Format.formatter -> t -> unit

    include Compare.S with type t := t

    module Map : Map.S with type key = t
  end

  module Address = Sc_rollup_repr.Address

  type t = Sc_rollup_repr.t

  type rollup := t

  val in_memory_size : t -> Cache_memory_helpers.sint

  val must_exist : context -> t -> context tzresult Lwt.t

  module Staker : S.SIGNATURE_PUBLIC_KEY_HASH with type t = public_key_hash

  module State_hash : sig
    include S.HASH

    val context_hash_to_state_hash : Context_hash.t -> t

    type unreachable = |

    val hash_bytes : unreachable -> t

    val hash_string : unreachable -> t
  end

  (** See {!Sc_rollup_metadata_repr}. *)
  module Metadata : sig
    type t = {address: rollup; origination_level: Raw_level.t}

    val pp : Format.formatter -> t -> unit

    val equal : t -> t -> bool

    val encoding : t Data_encoding.t
  end

  (** See {!Sc_rollup_inbox_message_repr}. *)
  module Inbox_message : sig
    type internal_inbox_message =
      | Transfer of {
          payload: Script.expr;
          sender: Contract_hash.t;
          source: public_key_hash;
          destination: t;
        }
      | Start_of_level
      | End_of_level
      | Info_per_level of {
          predecessor_timestamp: Time.t;
          predecessor: Block_hash.t;
        }

    type t = Internal of internal_inbox_message | External of string

    type serialized

    val encoding : t Data_encoding.t

    val unsafe_of_string : string -> serialized

    val unsafe_to_string : serialized -> string

    val serialize : t -> serialized tzresult

    val deserialize : serialized -> t tzresult

    module Hash : S.HASH

    val hash_serialized_message : serialized -> Hash.t
  end

  module Inbox_merkelized_payload_hashes : sig
    module Hash : S.HASH

    type t

    val encoding : t Data_encoding.t

    val equal : t -> t -> bool

    val hash : t -> Hash.t

    val get_payload_hash : t -> Inbox_message.Hash.t

    val get_index : t -> Z.t

    type merkelized_and_payload = {
      merkelized: t;
      payload: Inbox_message.serialized;
    }

    module History : sig
      include Bounded_history_repr.S with
      type key = Hash.t
      and type value = merkelized_and_payload

      val no_history : t
    end

    val genesis :
      History.t -> Inbox_message.serialized -> (History.t * t) tzresult

    val add_payload :
      History.t -> t -> Inbox_message.serialized -> (History.t * t) tzresult

    type proof = private t list

    val proof_encoding : proof Data_encoding.t

    val produce_proof :
      History.t -> index: Z.t -> t -> (merkelized_and_payload * proof) option

    val verify_proof : proof -> (t * t) tzresult

    module Internal_for_tests : sig
      val find_predecessor_payload : History.t -> index: Z.t -> t -> t option

      val make_proof : t list -> proof
    end
  end

  type inbox_message = {
    inbox_level: Raw_level.t;
    message_counter: Z.t;
    payload: Inbox_message.serialized;
  }

  type reveal_data =
    | Raw_data of string
    | Metadata of Metadata.t
    | Dal_page of Dal.Page.content option

  type input = Inbox_message of inbox_message | Reveal of reveal_data

  val pp_inbox_message : Format.formatter -> inbox_message -> unit

  val inbox_message_equal : inbox_message -> inbox_message -> bool

  val pp_reveal_data : Format.formatter -> reveal_data -> unit

  val pp_input : Format.formatter -> input -> unit

  val input_equal : input -> input -> bool

  val input_encoding : input Data_encoding.t

  module Input_hash : S.HASH

  type reveal =
    | Reveal_raw_data of Sc_rollup_reveal_hash.t
    | Reveal_metadata
    | Request_dal_page of Dal.Page.t

  type input_request =
    | No_input_required
    | Initial
    | First_after of Raw_level.t * Z.t
    | Needs_reveal of reveal

  val input_request_encoding : input_request Data_encoding.t

  val input_request_equal : input_request -> input_request -> bool

  val pp_input_request : Format.formatter -> input_request -> unit

  module Inbox : sig
    type t

    val pp : Format.formatter -> t -> unit

    val encoding : t Data_encoding.t

    val equal : t -> t -> bool

    val inbox_level : t -> Raw_level.t

    type history_proof

    val old_levels_messages : t -> history_proof

    val equal_history_proof : history_proof -> history_proof -> bool

    val pp_history_proof : Format.formatter -> history_proof -> unit

    module Hash : S.HASH

    module History :
      Bounded_history_repr.S with
      type key = Hash.t
      and type value = history_proof

    type serialized_proof

    val serialized_proof_encoding : serialized_proof Data_encoding.t

    val add_all_messages :
      predecessor_timestamp: Time.t ->
      predecessor: Block_hash.t ->
      History.t ->
      t ->
      Inbox_message.t list ->
      (Inbox_merkelized_payload_hashes.History.t
      * History.t
      * t
      * Inbox_merkelized_payload_hashes.t
      * Inbox_message.t list) tzresult

    val add_messages_no_history :
      Inbox_message.serialized list ->
      Inbox_merkelized_payload_hashes.t ->
      Inbox_merkelized_payload_hashes.t tzresult

    val take_snapshot : t -> history_proof

    type inclusion_proof

    val inclusion_proof_encoding : inclusion_proof Data_encoding.t

    val pp_inclusion_proof : Format.formatter -> inclusion_proof -> unit

    val verify_inclusion_proof :
      inclusion_proof -> history_proof -> history_proof tzresult

    type proof

    val pp_proof : Format.formatter -> proof -> unit

    val to_serialized_proof : proof -> serialized_proof

    val of_serialized_proof : serialized_proof -> proof option

    val verify_proof :
      Raw_level.t * Z.t ->
      history_proof ->
      proof ->
      inbox_message option tzresult

    val produce_proof :
      get_payloads_history:
      (Inbox_merkelized_payload_hashes.Hash.t ->
      Inbox_merkelized_payload_hashes.History.t Lwt.t) ->
      get_history: (Hash.t -> history_proof option Lwt.t) ->
      history_proof ->
      Raw_level.t * Z.t ->
      (proof * inbox_message option) tzresult Lwt.t

    val finalize_inbox_level_no_history :
      t -> Inbox_merkelized_payload_hashes.t -> t tzresult

    val init_witness_no_history : Inbox_merkelized_payload_hashes.t

    val add_info_per_level_no_history :
      predecessor_timestamp: Time.t ->
      predecessor: Block_hash.t ->
      Inbox_merkelized_payload_hashes.t ->
      Inbox_merkelized_payload_hashes.t tzresult

    val genesis :
      predecessor_timestamp: Time.t ->
      predecessor: Block_hash.t ->
      Raw_level.t ->
      t tzresult

    module Internal_for_tests : sig
      val produce_inclusion_proof :
        (Hash.t -> history_proof option Lwt.t) ->
        history_proof ->
        Raw_level.t ->
        (inclusion_proof * history_proof) tzresult Lwt.t

      val serialized_proof_of_string : string -> serialized_proof
    end

    val add_external_messages : context -> string list -> context tzresult Lwt.t

    val add_deposit :
      context ->
      payload: Script.expr ->
      sender: Contract_hash.t ->
      source: public_key_hash ->
      destination: rollup ->
      context tzresult Lwt.t

    val finalize_inbox_level : context -> context Lwt.t

    val add_info_per_level :
      predecessor: Block_hash.t -> context -> context Lwt.t

    val get_inbox : context -> (t * context) tzresult Lwt.t
  end

  module Outbox : sig
    (** See {!Sc_rollup_outbox_message_repr}. *)
    module Message : sig
      type transaction = {
        unparsed_parameters: Script.expr;
        destination: Contract_hash.t;
        entrypoint: Entrypoint.t;
      }

      type t = Atomic_transaction_batch of {transactions: transaction list}

      type serialized

      val unsafe_of_string : string -> serialized

      val unsafe_to_string : serialized -> string

      val deserialize : serialized -> t tzresult

      val serialize : t -> serialized tzresult
    end

    val record_applied_message :
      context ->
      t ->
      Raw_level.t ->
      message_index: int ->
      (Z.t * context) tzresult Lwt.t
  end

  type output = {
    outbox_level: Raw_level.t;
    message_index: Z.t;
    message: Outbox.Message.t;
  }

  val output_encoding : output Data_encoding.t

  module Dissection_chunk : sig
    type t = {state_hash: State_hash.t option; tick: Tick.t}

    val equal : t -> t -> bool

    val pp : Format.formatter -> t -> unit

    type error +=
      | Dissection_number_of_sections_mismatch of {expected: Z.t; given: Z.t}
      | Dissection_invalid_number_of_sections of Z.t
      | Dissection_start_hash_mismatch of {
          expected: State_hash.t option;
          given: State_hash.t option;
        }
      | Dissection_stop_hash_mismatch of State_hash.t option
      | Dissection_edge_ticks_mismatch of {
          dissection_start_tick: Tick.t;
          dissection_stop_tick: Tick.t;
          chunk_start_tick: Tick.t;
          chunk_stop_tick: Tick.t;
        }
      | Dissection_ticks_not_increasing
      | Dissection_invalid_distribution of Z.t
      | Dissection_invalid_successive_states_shape
  end

  module PVM : sig
    type boot_sector = string

    module type S = sig
      val parse_boot_sector : string -> boot_sector option

      val pp_boot_sector : Format.formatter -> boot_sector -> unit

      type state

      val pp : state -> (Format.formatter -> unit -> unit) Lwt.t

      type context

      type hash = State_hash.t

      type proof

      val proof_encoding : proof Data_encoding.t

      val proof_start_state : proof -> hash

      val proof_stop_state : proof -> hash

      val state_hash : state -> hash Lwt.t

      val initial_state : empty: state -> state Lwt.t

      val install_boot_sector : state -> string -> state Lwt.t

      val is_input_state : state -> input_request Lwt.t

      val set_input : input -> state -> state Lwt.t

      val eval : state -> state Lwt.t

      val verify_proof : input option -> proof -> input_request tzresult Lwt.t

      val produce_proof :
        context -> input option -> state -> proof tzresult Lwt.t

      val verify_origination_proof : proof -> string -> bool Lwt.t

      val produce_origination_proof : context -> string -> proof tzresult Lwt.t

      type output_proof

      val output_proof_encoding : output_proof Data_encoding.t

      val output_of_output_proof : output_proof -> output

      val state_of_output_proof : output_proof -> State_hash.t

      val verify_output_proof : output_proof -> bool Lwt.t

      val produce_output_proof :
        context -> state -> output -> (output_proof, error) result Lwt.t

      val check_dissection :
        default_number_of_sections: int ->
        start_chunk: Dissection_chunk.t ->
        stop_chunk: Dissection_chunk.t ->
        Dissection_chunk.t list ->
        unit tzresult

      val get_current_level : state -> Raw_level.t option Lwt.t

      module Internal_for_tests : sig
        val insert_failure : state -> state Lwt.t
      end
    end

    type ('state, 'proof, 'output) implementation =
      (
        module S with
        type state = 'state
        and type proof = 'proof
        and type output_proof = 'output
      )

    type t =
      Packed : ('state, 'proof, 'output) implementation -> t
    [@@unboxed]
  end

  module Kind : sig
    type t = Example_arith | Wasm_2_0_0

    val encoding : t Data_encoding.t

    val pp : Format.formatter -> t -> unit

    val pvm_of : t -> PVM.t

    val all : t list

    val of_string : string -> t option

    val to_string : t -> string

    val equal : t -> t -> bool
  end

  module ArithPVM : sig
    module type P = sig
      module Tree :
        Context.TREE with type key = string list and type value = bytes

      type tree = Tree.tree

      val hash_tree : tree -> State_hash.t

      type proof

      val proof_encoding : proof Data_encoding.t
      val proof_before : proof -> State_hash.t
      val proof_after : proof -> State_hash.t
      val verify_proof :
        proof -> (tree -> (tree * 'a) Lwt.t) -> (tree * 'a) option Lwt.t
      val produce_proof :
        Tree.t ->
        tree ->
        (tree -> (tree * 'a) Lwt.t) ->
        (proof * 'a) option Lwt.t
    end

    module Make (C : P) : sig
      include PVM.S with
      type context = C.Tree.t
      and type state = C.tree
      and type proof = C.proof

      val get_tick : state -> Tick.t Lwt.t

      type status =
        | Halted
        | Waiting_for_input_message
        | Waiting_for_reveal
        | Waiting_for_metadata
        | Parsing
        | Evaluating

      val get_status : state -> status Lwt.t

      val get_outbox : Raw_level.t -> state -> output list Lwt.t
    end

    val reference_initial_state_hash : State_hash.t

    module Protocol_implementation :
      PVM.S with
      type context = Context.t
      and type state = Context.tree
      and type proof = Context.Proof.tree Context.Proof.t
  end

  module Wasm_2_0_0PVM : sig
    val ticks_per_snapshot : Z.t

    val outbox_validity_period : int32

    val outbox_message_limit : Z.t

    val well_known_reveal_preimage : string

    val well_known_reveal_hash : Sc_rollup_reveal_hash.t

    module type P = sig
      module Tree :
        Context.TREE with type key = string list and type value = bytes

      type tree = Tree.tree

      type proof

      val proof_encoding : proof Data_encoding.t

      val proof_before : proof -> State_hash.t

      val proof_after : proof -> State_hash.t

      val verify_proof :
        proof -> (tree -> (tree * 'a) Lwt.t) -> (tree * 'a) option Lwt.t

      val produce_proof :
        Tree.t ->
        tree ->
        (tree -> (tree * 'a) Lwt.t) ->
        (proof * 'a) option Lwt.t
    end

    module type Make_wasm = module type of Wasm_2_0_0.Make

    module Make (Wasm_backend : Make_wasm) (C : P) : sig
      include PVM.S with
      type context = C.Tree.t
      and type state = C.tree
      and type proof = C.proof

      val get_tick : state -> Tick.t Lwt.t

      type status =
        | Computing
        | Waiting_for_input_message
        | Waiting_for_reveal of reveal

      val get_status : state -> status Lwt.t

      val get_outbox : Raw_level.t -> state -> output list Lwt.t

      val produce_proof :
        context -> input option -> state -> proof tzresult Lwt.t
    end

    module Protocol_implementation :
      PVM.S with
      type context = Context.t
      and type state = Context.tree
      and type proof = Context.Proof.tree Context.Proof.t

    val reference_initial_state_hash : State_hash.t
  end

  module Number_of_ticks : sig
    include Bounded.S with type ocaml_type := int64

    val zero : t
  end

  module Commitment : sig
    module Hash : S.HASH

    type t = {
      compressed_state: State_hash.t;
      inbox_level: Raw_level.t;
      predecessor: Hash.t;
      number_of_ticks: Number_of_ticks.t;
    }

    val encoding : t Data_encoding.t

    val pp : Format.formatter -> t -> unit

    val hash_uncarbonated : t -> Hash.t

    val hash : context -> t -> (context * Hash.t) tzresult

    val genesis_commitment :
      origination_level: Raw_level.t -> genesis_state_hash: State_hash.t -> t

    type genesis_info = {level: Raw_level.t; commitment_hash: Hash.t}

    val genesis_info_encoding : genesis_info Data_encoding.t

    val get_commitment :
      context -> rollup -> Hash.t -> (t * context) tzresult Lwt.t

    val last_cemented_commitment_hash_with_level :
      context -> rollup -> (Hash.t * Raw_level.t * context) tzresult Lwt.t

    val check_if_commitments_are_related :
      context ->
      rollup ->
      descendant: Hash.t ->
      ancestor: Hash.t ->
      (bool * context) tzresult Lwt.t
  end

  val originate :
    context ->
    kind: Kind.t ->
    parameters_ty: Script.lazy_expr ->
    genesis_commitment: Commitment.t ->
    (t * Z.t * Commitment.Hash.t * context) tzresult Lwt.t

  val parameters_type :
    context -> t -> (Script.lazy_expr option * context) tzresult Lwt.t

  val kind : context -> t -> (context * Kind.t) tzresult Lwt.t

  module Errors : sig
    type error += Sc_rollup_does_not_exist of t
  end

  module Proof : sig
    type reveal_proof =
      | Raw_data_proof of string
      | Metadata_proof
      | Dal_page_proof of {
          page_id: Dal.Page.t;
          proof: Dal.Slots_history.proof;
        }

    type input_proof =
      | Inbox_proof of {
          level: Raw_level.t;
          message_counter: Z.t;
          proof: Inbox.serialized_proof;
        }
      | Reveal_proof of reveal_proof
      | First_inbox_message

    type 'proof t = {pvm_step: 'proof; input_proof: input_proof option}

    type serialized = private string

    val serialize_pvm_step :
      pvm: ('state, 'proof, 'output) PVM.implementation ->
      'proof ->
      serialized tzresult

    val unserialize_pvm_step :
      pvm: ('state, 'proof, 'output) PVM.implementation ->
      serialized ->
      'proof tzresult

    val serialized_encoding : serialized Data_encoding.t

    val encoding : serialized t Data_encoding.t

    module type PVM_with_context_and_state = sig
      include PVM.S

      val context : context

      val state : state

      val proof_encoding : proof Data_encoding.t

      val reveal : Sc_rollup_reveal_hash.t -> string option Lwt.t

      module Inbox_with_history : sig
        val inbox : Inbox.history_proof

        val get_history : Inbox.Hash.t -> Inbox.history_proof option Lwt.t

        val get_payloads_history :
          Inbox_merkelized_payload_hashes.Hash.t ->
          Inbox_merkelized_payload_hashes.History.t Lwt.t
      end

      module Dal_with_history : sig
        val confirmed_slots_history : Dal.Slots_history.t

        val get_history :
          Dal.Slots_history.hash -> Dal.Slots_history.t option Lwt.t

        val page_info : (Dal.Page.content * Dal.Page.proof) option

        val dal_parameters : Dal.parameters

        val dal_attestation_lag : int
      end
    end

    type error += Sc_rollup_proof_check of string

    val valid :
      pvm: ('state, 'proof, 'output) PVM.implementation ->
      metadata: Metadata.t ->
      Inbox.history_proof ->
      Raw_level.t ->
      Dal.Slots_history.t ->
      Dal.parameters ->
      dal_attestation_lag: int ->
      'proof t ->
      (input option * input_request) tzresult Lwt.t

    val produce :
      metadata: Metadata.t ->
      (module PVM_with_context_and_state) ->
      Raw_level.t ->
      serialized t tzresult Lwt.t
  end

  module Game : sig
    type player = Alice | Bob

    val player_equal : player -> player -> bool

    val player_encoding : player Data_encoding.t

    type dissection_chunk = Dissection_chunk.t

    type game_state =
      | Dissecting of {
          dissection: dissection_chunk list;
          default_number_of_sections: int;
        }
      | Final_move of {
          agreed_start_chunk: dissection_chunk;
          refuted_stop_chunk: dissection_chunk;
        }

    val game_state_encoding : game_state Data_encoding.t

    val game_state_equal : game_state -> game_state -> bool

    type t = {
      turn: player;
      inbox_snapshot: Inbox.history_proof;
      dal_snapshot: Dal.Slots_history.t;
      start_level: Raw_level.t;
      inbox_level: Raw_level.t;
      game_state: game_state;
    }

    val pp_dissection : Format.formatter -> dissection_chunk list -> unit

    val pp : Format.formatter -> t -> unit

    module Index : sig
      type t = private {alice: Staker.t; bob: Staker.t}

      val encoding : t Data_encoding.t

      val make : Staker.t -> Staker.t -> t
    end

    val encoding : t Data_encoding.t

    val opponent : player -> player

    type step =
      | Dissection of dissection_chunk list
      | Proof of Proof.serialized Proof.t

    type refutation =
      | Start of {
          player_commitment_hash: Commitment.Hash.t;
          opponent_commitment_hash: Commitment.Hash.t;
        }
      | Move of {choice: Tick.t; step: step}

    val refutation_encoding : refutation Data_encoding.t

    val pp_refutation : Format.formatter -> refutation -> unit

    type reason = Conflict_resolved | Timeout

    val pp_reason : Format.formatter -> reason -> unit

    val reason_encoding : reason Data_encoding.t

    type game_result = Loser of {reason: reason; loser: Staker.t} | Draw

    val pp_game_result : Format.formatter -> game_result -> unit

    val game_result_encoding : game_result Data_encoding.t

    type status = Ongoing | Ended of game_result

    val pp_status : Format.formatter -> status -> unit

    val status_encoding : status Data_encoding.t

    val initial :
      Inbox.history_proof ->
      Dal.Slots_history.t ->
      start_level: Raw_level.t ->
      parent_commitment: Commitment.t ->
      defender_commitment: Commitment.t ->
      refuter: Staker.t ->
      defender: Staker.t ->
      default_number_of_sections: int ->
      t

    val play :
      Kind.t ->
      Dal.parameters ->
      dal_attestation_lag: int ->
      stakers: Index.t ->
      Metadata.t ->
      t ->
      step: step ->
      choice: Tick.t ->
      (game_result, t) Either.t tzresult Lwt.t

    type timeout = {alice: int; bob: int; last_turn_level: Raw_level.t}

    val timeout_encoding : timeout Data_encoding.t

    type error +=
      | Dissection_choice_not_found of Tick.t
      | Proof_unexpected_section_size of Z.t
      | Proof_start_state_hash_mismatch of {
          start_state_hash: State_hash.t option;
          start_proof: State_hash.t;
        }
      | Proof_stop_state_hash_failed_to_refute of {
          stop_state_hash: State_hash.t option;
          stop_proof: State_hash.t option;
        }
      | Proof_stop_state_hash_failed_to_validate of {
          stop_state_hash: State_hash.t option;
          stop_proof: State_hash.t option;
        }
      | Dissecting_during_final_move

    module Internal_for_tests : sig
      val check_dissection :
        default_number_of_sections: int ->
        start_chunk: dissection_chunk ->
        stop_chunk: dissection_chunk ->
        dissection_chunk list ->
        unit tzresult
    end
  end

  module Stake_storage : sig
    val find_staker :
      context ->
      t ->
      Staker.t ->
      (context * Commitment.Hash.t option) tzresult Lwt.t

    val publish_commitment :
      context ->
      t ->
      Staker.t ->
      Commitment.t ->
      (Commitment.Hash.t * Raw_level.t * context * Receipt.balance_updates) tzresult Lwt.t

    val cement_commitment :
      context ->
      t ->
      Commitment.Hash.t ->
      (context * Commitment.t) tzresult Lwt.t

    val withdraw_stake :
      context ->
      t ->
      Staker.t ->
      (context * Receipt.balance_updates) tzresult Lwt.t

    val stakers_commitments_uncarbonated :
      context -> t -> (Staker.t * Commitment.Hash.t option) list tzresult Lwt.t
  end

  module Refutation_storage : sig
    type point = {commitment: Commitment.t; hash: Commitment.Hash.t}

    type conflict_point = point * point

    type conflict = {
      other: Staker.t;
      their_commitment: Commitment.t;
      our_commitment: Commitment.t;
      parent_commitment: Commitment.Hash.t;
    }

    val conflict_encoding : conflict Data_encoding.t

    val conflicting_stakers_uncarbonated :
      context -> t -> Staker.t -> conflict list tzresult Lwt.t

    val get_ongoing_games_for_staker :
      context ->
      t ->
      Staker.t ->
      ((Game.t * Game.Index.t) list * context) tzresult Lwt.t

    val start_game :
      context ->
      t ->
      player: public_key_hash * Commitment.Hash.t ->
      opponent: public_key_hash * Commitment.Hash.t ->
      context tzresult Lwt.t

    val game_move :
      context ->
      t ->
      player: Staker.t ->
      opponent: Staker.t ->
      step: Game.step ->
      choice: Tick.t ->
      (Game.game_result option * context) tzresult Lwt.t

    val get_timeout :
      context -> t -> Game.Index.t -> (Game.timeout * context) tzresult Lwt.t

    val timeout :
      context ->
      t ->
      Game.Index.t ->
      (Game.game_result * context) tzresult Lwt.t

    val apply_game_result :
      context ->
      t ->
      Game.Index.t ->
      Game.game_result ->
      (Game.status * context * Receipt.balance_updates) tzresult Lwt.t

    module Internal_for_tests : sig
      val get_conflict_point :
        context ->
        t ->
        Staker.t ->
        Staker.t ->
        (conflict_point * context) tzresult Lwt.t
    end
  end

  val rpc_arg : t RPC_arg.t

  val list_unaccounted : context -> t list tzresult Lwt.t

  val genesis_info :
    context -> rollup -> (context * Commitment.genesis_info) tzresult Lwt.t

  (** This module discloses definitions that are only useful for tests and
    must not be used otherwise. *)
  module Internal_for_tests : sig
    val originated_sc_rollup : Origination_nonce.Internal_for_tests.t -> t
  end
end

(** This module re-exports definitions from {!Destination_repr}. *)
module Destination : sig
  type t =
    | Contract of Contract.t
    | Tx_rollup of Tx_rollup.t
    | Sc_rollup of Sc_rollup.t
    | Zk_rollup of Zk_rollup.t

  val encoding : t Data_encoding.t

  val pp : Format.formatter -> t -> unit

  val compare : t -> t -> int

  val equal : t -> t -> bool

  val to_b58check : t -> string

  val of_b58check : string -> t tzresult

  val in_memory_size : t -> Cache_memory_helpers.sint

  val must_exist : context -> t -> context tzresult Lwt.t

  type error += Invalid_destination_b58check of string
end

(** See {!Block_payload_repr}. *)
module Block_payload : sig
  (** See {!Block_payload_repr.hash}. *)
  val hash :
    predecessor_hash: Block_hash.t ->
    payload_round: Round.t ->
    Operation_list_hash.elt list ->
    Block_payload_hash.t
end

(** This module re-exports definitions from {!Block_header_repr}. *)
module Block_header : sig
  type contents = {
    payload_hash: Block_payload_hash.t;
    payload_round: Round.t;
    seed_nonce_hash: Nonce_hash.t option;
    proof_of_work_nonce: bytes;
    liquidity_baking_toggle_vote:
    Liquidity_baking_repr.liquidity_baking_toggle_vote;
  }

  type protocol_data = {contents: contents; signature: signature}

  type t = {shell: Block_header.shell_header; protocol_data: protocol_data}

  type block_header = t

  type raw = Block_header.t

  type shell_header = Block_header.shell_header

  type block_watermark = Block_header of Chain_id.t

  val to_watermark : block_watermark -> Signature.watermark

  val of_watermark : Signature.watermark -> block_watermark option

  module Proof_of_work : sig
    val check_hash : Block_hash.t -> int64 -> bool

    val check_header_proof_of_work_stamp :
      shell_header -> contents -> int64 -> bool

    val check_proof_of_work_stamp :
      proof_of_work_threshold: int64 -> block_header -> unit tzresult
  end

  val raw : block_header -> raw

  val hash : block_header -> Block_hash.t

  val hash_raw : raw -> Block_hash.t

  val encoding : block_header Data_encoding.encoding

  val raw_encoding : raw Data_encoding.t

  val contents_encoding : contents Data_encoding.t

  val unsigned_encoding : (shell_header * contents) Data_encoding.t

  val protocol_data_encoding : protocol_data Data_encoding.encoding

  val shell_header_encoding : shell_header Data_encoding.encoding

  (** The maximum size of block headers in bytes *)
  val max_header_length : int

  type error += Invalid_stamp

  val check_timestamp :
    Round.round_durations ->
    timestamp: Time.t ->
    round: Round.t ->
    predecessor_timestamp: Time.t ->
    predecessor_round: Round.t ->
    unit tzresult

  val check_signature : t -> Chain_id.t -> public_key -> unit tzresult

  val begin_validate_block_header :
    block_header: t ->
    chain_id: Chain_id.t ->
    predecessor_timestamp: Time.t ->
    predecessor_round: Round.t ->
    fitness: Fitness.t ->
    timestamp: Time.t ->
    delegate_pk: public_key ->
    round_durations: Round.round_durations ->
    proof_of_work_threshold: int64 ->
    expected_commitment: bool ->
    unit tzresult

  type locked_round_evidence = {
    preendorsement_round: Round.t;
    preendorsement_count: int;
  }

  type checkable_payload_hash =
    | No_check
    | Expected_payload_hash of Block_payload_hash.t

  val finalize_validate_block_header :
    block_header_contents: contents ->
    round: Round.t ->
    fitness: Fitness.t ->
    checkable_payload_hash: checkable_payload_hash ->
    locked_round_evidence: locked_round_evidence option ->
    consensus_threshold: int ->
    unit tzresult
end

(** This module re-exports definitions from {!Cache_repr}. *)
module Cache : sig
  type size = int

  type index = int

  type cache_nonce

  module Admin : sig
    type key

    type value

    val pp : Format.formatter -> context -> unit

    val sync : context -> cache_nonce -> context Lwt.t

    val future_cache_expectation :
      ?blocks_before_activation: int32 ->
      context ->
      time_in_blocks: int ->
      context tzresult Lwt.t

    val cache_size : context -> cache_index: int -> size option

    val cache_size_limit : context -> cache_index: int -> size option

    val value_of_key :
      context -> Context.Cache.key -> Context.Cache.value tzresult Lwt.t
  end

  type namespace = private string

  val create_namespace : string -> namespace

  type identifier = string

  module type CLIENT = sig
    type cached_value

    val cache_index : index

    val namespace : namespace

    val value_of_identifier :
      context -> identifier -> cached_value tzresult Lwt.t
  end

  module type INTERFACE = sig
    type cached_value

    val update :
      context -> identifier -> (cached_value * size) option -> context tzresult

    val find : context -> identifier -> cached_value option tzresult Lwt.t

    val list_identifiers : context -> (string * int) list

    val identifier_rank : context -> string -> int option

    val size : context -> int

    val size_limit : context -> int
  end

  val register_exn :
    (module CLIENT with type cached_value = 'a) ->
    (module INTERFACE with type cached_value = 'a)

  val cache_nonce_from_block_header :
    Block_header.shell_header -> Block_header.contents -> cache_nonce
end

(** This module re-exports definitions from {!Lazy_storage_kind}. *)
module Kind : sig
  type preendorsement_consensus_kind = Preendorsement_consensus_kind

  type endorsement_consensus_kind = Endorsement_consensus_kind

  type 'a consensus =
    | Preendorsement_kind : preendorsement_consensus_kind consensus
    | Endorsement_kind : endorsement_consensus_kind consensus

  type preendorsement = preendorsement_consensus_kind consensus

  type endorsement = endorsement_consensus_kind consensus

  type dal_attestation = Dal_attestation_kind

  type seed_nonce_revelation = Seed_nonce_revelation_kind

  type vdf_revelation = Vdf_revelation_kind

  type 'a double_consensus_operation_evidence =
    | Double_consensus_operation_evidence

  type double_endorsement_evidence =
    endorsement_consensus_kind double_consensus_operation_evidence

  type double_preendorsement_evidence =
    preendorsement_consensus_kind double_consensus_operation_evidence

  type double_baking_evidence = Double_baking_evidence_kind

  type activate_account = Activate_account_kind

  type proposals = Proposals_kind

  type ballot = Ballot_kind

  type reveal = Reveal_kind

  type transaction = Transaction_kind

  type origination = Origination_kind

  type delegation = Delegation_kind

  type event = Event_kind

  type set_deposits_limit = Set_deposits_limit_kind

  type increase_paid_storage = Increase_paid_storage_kind

  type update_consensus_key = Update_consensus_key_kind

  type drain_delegate = Drain_delegate_kind

  type failing_noop = Failing_noop_kind

  type register_global_constant = Register_global_constant_kind

  type tx_rollup_origination = Tx_rollup_origination_kind

  type tx_rollup_submit_batch = Tx_rollup_submit_batch_kind

  type tx_rollup_commit = Tx_rollup_commit_kind

  type tx_rollup_return_bond = Tx_rollup_return_bond_kind

  type tx_rollup_finalize_commitment = Tx_rollup_finalize_commitment_kind

  type tx_rollup_remove_commitment = Tx_rollup_remove_commitment_kind

  type tx_rollup_rejection = Tx_rollup_rejection_kind

  type tx_rollup_dispatch_tickets = Tx_rollup_dispatch_tickets_kind

  type transfer_ticket = Transfer_ticket_kind

  type dal_publish_slot_header = Dal_publish_slot_header_kind

  type sc_rollup_originate = Sc_rollup_originate_kind

  type sc_rollup_add_messages = Sc_rollup_add_messages_kind

  type sc_rollup_cement = Sc_rollup_cement_kind

  type sc_rollup_publish = Sc_rollup_publish_kind

  type sc_rollup_refute = Sc_rollup_refute_kind

  type sc_rollup_timeout = Sc_rollup_timeout_kind

  type sc_rollup_execute_outbox_message =
    | Sc_rollup_execute_outbox_message_kind

  type sc_rollup_recover_bond = Sc_rollup_recover_bond_kind

  type zk_rollup_origination = Zk_rollup_origination_kind

  type zk_rollup_publish = Zk_rollup_publish_kind

  type zk_rollup_update = Zk_rollup_update_kind

  type 'a manager =
    | Reveal_manager_kind : reveal manager
    | Transaction_manager_kind : transaction manager
    | Origination_manager_kind : origination manager
    | Delegation_manager_kind : delegation manager
    | Event_manager_kind : event manager
    | Register_global_constant_manager_kind : register_global_constant manager
    | Set_deposits_limit_manager_kind : set_deposits_limit manager
    | Increase_paid_storage_manager_kind : increase_paid_storage manager
    | Update_consensus_key_manager_kind : update_consensus_key manager
    | Tx_rollup_origination_manager_kind : tx_rollup_origination manager
    | Tx_rollup_submit_batch_manager_kind : tx_rollup_submit_batch manager
    | Tx_rollup_commit_manager_kind : tx_rollup_commit manager
    | Tx_rollup_return_bond_manager_kind : tx_rollup_return_bond manager
    | Tx_rollup_finalize_commitment_manager_kind :
      tx_rollup_finalize_commitment manager
    | Tx_rollup_remove_commitment_manager_kind :
      tx_rollup_remove_commitment manager
    | Tx_rollup_rejection_manager_kind : tx_rollup_rejection manager
    | Tx_rollup_dispatch_tickets_manager_kind :
      tx_rollup_dispatch_tickets manager
    | Transfer_ticket_manager_kind : transfer_ticket manager
    | Dal_publish_slot_header_manager_kind : dal_publish_slot_header manager
    | Sc_rollup_originate_manager_kind : sc_rollup_originate manager
    | Sc_rollup_add_messages_manager_kind : sc_rollup_add_messages manager
    | Sc_rollup_cement_manager_kind : sc_rollup_cement manager
    | Sc_rollup_publish_manager_kind : sc_rollup_publish manager
    | Sc_rollup_refute_manager_kind : sc_rollup_refute manager
    | Sc_rollup_timeout_manager_kind : sc_rollup_timeout manager
    | Sc_rollup_execute_outbox_message_manager_kind :
      sc_rollup_execute_outbox_message manager
    | Sc_rollup_recover_bond_manager_kind : sc_rollup_recover_bond manager
    | Zk_rollup_origination_manager_kind : zk_rollup_origination manager
    | Zk_rollup_publish_manager_kind : zk_rollup_publish manager
    | Zk_rollup_update_manager_kind : zk_rollup_update manager
end

(** All the definitions below are re-exported from {!Operation_repr}. *)

type 'a consensus_operation_type =
  | Endorsement : Kind.endorsement consensus_operation_type
  | Preendorsement : Kind.preendorsement consensus_operation_type

val pp_operation_kind :
  Format.formatter -> 'kind consensus_operation_type -> unit

type consensus_content = {
  slot: Slot.t;
  level: Raw_level.t;
  (* The level is not required to validate an endorsement when it corresponds
     to the current payload, but if we want to filter endorsements, we need
     the level. *)
  round: Round.t;
  block_payload_hash: Block_payload_hash.t;
}

val consensus_content_encoding : consensus_content Data_encoding.t

val pp_consensus_content : Format.formatter -> consensus_content -> unit

type 'kind operation = {
  shell: Operation.shell_header;
  protocol_data: 'kind protocol_data;
}

and 'kind protocol_data = {
  contents: 'kind contents_list;
  signature: signature option;
}

and _ contents_list =
  | Single : 'kind contents -> 'kind contents_list
  | Cons :
    'kind Kind.manager contents
    * 'rest Kind.manager contents_list ->
      ('kind * 'rest) Kind.manager contents_list

and _ contents =
  | Preendorsement : consensus_content -> Kind.preendorsement contents
  | Endorsement : consensus_content -> Kind.endorsement contents
  | Dal_attestation : Dal.Attestation.operation -> Kind.dal_attestation contents
  | Seed_nonce_revelation :
    {
      level: Raw_level.t;
      nonce: Nonce.t;
    } ->
      Kind.seed_nonce_revelation contents
  | Vdf_revelation :
    {
      solution: Seed.vdf_solution;
    } ->
      Kind.vdf_revelation contents
  | Double_preendorsement_evidence :
    {
      op1: Kind.preendorsement operation;
      op2: Kind.preendorsement operation;
    } ->
      Kind.double_preendorsement_evidence contents
  | Double_endorsement_evidence :
    {
      op1: Kind.endorsement operation;
      op2: Kind.endorsement operation;
    } ->
      Kind.double_endorsement_evidence contents
  | Double_baking_evidence :
    {
      bh1: Block_header.t;
      bh2: Block_header.t;
    } ->
      Kind.double_baking_evidence contents
  | Activate_account :
    {
      id: Ed25519.Public_key_hash.t;
      activation_code: Blinded_public_key_hash.activation_code;
    } ->
      Kind.activate_account contents
  | Proposals :
    {
      source: public_key_hash;
      period: int32;
      proposals: Protocol_hash.t list;
    } ->
      Kind.proposals contents
  | Ballot :
    {
      source: public_key_hash;
      period: int32;
      proposal: Protocol_hash.t;
      ballot: Vote.ballot;
    } ->
      Kind.ballot contents
  | Drain_delegate :
    {
      consensus_key: Signature.Public_key_hash.t;
      delegate: Signature.Public_key_hash.t;
      destination: Signature.Public_key_hash.t;
    } ->
      Kind.drain_delegate contents
  | Failing_noop : string -> Kind.failing_noop contents
  | Manager_operation :
    {
      source: public_key_hash;
      fee: Tez.tez;
      counter: Manager_counter.t;
      operation: 'kind manager_operation;
      gas_limit: Gas.Arith.integral;
      storage_limit: Z.t;
    } ->
      'kind Kind.manager contents

and _ manager_operation =
  | Reveal : public_key -> Kind.reveal manager_operation
  | Transaction :
    {
      amount: Tez.tez;
      parameters: Script.lazy_expr;
      entrypoint: Entrypoint.t;
      destination: Contract.t;
    } ->
      Kind.transaction manager_operation
  | Origination :
    {
      delegate: public_key_hash option;
      script: Script.t;
      credit: Tez.tez;
    } ->
      Kind.origination manager_operation
  | Delegation : public_key_hash option -> Kind.delegation manager_operation
  | Register_global_constant :
    {
      value: Script.lazy_expr;
    } ->
      Kind.register_global_constant manager_operation
  | Set_deposits_limit :
    Tez.t option ->
      Kind.set_deposits_limit manager_operation
  | Increase_paid_storage :
    {
      amount_in_bytes: Z.t;
      destination: Contract_hash.t;
    } ->
      Kind.increase_paid_storage manager_operation
  | Update_consensus_key :
    Signature.Public_key.t ->
      Kind.update_consensus_key manager_operation
  | Tx_rollup_origination : Kind.tx_rollup_origination manager_operation
  | Tx_rollup_submit_batch :
    {
      tx_rollup: Tx_rollup.t;
      content: string;
      burn_limit: Tez.tez option;
    } ->
      Kind.tx_rollup_submit_batch manager_operation
  | Tx_rollup_commit :
    {
      tx_rollup: Tx_rollup.t;
      commitment: Tx_rollup_commitment.Full.t;
    } ->
      Kind.tx_rollup_commit manager_operation
  | Tx_rollup_return_bond :
    {
      tx_rollup: Tx_rollup.t;
    } ->
      Kind.tx_rollup_return_bond manager_operation
  | Tx_rollup_finalize_commitment :
    {
      tx_rollup: Tx_rollup.t;
    } ->
      Kind.tx_rollup_finalize_commitment manager_operation
  | Tx_rollup_remove_commitment :
    {
      tx_rollup: Tx_rollup.t;
    } ->
      Kind.tx_rollup_remove_commitment manager_operation
  | Tx_rollup_rejection :
    {
      tx_rollup: Tx_rollup.t;
      level: Tx_rollup_level.t;
      message: Tx_rollup_message.t;
      message_position: int;
      message_path: Tx_rollup_inbox.Merkle.path;
      message_result_hash: Tx_rollup_message_result_hash.t;
      message_result_path: Tx_rollup_commitment.Merkle.path;
      previous_message_result: Tx_rollup_message_result.t;
      previous_message_result_path: Tx_rollup_commitment.Merkle.path;
      proof: Tx_rollup_l2_proof.serialized;
    } ->
      Kind.tx_rollup_rejection manager_operation
  | Tx_rollup_dispatch_tickets :
    {
      tx_rollup: Tx_rollup.t;
      level: Tx_rollup_level.t;
      context_hash: Context_hash.t;
      message_index: int;
      message_result_path: Tx_rollup_commitment.Merkle.path;
      tickets_info: Tx_rollup_reveal.t list;
    } ->
      Kind.tx_rollup_dispatch_tickets manager_operation
  | Transfer_ticket :
    {
      contents: Script.lazy_expr;
      ty: Script.lazy_expr;
      ticketer: Contract.t;
      amount: Ticket_amount.t;
      destination: Contract.t;
      entrypoint: Entrypoint.t;
    } ->
      Kind.transfer_ticket manager_operation
  | Dal_publish_slot_header :
    Dal.Operations.Publish_slot_header.t ->
      Kind.dal_publish_slot_header manager_operation
  | Sc_rollup_originate :
    {
      kind: Sc_rollup.Kind.t;
      boot_sector: string;
      origination_proof: Sc_rollup.Proof.serialized;
      parameters_ty: Script.lazy_expr;
    } ->
      Kind.sc_rollup_originate manager_operation
  | Sc_rollup_add_messages :
    {
      messages: string list;
    } ->
      Kind.sc_rollup_add_messages manager_operation
  | Sc_rollup_cement :
    {
      rollup: Sc_rollup.t;
      commitment: Sc_rollup.Commitment.Hash.t;
    } ->
      Kind.sc_rollup_cement manager_operation
  | Sc_rollup_publish :
    {
      rollup: Sc_rollup.t;
      commitment: Sc_rollup.Commitment.t;
    } ->
      Kind.sc_rollup_publish manager_operation
  | Sc_rollup_refute :
    {
      rollup: Sc_rollup.t;
      opponent: Sc_rollup.Staker.t;
      refutation: Sc_rollup.Game.refutation;
    } ->
      Kind.sc_rollup_refute manager_operation
  | Sc_rollup_timeout :
    {
      rollup: Sc_rollup.t;
      stakers: Sc_rollup.Game.Index.t;
    } ->
      Kind.sc_rollup_timeout manager_operation
  | Sc_rollup_execute_outbox_message :
    {
      rollup: Sc_rollup.t;
      cemented_commitment: Sc_rollup.Commitment.Hash.t;
      output_proof: string;
    } ->
      Kind.sc_rollup_execute_outbox_message manager_operation
  | Sc_rollup_recover_bond :
    {
      sc_rollup: Sc_rollup.t;
      staker: Signature.Public_key_hash.t;
    } ->
      Kind.sc_rollup_recover_bond manager_operation
  | Zk_rollup_origination :
    {
      public_parameters: Plonk.public_parameters;
      circuits_info: [`Public | `Private | `Fee] Zk_rollup.Account.SMap.t;
      init_state: Zk_rollup.State.t;
      nb_ops: int;
    } ->
      Kind.zk_rollup_origination manager_operation
  | Zk_rollup_publish :
    {
      zk_rollup: Zk_rollup.t;
      ops: (Zk_rollup.Operation.t * Zk_rollup.Ticket.t option) list;
    } ->
      Kind.zk_rollup_publish manager_operation
  | Zk_rollup_update :
    {
      zk_rollup: Zk_rollup.t;
      update: Zk_rollup.Update.t;
    } ->
      Kind.zk_rollup_update manager_operation

type packed_manager_operation =
  | Manager : 'kind manager_operation -> packed_manager_operation

type packed_contents = Contents : 'kind contents -> packed_contents

type packed_contents_list =
  | Contents_list : 'kind contents_list -> packed_contents_list

type packed_protocol_data =
  | Operation_data : 'kind protocol_data -> packed_protocol_data

type packed_operation = {
  shell: Operation.shell_header;
  protocol_data: packed_protocol_data;
}

val manager_kind : 'kind manager_operation -> 'kind Kind.manager

(** This module re-exports definitions from {!Operation_repr}. *)
module Operation : sig
  type nonrec 'kind contents = 'kind contents

  type nonrec packed_contents = packed_contents

  val contents_encoding : packed_contents Data_encoding.t

  type nonrec 'kind protocol_data = 'kind protocol_data

  type nonrec packed_protocol_data = packed_protocol_data

  type consensus_watermark =
    | Endorsement of Chain_id.t
    | Preendorsement of Chain_id.t
    | Dal_attestation of Chain_id.t

  val to_watermark : consensus_watermark -> Signature.watermark

  val of_watermark : Signature.watermark -> consensus_watermark option

  val protocol_data_encoding : packed_protocol_data Data_encoding.t

  val unsigned_encoding :
    (Operation.shell_header * packed_contents_list) Data_encoding.t

  type raw = Operation.t = {shell: Operation.shell_header; proto: bytes}

  val raw_encoding : raw Data_encoding.t

  val contents_list_encoding : packed_contents_list Data_encoding.t

  type 'kind t = 'kind operation = {
    shell: Operation.shell_header;
    protocol_data: 'kind protocol_data;
  }

  type nonrec packed = packed_operation

  val encoding : packed Data_encoding.t

  val raw : _ operation -> raw

  val hash : _ operation -> Operation_hash.t

  val hash_raw : raw -> Operation_hash.t

  val hash_packed : packed_operation -> Operation_hash.t

  val acceptable_pass : packed_operation -> int option

  val compare_by_passes : packed_operation -> packed_operation -> int

  type error += Missing_signature
  (* `Permanent *)

  type error += Invalid_signature
  (* `Permanent *)

  val check_signature : public_key -> Chain_id.t -> _ operation -> unit tzresult

  val pack : 'kind operation -> packed_operation

  val compare :
    Operation_hash.t * packed_operation ->
    Operation_hash.t * packed_operation ->
    int

  type ('a, 'b) eq = Eq : ('a, 'a) eq

  val equal : 'a operation -> 'b operation -> ('a, 'b) eq option

  module Encoding : sig
    type 'b case =
      | Case :
        {
          tag: int;
          name: string;
          encoding: 'a Data_encoding.t;
          select: packed_contents -> 'b contents option;
          proj: 'b contents -> 'a;
          inj: 'a -> 'b contents;
        } ->
          'b case

    val preendorsement_case : Kind.preendorsement case

    val endorsement_case : Kind.endorsement case

    val dal_attestation_case : Kind.dal_attestation case

    val seed_nonce_revelation_case : Kind.seed_nonce_revelation case

    val vdf_revelation_case : Kind.vdf_revelation case

    val double_preendorsement_evidence_case :
      Kind.double_preendorsement_evidence case

    val double_endorsement_evidence_case : Kind.double_endorsement_evidence case

    val double_baking_evidence_case : Kind.double_baking_evidence case

    val activate_account_case : Kind.activate_account case

    val proposals_case : Kind.proposals case

    val ballot_case : Kind.ballot case

    val drain_delegate_case : Kind.drain_delegate case

    val failing_noop_case : Kind.failing_noop case

    val reveal_case : Kind.reveal Kind.manager case

    val transaction_case : Kind.transaction Kind.manager case

    val origination_case : Kind.origination Kind.manager case

    val delegation_case : Kind.delegation Kind.manager case

    val update_consensus_key_case : Kind.update_consensus_key Kind.manager case

    val tx_rollup_origination_case :
      Kind.tx_rollup_origination Kind.manager case

    val tx_rollup_submit_batch_case :
      Kind.tx_rollup_submit_batch Kind.manager case

    val tx_rollup_commit_case : Kind.tx_rollup_commit Kind.manager case

    val tx_rollup_return_bond_case :
      Kind.tx_rollup_return_bond Kind.manager case

    val tx_rollup_finalize_commitment_case :
      Kind.tx_rollup_finalize_commitment Kind.manager case

    val tx_rollup_remove_commitment_case :
      Kind.tx_rollup_remove_commitment Kind.manager case

    val tx_rollup_rejection_case : Kind.tx_rollup_rejection Kind.manager case

    val tx_rollup_dispatch_tickets_case :
      Kind.tx_rollup_dispatch_tickets Kind.manager case

    val transfer_ticket_case : Kind.transfer_ticket Kind.manager case

    val dal_publish_slot_header_case :
      Kind.dal_publish_slot_header Kind.manager case

    val register_global_constant_case :
      Kind.register_global_constant Kind.manager case

    val set_deposits_limit_case : Kind.set_deposits_limit Kind.manager case

    val increase_paid_storage_case :
      Kind.increase_paid_storage Kind.manager case

    val sc_rollup_originate_case : Kind.sc_rollup_originate Kind.manager case

    val sc_rollup_add_messages_case :
      Kind.sc_rollup_add_messages Kind.manager case

    val sc_rollup_cement_case : Kind.sc_rollup_cement Kind.manager case

    val sc_rollup_publish_case : Kind.sc_rollup_publish Kind.manager case

    val sc_rollup_refute_case : Kind.sc_rollup_refute Kind.manager case

    val sc_rollup_timeout_case : Kind.sc_rollup_timeout Kind.manager case

    val sc_rollup_execute_outbox_message_case :
      Kind.sc_rollup_execute_outbox_message Kind.manager case

    val sc_rollup_recover_bond_case :
      Kind.sc_rollup_recover_bond Kind.manager case

    val zk_rollup_origination_case :
      Kind.zk_rollup_origination Kind.manager case

    val zk_rollup_publish_case : Kind.zk_rollup_publish Kind.manager case

    val zk_rollup_update_case : Kind.zk_rollup_update Kind.manager case

    module Manager_operations : sig
      type 'b case =
        | MCase :
          {
            tag: int;
            name: string;
            encoding: 'a Data_encoding.t;
            select: packed_manager_operation -> 'kind manager_operation option;
            proj: 'kind manager_operation -> 'a;
            inj: 'a -> 'kind manager_operation;
          } ->
            'kind case

      val reveal_case : Kind.reveal case

      val transaction_case : Kind.transaction case

      val origination_case : Kind.origination case

      val delegation_case : Kind.delegation case

      val update_consensus_key_tag : int

      val update_consensus_key_case : Kind.update_consensus_key case

      val register_global_constant_case : Kind.register_global_constant case

      val set_deposits_limit_case : Kind.set_deposits_limit case

      val increase_paid_storage_case : Kind.increase_paid_storage case

      val tx_rollup_origination_case : Kind.tx_rollup_origination case

      val tx_rollup_submit_batch_case : Kind.tx_rollup_submit_batch case

      val tx_rollup_commit_case : Kind.tx_rollup_commit case

      val tx_rollup_return_bond_case : Kind.tx_rollup_return_bond case

      val tx_rollup_finalize_commitment_case :
        Kind.tx_rollup_finalize_commitment case

      val tx_rollup_remove_commitment_case :
        Kind.tx_rollup_remove_commitment case

      val tx_rollup_rejection_case : Kind.tx_rollup_rejection case

      val tx_rollup_dispatch_tickets_case : Kind.tx_rollup_dispatch_tickets case

      val transfer_ticket_case : Kind.transfer_ticket case

      val dal_publish_slot_header_case : Kind.dal_publish_slot_header case

      val sc_rollup_originate_case : Kind.sc_rollup_originate case

      val sc_rollup_add_messages_case : Kind.sc_rollup_add_messages case

      val sc_rollup_cement_case : Kind.sc_rollup_cement case

      val sc_rollup_publish_case : Kind.sc_rollup_publish case

      val sc_rollup_refute_case : Kind.sc_rollup_refute case

      val sc_rollup_timeout_case : Kind.sc_rollup_timeout case

      val sc_rollup_execute_outbox_message_case :
        Kind.sc_rollup_execute_outbox_message case

      val sc_rollup_recover_bond_case : Kind.sc_rollup_recover_bond case

      val zk_rollup_origination_case : Kind.zk_rollup_origination case

      val zk_rollup_publish_case : Kind.zk_rollup_publish case

      val zk_rollup_update_case : Kind.zk_rollup_update case
    end
  end

  val of_list : packed_contents list -> packed_contents_list tzresult

  val to_list : packed_contents_list -> packed_contents list
end

(** This module re-exports definitions from {!Stake_storage},
    {!Delegate_storage} and {!Delegate}. *)
module Stake_distribution : sig
  val snapshot : context -> context tzresult Lwt.t

  val compute_snapshot_index :
    context -> Cycle.t -> max_snapshot_index: int -> int tzresult Lwt.t

  val baking_rights_owner :
    context ->
    Level.t ->
    round: Round.t ->
    (context * Slot.t * Consensus_key.pk) tzresult Lwt.t

  val slot_owner :
    context -> Level.t -> Slot.t -> (context * Consensus_key.pk) tzresult Lwt.t
end

(** This module re-exports definitions from {!Commitment_repr} and,
    {!Commitment_storage}. *)
module Commitment : sig
  type t = {
    blinded_public_key_hash: Blinded_public_key_hash.t;
    amount: Tez.tez;
  }

  (** See {!Commitment_storage.exists}. *)
  val exists : context -> Blinded_public_key_hash.t -> bool Lwt.t

  val encoding : t Data_encoding.t
end

(** This module re-exports definitions from {!Bootstrap_storage}. *)
module Bootstrap : sig
  val cycle_end : context -> Cycle.t -> context tzresult Lwt.t
end

(** This module re-exports definitions from {!Migration_repr}. *)
module Migration : sig
  type origination_result = {
    balance_updates: Receipt.balance_updates;
    originated_contracts: Contract_hash.t list;
    storage_size: Z.t;
    paid_storage_size_diff: Z.t;
  }
end

(** Create an [Alpha_context.t] from an untyped context (first block in the chain only). *)
val prepare_first_block :
  Chain_id.t ->
  Context.t ->
  typecheck:
  (context ->
  Script.t ->
  ((Script.t * Lazy_storage.diffs option) * context) tzresult Lwt.t) ->
  level: Int32.t ->
  timestamp: Time.t ->
  predecessor: Block_hash.t ->
  context tzresult Lwt.t

(** Create an [Alpha_context.t] from an untyped context. *)
val prepare :
  Context.t ->
  level: Int32.t ->
  predecessor_timestamp: Time.t ->
  timestamp: Time.t ->
  (context * Receipt.balance_updates * Migration.origination_result list) tzresult Lwt.t

(** All the definitions below are re-exported from {!Raw_context}. *)

val activate : context -> Protocol_hash.t -> context Lwt.t

val reset_internal_nonce : context -> context

val fresh_internal_nonce : context -> (context * int) tzresult

val record_internal_nonce : context -> int -> context

val internal_nonce_already_recorded : context -> int -> bool

val description : context Storage_description.t

val record_non_consensus_operation_hash : context -> Operation_hash.t -> context

val non_consensus_operations : context -> Operation_hash.t list

val record_dictator_proposal_seen : t -> t

val dictator_proposal_seen : t -> bool

(** Finalize an {{!t} [Alpha_context.t]}, producing a [validation_result].
 *)
val finalize :
  ?commit_message: string -> context -> Fitness.raw -> Updater.validation_result

(** Should only be used by [Main.current_context] to return a context usable for RPCs *)
val current_context : context -> Context.t

(** This module re-exports definitions from {!Parameters_repr}. *)
module Parameters : sig
  type bootstrap_account = {
    public_key_hash: public_key_hash;
    public_key: public_key option;
    amount: Tez.t;
    delegate_to: public_key_hash option;
    consensus_key: public_key option;
  }

  type bootstrap_contract = {
    delegate: public_key_hash option;
    amount: Tez.t;
    script: Script.t;
  }

  type t = {
    bootstrap_accounts: bootstrap_account list;
    bootstrap_contracts: bootstrap_contract list;
    commitments: Commitment.t list;
    constants: Constants.Parametric.t;
    security_deposit_ramp_up_cycles: int option;
    no_reward_cycles: int option;
  }

  val bootstrap_account_encoding : bootstrap_account Data_encoding.t

  val encoding : t Data_encoding.t
end

(** This module re-exports definitions from {!Liquidity_baking_repr} and
    {!Liquidity_baking_storage}. *)
module Liquidity_baking : sig
  type liquidity_baking_toggle_vote =
    Liquidity_baking_repr.liquidity_baking_toggle_vote =
    | LB_on
    | LB_off
    | LB_pass

  val liquidity_baking_toggle_vote_encoding :
    liquidity_baking_toggle_vote Data_encoding.encoding

  val get_cpmm_address : context -> Contract_hash.t tzresult Lwt.t

  module Toggle_EMA : sig
    type t

    val zero : t

    val to_int32 : t -> Int32.t

    val encoding : t Data_encoding.t
  end

  val on_subsidy_allowed :
    context ->
    toggle_vote: liquidity_baking_toggle_vote ->
    (context -> Contract_hash.t -> (context * 'a list) tzresult Lwt.t) ->
    (context * 'a list * Toggle_EMA.t) tzresult Lwt.t
end

(** This module re-exports definitions from {!Ticket_storage}. *)
module Ticket_balance : sig
  type error +=
    | Negative_ticket_balance of {key: Ticket_hash.t; balance: Z.t}
    | Used_storage_space_underflow

  val adjust_balance :
    context -> Ticket_hash.t -> delta: Z.t -> (Z.t * context) tzresult Lwt.t

  val adjust_storage_space :
    context -> storage_diff: Z.t -> (Z.t * context) tzresult Lwt.t

  val get_balance :
    context -> Ticket_hash.t -> (Z.t option * context) tzresult Lwt.t

  (** This module discloses definitions that are only useful for tests and
      must not be used otherwise. *)
  module Internal_for_tests : sig
    val used_storage_space : context -> Z.t tzresult Lwt.t

    val paid_storage_space : context -> Z.t tzresult Lwt.t
  end
end

module First_level_of_protocol : sig
  (** Get the level of the first block of this protocol. *)
  val get : context -> Raw_level.t tzresult Lwt.t
end

(** This module re-exports definitions from {!Raw_context.Consensus}. *)
module Consensus : sig
  include Raw_context.CONSENSUS with
  type t := t
  and type slot := Slot.t
  and type 'a slot_map := 'a Slot.Map.t
  and type slot_set := Slot.Set.t
  and type round := Round.t
  and type consensus_pk := Consensus_key.pk

  (** [store_endorsement_branch context branch] sets the "endorsement branch"
      (see {!Storage.Tenderbake.Endorsement_branch} to [branch] in both the disk
      storage and RAM. *)
  val store_endorsement_branch :
    context -> Block_hash.t * Block_payload_hash.t -> context Lwt.t

  (** [store_grand_parent_branch context branch] sets the "grand-parent branch"
      (see {!Storage.Tenderbake.Grand_parent_branch} to [branch] in both the
      disk storage and RAM. *)
  val store_grand_parent_branch :
    context -> Block_hash.t * Block_payload_hash.t -> context Lwt.t
end

(** This module re-exports definitions from {!Token}. *)
module Token : sig
  type container = [
    | `Contract of Contract.t
    | `Collected_commitments of Blinded_public_key_hash.t
    | `Delegate_balance of public_key_hash
    | `Frozen_deposits of public_key_hash
    | `Block_fees
    | `Frozen_bonds of Contract.t * Bond_id.t
  ]

  type source = [
    | `Invoice
    | `Bootstrap
    | `Initial_commitments
    | `Revelation_rewards
    | `Double_signing_evidence_rewards
    | `Endorsing_rewards
    | `Baking_rewards
    | `Baking_bonuses
    | `Minted
    | `Liquidity_baking_subsidies
    | `Tx_rollup_rejection_rewards
    | `Sc_rollup_refutation_rewards
    | container
  ]

  type sink = [
    | `Storage_fees
    | `Double_signing_punishments
    | `Lost_endorsing_rewards of public_key_hash * bool * bool
    | `Burned
    | `Tx_rollup_rejection_punishments
    | `Sc_rollup_refutation_punishments
    | container
  ]

  val allocated : context -> container -> (context * bool) tzresult Lwt.t

  val balance : context -> container -> (context * Tez.t) tzresult Lwt.t

  val transfer_n :
    ?origin: Receipt.update_origin ->
    context ->
    ([< source] * Tez.t) list ->
    [< sink] ->
    (context * Receipt.balance_updates) tzresult Lwt.t

  val transfer :
    ?origin: Receipt.update_origin ->
    context ->
    [< source] ->
    [< sink] ->
    Tez.t ->
    (context * Receipt.balance_updates) tzresult Lwt.t
end

(** This module re-exports definitions from {!Fees_storage}. *)
module Fees : sig
  val record_paid_storage_space :
    context -> Contract_hash.t -> (context * Z.t * Z.t) tzresult Lwt.t

  val record_global_constant_storage_space : context -> Z.t -> context * Z.t

  val burn_storage_fees :
    ?origin: Receipt.update_origin ->
    context ->
    storage_limit: Z.t ->
    payer: Token.source ->
    Z.t ->
    (context * Z.t * Receipt.balance_updates) tzresult Lwt.t

  val burn_storage_increase_fees :
    ?origin: Receipt_repr.update_origin ->
    context ->
    payer: Token.source ->
    Z.t ->
    (context * Receipt.balance_updates) tzresult Lwt.t

  val burn_origination_fees :
    ?origin: Receipt.update_origin ->
    context ->
    storage_limit: Z.t ->
    payer: Token.source ->
    (context * Z.t * Receipt.balance_updates) tzresult Lwt.t

  val burn_tx_rollup_origination_fees :
    ?origin: Receipt.update_origin ->
    context ->
    storage_limit: Z.t ->
    payer: Token.source ->
    (context * Z.t * Receipt.balance_updates) tzresult Lwt.t

  val burn_sc_rollup_origination_fees :
    ?origin: Receipt.update_origin ->
    context ->
    storage_limit: Z.t ->
    payer: Token.source ->
    Z.t ->
    (context * Z.t * Receipt.balance_updates) tzresult Lwt.t

  val burn_zk_rollup_origination_fees :
    ?origin: Receipt.update_origin ->
    context ->
    storage_limit: Z.t ->
    payer: Token.source ->
    Z.t ->
    (context * Z.t * Receipt.balance_updates) tzresult Lwt.t

  type error += Cannot_pay_storage_fee
  (* `Temporary *)

  type error += Operation_quota_exceeded
  (* `Temporary *)

  type error += Storage_limit_too_high
  (* `Permanent *)

  val check_storage_limit : context -> storage_limit: Z.t -> unit tzresult
end

(* Playing with nested structures *)
type foo =
  a ->
  (b -> c) ->
  d ->
  e

include module type of (struct include Time end)
