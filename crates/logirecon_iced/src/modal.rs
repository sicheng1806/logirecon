use iced::Element;

pub trait ModalView {
    type Message;

    fn modal_view(&self) -> Element<'_, Self::Message>;
}
