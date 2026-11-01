pub(crate) struct Column {
    pub(crate) name: String,
    pub(crate) data_type: ColumnDataType,
}

pub(crate) enum ColumnDataType {
    Text,
    Number,
}
