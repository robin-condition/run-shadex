pub enum ShaderScalarType {
    F32,
    I32,
    U8,
    U32,
}

pub enum ShaderVectorLength {
    Two,
    Three,
    Four,
}

pub struct ShaderVectorType(pub ShaderScalarType, pub ShaderVectorLength);

pub enum ShaderVariableType {
    Scalar(ShaderScalarType),
    Vector(ShaderVectorType),
}

pub struct ShaderEntryPoint {}

pub struct ShaderProgram {}
