use chrono::{naive::NaiveDateTime, prelude::*};
use sigstore::rekor::{
    apis::{configuration::Configuration, entries_api, index_api},
    models::SearchIndex,
};
use std::collections::HashMap;
use yew::prelude::*;
use yew_hooks::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <MainView />
    }
}

#[function_component(MainView)]
fn main_view() -> Html {
    let identity = use_state(|| None);
    let on_identity_update = {
        let identity = identity.clone();
        Callback::from(move |e: web_sys::InputEvent| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            identity.set(Some(input.value()));
        })
    };

    let entries = use_state(|| vec![]);
    let on_submit = {
        let entries = entries.clone();
        Callback::from(move |_| {
            let identity = identity.clone();
            let entries = entries.clone();
            if let Some(identity) = &*identity {
                let identity = identity.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let entries = entries.clone();
                    let query = SearchIndex {
                        email: Some(identity.to_string()),
                        hash: None,
                        public_key: None,
                    };
                    let configuration = Configuration::default();
                    match index_api::search_index(&configuration, query).await {
                        Ok(fetched_entries) => {
                            entries.set(fetched_entries);
                        }
                        Err(e) => {
                            log::warn!("Error fetching entries: {:?}", e);
                        }
                    }
                });
            }
        })
    };

    html!(
        <>
        <div>
            <img src="sigstore_logo.png" width = "400px"/>
            <br />
            <p>{"Search the Sigstore transparency log. Enter the identity to search for and click submit."}</p>
            <br />
            <label class="entry" for="email">
                 {"Identity (email):"}
                <input type="text" id="email" size="80" name="email" oninput={on_identity_update}/>
            </label>
            <br />
            <button id="button" onclick={on_submit.clone()}>{"Submit"}</button>

            <br />
            <br />
            <div class="entries">
                <EntriesView entries={(*entries).clone()} />
            </div>
        </div>
        </>
    )
}

#[derive(Properties, PartialEq)]
pub struct EntriesProps {
    pub entries: Vec<String>,
}

#[function_component(EntriesView)]
fn entries_view(props: &EntriesProps) -> Html {
    let entries = use_map(HashMap::new());
    let on_click = {
        let entries = entries.clone();
        Callback::from(move |e: web_sys::MouseEvent| {
            let input: web_sys::HtmlParagraphElement = e.target_unchecked_into();
            let entry = input.inner_text();
            let configuration = Configuration::default();
            let entries = entries.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = entries_api::get_log_entry_by_uuid(&configuration, &entry).await;
                if let Ok(result) = result {
                    entries.insert(entry, result);
                }
            });
        })
    };
    props
        .entries
        .iter()
        .map(|entry| {
            html! {
                <div>
                    <p class={"entry"} onclick={on_click.clone()}>{entry}</p>
                    {

                        if let Some(e) = entries.current().get(entry) {
                            html! {
                                <div>
                                    <p class={"details"}>{format!("Index: {}", e.log_index)}</p>
                                    <p class={"details"}>{format!("When: {}", DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(e.integrated_time, 0).unwrap(), Utc))}</p>
                                </div>
                            }
                        } else {
                            html! {<></>}
                        }
                    }
                </div>

            }
        })
        .collect()
}
