use crate::booking_store::SERVICES;
use crate::BookFormData;
use resuma::core::view::AttrValue;
use resuma::prelude::*;

#[component]
pub fn BookPage() -> View {
    load_boundary(
        use_book_form_load(),
        |data| {
            let fecha = data.fecha.clone();
            let servicio = data.servicio.clone();
            let slots = data.slots.clone();

            view! {
                <main>
                    <h1>"Book"</h1>
                    <Form submit={crate::book_slot} class="form">
                        <label>
                            "Name"
                            <input name="name" required=true />
                        </label>
                        <label>
                            "Phone"
                            <input name="phone" type="tel" required=true />
                        </label>
                        <fieldset>
                            <legend>"Service"</legend>
                            {(SERVICES.iter().map(|(key, label)| {
                                let checked = servicio == *key;
                                view! {
                                    <label>
                                        <input type="radio" name="service" value={(*key).to_string()} required=true checked={checked} />
                                        {*label}
                                    </label>
                                }
                            }).collect::<Vec<_>>())}
                        </fieldset>
                        <input type="hidden" name="date" value={fecha.clone()} />
                        <label>
                            "Date"
                            {loader_refresh_input(
                                "/book",
                                "fecha",
                                &fecha,
                                &["servicio"],
                                "date",
                                vec![("required", AttrValue::Bool(true))],
                            )}
                        </label>
                        {if slots.is_empty() {
                            view! { <p>"Pick a date to see available times."</p> }
                        } else {
                            view! {
                                <div class="slot-grid">
                                    {slots.iter().map(|t| {
                                        let t = t.clone();
                                        view! {
                                            <label>
                                                <input type="radio" name="time" value={t.clone()} required=true />
                                                {t}
                                            </label>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }
                        }}
                        <button type="submit" disabled={slots.is_empty()}>"Confirm"</button>
                    </Form>
                </main>
            }
        },
        |err| error_page(&FlowError::Loader(err)),
        || View::empty(),
    )
}
