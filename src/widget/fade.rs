use iced::advanced;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::gradient::{self, Gradient};
use iced::mouse;
use iced::{Background, Color, Degrees, Element, Length, Rectangle, Size, Vector};

pub fn fade<'a, Message>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message>
where
    Message: 'a,
{
    Fade {
        content: content.into(),
    }
    .into()
}

struct Fade<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Fade<'a, Message, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        self.content.as_widget().diff(tree)
    }

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.max();

        let content_layout = self.content.as_widget().layout(
            tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::INFINITY),
        );
        let content_size = content_layout.size();

        layout::Node::with_children(
            size,
            vec![content_layout.translate(Vector::new(
                (size.width - content_size.width) / 2.0,
                2.0 * size.height / 3.0 - content_size.height,
            ))],
        )
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as advanced::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();

        self.content.as_widget().draw(
            tree,
            renderer,
            theme,
            style,
            content_layout,
            cursor,
            viewport,
        );

        if content_bounds.height > 2.0 * bounds.height / 3.0 {
            renderer.with_layer(bounds, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: 0.0.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    Background::Gradient(Gradient::Linear(
                        gradient::Linear::new(Degrees(180.0))
                            .add_stop(0.0, Color::BLACK)
                            .add_stop(0.7, Color::TRANSPARENT),
                    )),
                );
            });
        }
    }
}

impl<'a, Message, Renderer> From<Fade<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(fade: Fade<'a, Message, Renderer>) -> Self {
        Element::new(fade)
    }
}
