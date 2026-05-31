use std::{path::PathBuf, time};

use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::Wrap,
};
use tmj_core::{
    pathes,
    script::{ContextRef, TypeName},
};

use crate::{
    LAYOUT,
    art::theme::THEME,
    pages::{
        dialogue::DialogueScene,
        behaviour::{
            Behaviour,
            animation::{Animation, img_trans::AniImgTrans, rect_trans::{AniRectTrans, RectTransCurve}},
            logical_area,
            ve_z_index::{Z_BG, Z_BG_EDGE},
            visual_element::{VisualElement, VisualElementCustomDrawer, VisualElementKind},
        },
        script_def::env::BG,
    },
};

#[derive(TypeName)]
pub struct BackgroundBehaviour {
    img_trans_ani: AniImgTrans,
    edge_top_ani: AniRectTrans,
    edge_bottom_ani: AniRectTrans,
}

impl Default for BackgroundBehaviour {
    fn default() -> Self {
        Self {
            img_trans_ani: AniImgTrans::default(),
            edge_top_ani: AniRectTrans::default(),
            edge_bottom_ani: AniRectTrans::default(),
        }
    }
}

impl BackgroundBehaviour {
    fn trans_string_path(s: String) -> Option<PathBuf> {
        if s.is_empty(){
            None
        } else {
            Some(pathes::path(s))
        }
    }
    pub fn export_trans_to(&mut self, new_img_path: String, duration: f64) {
        self.img_trans_ani.old_image = self.img_trans_ani.new_image.clone();
        self.img_trans_ani.new_image = Self::trans_string_path(new_img_path.clone());
        self.img_trans_ani.anim_time = time::Duration::from_secs_f64(duration);
        self.img_trans_ani.run_time = time::Duration::ZERO;
    }
    pub fn export_set(&mut self, new_img_path: String) {
        self.img_trans_ani.old_image = self.img_trans_ani.new_image.clone();
        self.img_trans_ani.new_image = Self::trans_string_path(new_img_path.clone());
        self.img_trans_ani.anim_time = time::Duration::ZERO;
        self.img_trans_ani.run_time = time::Duration::ZERO;
    }

    pub fn export_show_edge(&mut self, duration_secs: f64) {
        let area = logical_area();
        let h = LAYOUT.vertical_dark_edge;
        let top_start = Rect::new(area.x, area.y, area.width, 0);
        let top_target = Rect::new(area.x, area.y, area.width, h);
        let bottom_start = Rect::new(area.x, area.bottom(), area.width, 0);
        let bottom_target = Rect::new(area.x, area.bottom() - h, area.width, h);
        self.edge_top_ani.begin(top_start, top_target, duration_secs, RectTransCurve::SineInOut);
        self.edge_bottom_ani.begin(bottom_start, bottom_target, duration_secs, RectTransCurve::SineInOut);
    }

    pub fn export_hide_edge(&mut self, duration_secs: f64) {
        let area = logical_area();
        let h = LAYOUT.vertical_dark_edge;
        let top_start = Rect::new(area.x, area.y, area.width, h);
        let top_target = Rect::new(area.x, area.y, area.width, 0);
        let bottom_start = Rect::new(area.x, area.bottom() - h, area.width, h);
        let bottom_target = Rect::new(area.x, area.bottom(), area.width, 0);
        self.edge_top_ani.begin(top_start, top_target, duration_secs, RectTransCurve::SineInOut);
        self.edge_bottom_ani.begin(bottom_start, bottom_target, duration_secs, RectTransCurve::SineInOut);
    }
}

