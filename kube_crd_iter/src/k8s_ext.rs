use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::{
    CustomResourceDefinition, JSONSchemaProps,
};

use crate::{
    HasProperties, HasVersions, PropertyInfo, PropertyInfoInner, PropertyIter, VersionIter,
};

impl HasVersions for CustomResourceDefinition {
    fn version_iter(&self) -> VersionIter<'_> {
        self.spec.versions.iter()
    }
}

impl<'a> HasProperties<'a> for &'a JSONSchemaProps {
    fn property_iter(self) -> PropertyIter<'a> {
        if let Some(properties) = self.properties.as_ref() {
            Box::new(
                properties
                    .iter()
                    .map(move |(n, s)| (n, s, self))
                    .map(Into::<PropertyInfoInner>::into)
                    .map(|x| PropertyInfo(vec![x])),
            )
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn property_flat_iter(self) -> PropertyIter<'a> {
        Box::new(
            self.property_iter()
                .flat_map(PropertyInfo::property_flat_iter),
        )
    }
}
