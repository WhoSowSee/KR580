use crate::i18n::Lang;

pub(super) fn validate_parameter(
    minimum: Option<i64>,
    maximum: Option<i64>,
    value: &str,
    lang: Lang,
) -> Result<(), String> {
    if minimum.is_none() && maximum.is_none() {
        return Ok(());
    }
    let value = value.parse::<i64>().map_err(|_| match lang {
        Lang::Ru => "Значение должно быть целым числом".to_owned(),
        Lang::En => "The value must be an integer".to_owned(),
    })?;
    if minimum.is_some_and(|minimum| value < minimum)
        || maximum.is_some_and(|maximum| value > maximum)
    {
        let minimum = minimum.unwrap_or(i64::MIN);
        let maximum = maximum.unwrap_or(i64::MAX);
        return Err(match lang {
            Lang::Ru => format!("Значение должно быть от {minimum} до {maximum}"),
            Lang::En => format!("The value must be between {minimum} and {maximum}"),
        });
    }
    Ok(())
}
