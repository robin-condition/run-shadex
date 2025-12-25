use std::marker::PhantomData;

use wgpu::BufferUsages;

pub trait Const32BitSize {
    const SIZE: u32;
}

pub trait Scalar: Const32BitSize {}

pub struct ConstantF32 {}
impl Const32BitSize for ConstantF32 {
    const SIZE: u32 = 4;
}
impl Scalar for ConstantF32 {}
impl AllowedVectorEntry for ConstantF32 {}

pub struct ConstantBool {}
impl Const32BitSize for ConstantBool {
    const SIZE: u32 = 1;
}
impl Scalar for ConstantBool {}

pub struct ConstantU32 {}
impl Const32BitSize for ConstantU32 {
    const SIZE: u32 = 4;
}
impl Scalar for ConstantU32 {}

pub struct ConstantU8 {}
impl Const32BitSize for ConstantU8 {
    const SIZE: u32 = 1;
}
impl Scalar for ConstantU8 {}
impl AllowedVectorEntry for ConstantU8 {}

pub struct ConstantI32 {}
impl Const32BitSize for ConstantI32 {
    const SIZE: u32 = 4;
}
impl Scalar for ConstantI32 {}
impl AllowedVectorEntry for ConstantI32 {}

pub trait AllowedVectorEntry: Scalar {}

pub trait AllowedTextureEntry: Const32BitSize {}
impl<T: AllowedVectorEntry> AllowedTextureEntry for T {}

pub struct Vector<Entry: AllowedVectorEntry, const N: u32> {
    _m: PhantomData<Entry>,
}

impl<Entry: AllowedVectorEntry, const N: u32> Vector<Entry, N> {
    pub const SIZE: u32 = N * Entry::SIZE;
}

impl<T: AllowedVectorEntry, const N: u32> Const32BitSize for Vector<T, N> {
    const SIZE: u32 = Vector::<T, N>::SIZE;
}
impl<T: AllowedVectorEntry, const N: u32> AllowedTextureEntry for Vector<T, N> {}

pub struct Matrix<Entry: AllowedVectorEntry, const ROWS: u32, const COLS: u32> {
    _m: PhantomData<Entry>,
}

impl<Entry: AllowedVectorEntry, const ROWS: u32, const COLS: u32> Const32BitSize
    for Matrix<Entry, ROWS, COLS>
{
    const SIZE: u32 = Entry::SIZE * ROWS * COLS;
}

pub trait RuntimeU64Sizable {
    fn size(&self) -> u64;
}

impl<T: Const32BitSize> RuntimeU64Sizable for T {
    fn size(&self) -> u64 {
        Self::SIZE as u64
    }
}

pub struct TextureND<Entry: AllowedTextureEntry, const D: usize> {
    pub dims: [u32; D],
    _m: PhantomData<Entry>,
}

impl<Entry: AllowedTextureEntry, const D: usize> RuntimeU64Sizable for TextureND<Entry, D> {
    fn size(&self) -> u64 {
        let mut res = 1u64;
        for d in &self.dims {
            res *= *d as u64;
        }
        res
    }
}

pub trait ShaderBufferable: RuntimeU64Sizable {}

impl<S: Scalar> ShaderBufferable for S {}
impl<E: AllowedVectorEntry, const N: u32> ShaderBufferable for Vector<E, N> {}
impl<E: AllowedVectorEntry, const ROWS: u32, const COLS: u32> ShaderBufferable
    for Matrix<E, ROWS, COLS>
{
}
impl<E: AllowedTextureEntry, const DIM: usize> ShaderBufferable for TextureND<E, DIM> {}

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

fn make_buffer<T: ShaderBufferable>(dev: &mut wgpu::Device, q: &mut wgpu::Queue, v: T) {
    /*
    let desc = wgpu::BufferDescriptor {
        label: None,
        size: v.size(),
        usage: BufferUsages::COPY_SRC
            | BufferUsages::MAP_READ
            | BufferUsages::MAP_WRITE
            | BufferUsages::UNIFORM,
        mapped_at_creation: false,
    };
    let buf = dev.create_buffer(&desc);
    let enc = dev.create_command_encoder(desc);
    let pass = enc.begin_render_pass(desc);
    q.write_buffer(buffer, offset, data);
    */
}
