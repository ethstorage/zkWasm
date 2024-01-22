#[inline(always)]
pub(crate) fn name_of_params(k: u32) -> String {
    format!("K{}.params", k)
}

#[inline(always)]
pub(crate) fn name_of_config(name: &str) -> String {
    format!("{}.zkwasm.config", name)
}

#[cfg(not(feature = "continuation"))]
#[inline(always)]
pub(crate) fn name_of_circuit_data(name: &str) -> String {
    format!("{}.circuit.data", name)
}

#[cfg(feature = "continuation")]
#[inline(always)]
pub(crate) fn name_of_circuit_data(name: &str, is_last_circuit: bool) -> String {
    if is_last_circuit {
        format!("{}.circuit.finalized.data", name)
    } else {
        format!("{}.circuit.ongoing.data", name)
    }
}

// FIXME: adapt batcher crate, however the crate should provice this function
#[inline(always)]
pub(crate) fn name_of_loadinfo(name: &str) -> String {
    format!("{}.loadinfo.json", name)
}
