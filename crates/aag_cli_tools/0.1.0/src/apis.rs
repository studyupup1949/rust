use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct CurrencyResponse {
    rates: HashMap<String, f64>,
}

#[derive(Deserialize)]
struct Weather {
    weather: Vec<WeatherCondition>,
    main: WeatherMain,
    name: String,
}

#[derive(Deserialize)]
struct WeatherCondition {
    description: String,
}

#[derive(Deserialize)]
struct WeatherMain {
    temp: f64,
}

#[derive(Deserialize)]
struct Definition {
    definition: String,
}

#[derive(Deserialize)]
struct Meaning {
    definitions: Vec<Definition>,
}

#[derive(Deserialize)]
struct WordResponse {
    meanings: Vec<Meaning>,
}

pub fn currency_converter(amount: f64, from: &str, to: &str) {
    let api_url = format!("https://api.exchangerate-api.com/v4/latest/{}", from);
    let client = Client::new();
    match client.get(&api_url).send() {
        Ok(response) => {
            if let Ok(data) = response.json::<CurrencyResponse>() {
                if let Some(rate) = data.rates.get(to) {
                    println!("{} {} is {:.2} {}", amount, from, amount * rate, to);
                } else {
                    println!("Currency {} not found.", to);
                }
            } else {
                println!("Failed to parse the response.");
            }
        }
        Err(err) => println!("Error fetching exchange rates: {}", err),
    }
}

pub fn url_shortener(long_url: &str) {
    let api_url = format!("https://tinyurl.com/api-create.php?url={}", long_url);
    match reqwest::blocking::get(&api_url) {
        Ok(response) => {
            if let Ok(short_url) = response.text() {
                println!("Shortened URL: {}", short_url);
            } else {
                println!("Failed to shorten the URL.");
            }
        }
        Err(err) => println!("Error shortening URL: {}", err),
    }
}

pub fn fetch_definition(word: &str) -> Result<(), Box<dyn std::error::Error>> { 
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word); 
    let client = Client::new(); 
 
    let response: Vec<WordResponse> = client.get(&url).send()?.json()?; 
 
    if response.is_empty() { 
        println!("No definitions found for '{}'.", word); 
        return Ok(()); 
    }
 
    for meaning in &response[0].meanings { 
        for definition in &meaning.definitions { 
            println!("- {}", definition.definition); 
        } 
    }
 
    Ok(()) 
}

pub fn fetch_weather(city: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "fa5c8ca5209518a48516c4e5fc0b4277"; 
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=imperial",
        city, api_key
    );

    let client = Client::new();
    let response: Weather = client.get(&url).send()?.json()?;

    println!("Weather in {}:", response.name);
    println!("Temperature: {:.1}°F", response.main.temp);
    println!("Condition: {}", response.weather[0].description);

    Ok(())
}


