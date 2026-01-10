use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone)]
pub struct ConstantProgram<T: Clone + Copy> {
    pub val: T,
}

pub trait Computation {
    fn my_type(&self) -> impl SuperDetailedType;
    fn get_function_ref(&self) -> ();
}

pub trait Value {}

pub enum PrimitiveType {
    I32,
    F32,
    U32,
    U8,
}

#[derive(Clone, Copy)]
pub struct SuperDetailedComputationTypeId(usize);

pub struct SuperDetailedComputationTypeManager {
    pub types: Vec<SuperDetailedComputationType>,
    pub next_id: SuperDetailedComputationTypeId,
}

pub struct SuperDetailedComputationConstructor {
    pub output_type: SuperDetailedComputationTypeId,
    pub input_type: SuperDetailedComputationTypeId,
    pub concrete_comp: Rc<Program>,
}

pub struct Program;

pub struct SuperDetailedComputationType {
    pub concrete_computation_fn: Rc<Program>,
    pub remaining_to_specify: HashSet<String>,
    pub contextual_fields: Vec<SuperDetailedComputationTypeId>,
    pub already_computed_nexts: HashMap<String, SuperDetailedComputationConstructor>,
}

impl SuperDetailedComputationType {
    pub fn create_sum_type_no_check(
        t1: SuperDetailedComputationType,
        t2: SuperDetailedComputationType,
    ) -> SuperDetailedComputationType {
        // Don't do any smartness with using the same fields for the first round.
        // Just build a struct where only one field (plus the tag) is populated.
    }
}

pub trait SuperDetailedType: PartialEq + Eq {
    fn get_primitive_type(&self) -> PrimitiveType;

    fn accepts(&self, name: &String) -> bool;

    fn list_all_remaining_args(&self) -> impl Iterator<Item = &String>;

    //fn get_complete_computation(&self) -> impl Computation;

    fn get_computation_for_applying_arg(
        &self,
        arg_name: &String,
        arg_val_handle: impl Value,
    ) -> impl Computation;
}

pub struct ConstantFloatCompType {
    pub val: f32,
}

impl PartialEq for ConstantFloatCompType {
    fn eq(&self, other: &Self) -> bool {
        bytemuck::bytes_of(&self.val) == bytemuck::bytes_of(&other.val)
    }
}

impl Eq for ConstantFloatCompType {}

impl SuperDetailedType for ConstantFloatCompType {
    fn get_primitive_type(&self) -> PrimitiveType {
        PrimitiveType::F32
    }

    fn accepts(&self, name: &String) -> bool {
        false
    }

    fn list_all_remaining_args(&self) -> impl Iterator<Item = &String> {
        [].into_iter()
    }

    fn get_computation_for_applying_arg(
        &self,
        arg_name: &String,
        arg_val_handle: impl Value,
    ) -> impl Computation {
        todo!()
    }
}
