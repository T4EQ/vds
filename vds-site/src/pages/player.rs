use yew::prelude::*;
#[derive(yew::Properties, PartialEq, Eq)]
pub struct VideoPlayerProps {
    pub id: String,
}

#[function_component(VideoPlayer)]
pub fn video_player(VideoPlayerProps { id }: &VideoPlayerProps) -> Html {
    let path = format!("/api/content/{id}");
    html! {
        <video controls=true autoplay=true width={"250"}>
            <source src={path} type="video/mp4" />
        </video>
    }
}
