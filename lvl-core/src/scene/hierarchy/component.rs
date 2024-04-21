use std::any::{type_name, Any};

pub trait Component
where
    Self: Any,
{
    fn name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
