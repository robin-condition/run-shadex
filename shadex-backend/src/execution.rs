pub mod execution_types;
pub mod programs;
mod proof_of_concept;
mod wgpu_back;
pub use wgpu_back::WGPURunner;

pub use proof_of_concept::ExecutionInformation;
pub use proof_of_concept::Executor;
pub use proof_of_concept::ShaderProgram as NodeExecutionOutput;
