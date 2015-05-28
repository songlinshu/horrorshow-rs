use std::fmt;

use template::{TemplateBuilder, Template};


/// Something that can be rendered once.
pub trait RenderOnce {
    /// Render this into a template builder.
    fn render_once<'a>(self, tmpl: &mut TemplateBuilder<'a>) where Self: Sized;

    /// Returns a (very) rough estimate of how many bytes this Render will use.
    fn size_hint(&self) -> usize { 0 }
}

/// Something that can be rendered by mutable reference.
pub trait RenderMut: RenderOnce {
    /// Render this into a template builder.
    fn render_mut<'a>(&mut self, tmpl: &mut TemplateBuilder<'a>);
}

/// Something that can be rendered by reference.
pub trait Render: RenderMut {
    /// Render this into a template builder.
    fn render<'a>(&self, tmpl: &mut TemplateBuilder<'a>);
}

// RenderOnce is the trait we really care about. 

impl<'a, T: ?Sized> RenderOnce for &'a mut T where T: RenderMut {
    fn render_once(self, tmpl: &mut TemplateBuilder) {
        RenderMut::render_mut(self, tmpl)
    }
    fn size_hint(&self) -> usize {
        (**self).size_hint()
    }
}

impl<'a, T: ?Sized> RenderOnce for &'a T where T: Render {
    fn render_once(self, tmpl: &mut TemplateBuilder) {
        Render::render(self, tmpl)
    }
    fn size_hint(&self) -> usize {
        (**self).size_hint()
    }
}

// Box Stuff

/// Something that can be rendered once out of a box.
pub trait RenderBox {
    /// Do not call. Called by RenderOnce impl on Box<RenderBox>
    #[doc(hidden)]
    fn render_box<'a>(self: Box<Self>, tmpl: &mut TemplateBuilder<'a>);

    /// Do not call. Called by RenderOnce impl on Box<RenderBox>
    #[doc(hidden)]
    fn size_hint_box(&self) -> usize;
}


impl<T> RenderBox for T where T: RenderOnce {
    fn render_box<'a>(self: Box<T>, tmpl: &mut TemplateBuilder<'a>) {
        (*self).render_once(tmpl);
    }

    fn size_hint_box(&self) -> usize {
        RenderOnce::size_hint(self)
    }
}

// Box<RenderBox>

impl<'b> RenderOnce for Box<RenderBox + 'b> {
    #[inline]
    fn render_once<'a>(self, tmpl: &mut TemplateBuilder<'a>) {
        RenderBox::render_box(self, tmpl);
    }

    #[inline]
    fn size_hint(&self) -> usize {
        RenderBox::size_hint_box(self)
    }
}

// Box<RenderMut>

impl<'b> RenderOnce for Box<RenderMut + 'b> {
    #[inline]
    fn render_once<'a>(mut self, tmpl: &mut TemplateBuilder<'a>) {
        RenderMut::render_mut(&mut *self, tmpl);
    }

    #[inline]
    fn size_hint(&self) -> usize { 
        RenderMut::size_hint(&**self)
    }
}

impl<'b> RenderMut for Box<RenderMut + 'b> {
    #[inline]
    fn render_mut<'a>(&mut self, tmpl: &mut TemplateBuilder<'a>) {
        RenderMut::render_mut(&mut *self, tmpl);
    }
}

// Box<Render>

impl<'b> RenderOnce for Box<Render + 'b> {
    #[inline]
    fn render_once<'a>(self, tmpl: &mut TemplateBuilder<'a>) {
        Render::render(&*self, tmpl);
    }

    #[inline]
    fn size_hint(&self) -> usize { 
        Render::size_hint(&**self)
    }
}

impl<'b> RenderMut for Box<Render + 'b> {
    #[inline]
    fn render_mut<'a>(&mut self, tmpl: &mut TemplateBuilder<'a>) {
        Render::render(&*self, tmpl);
    }
}

