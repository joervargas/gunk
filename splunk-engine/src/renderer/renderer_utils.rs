

pub trait GfxRenderer
{
    fn init(&self);

    fn destroy(self);

    fn update(&mut self);

    fn render(&self);
}