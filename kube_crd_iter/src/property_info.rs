use std::fmt::{self, Display};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::{
    JSONSchemaProps,
    JSONSchemaPropsOrArray::{Schema, Schemas},
};

use crate::{HasProperties, PropertyIter};

#[derive(Debug, Clone)]
pub(crate) struct PropertyInfoInner<'a> {
    pub name: String,
    pub array_level: u8,
    pub is_required: bool,
    pub schema: &'a JSONSchemaProps,
}

impl<'a> From<(&String, &'a JSONSchemaProps, &'a JSONSchemaProps)> for PropertyInfoInner<'a> {
    fn from(
        (name, schema, parent_schema): (&String, &'a JSONSchemaProps, &'a JSONSchemaProps),
    ) -> Self {
        let mut schema = schema;
        let mut array_level = 0;
        while let Some(items) = schema.items.as_ref() {
            match items {
                Schema(s) => {
                    schema = s;
                    array_level += 1;
                }
                Schemas(_) => unimplemented!(),
            }
        }

        Self {
            name: name.clone(),
            array_level,
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
        for _ in 0..self.array_level {
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
    fn inner(&self) -> &PropertyInfoInner<'a> {
        self.0.last().unwrap()
    }
    pub fn name(&self) -> String {
        self.inner().to_string()
    }
    pub fn is_required(&self) -> bool {
        self.inner().is_required
    }
    pub fn schema(&self) -> &'a JSONSchemaProps {
        self.inner().schema
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
        let schema = self.schema();
        if let Some(properties) = schema.properties.as_ref() {
            Box::new(
                properties
                    .iter()
                    .map(move |(n, s)| (n, s, schema))
                    .map(Into::<PropertyInfoInner<'a>>::into)
                    .map(move |x| self.0.clone().into_iter().chain([x]).collect())
                    .map(PropertyInfo),
            )
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
