use std::marker::PhantomData;

pub struct ConstantF32 {}
impl Scalar for ConstantF32 {}
impl AllowedVectorEntry for ConstantF32 {}

pub struct ConstantBool {}
impl Scalar for ConstantBool {}

pub struct ConstantU32 {}
impl Scalar for ConstantU32 {}

pub struct ConstantU8 {}
impl Scalar for ConstantU8 {}
impl AllowedVectorEntry for ConstantU8 {}

pub struct ConstantI32 {}
impl Scalar for ConstantI32 {}
impl AllowedVectorEntry for ConstantI32 {}

pub trait Scalar {}
pub trait AllowedVectorEntry {}

pub trait AllowedTextureEntry {}
impl<T: AllowedVectorEntry> AllowedTextureEntry for T {}

pub struct Vector<Entry: AllowedVectorEntry, const N: u32> {
    _m: PhantomData<Entry>,
}

impl<T: AllowedVectorEntry, const N: u32> AllowedTextureEntry for Vector<T, N> {}

pub struct Matrix<Entry: AllowedVectorEntry, const ROWS: u32, const COLS: u32> {
    _m: PhantomData<Entry>,
}

pub struct TextureND<Entry: AllowedTextureEntry, const D: u32> {
    _m: PhantomData<Entry>,
}

pub trait ShaderBufferable {}

impl<S: Scalar> ShaderBufferable for S {}
impl<E: AllowedVectorEntry, const N: u32> ShaderBufferable for Vector<E, N> {}
impl<E: AllowedVectorEntry, const ROWS: u32, const COLS: u32> ShaderBufferable
    for Matrix<E, ROWS, COLS>
{
}
impl<E: AllowedTextureEntry, const DIM: u32> ShaderBufferable for TextureND<E, DIM> {}

pub trait WhichChip {}

pub struct ValueLivesOnCPU {}
impl WhichChip for ValueLivesOnCPU {}
pub struct ValueLivesOnGPU {}
impl WhichChip for ValueLivesOnGPU {}

pub struct LocatedShaderBufferable<Val: ShaderBufferable, Where: WhichChip> {
    _a: PhantomData<Val>,
    _b: PhantomData<Where>,
}

// Just a note for myself about how this kind of thing is meant to be used.
// I'm second guessing the whole "located shader bufferable" thing.
pub type DesiredOutputType =
    LocatedShaderBufferable<TextureND<Vector<ConstantF32, 3>, 2>, ValueLivesOnGPU>;
