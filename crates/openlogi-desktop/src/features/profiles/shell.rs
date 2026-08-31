//! Reusable profile tabs and feature-independent controls.

use gpui::{
    AnyElement, App, ClickEvent, Context, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled, Window,
    div, img, prelude::FluentBuilder as _, px,
};
use gpui_base::Button as BaseButton;
use gpui_component::{
    Icon, IconName, Sizable as _,
    h_flex,
    menu::{ContextMenuExt as _, PopupMenu, PopupMenuItem},
    spinner::Spinner,
};

use super::catalog::{AppCatalogPicker, AppIconState, ProfileIconCache};
use super::picker::add_app_popover;
use super::{ProfileChoice, ProfileScopeActions, ProfileScopeModel};
use crate::ui::theme::{self, Palette, SelectableStyle as _, Typography as _};

/// Icon edge inside single-line profile tabs.
const TAB_ICON_EDGE: f32 = 18.;

/// Feature-independent profile switcher. Feature adapters provide the
/// profiles and own selection/removal behavior through [`ProfileScopeActions`].
#[derive(IntoElement)]
pub(crate) struct ProfileScopeShell {
    id_base: &'static str,
    model: ProfileScopeModel,
    catalog: Entity<AppCatalogPicker>,
    icons: ProfileIconCache,
    actions: ProfileScopeActions,
}

impl ProfileScopeShell {
    pub(super) fn new(
        id_base: &'static str,
        model: ProfileScopeModel,
        catalog: Entity<AppCatalogPicker>,
        icons: ProfileIconCache,
        actions: ProfileScopeActions,
    ) -> Self {
        Self {
            id_base,
            model,
            catalog,
            icons,
            actions,
        }
    }
}

impl RenderOnce for ProfileScopeShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let pal = theme::palette(cx);
        profile_scope_content(self, pal)
    }
}

fn profile_scope_content(shell: ProfileScopeShell, pal: Palette) -> impl IntoElement + use<> {
    let default_selected = shell.model.editing_app.is_none();
    let profile_tabs = shell
        .model
        .profiles
        .iter()
        .map(|profile| {
            let selected = shell.model.editing_app.as_deref() == Some(profile.app.as_str());
            let app = profile.app.clone();
            let actions = shell.actions.clone();
            let tab_id = format!("{}:app:{}", shell.id_base, profile.app);
            let remove = profile.persisted.then(|| {
                (
                    format!("{tab_id}:remove"),
                    profile.clone(),
                    shell.actions.clone(),
                )
            });
            profile_tab(
                tab_id,
                profile.name.clone(),
                Some(application_mark(
                    shell.icons.state(&profile.app),
                    &profile.name,
                    TAB_ICON_EDGE,
                    pal,
                )),
                selected,
                pal,
                remove,
                move |_event, _window, cx| {
                    actions.select(Some(app.clone()), cx);
                },
            )
        })
        .collect::<Vec<_>>();
    let default_actions = shell.actions.clone();

    h_flex()
        .flex_shrink_0()
        .w_full()
        .items_center()
        .gap_2()
        .border_b_1()
        .border_color(pal.border)
        .bg(pal.panel)
        .px_4()
        .py_2()
        .child(
            div()
                .flex_none()
                .text_body()
                .text_color(pal.text_muted)
                .child(tr!("profiles.profile")),
        )
        .child(
            h_flex()
                .id(format!("{}:tabs-scroll", shell.id_base))
                .flex_1()
                .min_w_0()
                .items_center()
                .gap_1()
                .overflow_x_scroll()
                .child(
                    profile_tab(
                        format!("{}:default", shell.id_base),
                        tr!("common.default"),
                        None,
                        default_selected,
                        pal,
                        None,
                        move |_event, _window, cx| {
                            default_actions.select(None, cx);
                        },
                    ),
                )
                .children(profile_tabs),
        )
        .child(add_app_popover(
            shell.id_base,
            shell.model.choices,
            shell.catalog,
            shell.icons,
            shell.actions,
            pal,
        ))
}

