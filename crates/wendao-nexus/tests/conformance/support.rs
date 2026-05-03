use arrow_array::{
    Array, BooleanArray, Float64Array, RecordBatch, StringArray, TimestampNanosecondArray,
};
use wendao_nexus_flight::{
    NEXUS_FLIGHT_ROUTE_METADATA_KEY, NEXUS_FLIGHT_SCHEMA_VERSION,
    NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY,
};

pub(crate) fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a StringArray {
    let index = batch.schema().index_of(name).unwrap();
    batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap()
}

pub(crate) fn string_values(batch: &RecordBatch, name: &str) -> Vec<String> {
    let column = string_column(batch, name);
    (0..column.len())
        .map(|row| column.value(row).to_string())
        .collect()
}

pub(crate) fn bool_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a BooleanArray {
    let index = batch.schema().index_of(name).unwrap();
    batch
        .column(index)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap()
}

pub(crate) fn assert_batch_route(batch: &RecordBatch, route: &str) {
    assert_eq!(
        batch
            .schema()
            .metadata()
            .get(NEXUS_FLIGHT_ROUTE_METADATA_KEY)
            .map(String::as_str),
        Some(route)
    );
    assert_eq!(
        batch
            .schema()
            .metadata()
            .get(NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY)
            .map(String::as_str),
        Some(NEXUS_FLIGHT_SCHEMA_VERSION)
    );
}

pub(crate) fn compact_batch_snapshot(batch: &RecordBatch) -> String {
    batch
        .schema()
        .fields()
        .iter()
        .enumerate()
        .map(|(index, field)| format!("{}={}", field.name(), compact_value(batch, index)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn compact_value(batch: &RecordBatch, column_index: usize) -> String {
    let column = batch.column(column_index);
    if column.is_null(0) {
        return "<null>".to_string();
    }

    let data_type = format!("{:?}", batch.schema().field(column_index).data_type());
    match data_type.as_str() {
        "Utf8" => column
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap()
            .value(0)
            .to_string(),
        "Float64" => column
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap()
            .value(0)
            .to_string(),
        "Boolean" => column
            .as_any()
            .downcast_ref::<BooleanArray>()
            .unwrap()
            .value(0)
            .to_string(),
        timestamp if timestamp.starts_with("Timestamp(") => column
            .as_any()
            .downcast_ref::<TimestampNanosecondArray>()
            .unwrap()
            .value(0)
            .to_string(),
        unsupported => panic!("unsupported compact snapshot type: {unsupported}"),
    }
}
