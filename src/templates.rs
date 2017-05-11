use Context;
use routes::types::*;

use std::fmt::Display;

#[derive(BartDisplay)]
#[template = "templates/head.html"]
pub struct TemplateHead<'a, 'b> {
    pub name: &'a str,
    pub title: Option<&'b str>,
}

impl<'a, 'b> TemplateHead<'a, 'b> {
    pub fn new(ctx: &'a Context, title: Option<&'b str>) -> Self {
        TemplateHead {
            name: &ctx.name,
            title: title,
        }
    }
}

#[derive(BartDisplay)]
#[template = "templates/foot.html"]
pub struct TemplateFoot;

#[derive(BartDisplay)]
#[template_string = "{{head}}{{body}}{{foot}}"]
pub struct Template<'a, 'b, T: Display> {
    head: TemplateHead<'a, 'b>,
    body: T,
    foot: TemplateFoot,
}

impl<'a, 'b, T: Display> Template<'a, 'b, T> {
    pub fn new(ctx: &'a Context, title: Option<&'b str>, body: T) -> Self {
        let head = TemplateHead::new(ctx, title);
        Template {
            head: head,
            body: body,
            foot: TemplateFoot,
        }
    }
}

#[derive(BartDisplay)]
#[template = "templates/user.html"]
pub struct User {
    pub username: String,
    pub repos: Vec<Repo>,
}
