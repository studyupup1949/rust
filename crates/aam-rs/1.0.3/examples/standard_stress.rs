use std::time::Instant;
use aam_rs::aaml::AAML;
use aam_rs::builder::AAMBuilder;

fn main() {
    let count = 30_000_000;
    println!("🚀 Начинаем тест для {} строк...", count);

    let gen_start = Instant::now();

    let mut builder = AAMBuilder::with_capacity(count * 40);

    for i in 0..count {
        let key = format!("user_profile_setting_key_{}", i);
        let val = format!("value_string_number_{}", i);
        builder.add_line(&key, &val);
    }

    let content = builder.build();
    let gen_duration = gen_start.elapsed();
    println!("✅ Генерация:  {:?}", gen_duration);

    let parse_start = Instant::now();

    let aaml = AAML::parse(&content).expect("Ошибка парсинга");

    let parse_duration = parse_start.elapsed();
    println!("✅ Парсинг:    {:?}", parse_duration);

    let search_key = format!("user_profile_setting_key_{}", count - 1);

    let search_start = Instant::now();
    let result = aaml.find_obj(&search_key);
    let search_duration = search_start.elapsed();

    match result {
        Some(v) => println!("✅ Поиск:      {:?} (Найдено: {})", search_duration, v.as_str()),
        None => println!("❌ Поиск:      {:?} (Не найдено)", search_duration),
    }

    println!("---");
    println!("📊 Общее время (без учета вывода в консоль): {:?}", gen_duration + parse_duration + search_duration);

    let total_bytes = content.len();
    println!("📦 Размер строкового буфера: {:.2} MB", total_bytes as f64 / 1_048_576.0);
}