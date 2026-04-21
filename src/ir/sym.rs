pub enum LinkageType {
    Private,
    Internal,
    External,
}

pub struct SymbolData {
    linkage: LinkageType,
    name: String,
}

pub trait Symbol {
    fn get_symbol(&self) -> &SymbolData;
}
