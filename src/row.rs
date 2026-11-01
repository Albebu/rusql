pub(crate) struct Row {
    pub(crate) values: Vec<RowValue>,
}

pub(crate) enum RowValue {
    Text(String),
    Number(i64),
}
