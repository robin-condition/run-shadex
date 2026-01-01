#[derive(Clone)]
pub struct ShaderProgram {
    pub text: String,
    pub name: String,
}

pub struct NameGenerator {
    next_id: usize,
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self {
            next_id: Default::default(),
        }
    }
}

impl NameGenerator {
    pub fn generate_name(&mut self) -> String {
        let st = format!("id{}", self.next_id);
        self.next_id += 1;
        st
    }

    pub fn reset(&mut self) {
        self.next_id = 0;
    }
}

pub struct Executor {
    pub namer: NameGenerator,
}
pub enum ExecutionInformation {
    Add,
    Exp,
    Constant(f32),
    Attr(String),
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            namer: Default::default(),
        }
    }
}

impl Executor {
    pub fn run(&mut self) -> ShaderProgram {
        todo!()
    }
}
