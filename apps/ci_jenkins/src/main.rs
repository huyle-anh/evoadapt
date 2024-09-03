use common::utils;
use std::error::Error;
//use std::process; // -> //process::exit(1);
use std::env;
/*
const JENKIN_USER: &str = "hule";
const JENKIN_TOKEN: &str = "1113cacf21918e88d02a333232a08c8efb";
const JENKINS_URL: &str = "https://jenkin.com";
const DAILY_REBASE_CHECK: &str = "OBMC_Daily_Rebase_Check";
const DAILY_MAIN_CHECK: &str = "OBMC_Daily_Main_Check";
const DAILY_RERGESSION_TEST: &str = "OBMC_Daily_Regression_Test";
const JENKINS_JOB_NAME: &str = "temp";
*/

fn main() -> Result<(), Box<dyn Error>> {
    // Check argument to print usage for user

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <json Config File Path>", args[0]);
        return Ok(());
    }
    let json_cfg_file_path = &args[1];

    /*
    Call trigger Jenkin Job
    */
    let current_dir = env::current_dir().unwrap();
    println!("Current dir: {}", current_dir.display());
    let config = utils::read_json_cfg(json_cfg_file_path)?;
    println!(" Json config file path: {:?}", json_cfg_file_path);
    //    json_cfg_file_path
    //    "
    //            /evoadapt/jenkins/rust/botevoadapt/configs/jenkinsCfg.json",
    //)?;
    //.expect("Failed to read config file, check the file path and ensure the file exists.");
    // .as_ref() returns a reference to the value inside the Result,
    // .clone() creates a data-> String so it needs to be squeezed through & to call str in the func
    let jenkin_user = config.jenkin_user.clone();
    let jenkin_token = config.jenkin_token.clone();
    let jenkin_url = config.jenkin_url.clone();
    let daily_rebase_check = config.daily_rebase_check.clone();
    let daily_main_check = config.daily_main_check.clone();
    let daily_rergression_test = config.daily_rergression_test.clone();
    //let _timeout = config.as_ref().unwrap().timeout.clone();
    match utils::trigger_jenkins_job(
        &jenkin_user,
        &jenkin_token,
        &jenkin_url,
        &daily_rebase_check,
    ) {
        Ok((_trigger_url, build_id)) => {
            println!(
                "Trigger Rebase-URL successfully: {}/job/{}/{}",
                jenkin_url, daily_rebase_check, build_id
            );
            println!("Build ID: {}", build_id);
            match utils::check_jenkins_job_status(
                &json_cfg_file_path,
                &jenkin_user,
                &jenkin_token,
                &jenkin_url,
                &daily_rebase_check,
                &build_id, // | build_id.c_str()
                &daily_rergression_test,
            ) {
                Ok(_status) => {
                    println!("Next run daily_main_check");
                    match utils::trigger_jenkins_job(
                        &jenkin_user,
                        &jenkin_token,
                        &jenkin_url,
                        &daily_main_check,
                    ) {
                        Ok((_trigger_url, build_id)) => {
                            println!(
                                "Trigger Main-URL successfully: {}/job/{}/{}",
                                jenkin_url, daily_main_check, build_id
                            );
                        }
                        Err(e) => {
                            eprintln!("Failed to trigger Jenkins Job: {}", e);
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to check Status: {}", e);
                    return Err(e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to trigger Jenkins Job: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