fn profile_tab(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    leading: Option<gpui::Div>,
    selected: bool,
    pal: Palette,
    remove: Option<(String, ProfileChoice, ProfileScopeActions)>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let label = label.into();
    let has_remove = remove.is_some();
    let context_menu = remove.as_ref().map(|(_, profile, actions)| {
        profile_remove_context_menu(profile.clone(), actions.clone())
    });
    let tab = BaseButton::new(id)
        .role(Role::Tab)
        .selected(selected)
        .accessibility_label(label.clone())
        .aria_selected(selected)
        .flex()
        .flex_none()
        .items_center()
        .gap_1p5()
        .h(px(theme::CONTROL_H))
        .when(has_remove, |tab| tab.pl_2p5().pr_1())
        .when(!has_remove, |tab| tab.px_2p5())
        .rounded(pal.control_radius)
        .cursor_pointer()
        .text_body()
        .text_color(pal.text_primary)
        .selected_fill(selected)
        .hover(move |tab| {
            tab.bg(if selected {
                theme::accent_tint_hover()
            } else {
                pal.control_hover
            })
        })
        .focus_visible(move |tab| {
            tab.bg(if selected {
                theme::accent_tint_hover()
            } else {
                pal.control_hover
            })
        })
        .children(leading)
        .child({
            let remove_label = label.clone();
            if let Some((delete_id, profile, actions)) = &remove {
                let profile_for_button = profile.clone();
                let actions_for_button = actions.clone();
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .child(label)
                    .child(
                        BaseButton::new(delete_id.clone())
                            .accessibility_label(tr!(
                                "Remove %{name}",
                                name => remove_label.to_string()
                            ))
                            .px_0p5()
                            .rounded(pal.control_radius)
                            .text_color(pal.text_muted)
                            .hover(|button| button.text_color(pal.text_primary))
                            .focus_visible(|button| button.text_color(pal.text_primary))
                            .child(Icon::new(IconName::Close).size_3())
                            .on_click(move |_event, window, cx| {
                                cx.stop_propagation();
                                actions_for_button.remove(profile_for_button.clone(), window, cx);
                            }),
                    )
            } else {
                div().child(label)
            }
        })
        .on_click(on_click);

    if let Some(menu) = context_menu {
        div()
            .flex_none()
            .child(tab)
            .context_menu(menu)
            .into_any_element()
    } else {
        div().flex_none().child(tab).into_any_element()
    }
}

fn profile_remove_context_menu(
    profile: ProfileChoice,
    actions: ProfileScopeActions,
) -> impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + Clone + 'static {
    move |menu, _window, _cx| {
        let profile = profile.clone();
        let actions = actions.clone();
        menu.item(
            PopupMenuItem::new(tr!("Remove profile…"))
                .icon(IconName::Close)
                .on_click(move |_, window, cx| {
                    actions.remove(profile.clone(), window, cx);
                }),
        )
    }
}

pub(super) fn application_mark(
    icon: AppIconState,
    name: &str,
    edge: f32,
    pal: Palette,
) -> gpui::Div {
    let slot = h_flex()
        .size(px(edge))
        .flex_none()
        .items_center()
        .justify_center();
    match icon {
        AppIconState::Ready(icon) => slot.child(img(icon).size(px(edge)).flex_none()),
        AppIconState::Loading => slot.child(
            Spinner::new()
                .with_size(px(edge * 0.6))
                .color(pal.text_muted),
        ),
        AppIconState::Missing => {
            let initial = name
                .chars()
                .find(|character| !character.is_whitespace())
                .map_or_else(|| "?".to_string(), |character| character.to_string());
            slot.rounded(px(edge / 4.))
                .bg(pal.muted)
                .map(|tile| {
                    if edge < 24. {
                        tile.text_caption()
                    } else {
                        tile.text_body()
                    }
                })
                .text_color(pal.text_muted)
                .child(initial)
        }
    }
}
