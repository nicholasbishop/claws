use anyhow::{anyhow, Context, Error, Result};
use fehler::{throw, throws};
use rusoto_core::Region;
use rusoto_ec2::{
    DescribeInstancesRequest, Ec2 as _, Ec2Client, Instance,
    RebootInstancesRequest, StartInstancesRequest, StopInstancesRequest,
    TerminateInstancesRequest,
};
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogStreamsRequest,
};
use rusoto_s3::{S3Client, S3 as _};
use std::{thread, time};
use structopt::StructOpt;

fn get_instance_name(instance: &Instance) -> Option<String> {
    if let Some(tags) = &instance.tags {
        for tag in tags {
            if let Some(key) = &tag.key {
                if key == "Name" {
                    if let Some(value) = &tag.value {
                        return Some(value.into());
                    }
                }
            }
        }
    }
    None
}

fn get_instance_state_name(instance: &Instance) -> Option<String> {
    if let Some(state) = &instance.state {
        if let Some(name) = &state.name {
            return Some(name.into());
        }
    }
    None
}

#[throws]
fn ec2_list_instances() {
    let client = Ec2Client::new(Region::default());
    let output = client
        .describe_instances(DescribeInstancesRequest {
            ..Default::default()
        })
        .sync()
        .context("failed to list instances")?;
    let reservations =
        output.reservations.context("missing reservations field")?;
    struct Row {
        id: String,
        name: String,
        state: String,
    }
    let mut instances = Vec::new();
    for reservation in reservations {
        if let Some(res_instances) = reservation.instances {
            for instance in res_instances {
                let id = instance
                    .instance_id
                    .clone()
                    .unwrap_or_else(|| "i-?????????????????".to_string());
                let name = get_instance_name(&instance)
                    .unwrap_or_else(|| "<no-name>".into());
                let state = get_instance_state_name(&instance)
                    .unwrap_or_else(|| "unknown".into());
                instances.push(Row { id, name, state });
            }
        }
    }

    // Sort the instances by name
    instances.sort_unstable_by_key(|row| row.name.clone());

    // Get the maximum length of the state field
    let state_width = instances
        .iter()
        .map(|row| row.state.len())
        .max()
        .unwrap_or(0);

    for row in instances {
        println!(
            "{:19} {:state_width$} {}",
            row.id,
            row.state,
            row.name,
            state_width = state_width
        );
    }
}

#[throws]
fn ec2_show_addresses(instance_id: String) {
    println!("{}:", instance_id);
    let client = Ec2Client::new(Region::default());
    let output = client
        .describe_instances(DescribeInstancesRequest {
            instance_ids: Some(vec![instance_id]),
            ..Default::default()
        })
        .sync()
        .context("failed to get instance details")?;
    let reservations =
        output.reservations.context("missing reservations field")?;
    for reservation in reservations {
        if let Some(res_instances) = reservation.instances {
            for instance in res_instances {
                println!(
                    "  private IP: {}",
                    instance.private_ip_address.unwrap_or_else(String::new)
                );
                println!(
                    "  public IP: {}",
                    instance.public_ip_address.unwrap_or_else(String::new)
                );
            }
        }
    }
}

#[throws]
fn ec2_start_instance(instance_id: String) {
    let client = Ec2Client::new(Region::default());
    client
        .start_instances(StartInstancesRequest {
            instance_ids: vec![instance_id],
            ..Default::default()
        })
        .sync()
        .context("failed to start instance")?;
}

#[throws]
fn ec2_stop_instance(instance_id: String) {
    let client = Ec2Client::new(Region::default());
    client
        .stop_instances(StopInstancesRequest {
            instance_ids: vec![instance_id],
            ..Default::default()
        })
        .sync()
        .context("failed to stop instance")?;
}

#[throws]
fn ec2_terminate_instance(instance_id: String) {
    let client = Ec2Client::new(Region::default());
    client
        .terminate_instances(TerminateInstancesRequest {
            instance_ids: vec![instance_id],
            ..Default::default()
        })
        .sync()
        .context("failed to terminate instance")?;
}

#[throws]
fn ec2_reboot_instance(instance_id: String) {
    let client = Ec2Client::new(Region::default());
    client
        .reboot_instances(RebootInstancesRequest {
            instance_ids: vec![instance_id],
            ..Default::default()
        })
        .sync()
        .context("failed to reboot instance")?;
}

