mod scene;

use scene::Scene;

use iced::wgpu;
use iced::widget::{center, column, row, shader, text, button};
use iced::{Center, Element, Fill, Subscription};
use rfd;

fn main() -> iced::Result {
    iced::application(
        "Shader Widget - Test",
        App::update,
        App::view,
    )
    .subscription(App::subscription)
    .run()
}

struct App {
    scene: Scene,
    name: String,
}

#[derive(Debug, Clone)]
enum Message {
    SelectImage,
}

impl App {
    fn new() -> Self {
        Self {
            scene: Scene::new(),
            name: "Hello".to_string(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SelectImage => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image Files", &["png", "jpg", "jpeg"])
                    .pick_file() {
                    self.scene.set_image_path(path);
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let top_controls = row![
            text(self.name.clone()),
            button("Select Image").on_press(Message::SelectImage)
        ]
        .spacing(40);

        let controls = column![top_controls]
            .spacing(10)
            .padding(20)
            .align_x(Center);

        let shader = shader(&self.scene).width(Fill).height(Fill);

        center(column![controls, shader].align_x(Center)).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}