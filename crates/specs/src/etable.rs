use serde::Serialize;

use super::itable::InstructionTableEntry;
use crate::host_function::HostPlugin;
use crate::step::StepInfo;

#[derive(Clone, Debug, Serialize)]
pub struct EventTableEntry {
    pub eid: u32,
    pub sp: u32,
    pub allocated_memory_pages: u32,
    pub last_jump_eid: u32,
    pub inst: InstructionTableEntry,
    pub step_info: StepInfo,
}

pub struct RestMops {
    rest_mops: Vec<u64>,
}

impl Iterator for RestMops {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.rest_mops.pop()
    }
}

pub struct RestJops {
    rest_jops: Vec<u64>,
}

impl Iterator for RestJops {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.rest_jops.pop()
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct EventTable {
    entries: Vec<EventTableEntry>,
    pub latest_eid: u32
}

impl EventTable {
    pub fn new(entries: Vec<EventTableEntry>) -> Self {
        Self {entries, latest_eid: 0}
    }

    pub fn entries(&self) -> &Vec<EventTableEntry> {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<EventTableEntry> {
        &mut self.entries
    }

    pub fn filter_foreign_entries(&self, foreign: HostPlugin) -> Vec<EventTableEntry> {
        self.entries
            .clone()
            .into_iter()
            .filter(|entry| match entry.step_info {
                StepInfo::CallHost { plugin, .. } => plugin == foreign,
                _ => false,
            })
            .collect::<Vec<_>>()
    }

    pub fn get_latest_eid(&self) -> u32 {
        self.latest_eid
    }
}
