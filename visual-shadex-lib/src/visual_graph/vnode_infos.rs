use std::{collections::HashMap, fmt::Display, rc::Rc};

use egui::{
    Color32, FontId, InnerResponse, Label, Pos2, Rect, RichText, Sense, Separator, Shape, Stroke,
    Style, Vec2, epaint::RectShape, layers::PaintList, text::LayoutJob, vec2,
};
use shadex_backend::{
    nodegraph::{
        FallibleNodeTypeRc, InputInfo, NodeTypeInfo, NodeTypeRef, OutputInfo, PortTypeAnnotation,
        ValueRef,
    },
    typechecking::{
        InputTypeNotes, NodeInputReference, OutputPromotion, OutputTypeNotes,
        typetypes::{MaybeValueType, TypeError, ValueType},
    },
};

pub mod node_types;
pub use node_types::*;

use crate::{
    InteractionState,
    formal_graph_annotations::FormalGraph,
    visual_graph::{VNodeId, VNodeInputRef, VNodeOutputRef},
};

pub trait VisualNodeInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool;
    fn get_shadex_type(&self) -> FallibleNodeTypeRc;
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

    pub formal_type: Option<FallibleNodeTypeRc>,

    pub input_ports: Vec<VisualInputPort>,
    pub output_ports: Vec<VisualOutputPort>,
}

trait ColorfulTypeNotes {
    fn colors(&self) -> (&ValueType, HashMap<String, Color32>);
}

impl<T: ColorfulTypeNotes> RichTextPrintable for T {
    fn print_to_layout_job(&self, ui: &mut egui::Ui, layout: &mut LayoutJob) {
        colorful(self, ui, layout);
    }
}

trait RichTextPrintable {
    fn print_to_layout_job(&self, ui: &mut egui::Ui, layout: &mut LayoutJob);
}

trait NotesWithDotColor {
    fn pick_dot_color(&self) -> Color32;
}

impl<T: NotesWithDotColor> NotesWithDotColor for Result<T, TypeError> {
    fn pick_dot_color(&self) -> Color32 {
        match self {
            Ok(a) => a.pick_dot_color(),
            Err(_) => Color32::YELLOW,
        }
    }
}

impl<T: NotesWithDotColor> NotesWithDotColor for (&MaybeValueType, Option<&T>) {
    fn pick_dot_color(&self) -> Color32 {
        self.1
            .map(|f| f.pick_dot_color())
            .unwrap_or_else(|| match self.0 {
                Ok(_) => Color32::RED,
                Err(_) => Color32::YELLOW,
            })
    }
}

impl NotesWithDotColor for OutputTypeNotes {
    fn pick_dot_color(&self) -> Color32 {
        if !self.inputs_parameterized_by.is_empty() {
            Color32::LIGHT_BLUE
        } else {
            Color32::RED
        }
    }
}

impl NotesWithDotColor for InputTypeNotes {
    fn pick_dot_color(&self) -> Color32 {
        match &self.type_source {
            shadex_backend::typechecking::InputValueTypeSource::FreeVariable(_) => {
                Color32::DARK_GREEN
            }
            shadex_backend::typechecking::InputValueTypeSource::FromOutput(outp_prom) => {
                if !outp_prom.underspecified_args.is_empty() {
                    Color32::LIGHT_BLUE
                } else if !outp_prom.added_constant_wrt.is_empty() {
                    Color32::ORANGE
                } else {
                    Color32::RED
                }
            }
        }
    }
}

impl ColorfulTypeNotes for OutputTypeNotes {
    fn colors(&self) -> (&ValueType, HashMap<String, Color32>) {
        let mut colormap = HashMap::new();
        for name in self.inputs_parameterized_by.keys() {
            colormap.insert(name.clone(), Color32::LIGHT_BLUE);
        }
        (&self.formal_type, colormap)
    }
}

impl ColorfulTypeNotes for (&ValueType, &OutputPromotion) {
    fn colors(&self) -> (&ValueType, HashMap<String, Color32>) {
        let mut colormap = HashMap::new();
        for name in &self.1.underspecified_args {
            colormap.insert(name.clone(), Color32::LIGHT_BLUE);
        }
        for name in self.1.added_constant_wrt.keys() {
            colormap.insert(name.clone(), Color32::ORANGE);
        }
        (&self.0, colormap)
    }
}

impl RichTextPrintable for InputTypeNotes {
    fn print_to_layout_job(&self, ui: &mut egui::Ui, layout: &mut LayoutJob) {
        match &self.type_source {
            shadex_backend::typechecking::InputValueTypeSource::FreeVariable(_) => {
                RichText::new("Free variable!")
                    .color(ui.style().visuals.text_color())
                    .append_to(
                        layout,
                        ui.style(),
                        egui::FontSelection::Default,
                        egui::Align::Min,
                    )
            }
            shadex_backend::typechecking::InputValueTypeSource::FromOutput(output_promotion) => {
                (&self.formal_type, output_promotion).print_to_layout_job(ui, layout);
            }
        }
    }
}

