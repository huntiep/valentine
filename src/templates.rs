use Context;
use types::*;

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
#[template = "templates/error.html"]
pub struct TemplateError<'a> {
    msg: &'a str,
}

impl<'a> TemplateError<'a> {
    pub fn new(msg: &'a str) -> Self {
        TemplateError {
            msg: msg,
        }
    }
}

#[derive(BartDisplay)]
#[template = "templates/foot.html"]
pub struct TemplateFoot;

#[derive(BartDisplay)]
#[template_string = "{{{head}}}{{#error}}{{{.}}}{{/error}}{{{body}}}{{{foot}}}"]
pub struct Template<'a, 'b, 'c, T: Display> {
    head: TemplateHead<'a, 'b>,
    error: Option<TemplateError<'c>>,
    body: T,
    foot: TemplateFoot,
}

impl<'a, 'b, 'c, T: Display> Template<'a, 'b, 'c, T> {
    pub fn new(ctx: &'a Context,
               title: Option<&'b str>,
               error: Option<&'c str>,
               body: T)
        -> Self
    {
        let head = TemplateHead::new(ctx, title);
        let error = if let Some(msg) = error {
            Some(TemplateError::new(msg))
        } else {
            None
        };

        Template {
            head: head,
            error: error,
            body: body,
            foot: TemplateFoot,
        }
    }
}

#[derive(BartDisplay)]
#[template = "templates/home.html"]
pub struct HomeTmpl<'a, 'b> {
    pub name: &'a str,
    pub username: Option<&'b str>,
}

#[derive(BartDisplay)]
#[template = "templates/user.html"]
pub struct User<'a> {
    pub name: &'a str,
    pub auth: bool,
    pub username: String,
    pub repos: Vec<Repo>,
}

#[derive(BartDisplay)]
#[template = "templates/user/settings.html"]
pub struct UserSettings {
    pub username: String,
    pub email: String,
    pub keys: Vec<SshKey>,
}

#[derive(BartDisplay)]
#[template = "templates/repo/view.html"]
pub struct RepoTmpl<'a, 'b> {
    pub name: &'a str,
    pub username: &'b str,
    pub repo: Repo,
    pub items: Vec<RepoItem>,
    pub readme: Option<String>,
    pub empty: bool,
}

#[derive(BartDisplay)]
#[template = "templates/repo/settings.html"]
pub struct RepoSettingsTmpl<'a, 'b> {
    pub name: &'a str,
    pub username: &'b str,
    pub repo: Repo,
}

#[derive(BartDisplay)]
#[template = "templates/repo/log.html"]
pub struct RepoLogTmpl<'a, 'b> {
    pub name: &'a str,
    pub username: &'b str,
    pub repo: Repo,
    pub log: Vec<Commit>,
}

#[derive(BartDisplay)]
#[template = "templates/repo/src.html"]
pub struct RepoSrcTmpl<'a, 'b, 'c> {
    pub name: &'a str,
    pub username: &'b str,
    pub repo: Repo,
    pub filename: &'c str,
    pub src: RepoSrc,
}

#[derive(BartDisplay)]
#[template = "templates/repo/issues.html"]
pub struct IssuesTmpl<'a, 'b, 'c> {
    pub name: &'a str,
    pub username: &'b str,
    pub reponame: &'c str,
    pub issues: Vec<Issue>,
    pub auth: bool,
}

#[derive(BartDisplay)]
#[template = "templates/repo/issue.html"]
pub struct IssueTmpl<'a, 'b, 'c> {
    pub name: &'a str,
    pub username: &'b str,
    pub reponame: &'c str,
    pub thread: Vec<Issue>,
    pub auth: bool,
}
