#[macro_export]
macro_rules! oninput {
    ($state:expr) => {{
        let state = $state.clone();
        Callback::from(move |e: InputEvent| {
            state.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    }};
}
