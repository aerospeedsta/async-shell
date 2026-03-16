// Export the binary modules so PyO3 can build them if needed.
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pymodule]
fn async_shell(_py: Python, _m: &Bound<PyModule>) -> PyResult<()> {
    Ok(())
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
fn async_shell() {
    // Empty stub for node
}
