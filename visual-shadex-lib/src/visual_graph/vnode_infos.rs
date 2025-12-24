use std::rc::Rc;

use egui::{
    Color32, FontId, InnerResponse, Label, Pos2, Rect, RichText, Sense, Separator, Shape, Stroke,
    Vec2, epaint::RectShape, layers::PaintList, vec2,
};
use shadex_backend::nodegraph::{InputInfo, NodeTypeInfo, NodeTypeRef, OutputInfo};

pub mod node_types;
pub use node_types::*;

use crate::{
    InteractionState,
    visual_graph::{VNodeId, VNodeInputRef, VNodeOutputRef},
};

pub trait VisualNodeInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool;
    fn get_shadex_type(&self) -> Rc<NodeTypeInfo>;
    fn get_name(&self) -> &str;
}

#[derive(Clone)]
pub struct VisualInputPort {
    pub pos: Pos2,
    pub input_source: Option<VNodeOutputRef>,
}

#[derive(Clone)]
pub struct VisualOutputPort {
    pub pos: Pos2,
}

pub struct VisualNode {
    pub data: Box<dyn VisualNodeInfo>,
    pub position: Vec2,

    pub formal_type: Option<Rc<NodeTypeInfo>>,

    pub input_ports: Vec<VisualInputPort>,
    pub output_ports: Vec<VisualOutputPort>,
}

fn draw_input_port(
    ui: &mut egui::Ui,
    vref: &VNodeInputRef,
    vport: &mut VisualInputPort,
    port: &InputInfo,
    mode: &mut InteractionState,
    any_drag_stopped: &mut bool,
    mouse_pos: &mut Option<Pos2>,
) {
    ui.horizontal(|ui| {
        let (resp, ptr) = ui.allocate_painter(vec2(20f32, 20f32), Sense::hover() | Sense::drag());

        let hovering = resp.contains_pointer() && mode.dragging.hover_inputs();

        let color = if hovering {
            Color32::WHITE
        } else {
            Color32::RED
        };

        ptr.circle_filled(resp.rect.center(), resp.rect.size().x * 0.5f32, color);
        ui.add(Label::new(&port.name).selectable(false));

        if hovering {
            if let crate::DraggingState::DraggingLineFromOutputPort(_, outref) = &mode.dragging {
                mode.dragging =
                    crate::DraggingState::DraggingLineFromOutputPort(Some(*vref), outref.clone());
            }
        }

        if resp.drag_started() {
            mode.dragging = crate::DraggingState::DraggingLineFromInputPort(vref.clone(), None);
        }

        *mouse_pos = mouse_pos.or(resp.interact_pointer_pos());

        if resp.drag_stopped() {
            *any_drag_stopped = true;
        }

        vport.pos = resp.rect.center();

        resp.on_hover_text(RichText::new(format!("{}", port.value_type)));
    });
}

fn draw_output_ports(
    ui: &mut egui::Ui,
    node_ref: VNodeId,
    vports: &mut [VisualOutputPort],
    ports: &[OutputInfo],
    mode: &mut InteractionState,
    any_drag_stopped: &mut bool,
    mouse_pos: &mut Option<Pos2>,
) {
    for (i, p) in ports.iter().enumerate() {
        ui.horizontal(|ui| {
            let oref = VNodeOutputRef {
                source: node_ref,
                output_ind: i,
            };
            ui.add(Label::new(&p.name).selectable(false));

            let (resp, ptr) =
                ui.allocate_painter(vec2(20f32, 20f32), Sense::hover() | Sense::drag());

            let hovering = resp.contains_pointer() && mode.dragging.hover_outputs();
            let color = if hovering {
                Color32::WHITE
            } else {
                Color32::RED
            };

            if hovering {
                if let crate::DraggingState::DraggingLineFromInputPort(inpref, _) = mode.dragging {
                    mode.dragging =
                        crate::DraggingState::DraggingLineFromInputPort(inpref, Some(oref));
                }
            }

            if resp.drag_started() {
                mode.dragging = crate::DraggingState::DraggingLineFromOutputPort(None, oref);
            }

            *mouse_pos = mouse_pos.or(resp.interact_pointer_pos());

            if resp.drag_stopped() {
                *any_drag_stopped = true;
            }

            ptr.circle_filled(resp.rect.center(), resp.rect.size().x * 0.5f32, color);
            vports[i].pos = resp.rect.center();
            resp.on_hover_text(RichText::new(format!("{}", p.value_type)));
        });
    }
}

