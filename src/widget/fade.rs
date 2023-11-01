use iced::advanced;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::mouse;
use iced::{Element, Length, Rectangle, Size, Vector};

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
        self.content.as_widget().draw(
            tree,
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
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
