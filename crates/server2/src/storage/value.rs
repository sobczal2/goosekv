use goosekv_protocol::data_type::GString;

#[derive(Clone)]
pub struct Value {
    data: Data,
}

#[derive(Clone)]
pub enum Data {
    String(GString),
}
