use crate::pages::{PageAction, PagesRouter, Route};

pub struct ActionContext<'a> {
    pub pages_router: &'a mut PagesRouter,
}

pub fn handle_action(ctx: &mut ActionContext<'_>, action: PageAction) {
    match action {
        PageAction::Navigate(page) => handle_navigate(ctx.pages_router, page),
    }
}

fn handle_navigate(pages_router: &mut PagesRouter, page: Route) {
    pages_router.navigate(page);
}
