use crate::builder::{IRBuilder, InstBuilderExt};
use crate::contract::EventId;
use crate::types::Type;
use num_bigint::BigUint;

#[test]
fn test_event_emission() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("EventContract");

    let transfer_event = contract
        .event("Transfer")
        .indexed("from", Type::Address)
        .indexed("to", Type::Address)
        .data("amount", Type::Uint(256))
        .build();
    contract.add_event(transfer_event);

    let mut func = contract.function("transfer");
    func.param("to", Type::Address)
        .param("amount", Type::Uint(256));

    let to = func.get_param(0);
    let amount = func.get_param(1);
    let mut entry = func.entry_block();

    let from = entry.msg_sender();

    let event_id = EventId(0);
    entry.emit_event(event_id, vec![from, to], vec![amount]);

    entry.return_void().unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_anonymous_event() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("AnonEventContract");

    let data_log_event = contract
        .event("DataLog")
        .data("data", Type::Bytes(32))
        .anonymous()
        .build();
    contract.add_event(data_log_event);

    let mut func = contract.function("logData");
    func.param("data", Type::Bytes(32));

    let data = func.get_param(0);
    let mut entry = func.entry_block();

    let event_id = EventId(0);
    entry.emit_event(event_id, vec![], vec![data]);

    entry.return_void().unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_multiple_events() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("MultiEventContract");

    let deposit_event = contract
        .event("Deposit")
        .indexed("user", Type::Address)
        .data("amount", Type::Uint(256))
        .build();
    contract.add_event(deposit_event);

    let withdrawal_event = contract
        .event("Withdrawal")
        .indexed("user", Type::Address)
        .data("amount", Type::Uint(256))
        .build();
    contract.add_event(withdrawal_event);

    let state_changed_event = contract
        .event("StateChanged")
        .data("oldState", Type::Uint(8))
        .data("newState", Type::Uint(8))
        .build();
    contract.add_event(state_changed_event);

    let mut func = contract.function("complexOperation");
    func.param("user", Type::Address)
        .param("amount", Type::Uint(256));

    let user = func.get_param(0);
    let amount = func.get_param(1);
    let mut entry = func.entry_block();

    entry.emit_event(EventId(0), vec![user.clone()], vec![amount.clone()]);

    let old_state = entry.constant_uint(0, 8);
    let new_state = entry.constant_uint(1, 8);

    entry.emit_event(EventId(2), vec![], vec![old_state, new_state]);

    entry.emit_event(EventId(1), vec![user], vec![amount]);

    entry.return_void().unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}

#[test]
fn test_event_with_complex_data() {
    let mut builder = IRBuilder::new();
    let mut contract = builder.contract("ComplexEventContract");

    let complex_event = contract
        .event("ComplexEvent")
        .indexed("sender", Type::Address)
        .indexed("recipient", Type::Address)
        .indexed("tokenId", Type::Uint(256))
        .data("value", Type::Uint(256))
        .data("data", Type::Bytes(32))
        .data("timestamp", Type::Uint(256))
        .build();
    contract.add_event(complex_event);

    let mut func = contract.function("emitComplex");
    func.param("recipient", Type::Address)
        .param("tokenId", Type::Uint(256))
        .param("value", Type::Uint(256))
        .param("data", Type::Bytes(32));

    let recipient = func.get_param(0);
    let token_id = func.get_param(1);
    let value = func.get_param(2);
    let data = func.get_param(3);
    let mut entry = func.entry_block();

    let sender = entry.msg_sender();
    let timestamp = entry.block_timestamp();

    entry.emit_event(
        EventId(0),
        vec![sender, recipient, token_id],
        vec![value, data, timestamp],
    );

    entry.return_void().unwrap();
    func.build().unwrap();
    contract.build().unwrap();
}
