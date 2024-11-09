use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinitionVersion;

use crate::PropertyInfo;

pub type VersionIter<'a> = std::slice::Iter<'a, CustomResourceDefinitionVersion>;
pub trait HasVersions {
    fn version_iter(&self) -> VersionIter<'_>;
}

pub type PropertyIter<'a> = Box<dyn Iterator<Item = PropertyInfo<'a>> + 'a>;
pub trait HasProperties<'a> {
    fn property_flat_iter(self) -> PropertyIter<'a>;
    fn property_iter(self) -> PropertyIter<'a>;
}
