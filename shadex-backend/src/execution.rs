pub mod execution_types;
pub mod programs;
mod proof_of_concept;

pub use proof_of_concept::ExecutionInformation;
pub use proof_of_concept::Executor;
pub use proof_of_concept::ShaderProgram as NodeExecutionOutput;
