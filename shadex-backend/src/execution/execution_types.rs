use std::marker::PhantomData;

use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferUsages, TextureView,
};

pub trait Const32BitSize {
    const SIZE: u32;
}

pub trait CanLiveOnCPU {
    type CPUStoredType: Sized;
}

pub trait CanLiveOnGPU {
    type GPUStorageHandleType: Sized;
}

pub trait Scalar: Const32BitSize + CanLiveOnCPU {}

pub struct ConstantF32 {}
impl Const32BitSize for ConstantF32 {
    const SIZE: u32 = 4;
}
impl CanLiveOnCPU for ConstantF32 {
    type CPUStoredType = f32;
}
impl Scalar for ConstantF32 {}
impl AllowedVectorEntry for ConstantF32 {}

pub struct ConstantBool {}
impl Const32BitSize for ConstantBool {
    const SIZE: u32 = 1;
}
impl CanLiveOnCPU for ConstantBool {
    type CPUStoredType = bool;
}
impl Scalar for ConstantBool {}

pub struct ConstantU32 {}
impl Const32BitSize for ConstantU32 {
    const SIZE: u32 = 4;
}
impl CanLiveOnCPU for ConstantU32 {
    type CPUStoredType = u32;
}
impl Scalar for ConstantU32 {}

pub struct ConstantU8 {}
impl Const32BitSize for ConstantU8 {
    const SIZE: u32 = 1;
}
impl CanLiveOnCPU for ConstantU8 {
    type CPUStoredType = u8;
}
impl Scalar for ConstantU8 {}
impl AllowedVectorEntry for ConstantU8 {}

pub struct ConstantI32 {}
impl Const32BitSize for ConstantI32 {
    const SIZE: u32 = 4;
}
impl CanLiveOnCPU for ConstantI32 {
    type CPUStoredType = i32;
}
impl Scalar for ConstantI32 {}
impl AllowedVectorEntry for ConstantI32 {}

pub trait AllowedVectorEntry: Scalar {}

pub trait AllowedTextureEntry: Const32BitSize + CanLiveOnCPU + CanLiveOnGPU {}
impl<T: AllowedVectorEntry> AllowedTextureEntry for T {}

pub struct Vector<Entry: AllowedVectorEntry, const N: usize> {
    _m: PhantomData<Entry>,
}

impl<Entry: AllowedVectorEntry, const N: usize> Vector<Entry, N> {
    pub const SIZE: u32 = N as u32 * Entry::SIZE;
}

impl<T: AllowedVectorEntry, const N: usize> Const32BitSize for Vector<T, N> {
    const SIZE: u32 = Vector::<T, N>::SIZE;
}
impl<T: AllowedVectorEntry, const N: usize> AllowedTextureEntry for Vector<T, N> {}

pub struct Matrix<Entry: AllowedVectorEntry, const ROWS: usize, const COLS: usize> {
    _m: PhantomData<Entry>,
}

impl<Entry: AllowedVectorEntry, const ROWS: usize, const COLS: usize> Const32BitSize
    for Matrix<Entry, ROWS, COLS>
{
    const SIZE: u32 = Entry::SIZE * ROWS as u32 * COLS as u32;
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

pub trait ShaderTransferable: RuntimeU64Sizable + CanLiveOnCPU + CanLiveOnGPU {}

impl<S: Scalar> CanLiveOnGPU for S {
    type GPUStorageHandleType = Buffer;
}
impl<S: Scalar> ShaderTransferable for S {}
impl<E: AllowedVectorEntry, const N: usize> CanLiveOnGPU for Vector<E, N> {
    type GPUStorageHandleType = Buffer;
}
impl<E: AllowedVectorEntry, const N: usize> CanLiveOnCPU for Vector<E, N> {
    type CPUStoredType = [E::CPUStoredType; N];
}
impl<E: AllowedVectorEntry, const N: usize> ShaderTransferable for Vector<E, N> {}

impl<E: AllowedVectorEntry, const ROWS: usize, const COLS: usize> CanLiveOnGPU
    for Matrix<E, ROWS, COLS>
{
    type GPUStorageHandleType = Buffer;
}
impl<E: AllowedVectorEntry, const ROWS: usize, const COLS: usize> CanLiveOnCPU
    for Matrix<E, ROWS, COLS>
{
    type CPUStoredType = [[E::CPUStoredType; COLS]; ROWS];
}
impl<E: AllowedVectorEntry, const ROWS: usize, const COLS: usize> ShaderTransferable
    for Matrix<E, ROWS, COLS>
{
}

impl<E: AllowedTextureEntry, const DIM: usize> CanLiveOnGPU for TextureND<E, DIM> {
    type GPUStorageHandleType = TextureView;
}
impl<E: AllowedTextureEntry, const DIM: usize> CanLiveOnCPU for TextureND<E, DIM> {
    type CPUStoredType = Vec<E::CPUStoredType>;
}
impl<E: AllowedTextureEntry, const DIM: usize> ShaderTransferable for TextureND<E, DIM> {}

pub trait WhichChip {}

pub struct ValueLivesOnCPU {}
impl WhichChip for ValueLivesOnCPU {}
pub struct ValueLivesOnGPU {}
impl WhichChip for ValueLivesOnGPU {}

pub struct ValueOnCPU<T: CanLiveOnCPU> {
    pub item: T::CPUStoredType,
}

pub struct ValueOnGPU<T: CanLiveOnGPU> {
    pub handle: T::GPUStorageHandleType,
}

pub struct ValueMaybeOnCPU<T: CanLiveOnCPU> {
    pub opt: Option<ValueOnCPU<T>>,
}

pub struct ValueMaybeOnGPU<T: CanLiveOnGPU> {
    pub opt: Option<ValueOnGPU<T>>,
}

pub struct ValueMaybeOnEitherCPUorGPU<T: CanLiveOnCPU + CanLiveOnGPU> {
    pub cpu: ValueMaybeOnCPU<T>,
    pub gpu: ValueMaybeOnGPU<T>,
}

pub struct LocatedShaderBufferable<Val: ShaderTransferable, Where: WhichChip> {
    _a: PhantomData<Val>,
    _b: PhantomData<Where>,
}

// Just a note for myself about how this kind of thing is meant to be used.
// I'm second guessing the whole "located shader bufferable" thing.
pub type DesiredOutputType =
    LocatedShaderBufferable<TextureND<Vector<ConstantF32, 3>, 2>, ValueLivesOnGPU>;

fn make_buffer<T: ShaderTransferable>(dev: &mut wgpu::Device, q: &mut wgpu::Queue, v: T) {
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
    //q.write_texture(texture, data, data_layout, size);
    /*BindGroupDescriptor {
        label: todo!(),
        layout: todo!(),
        entries: &[BindGroupEntry {
            binding: todo!(),
            resource: BindingResource::TextureView(()),
        }],
    }*/
}
