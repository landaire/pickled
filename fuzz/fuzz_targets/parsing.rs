#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    pickled::value_from_slice(data, Default::default());
});
