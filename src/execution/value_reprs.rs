use std::{collections::HashMap};

use rpds::HashTrieMap;

#[derive(Clone)]
pub struct MultiValueIndex<'a> {
    pub param_inds: HashTrieMap<&'a str, usize>
}

pub struct InputSet {
    pub params: HashTrieMap<String, ParameterValueSet>,
}

struct InputSetIterator<'a> {
    order: Vec<&'a str>,
    lens: Vec<usize>,
    cur: Option<MultiValueIndex<'a>>,
    first: bool
}

impl<'a> Iterator for InputSetIterator<'a> {
    type Item = MultiValueIndex<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.cur {
            return None;
        }

        if self.first {
            self.first = false;
            return self.cur.clone();
        }

        for i in (0..self.lens.len()).rev() {
            let key = self.order[i];
            let cur_len = self.lens[i];
            let cur_val = self.cur.as_mut().unwrap().param_inds.get_mut(key).unwrap();
            *cur_val = *cur_val + 1;
            if *cur_val < cur_len {
                return self.cur.clone();
            }

            *cur_val = 0;
        }

        self.cur = None;
        None
    }
}

impl InputSet {
    pub fn iter_inds<'a>(&'a self) -> InputSetIterator<'a> {
        let kvs: Vec<(&str, usize)> = self.params.iter().map(|(a,b)| (a.as_ref(), b.cardinality() as usize)).collect();
        let (a, b): (Vec<&str>, Vec<usize>) = kvs.into_iter().unzip();
        let starting_map: HashTrieMap<&str, usize> = a.iter().map(|s| (*s, 0usize)).collect();
        InputSetIterator { order: a, lens: b, cur: Some(MultiValueIndex { param_inds: starting_map }), first: true }
    }
}

enum ParameterValueSet {
    I32Range([i32; 2]),
    U32Range([u32; 2]),

    I32List(Vec<i32>),
    F32List(Vec<f32>),

    FnList(Vec<super::FunctionRepresentation>),
}

impl ParameterValueSet {
    fn cardinality(&self) -> u64 {
        match self {
            // Upper bound is exclusive
            ParameterValueSet::I32Range(bounds) => (bounds[1] - bounds[0]) as u64,
            ParameterValueSet::U32Range(bounds) => (bounds[1] - bounds[0]) as u64,
            ParameterValueSet::I32List(items) => items.len() as u64,
            ParameterValueSet::F32List(items) => items.len() as u64,
            ParameterValueSet::FnList(function_representations) => {
                function_representations.len() as u64
            }
        }
    }
}

impl InputSet {
    pub fn cardinality(&self) -> u64 {
        self.params
            .values()
            .into_iter()
            .map(ParameterValueSet::cardinality)
            .product()
    }
}


pub trait AtLeastOneValueRepresentation<El: SingleValueRepresentation> {
    fn get<'a>(&self, ind: MultiValueIndex<'a>) -> El;
}

pub trait PrimitiveValue: Copy + std::fmt::Debug + PartialEq {

}

impl PrimitiveValue for i32 {}
impl PrimitiveValue for u8 {}
impl PrimitiveValue for u32 {}
impl PrimitiveValue for f32 {}

pub trait SingleValueRepresentation: Sized + Clone {
    type MultiValueRepresentations: MultiValueRepresentationTrait<Self>;
}

pub trait MultiValueRepresentationTrait<El: SingleValueRepresentation>: AtLeastOneValueRepresentation<El> {

}

pub enum GeneralPurposeMultiValueRepresentation<El: SingleValueRepresentation> {
    Array(ValueArray<El>),
}

impl<El: SingleValueRepresentation> AtLeastOneValueRepresentation<El> for GeneralPurposeMultiValueRepresentation<El> {
    fn get<'a>(&self, ind: MultiValueIndex<'a>) -> El {
        match self {
            GeneralPurposeMultiValueRepresentation::Array(value_array) => value_array.get(ind),
        }
    }
}

impl<El: SingleValueRepresentation> MultiValueRepresentationTrait<El> for GeneralPurposeMultiValueRepresentation<El> {

}

pub struct TextureMultivalue<El: PrimitiveValue> {
    x_name: String,
    y_name: String,
    comp_name: Option<String>,
    val: El // Placeholder for texture handle
}

pub enum PrimitiveMultivalue<El: PrimitiveValue> {
    General(GeneralPurposeMultiValueRepresentation<El>),
    Texture(TextureMultivalue<El>)
}

impl<El: PrimitiveValue> AtLeastOneValueRepresentation<El> for PrimitiveMultivalue<El> {
    fn get<'a>(&self, ind: MultiValueIndex<'a>) -> El {
        match self {
            PrimitiveMultivalue::General(general_purpose_multi_value_representation) => general_purpose_multi_value_representation.get(ind),
            PrimitiveMultivalue::Texture(texture_multivalue) => todo!(),
        }
    }
}

impl<El: PrimitiveValue> MultiValueRepresentationTrait<El> for PrimitiveMultivalue<El> {

}

impl<Prim: PrimitiveValue> SingleValueRepresentation for Prim {
    type MultiValueRepresentations = PrimitiveMultivalue<Prim>;
}

pub enum NOrderValueRepresentation<El: SingleValueRepresentation> {
    Single(El),
    Many(El::MultiValueRepresentations)
}

impl<El: SingleValueRepresentation> AtLeastOneValueRepresentation<El> for NOrderValueRepresentation<El> {
    fn get<'a>(&self, ind: MultiValueIndex<'a>) -> El {
        match self {
            NOrderValueRepresentation::Single(el) => el.clone(),
            NOrderValueRepresentation::Many(m) => m.get(ind),
        }
    }
}

pub struct ValueArray<El: SingleValueRepresentation> {
    arg_name: String,
    els: Vec<NOrderValueRepresentation<El>>
}

impl<El: SingleValueRepresentation> AtLeastOneValueRepresentation<El> for ValueArray<El> {
    fn get<'a>(&self, ind: MultiValueIndex<'a>) -> El {
        self.els[*ind.param_inds.get(AsRef::<str>::as_ref(&self.arg_name)).unwrap()].get(ind)
    }
}