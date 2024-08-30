/// The Library includes the function for common using.
pub mod utils {
    use reqwest::{blocking::Client, header::CONTENT_TYPE};
    //use reqwest::blocking::Client;
    //use reqwest::header::CONTENT_TYPE;
    use chrono::Utc;
    use serde_json::Value;
    use std::error::Error;
    use std::thread::sleep;
    use std::time::Duration; // for get Time
                             //const TIMEOUT: u16 = 36000; // 10 * 60 * 60 = 36000 seconds

    pub fn hello() {
        println!("Hello, I'm libs in common");
    }

    pub fn trigger_jenkins_job(
        jenkins_user: &str,
        jenkins_token: &str,
        jenkins_url: &str,
        jenkins_job_name: &str,
    ) -> Result<(String, String), Box<dyn Error>> {
        let client = Client::new();

        /*
        Trigging job to jenkins
        */
        println!("Triggering Jenkins Job Name: {}", jenkins_job_name);
        let build_response = client
            .post(format!("{}/job/{}/build", jenkins_url, jenkins_job_name))
            .basic_auth(jenkins_user, Some(jenkins_token))
            .header(CONTENT_TYPE, "application/json")
            .send()?;

        /*
        Check status job trigger before
        */
        let trigger_url = build_response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|location| location.to_str().ok())
            .map(|s| s.to_string());
        if trigger_url.is_none() {
            println!("Failed to trigger Jenkins Job {}", jenkins_job_name);
            return Err("Failed to trigger Jenkins Job".into());
        }
        // Method .unwrap to open value inside Option or Result, if None -> error panic
        let trigger_url = trigger_url.unwrap();

        /*
        Get BUILD_ID of Jenkin Job
        */
        let mut build_id = String::new();
        while build_id.is_empty() {
            sleep(Duration::from_secs(15));
            let last_build_response = client
                .get(format!(
                    "{}/job/{}/lastBuild/api/json",
                    jenkins_url, jenkins_job_name
                ))
                .basic_auth(jenkins_user, Some(jenkins_token))
                .send()?;
            let json: Value = last_build_response.json()?;
            build_id = json["id"].as_str().unwrap_or("").to_string();
        }

        println!(
            "Triggered Jenkins BUILD URL: {}/job/{}/{}",
            jenkins_url, jenkins_job_name, build_id
        );

        Ok((trigger_url, build_id))
    }

    pub fn get_last_build_id(
        jenkins_user: &str,
        jenkins_token: &str,
        jenkins_url: &str,
        job_name: &str,
    ) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/job/{}/lastBuild/api/json", jenkins_url, job_name);
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .basic_auth(jenkins_user, Some(jenkins_token))
            .send()?
            .text()?;
        let json: Value = serde_json::from_str(&response)?;
        Ok(json["id"].as_str().unwrap_or("").to_string())
    }

    pub fn check_jenkins_job_status(
        file_path: &str,
        jenkins_user: &str,
        jenkins_token: &str,
        jenkins_url: &str,
        jenkins_job_name: &str,
        build_id: &str,
        daily_regression_test: &str,
    ) -> Result<(), Box<dyn Error>> {
        let start_time = Utc::now();
        println!("check_jenkins_job_status...");
        loop {
            // Get status of Jenkin Job
            println!("loop...");
            let url = format!(
                "{}/job/{}/{}/api/json",
                jenkins_url, jenkins_job_name, build_id
            );
            let client = reqwest::blocking::Client::new();
            let response = client
                .get(&url)
                .basic_auth(jenkins_user, Some(jenkins_token))
                .send()?
                .text()?;

            // Analyzer to get status of Job
            let json: Value = serde_json::from_str(&response)?;
            let status = json["result"].as_str().unwrap_or("null");
            if status != "null" {
                // Job complete with status SUCCESS | ABORTED | FAILURE
                println!("Jenkins job completed with status: {}.", status);
                return Ok(());
            }

            let end_time = Utc::now();
            let elapsed_time = (end_time.timestamp() - start_time.timestamp()) as u16;
            println!("Elapse_time: {}", elapsed_time);
            //let config = read_json_cfg("../configs/jenkinsCfg.json");
            let config = read_json_cfg(file_path)?;
            //let config = read_json_cfg(
            //    "
            //        /evoadapt/jenkins/rust/botevoadapt/configs/jenkinsCfg.json",
            //)
            //.expect(
            //    "Lib: Failed to read config file,
            //                check the file path and ensure the file exists.",
            //);
            let timeout: u16 = config.time_out;

            if elapsed_time >= timeout {
                println!(
                    "
                        Jenkins job exceeded maximum time allowed ({} seconds).
                        Attempting to abort job...",
                    timeout
                );
                // Stop job Jenkins
                let stop_url =
                    format!("{}/job/{}/{}/stop", jenkins_url, jenkins_job_name, build_id);
                client
                    .post(&stop_url)
                    .basic_auth(jenkins_user, Some(jenkins_token))
                    .send()?;
                sleep(Duration::from_secs(60));
                /*
                Get to latest ID job daily_regression_test to Stop it
                */
                let regression_build_id = get_last_build_id(
                    jenkins_user,
                    jenkins_token,
                    jenkins_url,
                    daily_regression_test,
                )?;
                println!("IDR: {}", regression_build_id);
                let stop_regression_url = format!(
                    "{}/job/{}/{}/stop",
                    jenkins_url, daily_regression_test, regression_build_id
                );
                client
                    .post(&stop_regression_url)
                    .basic_auth(jenkins_user, Some(jenkins_token))
                    .send()?;
                sleep(Duration::from_secs(60));
                return Ok(());
            }
            // Sleep 30 seconds then re-run
            sleep(Duration::from_secs(30));
        }
    }

    /// Do read JSON file.
    ///
    /// # Arguments
    ///
    /// - `file_path`: file path to configure JSON.
    ///
    /// # Return:
    ///
    /// Return `Result` inculde configure or error.
    ///
    /// # Example:
    ///
    /// ```
    /// ci_jenkins path/jenkinsCfg.json
    /// ```

    use serde::{Deserialize, Serialize};
    use std::fs::File;
    use std::io::Read;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Config {
        pub jenkin_user: String,
        pub jenkin_token: String,
        pub jenkin_url: String,
        pub daily_rebase_check: String,
        pub daily_main_check: String,
        pub daily_rergression_test: String,
        pub time_out: u16,
    }
    pub fn read_json_cfg(file_path: &str) -> Result<Config, Box<dyn Error>> {
        let mut file = File::open(file_path).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;
        let config: Config =
            serde_json::from_str(&contents).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        Ok(config)
    }
}
