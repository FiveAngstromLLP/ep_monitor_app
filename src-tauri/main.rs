
use std::fs::File;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tauri::State;
use csv::Writer;
use tar::Builder;
use aws_sdk_s3::Client;
use tokio::runtime::Runtime;

struct TimeTracker {
    start_time: Option<std::time::Instant>,
    end_time: Option<std::time::Instant>,
}

#[derive(Default)]
struct AppState {
    tracker: Arc<Mutex<TimeTracker>>,
}

// Start time tracking (login)
#[tauri::command]
fn start_time_tracking(state: State<AppState>) {
    let mut tracker = state.tracker.lock().unwrap();
    tracker.start_time = Some(std::time::Instant::now());
}

// Stop time tracking (logout)
#[tauri::command]
fn stop_time_tracking(state: State<AppState>) {
    let mut tracker = state.tracker.lock().unwrap();
    tracker.end_time = Some(std::time::Instant::now());
}

// Save data to CSV and pack it into tar
#[tauri::command]
fn save_csv_and_pack_to_tar(employee_name: &str, employee_code: &str, time_elapsed: i64, apps: Vec<String>) -> Result<(), Box<dyn Error>> {
    let csv_path = "employee_log.csv";
    save_data_to_csv(employee_name, employee_code, time_elapsed, apps)?;

    let tar_path = "employee_log.tar";
    pack_csv_into_tar(csv_path, tar_path)?;
    Ok(())
}

// Save data to CSV
fn save_data_to_csv(employee_name: &str, employee_code: &str, time_elapsed: i64, apps: Vec<String>) -> Result<(), Box<dyn Error>> {
    let file = File::create("employee_log.csv")?;
    let mut writer = Writer::from_writer(file);
    writer.write_record(&["Employee Name", "Employee Code", "Time Elapsed", "Active Applications"])?;
    writer.write_record(&[employee_name, employee_code, &time_elapsed.to_string(), &apps.join(", ")])?;
    writer.flush()?;
    Ok(())
}

// Pack CSV file into tar
fn pack_csv_into_tar(csv_path: &str, tar_path: &str) -> Result<(), Box<dyn Error>> {
    let tar_gz = File::create(tar_path)?;
    let mut tar = Builder::new(tar_gz);
    tar.append_file(csv_path, &mut File::open(csv_path)?)?;
    tar.finish()?;
    Ok(())
}

// Simulate fetching active applications (you can replace this with actual system API calls)
#[tauri::command]
fn get_active_apps() -> Vec<String> {
    vec!["Chrome", "VSCode", "Slack"].iter().map(|s| s.to_string()).collect()
}

// Upload the tar file to AWS S3
async fn upload_tar_to_s3() -> Result<(), Box<dyn Error>> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let bucket_name = "your-bucket-name";
    let key = "employee_log.tar";
    let body = std::fs::read("employee_log.tar")?;

    client.put_object().bucket(bucket_name).key(key).body(body.into()).send().await?;
    Ok(())
}

#[tauri::command]
fn send_tar_to_aws() {
    let rt = Runtime::new().unwrap();
    rt.block_on(upload_tar_to_s3()).unwrap();
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_time_tracking,
            stop_time_tracking,
            get_active_apps,
            save_csv_and_pack_to_tar,
            send_tar_to_aws
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
