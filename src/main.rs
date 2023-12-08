use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::io::{BufRead, BufReader, Lines};

struct Entry {
    temperatur: u16,
    year: u16,
    month: u8,
    day: u8,
    station_id: u32,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}-{:0>2}-{:0>2} {}°C at {}",
            self.year,
            self.month,
            self.day,
            self.temperatur as f32 / 10.,
            self.station_id,
        ))
    }
}

fn line_to_entry(line: &str) -> Option<Entry> {
    let split: Vec<&str> = line.split(',').collect();
    // STAID, SOUID,    DATE,   TX, Q_TX
    if split.len() != 5 {
        return None;
    }
    let staid = split[0].trim();
    let date = split[2].trim();
    let tx = split[3].trim();
    let quality: u32 = split[4].trim().parse().ok()?;
    if quality != 0 {
        return None;
    }
    if date.len() != 2 + 2 + 4 {
        return None;
    }
    let year = &date[0..4];
    let month = &date[4..6];
    let day = &date[6..8];
    Some(Entry {
        temperatur: tx.parse().ok()?,
        year: year.parse().ok()?,
        month: month.parse().ok()?,
        day: day.parse().ok()?,
        station_id: staid.parse().ok()?,
    })
}

// plan
// => Station Id mappen auf koordinaten
// => koordinaten in Detailgrad von 2° auflösen (52° - 54°)
// => Durchschnitt erheben für Jede Kategorie
//  TODO: TX klassen abchecken
// => als CSV ausgeben

// use elements.txt, sources.txt, stations.txt,
// similiar for all categories

fn main() {
    let file = fs::File::open("ECA_blend_tx.zip").unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();

    let max_tx = 21;
    let mut tx_counter = vec![0; max_tx];

    let mut day_count: HashMap<(u16, u8, u8), usize> = HashMap::new();

    for i in 0..zip.len() {
        let mut zf = zip.by_index(i).unwrap();

        if zf.name().chars().take(2).all(|char| char.is_uppercase()) {
            continue;
        }
        println!("====== FIle {}", zf.name());
        // std::io::copy(&mut zf, &mut std::io::stdout()).unwrap();
        // let re = BufReader::new(zf);
        // for line in re.lines() {
        //     let line = line.unwrap();
        //     // if let Some(entry) = line_to_entry(&line) {
        //     //     let key = (entry.year, entry.month, entry.day);
        //     //     match day_count.get_mut(&key) {
        //     //         Some(old) => *old += 1,
        //     //         None => {
        //     //             day_count.insert(key, 1);
        //     //         }
        //     //     }
        //     //     // println!("{}", entry);
        //     // }
        // }
    }
    let mut days: Vec<_> = day_count.keys().collect();
    days.sort();
    for day in days {
        println!(
            "Day {}-{}-{} has {} entries",
            day.0,
            day.1,
            day.2,
            day_count.get(day).unwrap()
        )
    }
}