impl<'b> Render for Box<Render + 'b> {
    #[inline]
    fn render<'a>(&self, tmpl: &mut TemplateBuilder<'a>) {
        Render::render(&*self, tmpl);
    }
}

// {{{ Renderer

/// A template renderer. The `html! {}` macro returns a `Renderer`.
pub struct Renderer<F> {
    renderer: F,
    expected_size: usize,
}

impl<F> RenderOnce for Renderer<F> where F: FnOnce(&mut TemplateBuilder) {
    fn render_once(self, tmpl: &mut TemplateBuilder) {
        (self.renderer)(tmpl)
    }

    fn size_hint(&self) -> usize {
        self.expected_size
    }
}

impl<F> RenderMut for Renderer<F> where F: FnMut(&mut TemplateBuilder) {
    fn render_mut(&mut self, tmpl: &mut TemplateBuilder) {
        (self.renderer)(tmpl)
    }
}

impl<F> Render for Renderer<F> where F: Fn(&mut TemplateBuilder) {
    fn render(&self, tmpl: &mut TemplateBuilder) {
        (self.renderer)(tmpl)
    }
}

// I'd like to be able to say impl Display for T where T: Render but coherence.
impl<F> fmt::Display for Renderer<F> where Renderer<F>: Render {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Adapter<'a, 'b>(&'a mut fmt::Formatter<'b>) where 'b: 'a;
        impl<'a, 'b> fmt::Write for Adapter<'a, 'b> {
            #[inline]
            fn write_str(&mut self, text: &str) -> fmt::Result {
                self.0.write_str(text)
            }
            #[inline]
            fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
                self.0.write_fmt(args)
            }
        }
        self.write_to_fmt(&mut Adapter(f))
    }
}

/// Used by the `html! {}` macro
#[doc(hidden)]
pub fn __new_renderer<F: FnOnce(&mut TemplateBuilder)>(expected_size: usize, f: F) -> Renderer<F> {
    Renderer {
        renderer: f,
        expected_size: expected_size,
    }
}

/// Used by the `html! {}` macro
#[doc(hidden)]
pub fn __new_boxed_renderer<F: FnOnce(&mut TemplateBuilder)>(expected_size: usize, f: F) -> Box<Renderer<F>> {
    Box::new(Renderer {
        renderer: f,
        expected_size: expected_size,
    })
}

// }}}

/// Raw content marker.
///
/// When rendered, raw content will not be escaped.
pub struct Raw<S: AsRef<str>>(S);

impl<S> Raw<S> where S: AsRef<str> {
    /// Mark as raw.
    pub fn new(content: S) -> Raw<S> {
        Raw(content)
    }
}

impl<S> RenderOnce for Raw<S> where S: AsRef<str> {
    fn render_once(self, tmpl: &mut TemplateBuilder) {
        tmpl.write_raw(self.0.as_ref())
    }
    fn size_hint(&self) -> usize {
        self.0.as_ref().len()
    }
}

impl<S> RenderMut for Raw<S> where S: AsRef<str> {
    fn render_mut(&mut self, tmpl: &mut TemplateBuilder) {
        tmpl.write_raw(self.0.as_ref())
    }
}

impl<S> Render for Raw<S> where S: AsRef<str> {
    fn render(&self, tmpl: &mut TemplateBuilder) {
        tmpl.write_raw(self.0.as_ref())
    }
}

impl<'a> RenderOnce for &'a str {
    #[inline]
    fn render_once(self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(self)
    }

    #[inline]
    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl<'a> RenderMut for &'a str {
    #[inline]
    fn render_mut(&mut self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(self)
    }
}

impl<'a> Render for &'a str {
    #[inline]
    fn render(&self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(self)
    }
}

impl RenderOnce for String {
    #[inline]
    fn render_once(self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(&self)
    }
    #[inline]
    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl RenderMut for String {
    #[inline]
    fn render_mut(&mut self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(self)
    }
}

impl Render for String {
    #[inline]
    fn render(&self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(self)
    }
}