impl Behaviour for BackgroundBehaviour {
    fn is_animating(&self) -> bool {
        self.img_trans_ani.is_animing()
            || self.edge_top_ani.is_animing()
            || self.edge_bottom_ani.is_animing()
    }
    fn sync_from_ctx(&mut self, ctx: tmj_core::script::ContextRef) -> anyhow::Result<()> {
        let mut vars = self.get_bind_vars(&ctx);
        let is_edge = vars
            .pop()
            .transpose()?
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("{}.{} missing or not bool", BG, Self::BG_IS_EDGE))?;
        let img_path = vars
            .pop()
            .transpose()?
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow::anyhow!("{}.{} missing or not string", BG, Self::BG_IMAGE))?;
        if is_edge {
            self.export_show_edge(0.0);
        } else {
            self.export_hide_edge(0.0);
        }
        self.img_trans_ani.reset();
        self.img_trans_ani.new_image = Self::trans_string_path(img_path);
        Ok(())
    }

    fn binding_vars(&self) -> &'static [&'static str] {
        &[BG, Self::BG_IMAGE, Self::BG_IS_EDGE]
    }

    fn build_elements(
        &self,
        ctx: &tmj_core::script::ContextRef,
    ) -> anyhow::Result<Vec<VisualElement>> {
        let mut args = self.get_bind_vars(ctx);
        let is_edge_show = args
            .pop()
            .transpose()?
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("bg edge bind failed"))?;
        let area = logical_area();

        let [up, _, down] = area.layout(&Layout::vertical([
            Constraint::Length(LAYOUT.vertical_dark_edge),
            Constraint::Fill(1),
            Constraint::Length(LAYOUT.vertical_dark_edge),
        ]));

        let (top_rect, bottom_rect) = if is_edge_show {
            (up, down)
        } else {
            (
                Rect::new(area.x, area.y, area.width, 0),
                Rect::new(area.x, area.bottom(), area.width, 0),
            )
        };

        Ok(vec![
            VisualElement {
                name: Self::VE_BG.to_string(),
                z_index: Z_BG,
                rect: area,
                text_wrap: Some(Wrap { trim: false }),
                kind: VisualElementKind::Custom {
                    drawer: VisualElementCustomDrawer::from(|_, _, _| Ok(())),
                },
                style: THEME.dialouge.background,
                ..Default::default()
            },
            VisualElement {
                name: Self::VE_EDGE_TOP.to_string(),
                visible: true,
                z_index: Z_BG_EDGE,
                rect: top_rect,
                clear_before_draw: true,
                text_wrap: Some(Wrap { trim: false }),
                kind: VisualElementKind::Fill,
                style: THEME.dialouge.black_edge,
                ..Default::default()
            },
            VisualElement {
                name: Self::VE_EDGE_BOTTOM.to_string(),
                visible: true,
                z_index: Z_BG_EDGE,
                rect: bottom_rect,
                clear_before_draw: true,
                text_wrap: Some(Wrap { trim: false }),
                kind: VisualElementKind::Fill,
                style: THEME.dialouge.black_edge,
                ..Default::default()
            },
        ])
    }

    fn update_elements(
        &self,
        _screen: &DialogueScene,
        _ctx: &tmj_core::script::ContextRef,
        elements: &mut Vec<VisualElement>,
    ) -> anyhow::Result<()> {
        if let Some(bg) = elements.iter_mut().find(|x| x.name == Self::VE_BG) {
            self.img_trans_ani.apply_to_ve(bg)?;
        }
        if let Some(top) = elements.iter_mut().find(|x| x.name == Self::VE_EDGE_TOP) {
            self.edge_top_ani.apply_to_ve(top)?;
        }
        if let Some(bottom) = elements.iter_mut().find(|x| x.name == Self::VE_EDGE_BOTTOM) {
            self.edge_bottom_ani.apply_to_ve(bottom)?;
        }

        Ok(())
    }

    fn tick_update(&mut self, _ctx: ContextRef, delta_time: std::time::Duration) {
        self.img_trans_ani.update(delta_time);
        self.edge_top_ani.update(delta_time);
        self.edge_bottom_ani.update(delta_time);
    }

    fn on_force_over_animation(&mut self) -> anyhow::Result<()> {
        self.img_trans_ani.force_over();
        self.edge_top_ani.force_over();
        self.edge_bottom_ani.force_over();
        Ok(())
    }

    fn on_end_dialouge(&mut self) -> anyhow::Result<()> {
        self.img_trans_ani.reset();
        // self.edge_top_ani.reset();
        // self.edge_bottom_ani.reset();
        Ok(())
    }

    fn on_end_session(&mut self, _ctx: tmj_core::script::ContextRef) -> anyhow::Result<()> {
        self.img_trans_ani.reset();
        // self.edge_top_ani.reset();
        // self.edge_bottom_ani.reset();
        Ok(())
    }
}

impl BackgroundBehaviour {
    pub const BG_IMAGE: &'static str =
        constcat::concat!(BG, ".", crate::pages::script_def::var_bg::M_IMAGE);
    pub const BG_IS_EDGE: &'static str =
        constcat::concat!(BG, ".", crate::pages::script_def::var_bg::M_IS_EDGE);
    pub const VE_BG: &'static str = Self::BG_IMAGE;
    pub const VE_EDGE_TOP: &'static str = "bg.edge.top";
    pub const VE_EDGE_BOTTOM: &'static str = "bg.edge.bottom";
}
