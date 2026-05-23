pub mod alpha_shift;
pub mod bytes_stream;
pub mod error;
pub mod img_trans;
pub mod offset_shift;
pub mod rect_trans;
pub mod typewriter;

pub use offset_shift::OffsetShift;
pub use rect_trans::{
    AniRectTrans, RectTransCurve, ResizeAnchor, SlideDirection, resize_target, slide_target,
};

use std::time::Duration;

/// 确定性动画的归一化进度 [0, 1]（`anim_time` 为 0 时视为已完成）。
pub(crate) fn anim_normalized_time(run_time: Duration, anim_time: Duration) -> f32 {
    if anim_time.is_zero() {
        1.0
    } else {
        (run_time.as_secs_f32() / anim_time.as_secs_f32()).clamp(0.0, 1.0)
    }
}
use tmj_core::script::TypeName;

use crate::pages::behaviour::visual_element::VisualElement;
use std::{any::Any, collections::HashMap};

pub trait Animation {
    fn update(&mut self, tick_delta: std::time::Duration);
    fn apply_to_ve(&self, ve: &mut VisualElement) -> anyhow::Result<()>;
    fn force_over(&mut self);
    fn reset(&mut self);
    fn is_animing(&self) -> bool;

    /// determinate 确定性动画,即有明显意义和进度,显式结束
    /// indeterminate 没有进度和生命周期的动画,对于装饰性动画和效果,这里返回true, 不能重置和强制结束
    /// 这里为了一看就懂直接用unforceable, 不可强制结束的
    fn is_indeterminate(&self) -> bool {
        false
    }
}

pub trait AnyAnimation: Any + Animation {}

pub trait VeAniMap {
    fn get_ani<T>(&self, ve_name: &String) -> anyhow::Result<&T>
    where
        T: AnyAnimation + TypeName;
    fn get_ani_mut<T>(&mut self, ve_name: &String) -> anyhow::Result<&mut T>
    where
        T: AnyAnimation + TypeName;

    fn insert_ani<T>(&mut self, ve_name: &String, ani_ins: T) -> anyhow::Result<()>
    where
        T: AnyAnimation + TypeName;
    fn remove_ani<T>(&mut self, ve_name: &String) ->anyhow::Result<Option<T>>
    where
        T: AnyAnimation + TypeName;
}

/// 动画容器
pub type VeTypedAnimationMap = HashMap<String, HashMap<String, Box<dyn AnyAnimation>>>;
impl VeAniMap for VeTypedAnimationMap {
    fn get_ani<T>(&self, ve_name: &String) -> anyhow::Result<&T>
    where
        T: AnyAnimation + TypeName,
    {
        let name = T::TYPE_NAME;
        let map = self
            .get(ve_name)
            .ok_or(anyhow::anyhow!("ve {ve_name} not in Map"))?;
        let res = map
            .get(name)
            .ok_or(anyhow::anyhow!("{name} not in Animation Map"))?;
        let r = res.as_ref() as &dyn Any;
        r.downcast_ref::<T>().ok_or(anyhow::anyhow!(
            "{name} Animation is not a {} instance",
            std::any::type_name::<T>()
        ))
    }

    fn insert_ani<T>(&mut self, ve_name: &String, ani_ins: T) -> anyhow::Result<()>
    where
        T: AnyAnimation + TypeName,
    {
        let name = T::TYPE_NAME;
        let map = self
            .entry(ve_name.clone())
            .or_insert_with(HashMap::<String, Box<dyn AnyAnimation>>::default);
        map.insert(name.to_string(), Box::new(ani_ins));
        Ok(())
    }

    fn remove_ani<T>(&mut self, ve_name: &String) -> anyhow::Result<Option<T>>
    where
        T: AnyAnimation + TypeName,
    {
        let name = T::TYPE_NAME;
        let map = self
            .entry(ve_name.clone())
            .or_insert_with(HashMap::<String, Box<dyn AnyAnimation>>::default);
        let res = map.remove(name);
        match res {
            Some(ani) => {
                let r = ani as Box<dyn Any>;
                match r.downcast::<T>() {
                    Ok(res) => Ok(Some(*res)),
                    Err(_) => Err(anyhow::anyhow!(
                        "{name} Animation is not a {} instance",
                        std::any::type_name::<T>()
                    )),
                }
            }
            None => Ok(None),
        }
    }

    fn get_ani_mut<T>(&mut self, ve_name: &String) -> anyhow::Result<&mut T>
    where
        T: Animation + Any + TypeName,
    {
        let map = self
            .get_mut(ve_name)
            .ok_or(anyhow::anyhow!("ve {ve_name} not in Map"))?;
        let name = T::TYPE_NAME;
        let res = map
            .get_mut(name)
            .ok_or(anyhow::anyhow!("{name} not in Animation Map"))?;
        let r = res.as_mut() as &mut dyn Any;
        r.downcast_mut::<T>().ok_or(anyhow::anyhow!(
            "{name} Animation is not a {} instance",
            std::any::type_name::<T>()
        ))
    }
}
