use hdi::prelude::*;


#[hdk_entry_defs]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
  DemoEntry(DemoEntry)
}

#[derive(Clone)]
#[hdk_entry_helper]
pub struct DemoEntry(pub String);

#[hdk_link_types]
pub enum LinkTypes {
  Ranking
}