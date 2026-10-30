use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <main>
            <h1>"coach-tools"</h1>
            <p>"TABATA TIMER"</p>
        </main>
    }
}
