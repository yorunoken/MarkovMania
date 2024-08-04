use rand::Rng;
use std::fs::File;
use std::io::{self, Read};

use druid::widget::{Button, Flex, Label, TextBox};
use druid::{
    commands, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, Handled, Lens,
    Target, Widget, WidgetExt, WindowDesc,
};
use druid::{FileDialogOptions, FileSpec};

mod markov_chain;

#[derive(Clone, Data, Lens)]
struct AppState {
    custom_sentence: String,
    data_file_path: String,
    status: String,
    generated_text: String,
}

struct Delegate;

fn main() {
    let window = WindowDesc::new(main_ui())
        .window_size((700.0, 500.0))
        .title("MarkovMania");

    let data = AppState {
        custom_sentence: String::new(),
        data_file_path: String::new(),
        generated_text: String::new(),
        status: String::new(),
    };
    AppLauncher::with_window(window)
        .delegate(Delegate)
        .launch(data)
        .expect("Failed to launch application");
}

fn main_ui() -> impl Widget<AppState> {
    let title = Label::new("Markov Mania").with_text_size(40.0).center();
    let desc = Label::new("Generate sentences using custom data.")
        .with_text_size(18.0)
        .with_text_color(Color::rgba8(156, 163, 175, 160))
        .center();

    let upload_button = Button::new("Upload custom data")
        .on_click(|ctx, _data: &mut AppState, _env| {
            let options = FileDialogOptions::new()
                .allowed_types(vec![FileSpec::new("Text file", &["txt"])])
                .default_type(FileSpec::new("Text file", &["txt"]))
                .name_label("Custom data")
                .title("Choose a custom data file")
                .button_text("Open");

            ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(options.clone()))
        })
        .padding(10.0);

    let file_status_label = Label::new(|data: &AppState, _env: &Env| {
        if data.data_file_path.is_empty() {
            "No data file selected".to_string()
        } else {
            format!("Selected data file: {}", data.data_file_path)
        }
    })
    .padding(10.0);

    let status_label = Label::new(|data: &AppState, _env: &Env| data.status.clone())
        .with_text_color(Color::rgba8(156, 163, 175, 160))
        .padding(10.0);

    let generated_text_label =
        Label::new(|data: &AppState, _env: &Env| data.generated_text.clone())
            .with_text_size(18.0)
            .padding(10.0);

    let custom_sentence_input = TextBox::new()
        .with_placeholder("Enter custom sentence to generate from (optional)")
        .lens(AppState::custom_sentence)
        .fix_width(350.0)
        .padding(10.0);

    let generate_button = Button::new("Generate text")
        .on_click(|ctx, data: &mut AppState, _env| {
            match read_file_to_string(&data.data_file_path) {
                Ok(file) => {
                    let mut rng = rand::thread_rng();

                    let sentences: Vec<&str> =
                        file.split("\n").filter(|s| !s.trim().is_empty()).collect();

                    let mut markov_chain = markov_chain::Chain::new();
                    markov_chain.train(sentences);

                    let custom_word = if data.custom_sentence.is_empty() {
                        None
                    } else {
                        Some(data.custom_sentence.clone())
                    };

                    let max_words = rng.gen_range(1..15);
                    let content = markov_chain.generate(max_words, custom_word);
                    data.status = content;
                }
                Err(e) => {
                    data.status = format!("Error while opening file: \n{}", e);
                    ctx.new_window(show_error_dialog(data.status.clone()));
                }
            }
        })
        .padding(10.0);

    Flex::column()
        .with_child(title)
        .with_child(desc)
        .with_spacer(20.0)
        .with_child(
            Flex::column()
                .with_child(upload_button)
                .with_child(file_status_label),
        )
        .with_child(custom_sentence_input)
        .with_child(generate_button)
        .with_child(status_label)
        .with_child(generated_text_label)
        .center()
        .padding(20.0)
}

fn read_file_to_string(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn show_error_dialog<S: Into<String>>(error_message: S) -> WindowDesc<AppState> {
    WindowDesc::new(
        Flex::column()
            .with_child(
                Label::new("An error has occured!")
                    .with_text_size(26.0)
                    .with_text_color(Color::RED)
                    .padding(10.0),
            )
            .with_child(
                Label::new(error_message.into())
                    .with_text_size(18.0)
                    .padding(10.0),
            )
            .with_child(
                Button::new("Ok")
                    .on_click(|ctx, _data: &mut AppState, _env| {
                        ctx.window().close();
                    })
                    .padding(10.0),
            )
            .padding(10.0),
    )
    .title("Error")
    .window_size((400.0, 200.0))
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            match file_info.path.to_str() {
                Some(path) => data.data_file_path = path.into(),
                None => ctx.new_window(show_error_dialog("Path isn't valid unicode")),
            }
            return Handled::Yes;
        }
        Handled::No
    }
}
