use super::{LanguageParser, ParsedSymbol};
use anyhow::Result;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

pub struct RustParser;

struct RustVisitor {
    items: Vec<ParsedSymbol>,
    path_stack: Vec<String>,
}

impl RustVisitor {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            path_stack: Vec::new(),
        }
    }

    fn current_path(&self) -> String {
        self.path_stack.join("::")
    }

    fn push_path(&mut self, name: String) {
        self.path_stack.push(name);
    }

    fn pop_path(&mut self) {
        self.path_stack.pop();
    }
}

impl<'ast> Visit<'ast> for RustVisitor {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        let name = i.ident.to_string();
        let start = i.span().start().line;
        let end = i.span().end().line;
        self.push_path(name.clone());
        if i.content.is_some() {
            self.items.push(ParsedSymbol {
                name: self.current_path(),
                start_line: start,
                end_line: end,
            });
        }
        visit::visit_item_mod(self, i);
        self.pop_path();
    }

    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        let name = i.sig.ident.to_string();
        let start = i.span().start().line;
        let end = i.span().end().line;
        let full_name = if self.path_stack.is_empty() {
            name
        } else {
            format!("{}::{}", self.current_path(), name)
        };
        self.items.push(ParsedSymbol {
            name: full_name,
            start_line: start,
            end_line: end,
        });
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        let name = i.ident.to_string();
        let start = i.span().start().line;
        let end = i.span().end().line;
        let full_name = if self.path_stack.is_empty() {
            name
        } else {
            format!("{}::{}", self.current_path(), name)
        };
        self.items.push(ParsedSymbol {
            name: full_name,
            start_line: start,
            end_line: end,
        });
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        let name = i.ident.to_string();
        let start = i.span().start().line;
        let end = i.span().end().line;
        let full_name = if self.path_stack.is_empty() {
            name
        } else {
            format!("{}::{}", self.current_path(), name)
        };
        self.items.push(ParsedSymbol {
            name: full_name,
            start_line: start,
            end_line: end,
        });
    }

    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        let name = i.ident.to_string();
        let start = i.span().start().line;
        let end = i.span().end().line;
        let full_name = if self.path_stack.is_empty() {
            name
        } else {
            format!("{}::{}", self.current_path(), name)
        };
        self.items.push(ParsedSymbol {
            name: full_name,
            start_line: start,
            end_line: end,
        });
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        let self_ty = &i.self_ty;
        let type_name = quote::quote!(#self_ty).to_string().replace(" ", "");
        self.push_path(type_name.clone());
        let start = i.span().start().line;
        let end = i.span().end().line;
        self.items.push(ParsedSymbol {
            name: self.current_path(),
            start_line: start,
            end_line: end,
        });
        visit::visit_item_impl(self, i);
        self.pop_path();
    }

    fn visit_impl_item_fn(&mut self, i: &'ast syn::ImplItemFn) {
        let name = i.sig.ident.to_string();
        let start = i.span().start().line;
        let end = i.span().end().line;
        let full_name = format!("{}::{}", self.current_path(), name);
        self.items.push(ParsedSymbol {
            name: full_name,
            start_line: start,
            end_line: end,
        });
    }
}

impl LanguageParser for RustParser {
    fn parse(&self, content: &str) -> Result<Vec<ParsedSymbol>> {
        let file = syn::parse_file(content)?;
        let mut visitor = RustVisitor::new();
        visitor.visit_file(&file);
        Ok(visitor.items)
    }
}
