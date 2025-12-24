use std::collections::HashMap;

use shadex_backend::{
    nodegraph::NodeTypeInfo,
    typechecking::typetypes::{PrimitiveType, U32Boundedness, ValueType},
};

// TODO: Replace this with some kind of "template-reduction-time" expression
type TemplatePlaceholderIdentifier = String;

pub enum TemplatePlaceholder {
    U32Bound(u32),
    PrimitiveType(PrimitiveType),
    ArgumentName(String),
    ValueType(ValueType),
    ArgsList(Vec<(String, ValueType)>),

    MiscellaneousF32s(Vec<f32>),
}

pub enum U32BoundednessTemplate {
    Unbounded,
    Bounded(u32),
    BoundedPlaceholder(TemplatePlaceholderIdentifier),
}

impl U32BoundednessTemplate {
    pub fn evaluate(
        &self,
        ctx: &HashMap<TemplatePlaceholderIdentifier, TemplatePlaceholder>,
    ) -> Result<U32Boundedness, ()> {
        match self {
            U32BoundednessTemplate::Unbounded => Ok(U32Boundedness::Unbounded),
            U32BoundednessTemplate::Bounded(bd) => Ok(U32Boundedness::Bounded(*bd)),
            U32BoundednessTemplate::BoundedPlaceholder(id) => {
                Ok(U32Boundedness::Bounded(match ctx.get(id) {
                    Some(TemplatePlaceholder::U32Bound(bd)) => Ok(*bd),
                    _ => Err(()),
                }?))
            }
        }
    }
}

pub enum PrimitiveTypeTemplate {
    Placeholder(TemplatePlaceholderIdentifier),
    F32,
    I32,
    U32(U32BoundednessTemplate),
}

impl PrimitiveTypeTemplate {
    pub fn evaluate(
        &self,
        ctx: &HashMap<TemplatePlaceholderIdentifier, TemplatePlaceholder>,
    ) -> Result<PrimitiveType, ()> {
        match self {
            PrimitiveTypeTemplate::Placeholder(id) => match ctx.get(id) {
                Some(TemplatePlaceholder::PrimitiveType(typ)) => Ok(*typ),
                _ => Err(()),
            },
            PrimitiveTypeTemplate::F32 => Ok(PrimitiveType::F32),
            PrimitiveTypeTemplate::I32 => Ok(PrimitiveType::I32),
            PrimitiveTypeTemplate::U32(u32_boundedness_template) => {
                Ok(PrimitiveType::U32(u32_boundedness_template.evaluate(ctx)?))
            }
        }
    }
}

pub enum StringTemplate {
    Literal(String),
    Placeholder(TemplatePlaceholderIdentifier),
}

impl StringTemplate {
    pub fn evaluate(
        &self,
        ctx: &HashMap<TemplatePlaceholderIdentifier, TemplatePlaceholder>,
    ) -> Result<String, ()> {
        match self {
            StringTemplate::Literal(lit) => Ok(lit.clone()),
            StringTemplate::Placeholder(id) => match ctx.get(id) {
                Some(TemplatePlaceholder::ArgumentName(name)) => Ok(name.clone()),
                _ => Err(()),
            },
        }
    }
}

pub enum ValueTypeTemplate {
    Placeholder(TemplatePlaceholderIdentifier),
    ConcreteShape(ShapedValueTypeTemplate),
}

impl ValueTypeTemplate {
    pub fn evaluate(
        &self,
        ctx: &HashMap<TemplatePlaceholderIdentifier, TemplatePlaceholder>,
    ) -> Result<ValueType, ()> {
        match self {
            ValueTypeTemplate::Placeholder(id) => match ctx.get(id) {
                Some(TemplatePlaceholder::ValueType(vt)) => Ok(vt.clone()),
                _ => Err(()),
            },
            ValueTypeTemplate::ConcreteShape(shaped_value_type_template) => {
                shaped_value_type_template.evaluate(ctx)
            }
        }
    }
}

pub struct ShapedValueTypeTemplate {
    pub args: Vec<(StringTemplate, ValueTypeTemplate)>,
    pub args_to_absorb: Vec<TemplatePlaceholderIdentifier>,
    pub out: PrimitiveTypeTemplate,
}

impl ShapedValueTypeTemplate {
    pub fn evaluate(
        &self,
        ctx: &HashMap<TemplatePlaceholderIdentifier, TemplatePlaceholder>,
    ) -> Result<ValueType, ()> {
        let mut args = HashMap::new();
        for (key, v) in &self.args {
            let name = key.evaluate(ctx)?;
            let vt = v.evaluate(ctx)?;
            args.insert(name, Box::new(vt));
        }

        for id in &self.args_to_absorb {
            let list = match ctx.get(id) {
                Some(TemplatePlaceholder::ArgsList(list)) => Ok(list),
                _ => Err(()),
            }?;

            for (n, v) in list {
                args.insert(n.clone(), Box::new(v.clone()));
            }
        }

        Ok(ValueType {
            inputs: args,
            output: self.out.evaluate(ctx)?,
        })
    }
}

pub struct TemplatedInputInfo {
    name: StringTemplate,
    val_type: ValueTypeTemplate,
}

pub struct TemplatedOutputInfo {
    name: StringTemplate,
    val_type: ValueTypeTemplate,
}

pub struct TemplatedNodeTypeInfo {
    input_types: Vec<TemplatedInputInfo>,
    output_types: Vec<TemplatedOutputInfo>,
}