// Not using #[throws] here because of
// github.com/withoutboats/fehler/issues/52
fn logs_recent_streams(args: RecentLogStreams) -> Result<(), Error> {
    let client = CloudWatchLogsClient::new(Region::default());
    let mut next_token = None;
    let mut remaining = args.limit as i64;
    loop {
        // The describe-log-streams operation has a limit of five
        // transactions per second, so attempt up to five requests and
        // then sleep for 1 second.
        for _ in 0..5 {
            // 50 is the maximum for a single request
            let limit = std::cmp::min(remaining, 50);
            let resp = client
                .describe_log_streams(DescribeLogStreamsRequest {
                    log_group_name: args.log_group_name.clone(),
                    limit: Some(limit),
                    order_by: Some("LastEventTime".to_string()),
                    next_token: next_token.clone(),
                    descending: Some(true),
                    ..Default::default()
                })
                .sync()
                .context("failed to list streams")?;
            if let Some(log_streams) = resp.log_streams {
                for log_stream in log_streams {
                    println!(
                        "{}",
                        log_stream.log_stream_name.ok_or_else(|| anyhow!(
                            "missing log stream name"
                        ))?
                    );
                }
            }
            // Finish if there are no more results
            if resp.next_token.is_none() {
                return Ok(());
            }
            remaining -= limit;
            // Finish if the number of requested streams has already
            // been shown
            if remaining <= 0 {
                return Ok(());
            }
            next_token = resp.next_token;
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}

#[throws]
fn s3_list_buckets() {
    let client = S3Client::new(Region::default());
    let output = client
        .list_buckets()
        .sync()
        .context("failed to list buckets")?;
    let buckets = output.buckets.context("missing buckets field")?;
    for bucket in buckets {
        let name = bucket.name.context("missing bucket name")?;
        println!("{}", name);
    }
}

#[derive(Debug, StructOpt)]
enum Ec2 {
    /// List instances.
    Instances,
    /// Show an instance's IP address(es)
    Addr { instance_ids: Vec<String> },
    /// Start an instance.
    Start { instance_ids: Vec<String> },
    /// Stop an instance.
    Stop { instance_ids: Vec<String> },
    /// Terminate an instance.
    Terminate { instance_ids: Vec<String> },
    /// Reboot an instance.
    Reboot { instance_ids: Vec<String> },
}

#[derive(Debug, StructOpt)]
struct RecentLogStreams {
    log_group_name: String,
    #[structopt(long, default_value = "10")]
    limit: usize,
}

#[derive(Debug, StructOpt)]
enum Logs {
    /// List recent CloudWatch Logs streams.
    RecentStreams(RecentLogStreams),
}

#[derive(Debug, StructOpt)]
enum S3 {
    /// List buckets.
    Buckets,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "AWS command-line tool")]
enum Command {
    Ec2(Ec2),
    Logs(Logs),
    S3(S3),
}

#[throws]
fn for_each<F: Fn(String) -> Result<()>>(
    func: F,
    mut instance_ids: Vec<String>,
) {
    let mut any_errors = false;
    for id in instance_ids.drain(..) {
        if let Err(err) = func(id) {
            eprintln!("{}", err);
            any_errors = true;
        }
    }
    if any_errors {
        throw!(anyhow!("one or more operations failed"));
    }
}

fn main() -> Result<(), Error> {
    match Command::from_args() {
        Command::Ec2(Ec2::Instances) => ec2_list_instances(),
        Command::Ec2(Ec2::Addr { instance_ids }) => {
            for_each(ec2_show_addresses, instance_ids)
        }
        Command::Ec2(Ec2::Start { instance_ids }) => {
            for_each(ec2_start_instance, instance_ids)
        }
        Command::Ec2(Ec2::Stop { instance_ids }) => {
            for_each(ec2_stop_instance, instance_ids)
        }
        Command::Ec2(Ec2::Terminate { instance_ids }) => {
            for_each(ec2_terminate_instance, instance_ids)
        }
        Command::Ec2(Ec2::Reboot { instance_ids }) => {
            for_each(ec2_reboot_instance, instance_ids)
        }
        Command::Logs(Logs::RecentStreams(args)) => logs_recent_streams(args),
        Command::S3(S3::Buckets) => s3_list_buckets(),
    }
}