impl VisualNode {
    pub fn show_box(
        &mut self,
        ui: &mut egui::Ui,
        selfref: VNodeId,
        mode: &mut InteractionState,
        any_drag_stopped: &mut bool,
        mouse_pos: &mut Option<Pos2>,
        deleted: &mut bool,
    ) -> InnerResponse<bool> {
        let idx = ui.painter().add(Shape::Noop);
        let changed = &mut false;

        self.formal_type = Some(self.data.get_shadex_type());
        if let Some(formal_type) = &self.formal_type {
            self.input_ports.resize(
                formal_type.inputs.len(),
                VisualInputPort {
                    pos: Pos2::ZERO,
                    input_source: None,
                },
            );
            self.output_ports.resize(
                formal_type.outputs.len(),
                VisualOutputPort { pos: Pos2::ZERO },
            );
        }

        let inner_resp = ui.scope_builder(
            egui::UiBuilder::new()
                .sense(Sense::click_and_drag())
                .max_rect(Rect::from_min_size(self.position.to_pos2(), Vec2::INFINITY)),
            |ui| {
                ui.vertical(|ui| {
                    ui.add_space(5f32);
                    ui.horizontal(|ui| {
                        ui.add_space(15f32);
                        // Find out how to make title label centered and/or distinguishable.

                        ui.add(
                            Label::new(
                                RichText::new(self.data.get_name())
                                    .font(FontId::proportional(15f32))
                                    .underline(),
                            )
                            .selectable(false),
                        );
                    });

                    ui.horizontal_top(|ui| {
                        ui.add_space(5f32);

                        ui.vertical(|ui| {
                            // Do input ports
                            // TODO: Ports selectable.

                            if let Some(formal_type) = &self.formal_type {
                                for (i, p) in formal_type.inputs.iter().enumerate() {
                                    draw_input_port(
                                        ui,
                                        &VNodeInputRef {
                                            dest: selfref,
                                            input_ind: i,
                                        },
                                        &mut self.input_ports[i],
                                        p,
                                        mode,
                                        any_drag_stopped,
                                        mouse_pos,
                                    );
                                }
                            }
                        });
                        ui.add_space(5f32);
                        ui.vertical(|ui| {
                            *changed = self.data.show(ui);
                        });

                        ui.add_space(5f32);
                        ui.vertical(|ui| {
                            // Do output ports
                            // TODO: Ports selectable.
                            if let Some(formal_type) = &self.formal_type {
                                draw_output_ports(
                                    ui,
                                    selfref,
                                    &mut self.output_ports,
                                    &formal_type.outputs,
                                    mode,
                                    any_drag_stopped,
                                    mouse_pos,
                                );
                            }
                        });
                    });
                });
            },
        );

        let initial_rect = inner_resp.response.rect;
        let initial_rect = initial_rect.shrink2(vec2(10f32, 0f32));

        ui.painter().set(
            idx,
            RectShape::filled(initial_rect, 10.0f32, Color32::BLACK),
        );

        let response = inner_resp.response;

        if response.dragged() {
            let delt = response.drag_delta();
            self.position += delt;
        }

        response.context_menu(|ui| {
            if ui.button("Delete").clicked() {
                *deleted = true;
            }
        });

        //helpers::draw_text(painter, text, pos, font_size, halign, valign).galley.rect.width();

        InnerResponse {
            inner: *changed,
            response: response,
        }
    }
}
