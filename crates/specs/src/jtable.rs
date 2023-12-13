use super::itable::InstructionTableEntry;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Serialize, Debug, Clone, Deserialize, PartialEq)]
pub struct StaticFrameEntry {
    pub enable: bool,
    pub frame_id: u32,
    pub next_frame_id: u32,
    pub callee_fid: u32,
    pub fid: u32,
    pub iid: u32,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub struct JumpTableEntry {
    // caller eid (unique)
    pub eid: u32,
    pub last_jump_eid: u32,
    pub callee_fid: u32,
    pub inst: Box<InstructionTableEntry>,
}

impl JumpTableEntry {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct JumpTable(Vec<JumpTableEntry>);

impl JumpTable {
    pub fn new(entries: Vec<JumpTableEntry>) -> Self{
        JumpTable(entries)
    }

    pub fn entries(&self) -> &Vec<JumpTableEntry> {
        &self.0
    }

    pub fn push(&mut self, entry: JumpTableEntry) {
        self.0.push(entry)
    }
}
