use std::fmt::{self, Display};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::{
    JSONSchemaProps,
    JSONSchemaPropsOrArray::{Schema, Schemas},
};

use crate::{HasProperties, PropertyIter};

#[derive(Debug, Clone)]
pub(crate) struct PropertyInfoInner<'a> {
    pub name: String,
    pub is_array: bool,
    pub is_required: bool,
    pub schema: &'a JSONSchemaProps,
}

impl<'a> From<(&String, &'a JSONSchemaProps, &'a JSONSchemaProps)> for PropertyInfoInner<'a> {
    fn from(
        (name, schema, parent_schema): (&String, &'a JSONSchemaProps, &'a JSONSchemaProps),
    ) -> Self {
        Self {
            name: name.clone(),
            is_array: false,
            is_required: parent_schema
                .required
                .as_ref()
                .map(|r| r.contains(name))
                .unwrap_or(false),
            schema,
        }
    }
}
impl fmt::Display for PropertyInfoInner<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.is_array {
            write!(f, "[]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PropertyInfo<'a>(pub(crate) Vec<PropertyInfoInner<'a>>);

impl Display for PropertyInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for p in &self.0 {
            if !first {
                write!(f, ".")?;
            }
            write!(f, "{}", p)?;
            first = false;
        }
        Ok(())
    }
}

impl<'a> PropertyInfo<'a> {
    fn info(&self) -> &PropertyInfoInner<'a> {
        self.0.last().unwrap()
    }
    fn info_mut(&mut self) -> &mut PropertyInfoInner<'a> {
        self.0.last_mut().unwrap()
    }
    pub fn name(&self) -> &str {
        self.info().name.as_str()
    }
    pub fn is_required(&self) -> bool {
        self.info().is_required
    }
    pub fn schema(&self) -> &'a JSONSchemaProps {
        self.info().schema
    }
    pub fn type_(&self) -> &'a str {
        self.schema().type_.as_deref().unwrap_or("object")
    }
    pub fn full_name(&self) -> String {
        self.to_string()
    }
}

impl<'a> HasProperties<'a> for PropertyInfo<'a> {
    fn property_iter(self) -> PropertyIter<'a> {
        if let Some(properties) = self.schema().properties.as_ref() {
            let parent_schema = self.schema();
            Box::new(
                properties
                    .iter()
                    .map(move |(n, s)| (n, s, parent_schema))
                    .map(Into::<PropertyInfoInner<'a>>::into)
                    .map(move |x| self.0.clone().into_iter().chain([x]).collect())
                    .map(PropertyInfo),
            )
        } else if let Some(items) = self.schema().items.as_ref() {
            let mut self_mut = self;
            match items {
                Schema(s) => {
                    self_mut.info_mut().is_array = true;
                    self_mut.info_mut().schema = s;
                }
                Schemas(_) => unimplemented!(),
            }
            self_mut.property_flat_iter()
        } else {
            Box::new(std::iter::empty())
        }
    }
    fn property_flat_iter(self) -> PropertyIter<'a> {
        let self_iter = std::iter::once(self.clone());
        let prop_iter = self
            .property_iter()
            .flat_map(PropertyInfo::property_flat_iter);
        Box::new(self_iter.chain(prop_iter))
    }
}
