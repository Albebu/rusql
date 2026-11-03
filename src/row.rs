#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Row {
    pub(crate) values: Vec<RowValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RowValue {
    Text(String),
    Number(i64),
}
