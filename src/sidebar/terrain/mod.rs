use crate::{
    gui::{BuildContext, Ui, UiMessage, UiNode},
    scene::{
        commands::terrain::{AddTerrainLayerCommand, DeleteTerrainLayerCommand},
        commands::SceneCommand,
    },
    send_sync_message,
    sidebar::{terrain::brush::BrushSection, terrain::layer::LayerSection, ROW_HEIGHT},
    Message,
};
use rg3d::{
    core::{algebra::Vector2, pool::Handle, scope_profile},
    engine::resource_manager::ResourceManager,
    gui::{
        border::BorderBuilder,
        button::ButtonBuilder,
        decorator::DecoratorBuilder,
        grid::{Column, GridBuilder, Row},
        list_view::ListViewBuilder,
        message::{ButtonMessage, ListViewMessage, MessageDirection, UiMessageData, WidgetMessage},
        stack_panel::StackPanelBuilder,
        text::TextBuilder,
        widget::WidgetBuilder,
        Orientation,
    },
    scene::{graph::Graph, node::Node, terrain::BrushMode},
};
use std::sync::mpsc::Sender;

mod brush;
mod layer;

pub struct TerrainSection {
    pub section: Handle<UiNode>,
    pub brush_section: BrushSection,
    layers: Handle<UiNode>,
    add_layer: Handle<UiNode>,
    remove_layer: Handle<UiNode>,
    current_layer: Option<usize>,
    layer_section: LayerSection,
}

impl TerrainSection {
    pub fn new(ctx: &mut BuildContext) -> Self {
        let brush_section = BrushSection::new(ctx);
        let layer_section = LayerSection::new(ctx);

        let layers;
        let add_layer;
        let remove_layer;
        let section = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_child(
                    GridBuilder::new(
                        WidgetBuilder::new()
                            .with_child(
                                StackPanelBuilder::new(
                                    WidgetBuilder::new()
                                        .with_child({
                                            add_layer = ButtonBuilder::new(WidgetBuilder::new())
                                                .with_text("Add Layer")
                                                .build(ctx);
                                            add_layer
                                        })
                                        .with_child({
                                            remove_layer = ButtonBuilder::new(WidgetBuilder::new())
                                                .with_text("Remove Layer")
                                                .build(ctx);
                                            remove_layer
                                        }),
                                )
                                .with_orientation(Orientation::Horizontal)
                                .build(ctx),
                            )
                            .with_child({
                                layers = ListViewBuilder::new(
                                    WidgetBuilder::new()
                                        .with_min_size(Vector2::new(0.0, ROW_HEIGHT * 3.0))
                                        .on_row(1)
                                        .on_column(0),
                                )
                                .build(ctx);
                                layers
                            }),
                    )
                    .add_row(Row::strict(ROW_HEIGHT))
                    .add_row(Row::stretch())
                    .add_column(Column::stretch())
                    .build(ctx),
                )
                .with_child(brush_section.section)
                .with_child(layer_section.section),
        )
        .with_orientation(Orientation::Vertical)
        .build(ctx);

        Self {
            section,
            layers,
            add_layer,
            brush_section,
            remove_layer,
            layer_section,
            current_layer: None,
        }
    }

    pub fn sync_to_model(&mut self, node: &Node, ui: &mut Ui) {
        send_sync_message(
            ui,
            WidgetMessage::visibility(self.section, MessageDirection::ToWidget, node.is_terrain()),
        );

        if let Node::Terrain(terrain) = node {
            let layer_items = terrain
                .chunks_ref()
                .first()
                .unwrap()
                .layers()
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    DecoratorBuilder::new(BorderBuilder::new(
                        WidgetBuilder::new().with_child(
                            TextBuilder::new(WidgetBuilder::new())
                                .with_text(format!("Layer {}", i))
                                .build(&mut ui.build_ctx()),
                        ),
                    ))
                    .build(&mut ui.build_ctx())
                })
                .collect::<Vec<_>>();

            ui.send_message(ListViewMessage::items(
                self.layers,
                MessageDirection::ToWidget,
                layer_items,
            ));

            self.layer_section.sync_to_model(
                self.current_layer
                    .and_then(|i| terrain.chunks_ref().first().unwrap().layers().get(i)),
                ui,
            );
        }

        self.brush_section.sync_to_model(ui);
    }

    pub fn handle_message(
        &mut self,
        message: &UiMessage,
        ui: &mut Ui,
        resource_manager: ResourceManager,
        node: &Node,
        graph: &Graph,
        handle: Handle<Node>,
        sender: &Sender<Message>,
    ) {
        scope_profile!();

        if let Some(index) = self.current_layer {
            self.layer_section
                .handle_message(message, ui, index, resource_manager, handle, sender);
        }

        self.brush_section.handle_message(message);

        let mut brush = self.brush_section.brush.lock().unwrap();
        if let BrushMode::DrawOnMask { layer, .. } = &mut brush.mode {
            *layer = self.current_layer.unwrap_or(0);
        }
        drop(brush);

        if let Node::Terrain(_) = node {
            match *message.data() {
                UiMessageData::Button(ButtonMessage::Click) => {
                    if message.destination() == self.add_layer {
                        sender
                            .send(Message::DoSceneCommand(SceneCommand::AddTerrainLayer(
                                AddTerrainLayerCommand::new(handle, graph),
                            )))
                            .unwrap();
                    } else if message.destination() == self.remove_layer {
                        if let Some(index) = self.current_layer {
                            sender
                                .send(Message::DoSceneCommand(SceneCommand::DeleteTerrainLayer(
                                    DeleteTerrainLayerCommand::new(handle, index),
                                )))
                                .unwrap()
                        }
                    }
                }
                UiMessageData::ListView(ListViewMessage::SelectionChanged(layer_index)) => {
                    if message.destination() == self.layers && self.current_layer != layer_index {
                        self.current_layer = layer_index;
                        self.sync_to_model(node, ui);
                    }
                }
                _ => {}
            }
        }
    }
}
