pub mod sample_generation {
    use std::{error::Error, fs::OpenOptions};

    use serde::Deserialize;
    use csv::ReaderBuilder;
    use rand::{seq::SliceRandom, Rng};
    use chrono::{Datelike, Days, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

    use crate::{config::Config, errors::{PickingEmptySamplesCollection, RandomDateElementOutRange}, history::HistoryRecord, utils::{process_path, serde::parse_duration_from_minutes}};
    
    #[derive(Clone, Deserialize)]
    struct SampleEntry {
        title: String,
        subject: Option<String>,
        #[serde(deserialize_with = "parse_duration_from_minutes", serialize_with = "serialize_duration_as_minutes")]
        minimum_duration: Duration,
    }

    fn get_sample_choices(sample_path: &str) -> Result<Vec<SampleEntry>, Box<dyn Error>> {
        let file = OpenOptions::new()
        .read(true)
        .open(sample_path)?;
        let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .quote(b'"')
        .has_headers(true)
        .from_reader(file);
        let mut res: Vec<SampleEntry> = Vec::new();
        for entry in reader.deserialize() {
            res.push(entry?);
        }
        Ok(res)
    }
    
    fn get_days_amount(start: &NaiveDate, end: &NaiveDate) -> usize {
        let start_days = start.num_days_from_ce();
        let end_days = end.num_days_from_ce();
        (end_days - start_days) as usize
    }

    fn choose_task(samples: &Vec<SampleEntry>) -> Result<&SampleEntry, PickingEmptySamplesCollection> {
        let mut rng = rand::thread_rng();
        samples.choose(&mut rng)
        .ok_or(PickingEmptySamplesCollection)
    }

    fn choose_time(start: &NaiveTime, end: &NaiveTime) -> Result<NaiveTime, RandomDateElementOutRange> {
        let start_seconds = start.num_seconds_from_midnight();
        let end_seconds = end.num_seconds_from_midnight();
        if start_seconds > end_seconds {
            return Err(RandomDateElementOutRange::Times(start.to_string(), end.to_string()));
        }
        let random_seconds = rand::thread_rng().gen_range(start_seconds..=end_seconds);
    
        NaiveTime::from_num_seconds_from_midnight_opt(random_seconds, 0)
        .ok_or(RandomDateElementOutRange::Dates(start.to_string(), start.to_string()))
    }

    fn generate_samples(
        start_day: &NaiveDate, end_day: &NaiveDate,
        minimum_task_amount_per_day: usize,
        start_day_time: &NaiveTime, end_day_time: &NaiveTime,
        samples: &Vec<SampleEntry>
    ) -> Result<Vec<HistoryRecord>, Box<dyn Error>> {
        let mut res = Vec::new();
        let mut rng = rand::thread_rng();
        
        let days_amount = get_days_amount(
            start_day,
            end_day
        );

        for d in 0..days_amount {
            let current_day = start_day
            .checked_add_days(Days::new(d as u64))
            .ok_or(
                format!("From {}, there is no date given {} days after", 
                start_day, d))?;
            
            let task_amount = minimum_task_amount_per_day
            + rng.gen_range(0..=5);
            let mut last_time = *start_day_time;
            for _ in 0..task_amount {
                let sample = choose_task(samples)?;
                if let Ok(chosen_start_time) =  choose_time(
                    &last_time, 
                    end_day_time
                ) {
                let added_minutes = rng.gen_range(0..60);
                let chosen_end_time = chosen_start_time
                + sample.minimum_duration
                + Duration::minutes(added_minutes);
                res.push(HistoryRecord {
                    start_date: NaiveDateTime::new(current_day, chosen_start_time)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                    end_date: NaiveDateTime::new(current_day, chosen_end_time)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                    title: sample.title.clone(),
                    section: sample.subject.clone(),
                });
                last_time = chosen_end_time;
                }
            }
        }
        Ok(res)
    }

    fn save_samples(samples_output_path: &str, sample_records: Vec<HistoryRecord>) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::WriterBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .from_path(samples_output_path)?;
        for rec in sample_records {
            writer.serialize(rec)?;
        }
        Ok(())
    }

    pub fn create_samples(config: &Config) -> Result<(), Box<dyn Error>> {
        let input = process_path(&config.samples.sample_file_path)?;
        let output = process_path(&config.samples.sample_output_path)?;
        let sample_choices = get_sample_choices(&input)?;
        let samples = generate_samples(
            &config.samples.start_day, &config.samples.end_day,
            config.samples.minimum_task_amount_per_day,
            &config.samples.start_day_time,
            &config.samples.end_day_time,
            &sample_choices)?;
        save_samples(&output, samples)?;
        Ok(())
    }
}