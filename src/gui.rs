use crate::load_image;
use rg3d::{
    core::pool::Handle,
    gui::{
        border::BorderBuilder,
        button::{ButtonBuilder, ButtonMessage},
        decorator::DecoratorBuilder,
        define_constructor,
        grid::{Column, GridBuilder, Row},
        image::ImageBuilder,
        message::{MessageDirection, UiMessage},
        text::TextBuilder,
        widget::{Widget, WidgetBuilder},
        BuildContext, Control, HorizontalAlignment, NodeHandleMapping, Thickness, UiNode,
        UserInterface, VerticalAlignment,
    },
};
use std::any::{Any, TypeId};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq)]
pub enum AssetItemMessage {
    Select(bool),
}

pub fn make_dropdown_list_option(ctx: &mut BuildContext, name: &str) -> Handle<UiNode> {
    DecoratorBuilder::new(BorderBuilder::new(
        WidgetBuilder::new().with_height(26.0).with_child(
            TextBuilder::new(WidgetBuilder::new())
                .with_vertical_text_alignment(VerticalAlignment::Center)
                .with_horizontal_text_alignment(HorizontalAlignment::Center)
                .with_text(name)
                .build(ctx),
        ),
    ))
    .build(ctx)
}

impl AssetItemMessage {
    define_constructor!(AssetItemMessage:Select => fn select(bool), layout: false);
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeletableItemMessage {
    Delete,
}

impl DeletableItemMessage {
    define_constructor!(DeletableItemMessage:Delete => fn delete(), layout: false);
}

/// An item that has content and a button to request deletion.
#[derive(Debug, Clone)]
pub struct DeletableItem<D: Clone> {
    widget: Widget,
    pub delete: Handle<UiNode>,
    pub data: Option<D>,
}

impl<D: Clone> Deref for DeletableItem<D> {
    type Target = Widget;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<D: Clone> DerefMut for DeletableItem<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

impl<D: Clone + 'static> Control for DeletableItem<D> {
    fn query_component(&self, type_id: TypeId) -> Option<&dyn Any> {
        if type_id == TypeId::of::<Self>() {
            Some(self)
        } else {
            None
        }
    }

    fn resolve(&mut self, node_map: &NodeHandleMapping) {
        node_map.resolve(&mut self.delete);
    }

    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);

        if let Some(ButtonMessage::Click) = message.data::<ButtonMessage>() {
            if message.destination() == self.delete {
                ui.send_message(DeletableItemMessage::delete(
                    self.handle(),
                    MessageDirection::FromWidget,
                ));
            }
        }
    }
}

pub struct DeletableItemBuilder<D> {
    widget_builder: WidgetBuilder,
    content: Handle<UiNode>,
    data: Option<D>,
}

impl<D: Clone + 'static> DeletableItemBuilder<D> {
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self {
            widget_builder,
            content: Handle::NONE,
            data: None,
        }
    }

    pub fn with_data(mut self, data: D) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_content(mut self, content: Handle<UiNode>) -> Self {
        self.content = content;
        self
    }

    pub fn build(self, ctx: &mut BuildContext) -> DeletableItem<D> {
        let delete = ButtonBuilder::new(WidgetBuilder::new().on_column(1))
            .with_content(
                ButtonBuilder::new(
                    WidgetBuilder::new()
                        .with_width(16.0)
                        .with_height(16.0)
                        .with_margin(Thickness::uniform(1.0)),
                )
                .with_content(
                    ImageBuilder::new(WidgetBuilder::new())
                        .with_opt_texture(load_image(include_bytes!(
                            "../resources/embed/cross.png"
                        )))
                        .build(ctx),
                )
                .build(ctx),
            )
            .build(ctx);
        DeletableItem {
            widget: self
                .widget_builder
                .with_child(
                    DecoratorBuilder::new(BorderBuilder::new(
                        WidgetBuilder::new().with_child(
                            GridBuilder::new(
                                WidgetBuilder::new()
                                    .with_child(self.content)
                                    .with_child(delete),
                            )
                            .add_column(Column::stretch())
                            .add_column(Column::strict(16.0))
                            .add_row(Row::stretch())
                            .build(ctx),
                        ),
                    ))
                    .build(ctx),
                )
                .build(),
            delete,
            data: self.data,
        }
    }
}
