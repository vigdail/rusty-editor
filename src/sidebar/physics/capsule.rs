use crate::{
    gui::{BuildContext, Ui, UiMessage, UiNode},
    physics::Collider,
    scene::commands::{
        physics::{SetCapsuleBeginCommand, SetCapsuleEndCommand, SetCapsuleRadiusCommand},
        SceneCommand,
    },
    send_sync_message,
    sidebar::{
        make_f32_input_field, make_text_mark, make_vec3_input_field, COLUMN_WIDTH, ROW_HEIGHT,
    },
    Message,
};
use rg3d::{
    core::pool::Handle,
    gui::{
        grid::{Column, GridBuilder, Row},
        message::{MessageDirection, NumericUpDownMessage, UiMessageData, Vec3EditorMessage},
        widget::WidgetBuilder,
    },
    scene::physics::desc::CapsuleDesc,
};
use std::sync::mpsc::Sender;

pub struct CapsuleSection {
    pub section: Handle<UiNode>,
    begin: Handle<UiNode>,
    end: Handle<UiNode>,
    radius: Handle<UiNode>,
    sender: Sender<Message>,
}

impl CapsuleSection {
    pub fn new(ctx: &mut BuildContext, sender: Sender<Message>) -> Self {
        let begin;
        let end;
        let radius;
        let section = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(make_text_mark(ctx, "Begin", 0))
                .with_child({
                    begin = make_vec3_input_field(ctx, 0);
                    begin
                })
                .with_child(make_text_mark(ctx, "End", 1))
                .with_child({
                    end = make_vec3_input_field(ctx, 1);
                    end
                })
                .with_child(make_text_mark(ctx, "Radius", 2))
                .with_child({
                    radius = make_f32_input_field(ctx, 2, 0.0, std::f32::MAX, 0.1);
                    radius
                }),
        )
        .add_column(Column::strict(COLUMN_WIDTH))
        .add_column(Column::stretch())
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .build(ctx);

        Self {
            section,
            sender,
            begin,
            end,
            radius,
        }
    }

    pub fn sync_to_model(&mut self, capsule: &CapsuleDesc, ui: &mut Ui) {
        send_sync_message(
            ui,
            Vec3EditorMessage::value(self.begin, MessageDirection::ToWidget, capsule.begin),
        );

        send_sync_message(
            ui,
            Vec3EditorMessage::value(self.end, MessageDirection::ToWidget, capsule.end),
        );

        send_sync_message(
            ui,
            NumericUpDownMessage::value(self.radius, MessageDirection::ToWidget, capsule.radius),
        );
    }

    pub fn handle_message(
        &mut self,
        message: &UiMessage,
        capsule: &CapsuleDesc,
        handle: Handle<Collider>,
    ) {
        match message.data() {
            &UiMessageData::NumericUpDown(NumericUpDownMessage::Value(value)) => {
                if message.direction() == MessageDirection::FromWidget
                    && message.destination() == self.radius
                    && capsule.radius.ne(&value)
                {
                    self.sender
                        .send(Message::DoSceneCommand(SceneCommand::SetCapsuleRadius(
                            SetCapsuleRadiusCommand::new(handle, value),
                        )))
                        .unwrap();
                }
            }
            UiMessageData::Vec3Editor(Vec3EditorMessage::Value(value)) => {
                if message.destination() == self.begin && capsule.begin.ne(value) {
                    self.sender
                        .send(Message::DoSceneCommand(SceneCommand::SetCapsuleBegin(
                            SetCapsuleBeginCommand::new(handle, *value),
                        )))
                        .unwrap();
                } else if message.destination() == self.end && capsule.end.ne(value) {
                    self.sender
                        .send(Message::DoSceneCommand(SceneCommand::SetCapsuleEnd(
                            SetCapsuleEndCommand::new(handle, *value),
                        )))
                        .unwrap();
                }
            }
            _ => {}
        }
    }
}
