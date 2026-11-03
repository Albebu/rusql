#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Column {
    pub(crate) name: String,
    pub(crate) data_type: ColumnDataType,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ColumnDataType {
    Text,
    Number,
}