fn colorful<T: ColorfulTypeNotes>(notes: &T, ui: &mut egui::Ui, layout: &mut LayoutJob) {
    let (formal_type, special_colors) = notes.colors();
    if formal_type.inputs.len() == 0 {
        RichText::new(format!("{}", formal_type))
            .color(ui.style().visuals.text_color())
            .append_to(
                layout,
                ui.style(),
                egui::FontSelection::Default,
                egui::Align::Min,
            );
        return;
    }

    let mut args: Vec<_> = formal_type.inputs.iter().collect();
    args.sort_by(|a, b| a.0.cmp(b.0));

    for (i, (n, t)) in args.iter().enumerate() {
        if i > 0 {
            RichText::new(", ")
                .color(ui.style().visuals.text_color())
                .append_to(
                    layout,
                    ui.style(),
                    egui::FontSelection::Default,
                    egui::Align::Min,
                );
        }
        RichText::new(format!("{}: {}", **n, t))
            .color(
                special_colors
                    .get(*n)
                    .map(|a| *a)
                    .unwrap_or(ui.style().visuals.text_color()),
            )
            .append_to(
                layout,
                ui.style(),
                egui::FontSelection::Default,
                egui::Align::Min,
            );
    }

    RichText::new(format!(" -> {}", formal_type.output))
        .color(ui.style().visuals.text_color())
        .append_to(
            layout,
            ui.style(),
            egui::FontSelection::Default,
            egui::Align::Min,
        );
}

fn richtext_type_desc<T: RichTextPrintable>(
    ui: &mut egui::Ui,
    spec_type: &MaybeValueType,
    typecheck_type: Option<&Result<T, TypeError>>,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    match spec_type {
        Ok(t) => {
            RichText::new(format!("Spec: {}\n", t))
                .color(ui.style().visuals.text_color())
                .append_to(
                    &mut job,
                    ui.style(),
                    egui::FontSelection::Default,
                    egui::Align::Min,
                );
        }
        Err(e) => {
            RichText::new(format!("Spec error! {}", e))
                .color(ui.style().visuals.text_color())
                .append_to(
                    &mut job,
                    ui.style(),
                    egui::FontSelection::Default,
                    egui::Align::Min,
                );
            return job;
        }
    }

    match typecheck_type {
        Some(Ok(tr)) => tr.print_to_layout_job(ui, &mut job),
        Some(Err(e)) => RichText::new(format!("Typecheck error! {}", e))
            .color(ui.style().visuals.text_color())
            .append_to(
                &mut job,
                ui.style(),
                egui::FontSelection::Default,
                egui::Align::Min,
            ),
        None => RichText::new("No typecheck")
            .color(ui.style().visuals.text_color())
            .append_to(
                &mut job,
                ui.style(),
                egui::FontSelection::Default,
                egui::Align::Min,
            ),
    }

    job
}

fn draw_input_port(
    ui: &mut egui::Ui,
    vref: &VNodeInputRef,
    vport: &mut VisualInputPort,
    port: &InputInfo<MaybeValueType>,
    mode: &mut InteractionState,
    any_drag_stopped: &mut bool,
    mouse_pos: &mut Option<Pos2>,

    formal_graph: Option<&FormalGraph>,
) {
    ui.horizontal(|ui| {
        let (resp, ptr) = ui.allocate_painter(vec2(20f32, 20f32), Sense::hover() | Sense::drag());

        let hovering = resp.contains_pointer() && mode.dragging.hover_inputs();

        let detailed_type = formal_graph.and_then(|f| {
            let fnode_id = f.vnode_to_fnode.get(&vref.dest)?;
            let notes = f.typecheck.input_type_notes.get(&NodeInputReference {
                source_node: *fnode_id,
                input_ind: vref.input_ind,
            })?;
            Some(notes)
        });

        let color = if hovering {
            Color32::WHITE
        } else {
            (&port.value_type, detailed_type).pick_dot_color()
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

        resp.on_hover_ui(|ui| {
            let label = Label::new(richtext_type_desc(ui, &port.value_type, detailed_type));
            ui.add(label);
        });
    });
}

fn draw_output_ports(
    ui: &mut egui::Ui,
    node_ref: VNodeId,
    vports: &mut [VisualOutputPort],
    ports: &[OutputInfo<MaybeValueType>],
    mode: &mut InteractionState,
    any_drag_stopped: &mut bool,
    mouse_pos: &mut Option<Pos2>,

    formal_graph: Option<&FormalGraph>,
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

            let detailed_type = formal_graph.and_then(|f| {
                let fnode_id = f.vnode_to_fnode.get(&node_ref)?;
                let notes = f.typecheck.output_type_notes.get(&ValueRef {
                    node: *fnode_id,
                    output_index: i,
                })?;
                Some(notes)
            });

            let hovering = resp.contains_pointer() && mode.dragging.hover_outputs();
            let color = if hovering {
                Color32::WHITE
            } else {
                (&p.value_type, detailed_type).pick_dot_color()
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
            resp.on_hover_ui(|ui| {
                let label = Label::new(richtext_type_desc(ui, &p.value_type, detailed_type));
                ui.add(label);
            });
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

        formal_graph: Option<&FormalGraph>,
    ) -> InnerResponse<bool> {
        let idx = ui.painter().add(Shape::Noop);
        let changed = &mut false;

        self.formal_type = Some(self.data.get_shadex_type());
        if let Some(Ok(formal_type)) = &self.formal_type {
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

                            if let Some(Ok(formal_type)) = &self.formal_type {
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
                                        formal_graph,
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
                            if let Some(Ok(formal_type)) = &self.formal_type {
                                draw_output_ports(
                                    ui,
                                    selfref,
                                    &mut self.output_ports,
                                    &formal_type.outputs,
                                    mode,
                                    any_drag_stopped,
                                    mouse_pos,
                                    formal_graph,
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
