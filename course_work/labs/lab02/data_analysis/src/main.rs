use std::collections::{HashMap, HashSet};
use csv::{ReaderBuilder};
use serde::{Deserialize};

#[derive(Deserialize, Debug, Clone)]
struct Enrolment {
    course: String,
    z_id: String,
    name: String,
    program: String,
    plan: String,
    wam: f32,
    session: String,
    dob: i32,
    sex: String
}
const ENROLMENTS_PATH: &str = "enrolments.psv";

fn main() {

    let mut rdr = ReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .from_path(ENROLMENTS_PATH).unwrap();

    let mut enrolment_vec = Vec::new();
    for result in rdr.records() {
        let enrolment: Enrolment = result.unwrap().deserialize(None).unwrap();
        enrolment_vec.push(enrolment);
    }

    let students = filter_unique(enrolment_vec.clone());
    let course_map = count_course(enrolment_vec.clone());
    let most_common_course = course_map.iter().max_by_key(|entry| entry.1).unwrap();
    let least_common_course = course_map.iter().min_by_key(|entry| entry.1).unwrap();
    let average_wam = calculate_average(students.clone());


    println!("Number of students: {}", students.len());
    println!("Most common course: {} with {} students", most_common_course.0, most_common_course.1);
    println!("Least common course: {} with {} students", least_common_course.0, least_common_course.1);
    println!("Average WAM: {:.2}", average_wam);
}

fn filter_unique(vec: Vec<Enrolment>) -> Vec<(String, String)> {
    vec.into_iter()
        .map(|enrolment| (enrolment.z_id, enrolment.wam.to_string()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn count_course(vec: Vec<Enrolment>) -> HashMap<String, i32> {
    vec.into_iter()
        .map(|enrolment| enrolment.course)
        .fold(HashMap::new(), |mut acc, c| {
            *acc.entry(c).or_insert(0) += 1;
            acc
        })
}

fn calculate_average(vec: Vec<(String, String)>) -> f32 {
    let len = vec.len() as f32;
    vec.iter().map(|student| student.1.parse::<f32>().unwrap()).sum::<f32>() / len
}
