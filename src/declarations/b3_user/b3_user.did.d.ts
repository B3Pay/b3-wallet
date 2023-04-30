import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Account {
  'id' : string,
  'keys' : Keys,
  'name' : string,
  'ecdsa' : Ecdsa,
  'canisters' : Array<[Principal, Allowance]>,
  'requests' : Array<SignRequest>,
  'signed' : SignedTransaction,
}
export interface Allowance {
  'updated_at' : bigint,
  'metadata' : Array<[string, string]>,
  'created_at' : bigint,
  'limit' : [] | [number],
  'expires_at' : [] | [bigint],
}
export interface CanisterStatus {
  'id' : Principal,
  'status' : CanisterStatusResponse,
  'status_at' : bigint,
}
export interface CanisterStatusResponse {
  'status' : CanisterStatusType,
  'memory_size' : bigint,
  'cycles' : bigint,
  'settings' : DefiniteCanisterSettings,
  'idle_cycles_burned_per_day' : bigint,
  'module_hash' : [] | [Uint8Array | number[]],
}
export type CanisterStatusType = { 'stopped' : null } |
  { 'stopping' : null } |
  { 'running' : null };
export interface DefiniteCanisterSettings {
  'freezing_threshold' : bigint,
  'controllers' : Array<Principal>,
  'memory_allocation' : bigint,
  'compute_allocation' : bigint,
}
export interface Ecdsa { 'env' : Environment, 'path' : Uint8Array | number[] }
export type Environment = { 'Production' : null } |
  { 'Development' : null } |
  { 'Staging' : null };
export interface Keys {
  'eth_address' : string,
  'bytes' : Uint8Array | number[],
  'btc_address' : string,
}
export type Result = { 'Ok' : Account } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : Uint8Array | number[] } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : SignedTransaction } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : CanisterStatus } |
  { 'Err' : string };
export interface SignRequest {
  'id' : bigint,
  'destination' : Principal,
  'public_key' : Keys,
  'data' : Uint8Array | number[],
  'deadline' : bigint,
  'cycles' : bigint,
  'chain_id' : bigint,
  'nonce' : bigint,
}
export interface SignedTransaction {
  'data' : Uint8Array | number[],
  'timestamp' : bigint,
}
export interface _SERVICE {
  'change_owner' : ActorMethod<[Principal], undefined>,
  'create_account' : ActorMethod<[[] | [Environment], [] | [string]], Result>,
  'get_account' : ActorMethod<[string], Account>,
  'get_accounts' : ActorMethod<[], Array<Account>>,
  'get_caller' : ActorMethod<[], Principal>,
  'get_owner' : ActorMethod<[], Principal>,
  'get_public_key' : ActorMethod<[string], Keys>,
  'get_signed' : ActorMethod<[string], SignedTransaction>,
  'number_of_accounts' : ActorMethod<[], number>,
  'sign_message' : ActorMethod<[string, Uint8Array | number[]], Result_1>,
  'sign_transaction' : ActorMethod<
    [string, Uint8Array | number[], bigint],
    Result_2
  >,
  'status' : ActorMethod<[], Result_3>,
  'version' : ActorMethod<[], string>,
}
