use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::io::{copy, stdout, BufRead, BufReader, Read};
use std::iter::empty;

struct Entry {
    temperatur: u16,
    year: u16,
    month: u8,
    day: u8,
    station_id: u32,
}

#[derive(Debug, Clone)]
struct Source {
    pub station_id: usize,
    pub source_id: usize,
    pub source_name: String,
    pub country: String,
    pub lat_sec: i32,
    pub lon_sec: i32,
    pub height: i32,
    pub element_identifier: usize, // TX2 => 2, FX12 => 12
    pub start: usize,
    pub stop: usize,
}

#[derive(Debug, Clone)]
struct Station {
    pub id: usize,
    pub name: String,
    pub cn: String,
    pub lat_sec: i32,
    pub lon_sec: i32,
    pub height: i32,
}

fn coord_tuple_to_secs(t: (i32, u8, u8)) -> i32 {
    if t.0 > 0 {
        t.0 * 3600 + t.1 as i32 * 60 + t.2 as i32
    } else {
        t.0 * 3600 - t.1 as i32 * 60 - t.2 as i32
    }
}

fn from_raw_values_to_station(v: &[String]) -> Option<Station> {
    let lat: Vec<_> = v[3].trim().split(':').collect();
    let lon: Vec<_> = v[4].trim().split(':').collect();
    let lat_tuple: (i32, u8, u8) = (
        lat.get(0)?.parse().ok()?,
        lat.get(1)?.parse().ok()?,
        lat.get(2)?.parse().ok()?,
    );
    let lon_tuple: (i32, u8, u8) = (
        lon.get(0)?.parse().ok()?,
        lon.get(1)?.parse().ok()?,
        lon.get(2)?.parse().ok()?,
    );
    Some(Station {
        id: v[0].trim().parse().ok()?,
        name: v[1].trim().parse().ok()?,
        cn: v[2].trim().parse().ok()?,
        lat_sec: coord_tuple_to_secs(lat_tuple),
        lon_sec: coord_tuple_to_secs(lon_tuple),
        height: v[5].trim().parse().ok()?,
    })
}

fn from_raw_values_to_sources(v: &[String]) -> Option<Source> {
    // Staid, Souid, Souname, Cn, Lat, Long, Height, Elei, Start, Stop, Pardid, Parname
    let lat: Vec<_> = v.get(4)?.trim().split(':').collect();
    let lon: Vec<_> = v.get(5)?.trim().split(':').collect();
    let lat_tuple: (i32, u8, u8) = (
        lat.get(0)?.parse().ok()?,
        lat.get(1)?.parse().ok()?,
        lat.get(2)?.parse().ok()?,
    );
    let lon_tuple: (i32, u8, u8) = (
        lon.get(0)?.parse().ok()?,
        lon.get(1)?.parse().ok()?,
        lon.get(2)?.parse().ok()?,
    );
    Some(Source {
        station_id: v.get(0)?.parse().ok()?,
        source_id: v.get(1)?.parse().ok()?,
        source_name: v.get(2)?.to_string(),
        country: v.get(3)?.to_string(),
        lat_sec: coord_tuple_to_secs(lat_tuple),
        lon_sec: coord_tuple_to_secs(lon_tuple),
        height: v.get(6)?.parse().ok()?,
        element_identifier: v.get(7)?[2..].parse().ok()?,
        start: v.get(8)?.parse().ok()?,
        stop: v.get(9)?.parse().ok()?,
    })
}

fn read_sources<R>(mut reader: BufReader<R>) -> impl Iterator<Item = Source>
where
    R: Read,
{
    reader
        .lines()
        .map(Result::unwrap)
        .map(|line| {
            line.split(',')
                .into_iter()
                .map(|v| v.trim())
                .map(ToString::to_string)
                .collect()
        })
        .filter(|words: &Vec<_>| words.len() > 3)
        .map(|words| from_raw_values_to_sources(words.as_slice()))
        .filter(|station| station.is_some())
        .map(Option::unwrap)
}

fn read_stations<R>(mut reader: BufReader<R>) -> impl Iterator<Item = Station>
where
    R: Read,
{
    reader
        .lines()
        .map(Result::unwrap)
        .map(|line| {
            line.split(',')
                .into_iter()
                .map(|v| v.trim())
                .map(ToString::to_string)
                .collect()
        })
        .filter(|words: &Vec<_>| words.len() == 6)
        .map(|words| from_raw_values_to_station(words.as_slice()))
        .filter(|station| station.is_some())
        .map(Option::unwrap)
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

    let stations: HashMap<usize, Station> =
        read_stations(BufReader::new(zip.by_name("stations.txt").unwrap()))
            .map(|st| (st.id, st))
            .collect();
    let all_sources = read_sources(BufReader::new(zip.by_name("sources.txt").unwrap()));
    let mut source_counter = 0;
    let mut sources_to_keep: HashMap<usize, Source> = HashMap::new();
    for source in all_sources {
        source_counter += 1;
        match sources_to_keep.get_mut(&source.station_id) {
            None => {sources_to_keep.insert(source.station_id, source);},
            Some(old) => {
                if old.element_identifier > source.element_identifier {
                    *old = source
                }
            }
        }
    }
    println!("Working with {}/{} sources", sources_to_keep.len(), source_counter);

    for i in 0..zip.len() {
        let mut zf = zip.by_index(i).unwrap();
        if !zf.name().chars().take(2).all(|char| char.is_uppercase()) {
            continue;
        }
        println!("F: {}", zf.name())
    }
}
