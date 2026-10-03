use evobase_appspec::{AppId, FieldId, FieldType, RecordId, RecordRef, Scope, TableId, Value};
use evobase_protocol::appspec::*;
use std::collections::BTreeMap;

const COMMAND: &str = r#"{"api_version":1,"command_id":"cmd_update","record_id":"rec_a","params":{},"request_key":"request_a","expected_revision":"0","release_id":"sha256:abc"}"#;

fn command() -> CommandRequestDto {
    CommandRequestDto::decode(COMMAND.as_bytes()).unwrap()
}

fn assert_json_error(bytes: &[u8], fragment: &str) {
    match CommandRequestDto::decode(bytes).unwrap_err() {
        WireError::InvalidJson { message } => assert!(message.contains(fragment), "{message}"),
        error => panic!("expected InvalidJson containing {fragment:?}, got {error:?}"),
    }
}

#[test]
fn reviewed_command_golden_is_a_fixed_point() {
    let request = command();
    assert_eq!(request.encode().unwrap(), COMMAND.as_bytes());
    assert_eq!(request.expected_revision.get(), 0);
    assert_eq!(
        CommandRequestDto::decode(&request.encode().unwrap()).unwrap(),
        request
    );
}

#[test]
fn exact_i64_tagged_values_survive_the_json_serializer_and_decoder() {
    let mut request = command();
    let entries = [
        ("fld_a", ValueDto::Integer(i64::MIN)),
        ("fld_b", ValueDto::Integer(i64::MAX)),
        ("fld_c", ValueDto::Money(i64::MIN)),
        ("fld_d", ValueDto::Money(i64::MAX)),
        ("fld_e", ValueDto::Blank),
        ("fld_f", ValueDto::Null),
        ("fld_g", ValueDto::Text("null\\\"\n🦀".to_owned())),
        ("fld_h", ValueDto::Bool(false)),
        (
            "fld_i",
            ValueDto::Ref(RefDto {
                table_id: TableId::new("tbl_target").unwrap(),
                record_id: RecordId::new("rec_target").unwrap(),
            }),
        ),
    ];
    request.params = entries
        .into_iter()
        .map(|(field, value)| (FieldId::new(field).unwrap(), value))
        .collect();
    request.expected_revision = RevisionDto::new(u64::MAX);
    // This is the same serde_json serializer used by the HTTP JSON response adapter.
    let bytes = serde_json::to_vec(&request).unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    assert!(text.contains("-9223372036854775808"));
    assert!(text.contains("9223372036854775807"));
    assert!(text.contains(r#""expected_revision":"18446744073709551615""#));
    assert!(!text.contains("tenant_id"));
    assert_eq!(CommandRequestDto::decode(&bytes).unwrap(), request);
    assert_eq!(request.encode().unwrap(), bytes);
    assert!(
        !request
            .params
            .contains_key(&FieldId::new("fld_absent").unwrap())
    );
    assert_ne!(
        request.params[&FieldId::new("fld_e").unwrap()],
        request.params[&FieldId::new("fld_f").unwrap()]
    );
}

#[test]
fn numeric_overflow_float_and_exponent_do_not_become_integer_values() {
    for number in ["9223372036854775808", "-9223372036854775809", "1.0", "1e0"] {
        let bytes = COMMAND.replace(
            "\"params\":{}",
            &format!(r#""params":{{"fld_a":{{"type":"integer","value":{number}}}}}"#),
        );
        assert_json_error(bytes.as_bytes(), "expected i64");
    }
    for revision in ["00", "-1", "1e0", "18446744073709551616", ""] {
        let bytes = COMMAND.replace(
            "\"expected_revision\":\"0\"",
            &format!(r#""expected_revision":"{revision}""#),
        );
        assert_json_error(bytes.as_bytes(), "revision");
    }
    assert_json_error(
        COMMAND
            .replace("\"expected_revision\":\"0\"", "\"expected_revision\":0")
            .as_bytes(),
        "expected a string",
    );
}

#[test]
fn future_versions_unknown_tags_ids_and_authority_fields_are_rejected() {
    assert_eq!(
        CommandRequestDto::decode(
            COMMAND
                .replace("\"api_version\":1", "\"api_version\":2")
                .as_bytes()
        )
        .unwrap_err(),
        WireError::UnsupportedVersion { found: 2 }
    );
    for field in [
        "tenant_id",
        "actor",
        "grants",
        "trusted_context",
        "checked_batch",
    ] {
        let bytes = COMMAND.replacen('{', &format!(r#"{{"{field}":"forged","#), 1);
        assert_json_error(bytes.as_bytes(), &format!("unknown field `{field}`"));
    }
    assert_json_error(
        COMMAND.replace("rec_a", "tbl_a").as_bytes(),
        "invalid record identity",
    );
    assert_json_error(
        COMMAND
            .replace(
                "\"params\":{}",
                r#""params":{"fld_a":{"type":"float","value":1}}"#,
            )
            .as_bytes(),
        "unknown variant `float`",
    );
    assert_json_error(COMMAND.replace("\"params\":{}", r#""params":{"fld_a":{"type":"ref","value":{"table_id":"tbl_a","record_id":"rec_a","scope":{"tenant_id":"forged","app_id":"app_a"}}}}"#).as_bytes(), "unknown field `scope`");
}

#[test]
fn duplicate_keys_are_rejected_at_every_object_level_including_escaped_keys() {
    let vectors = [
        (
            COMMAND.replacen('{', r#"{"api_version":1,"#, 1),
            "api_version",
        ),
        (
            COMMAND.replacen('{', r#"{"\u0061pi_version":1,"#, 1),
            "api_version",
        ),
        (
            COMMAND.replace(
                "\"params\":{}",
                r#""params":{"fld_a":{"type":"blank"},"fld_a":{"type":"null"}}"#,
            ),
            "fld_a",
        ),
        (
            COMMAND.replace(
                "\"params\":{}",
                r#""params":{"fld_a":{"type":"integer","value":1,"value":2}}"#,
            ),
            "value",
        ),
        (
            COMMAND.replace(
                "\"params\":{}",
                r#""params":{"fld_a":{"type":"blank","type":"null"}}"#,
            ),
            "type",
        ),
    ];
    for (bytes, key) in vectors {
        assert_eq!(
            CommandRequestDto::decode(bytes.as_bytes()).unwrap_err(),
            WireError::DuplicateKey {
                key: key.to_owned()
            }
        );
    }
}

#[test]
fn byte_depth_node_collection_and_text_limits_fail_with_named_resources() {
    assert_eq!(
        CommandRequestDto::decode(&vec![b' '; MAX_WIRE_BYTES + 1]).unwrap_err(),
        WireError::LimitExceeded {
            resource: "bytes",
            limit: MAX_WIRE_BYTES
        }
    );
    let depth = format!(
        "{}0{}",
        "[".repeat(MAX_WIRE_DEPTH + 1),
        "]".repeat(MAX_WIRE_DEPTH + 1)
    );
    assert_eq!(
        CommandRequestDto::decode(depth.as_bytes()).unwrap_err(),
        WireError::LimitExceeded {
            resource: "depth",
            limit: MAX_WIRE_DEPTH
        }
    );
    let nodes = format!("[{}0]", "0,".repeat(MAX_WIRE_NODES));
    assert_eq!(
        CommandRequestDto::decode(nodes.as_bytes()).unwrap_err(),
        WireError::LimitExceeded {
            resource: "nodes",
            limit: MAX_WIRE_NODES
        }
    );
    let mut request = command();
    request.params = (0..=evobase_appspec::MAX_FIELDS_PER_TABLE)
        .map(|index| {
            (
                FieldId::new(format!("fld_{index}")).unwrap(),
                ValueDto::Blank,
            )
        })
        .collect();
    assert_eq!(
        CommandRequestDto::decode(&serde_json::to_vec(&request).unwrap()).unwrap_err(),
        WireError::LimitExceeded {
            resource: "field values",
            limit: evobase_appspec::MAX_FIELDS_PER_TABLE
        }
    );
    request.params = BTreeMap::from([(
        FieldId::new("fld_a").unwrap(),
        ValueDto::Text("a".repeat(evobase_appspec::MAX_TEXT_BYTES + 1)),
    )]);
    assert_eq!(
        CommandRequestDto::decode(&serde_json::to_vec(&request).unwrap()).unwrap_err(),
        WireError::LimitExceeded {
            resource: "text bytes",
            limit: evobase_appspec::MAX_TEXT_BYTES
        }
    );
}

#[test]
fn diagnostics_are_bounded_even_for_hostile_unknown_and_duplicate_keys() {
    let key = "🦀".repeat(20_000);
    let unknown = COMMAND.replacen('{', &format!(r#"{{"{key}":0,"#), 1);
    match CommandRequestDto::decode(unknown.as_bytes()).unwrap_err() {
        WireError::InvalidJson { message } => assert!(message.len() <= 1_024),
        error => panic!("expected InvalidJson, got {error:?}"),
    }
    let duplicate = format!(r#"{{"{key}":0,"{key}":1}}"#);
    match CommandRequestDto::decode(duplicate.as_bytes()).unwrap_err() {
        WireError::DuplicateKey { key } => assert!(key.len() <= 256),
        error => panic!("expected DuplicateKey, got {error:?}"),
    }
}

#[test]
fn reference_conversion_uses_the_host_scope_and_rejects_cross_scope_output() {
    let scope = Scope::new("configured_tenant", AppId::new("app_a").unwrap()).unwrap();
    let wire = ValueDto::Ref(RefDto {
        table_id: TableId::new("tbl_a").unwrap(),
        record_id: RecordId::new("rec_a").unwrap(),
    });
    let domain = wire.to_domain(&scope);
    assert_eq!(ValueDto::from_domain(&domain, &scope).unwrap(), wire);
    let foreign = Value::Ref(RecordRef {
        scope: Scope::new("foreign_tenant", AppId::new("app_a").unwrap()).unwrap(),
        table_id: TableId::new("tbl_a").unwrap(),
        record_id: RecordId::new("rec_a").unwrap(),
    });
    assert_eq!(
        ValueDto::from_domain(&foreign, &scope).unwrap_err(),
        WireError::Validation {
            field: "value",
            message: "cross-scope Ref".to_owned()
        }
    );
}

#[test]
fn query_and_request_identity_validation_is_explicit() {
    assert_eq!(
        ListQueryDto { limit: None }.limit_or_default().unwrap(),
        100
    );
    assert_eq!(
        ListQueryDto { limit: Some(256) }
            .limit_or_default()
            .unwrap(),
        256
    );
    for value in [0, 257] {
        assert_eq!(
            ListQueryDto { limit: Some(value) }
                .limit_or_default()
                .unwrap_err(),
            WireError::Validation {
                field: "limit",
                message: "expected 1..256".to_owned()
            }
        );
    }
    let mut request = command();
    request.request_key = "new/key".to_owned();
    assert_eq!(
        request.encode().unwrap_err(),
        WireError::Validation {
            field: "request_key",
            message: "expected 1..96 ASCII letters, digits, '_' or '-'".to_owned()
        }
    );
    request = command();
    request.command_id = "SubmitOrder".to_owned();
    assert_eq!(
        request.encode().unwrap_err(),
        WireError::Validation {
            field: "command_id",
            message: "expected a stable cmd_ identity".to_owned()
        }
    );
}

#[test]
fn list_receipt_and_typed_errors_have_exact_round_trips() {
    let record = RecordDto {
        record_id: RecordId::new("rec_a").unwrap(),
        values: BTreeMap::from([(FieldId::new("fld_a").unwrap(), ValueDto::Integer(i64::MAX))]),
    };
    let mut list = ListResponseDto {
        api_version: API_VERSION,
        app_id: AppId::new("app_a").unwrap(),
        table_id: TableId::new("tbl_a").unwrap(),
        release_id: "sha256:abc".to_owned(),
        revision: RevisionDto::new(u64::MAX),
        records: vec![record.clone()],
        has_more: false,
    };
    assert_eq!(
        ListResponseDto::decode(&list.encode().unwrap()).unwrap(),
        list
    );
    list.records.push(record);
    assert_eq!(
        list.encode().unwrap_err(),
        WireError::Validation {
            field: "records",
            message: "duplicate record identity".to_owned()
        }
    );
    let receipt = ReceiptResponseDto {
        api_version: API_VERSION,
        app_id: AppId::new("app_a").unwrap(),
        command_id: "cmd_update".to_owned(),
        record_id: RecordId::new("rec_a").unwrap(),
        release_id: "sha256:abc".to_owned(),
        request_key: "request_a".to_owned(),
        revision: RevisionDto::new(42),
        replayed: true,
    };
    assert_eq!(
        ReceiptResponseDto::decode(&receipt.encode().unwrap()).unwrap(),
        receipt
    );
    for code in [
        ErrorCodeDto::Denied,
        ErrorCodeDto::Unsupported,
        ErrorCodeDto::Conflict,
        ErrorCodeDto::Validation,
        ErrorCodeDto::NotFound,
        ErrorCodeDto::Internal,
    ] {
        let error = ApiErrorDto::new(code, "bounded diagnostic", Some("params.fld_a".to_owned()));
        assert_eq!(
            ApiErrorDto::decode(&error.encode().unwrap()).unwrap(),
            error
        );
    }
}

#[test]
fn metadata_preserves_stable_ids_and_rejects_ambiguous_eligibility() {
    let field = FieldMetadataDto {
        field_id: FieldId::new("fld_a").unwrap(),
        name: "Count".to_owned(),
        field_type: FieldType::Integer,
        required: true,
    };
    let mut metadata = MetadataResponseDto {
        api_version: API_VERSION,
        app_id: AppId::new("app_a").unwrap(),
        release_id: "sha256:abc".to_owned(),
        revision: RevisionDto::new(0),
        canonical_appspec: None,
        supported: vec!["list".to_owned(), "command".to_owned()],
        tables: vec![TableMetadataDto {
            table_id: TableId::new("tbl_a").unwrap(),
            name: "Items".to_owned(),
            fields: vec![field.clone()],
        }],
        commands: vec![CommandMetadataDto {
            command_id: "cmd_update".to_owned(),
            name: "Update".to_owned(),
            table_id: TableId::new("tbl_a").unwrap(),
            input_fields: vec![field],
            eligible_record_ids: vec![RecordId::new("rec_a").unwrap()],
        }],
    };
    assert_eq!(
        MetadataResponseDto::decode(&metadata.encode().unwrap()).unwrap(),
        metadata
    );
    metadata.tables[0].name = "Renamed items".to_owned();
    metadata.tables[0].fields[0].name = "Localized count".to_owned();
    let decoded = MetadataResponseDto::decode(&metadata.encode().unwrap()).unwrap();
    assert_eq!(decoded.tables[0].table_id, TableId::new("tbl_a").unwrap());
    assert_eq!(
        decoded.tables[0].fields[0].field_id,
        FieldId::new("fld_a").unwrap()
    );
    metadata.commands[0]
        .eligible_record_ids
        .push(RecordId::new("rec_a").unwrap());
    assert_eq!(
        metadata.encode().unwrap_err(),
        WireError::Validation {
            field: "eligible_record_ids",
            message: "duplicate record identity".to_owned()
        }
    );
}
