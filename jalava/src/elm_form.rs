pub use jalava_derive::ElmForm;

pub trait ElmForm {
    /// The type of the form in Elm.
    fn form_type() -> String;

    /// A function that takes the form type and return an Elm Body.
    fn prepare_form() -> String;
}

pub trait ElmFormField {
    fn part_type() -> &'static str;
    fn to_form_fields(field_name: &str) -> String {
        format!("[ {0} \"{1}\" form.{1} ]", Self::part_type(), field_name)
    }
}

impl ElmFormField for String {
    fn part_type() -> &'static str {
        "Http.stringPart"
    }
}

impl ElmFormField for &str {
    fn part_type() -> &'static str {
        "Http.stringPart"
    }
}

impl<T: ElmFormField> ElmFormField for Vec<T> {
    fn part_type() -> &'static str {
        T::part_type()
    }

    fn to_form_fields(field_name: &str) -> String {
        format!(
            "List.map (\\x -> {0} \"{1}\" x) form.{1}",
            T::part_type(),
            field_name
        )
    }
}
