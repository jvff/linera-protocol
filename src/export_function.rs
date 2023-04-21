#![allow(clippy::let_unit_value)]

use crate::RuntimeError;

pub trait ExportFunction<Handler, Parameters, Results> {
    fn export(
        &mut self,
        module_name: &str,
        function_name: &str,
        handler: Handler,
    ) -> Result<(), RuntimeError>;
}
