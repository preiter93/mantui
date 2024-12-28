pub(super) mod help;
pub(super) mod home;
pub(super) mod list;
pub(super) mod manual;
mod utils;

pub(super) use home::{HomePage, HomePageState};
pub(super) use list::{ListPage, ListPageState};
pub(super) use manual::{ManPage, ManPageState};

// pub(super) enum Page {
//     None,
//     Home(HomePageState),
//     List(ListPageState),
//     Desc(ManPageState),
// }
//
// fn drop_page(ctx: &mut AppContext) {
//     match &ctx.current_page {
//         Page::Home(_) => {
//             HomePageState::on_drop(ctx);
//         }
//         Page::List(_) => {
//             ListPageState::on_drop(ctx);
//         }
//         Page::Desc(_) => {
//             ManPageState::on_drop(ctx);
//         }
//         Page::None => {}
//     }
// }

// fn switch_page(ctx: &mut AppContext, next_page: Page) {
//     let prev_page = std::mem::replace(&mut ctx.current_page, next_page);
//
//     match prev_page {
//         Page::Home(state) => {
//             HomePageState::on_drop(ctx);
//         }
//         // Page::SearchPage(state) => {
//         //     state.on_drop(ctx);
//         // }
//         Page::Man(state) => {
//             ManPageState::on_drop(ctx);
//         }
//         Page::None => {}
//     }
// }
