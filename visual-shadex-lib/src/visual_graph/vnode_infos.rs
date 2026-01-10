use std::{collections::HashMap, fmt::Display, rc::Rc};

use egui::{
    Color32, FontId, InnerResponse, Label, Pos2, Rect, RichText, Sense, Separator, Shape, Stroke,
    Style, Vec2, epaint::RectShape, layers::PaintList, text::LayoutJob, vec2,
};
use serde::{Deserialize, Serialize, de::Visitor, ser::SerializeStruct};
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

pub const UNDERSPECIFIED_COLOR: Color32 = Color32::BLUE;
pub const ERROR_COLOR: Color32 = Color32::YELLOW;
pub const HOVER_COLOR: Color32 = Color32::WHITE;
pub const OVERSPECIFIED_COLOR: Color32 = Color32::ORANGE;
pub const OK_COLOR: Color32 = Color32::LIGHT_RED;
pub const FREE_VARIABLE_COLOR: Color32 = Color32::DARK_GREEN;

pub const PORT_HB_WIDTH: f32 = 20f32;
pub const PORT_VIS_RADIUS: f32 = 5f32;

pub mod node_types;
pub use node_types::*;

use crate::{
    InteractionState,
    formal_graph_annotations::FormalGraph,
    visual_graph::{VNodeId, VNodeInputRef, VNodeOutputRef},
};

#[typetag::serde(tag = "type")]
pub trait VisualNodeInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool;
    fn get_shadex_type(&self) -> FallibleNodeTypeRc;
    fn get_name(&self) -> &str;
}

#[derive(Clone, Deserialize, Serialize)]
pub struct VisualInputPort {
    pub pos: Pos2,
    pub input_source: Option<VNodeOutputRef>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct VisualOutputPort {
    pub pos: Pos2,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct VisualNode {
    pub data: Box<dyn VisualNodeInfo>,
    pub position: Vec2,

    pub input_ports: Vec<VisualInputPort>,
    pub output_ports: Vec<VisualOutputPort>,
}

/*
impl Serialize for Box<dyn VisualNodeInfo> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("VNodeInfo", 2)?;
        s.serialize_field("Name", self.get_name())?;
        s.serialize_field("Content", &self.to_serialization_data())?;

        s.end()
    }
}
    */

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
            Err(_) => ERROR_COLOR,
        }
    }
}

impl<T: NotesWithDotColor> NotesWithDotColor for (&MaybeValueType, Option<&T>) {
    fn pick_dot_color(&self) -> Color32 {
        self.1
            .map(|f| f.pick_dot_color())
            .unwrap_or_else(|| match self.0 {
                Ok(_) => OK_COLOR,
                Err(_) => ERROR_COLOR,
            })
    }
}

impl NotesWithDotColor for OutputTypeNotes {
    fn pick_dot_color(&self) -> Color32 {
        if !self.inputs_parameterized_by.is_empty() {
            UNDERSPECIFIED_COLOR
        } else {
            OK_COLOR
        }
    }
}

impl NotesWithDotColor for InputTypeNotes {
    fn pick_dot_color(&self) -> Color32 {
        match &self.type_source {
            shadex_backend::typechecking::InputValueTypeSource::FreeVariable(_) => {
                FREE_VARIABLE_COLOR
            }
            shadex_backend::typechecking::InputValueTypeSource::FromOutput(outp_prom) => {
                if !outp_prom.underspecified_args.is_empty() {
                    UNDERSPECIFIED_COLOR
                } else if !outp_prom.added_constant_wrt.is_empty() {
                    OVERSPECIFIED_COLOR
                } else {
                    OK_COLOR
                }
            }
        }
    }
}

impl ColorfulTypeNotes for OutputTypeNotes {
    fn colors(&self) -> (&ValueType, HashMap<String, Color32>) {
        let mut colormap = HashMap::new();
        for name in self.inputs_parameterized_by.keys() {
            colormap.insert(name.clone(), UNDERSPECIFIED_COLOR);
        }
        (&self.formal_type, colormap)
    }
}

impl ColorfulTypeNotes for (&ValueType, &OutputPromotion) {
    fn colors(&self) -> (&ValueType, HashMap<String, Color32>) {
        let mut colormap = HashMap::new();
        for name in &self.1.underspecified_args {
            colormap.insert(name.clone(), UNDERSPECIFIED_COLOR);
        }
        for name in self.1.added_constant_wrt.keys() {
            colormap.insert(name.clone(), OVERSPECIFIED_COLOR);
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
    name: &String,
    ui: &mut egui::Ui,
    spec_type: &MaybeValueType,
    typecheck_type: Option<&Result<T, TypeError>>,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    match spec_type {
        Ok(t) => {
            RichText::new(format!("{} @ {}\n", name, t))
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
    changed: &mut bool,

    formal_graph: Option<&FormalGraph>,
) {
    ui.horizontal(|ui| {
        let (resp, ptr) = ui.allocate_painter(
            vec2(PORT_HB_WIDTH, PORT_HB_WIDTH),
            Sense::hover() | Sense::drag(),
        );

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
            HOVER_COLOR
        } else {
            (&port.value_type, detailed_type).pick_dot_color()
        };

        ptr.circle_filled(resp.rect.center(), PORT_VIS_RADIUS, color);
        ui.add(Label::new(&port.name).selectable(false));

        if hovering {
            if let crate::DraggingState::DraggingLineFromOutputPort(_, outref) = &mode.dragging {
                mode.dragging =
                    crate::DraggingState::DraggingLineFromOutputPort(Some(*vref), outref.clone());
            }
        }

        if resp.drag_started() {
            if let Some(src) = &vport.input_source {
                mode.dragging = crate::DraggingState::DraggingLineFromOutputPort(None, *src);
                vport.input_source = None;
                *changed = true;
            } else {
                mode.dragging = crate::DraggingState::DraggingLineFromInputPort(vref.clone(), None);
            }
        }

        *mouse_pos = mouse_pos.or(resp.interact_pointer_pos());

        if resp.drag_stopped() {
            *any_drag_stopped = true;
        }

        vport.pos = resp.rect.center();

        resp.on_hover_ui(|ui| {
            let label = Label::new(richtext_type_desc(
                &port.name,
                ui,
                &port.value_type,
                detailed_type,
            ));
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
            match &p.name {
                Some(txt) => {
                    ui.add(Label::new(txt).selectable(false));
                }
                None => {}
            }

            let (resp, ptr) = ui.allocate_painter(
                vec2(PORT_HB_WIDTH, PORT_HB_WIDTH),
                Sense::hover() | Sense::drag(),
            );

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
                HOVER_COLOR
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

            ptr.circle_filled(resp.rect.center(), PORT_VIS_RADIUS, color);
            vports[i].pos = resp.rect.center();
            let strin = match &p.name {
                Some(txt) => txt,
                None => &format!("{}", i),
            };
            resp.on_hover_ui(|ui| {
                let label = Label::new(richtext_type_desc(strin, ui, &p.value_type, detailed_type));
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

        if let Ok(formal_type) = &self.data.get_shadex_type() {
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

                            if let Ok(formal_type) = &self.data.get_shadex_type() {
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
                                        changed,
                                        formal_graph,
                                    );
                                }
                            }
                        });
                        ui.add_space(5f32);
                        ui.vertical(|ui| {
                            *changed = self.data.show(ui) | *changed;
                        });

                        ui.add_space(5f32);
                        ui.vertical(|ui| {
                            // Do output ports
                            // TODO: Ports selectable.
                            if let Ok(formal_type) = &self.data.get_shadex_type() {
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
