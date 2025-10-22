/*! Test coverage for core IR operations.
 *
 * IR builders and transformations need thorough testing to catch edge cases. These tests verify
 * everything from basic arithmetic to storage operations, ensuring the IR behaves correctly across
 * the full range of Solidity patterns.
 */

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

mod arithmetic_tests;
mod builder_api_tests;
mod call_tests;
mod contract_tests;
mod control_flow_tests;
mod cryptographic_tests;
mod event_tests;
mod memory_tests;
mod storage_tests;
mod type_tests;
