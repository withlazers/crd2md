mod util;

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::{
    CustomResourceDefinition, CustomResourceDefinitionVersion,
};
use kube_crd_iter::{HasProperties, HasVersions, PropertyInfo, PropertyIter};
use markdown_ast::{
    ast_to_markdown, markdown_to_ast,
    Block::{self, *},
    HeadingLevel::*,
    Inline, Inlines,
};
use pulldown_cmark::{Alignment, LinkType};
use util::to_anchor;

fn property_table(
    prop: PropertyIter<'_>,
    f: impl Fn(&PropertyInfo) -> String,
) -> Block {
    Table {
        alignments: vec![Alignment::Left, Alignment::Left, Alignment::Center],
        headers: vec![
            Inlines(vec![Inline::plain_text("Property")]),
            Inlines(vec![Inline::plain_text("Type")]),
            Inlines(vec![Inline::plain_text("Required")]),
        ],
        rows: prop.map(|x| property_table_row(x, &f)).collect(),
    }
}

fn property_detail(prop: PropertyInfo) -> Vec<Block> {
    let full_name = prop.full_name();
    let mut blocks = vec![];

    blocks.extend([
        Rule,
        Heading(H3, Inlines(vec![Inline::plain_text(&full_name)])),
        Paragraph(Inlines(vec![
            Inline::plain_text("Type: "),
            Inline::plain_text(prop.type_()),
        ])),
    ]);
    if prop.schema().properties.is_some() {
        blocks.push(property_table(prop.clone().property_iter(), |p| {
            p.name().to_string()
        }));
    }

    if let Some(validations) = &prop.schema().x_kubernetes_validations {
        blocks.extend([
            Heading(H4, Inlines(vec![Inline::plain_text("Validations")])),
            Table {
                alignments: vec![Alignment::Left, Alignment::Left],
                headers: vec![
                    Inlines(vec![Inline::plain_text("Rule")]),
                    Inlines(vec![Inline::plain_text("Error Message")]),
                ],
                rows: validations
                    .iter()
                    .map(|v| {
                        vec![
                            Inlines(vec![Inline::plain_text(&v.rule)]),
                            Inlines(vec![Inline::plain_text(
                                v.message.clone().unwrap_or_default(),
                            )]),
                        ]
                    })
                    .collect(),
            },
        ]);
    }

    blocks.extend(markdown_to_ast(
        prop.schema()
            .description
            .as_ref()
            .unwrap_or(&"*missing*".to_string()),
    ));

    blocks
}

fn property_table_row(
    prop: PropertyInfo,
    f: &impl Fn(&PropertyInfo) -> String,
) -> Vec<Inlines> {
    let full_name = prop.full_name();

    let link = Inline::Link {
        link_type: LinkType::Inline,
        dest_url: to_anchor(&full_name),
        title: "".to_string(),
        id: "".to_string(),
        content_text: Inlines(vec![Inline::plain_text(f(&prop))]),
    };
    vec![
        Inlines(vec![link]),
        Inlines(vec![Inline::plain_text(prop.type_())]),
        Inlines(vec![Inline::plain_text({
            if prop.is_required() {
                "✅"
            } else {
                ""
            }
        })]),
    ]
}

fn version(version: &CustomResourceDefinitionVersion) -> Vec<Block> {
    let mut blocks = vec![];
    blocks.extend(vec![Heading(
        H2,
        markdown_ast::Inlines(vec![Inline::plain_text(version.name.clone())]),
    )]);
    let schema = version
        .schema
        .as_ref()
        .and_then(|s| s.open_api_v3_schema.as_ref())
        .unwrap();
    blocks.extend(markdown_to_ast(
        schema
            .description
            .as_ref()
            .unwrap_or(&"*missing*".to_string()),
    ));
    blocks.push(property_table(schema.property_flat_iter(), |p| {
        p.full_name()
    }));

    blocks.extend(schema.property_flat_iter().flat_map(property_detail));

    blocks
}

fn crd(crd: &CustomResourceDefinition) -> Vec<Block> {
    let mut blocks = vec![Heading(
        H1,
        markdown_ast::Inlines(vec![Inline::plain_text(&crd.spec.names.kind)]),
    )];
    blocks.extend(crd.version_iter().flat_map(version));
    blocks
}

pub trait ToMarkdown {
    fn to_markdown(&self) -> String;
}

impl ToMarkdown for CustomResourceDefinition {
    fn to_markdown(&self) -> String {
        ast_to_markdown(&crd(self))
    }
}
