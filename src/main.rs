use rusoto_core::Region;
use rusoto_ec2::{
    DescribeInstancesRequest, Ec2 as _, Ec2Client, Instance,
    StartInstancesRequest, StopInstancesRequest,
};
use rusoto_s3::{S3Client, S3 as _};
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

fn ec2_list_instances() {
    let client = Ec2Client::new(Region::default());
    let output = client
        .describe_instances(DescribeInstancesRequest {
            ..Default::default()
        })
        .sync()
        .expect("failed to list instances");
    let reservations = output.reservations.expect("missing reservations field");
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

fn ec2_show_addresses(instance_id: String) {
    let client = Ec2Client::new(Region::default());
    let output = client
        .describe_instances(DescribeInstancesRequest {
            instance_ids: Some(vec![instance_id]),
            ..Default::default()
        })
        .sync()
        .expect("failed to get instance details");
    let reservations = output.reservations.expect("missing reservations field");
    for reservation in reservations {
        if let Some(res_instances) = reservation.instances {
            for instance in res_instances {
                println!(
                    "private IP: {}",
                    instance.private_ip_address.unwrap_or_else(String::new)
                );
                println!(
                    "public IP: {}",
                    instance.public_ip_address.unwrap_or_else(String::new)
                );
            }
        }
    }
}

fn ec2_start_instance(instance_id: String) {
    let client = Ec2Client::new(Region::default());
    client
        .start_instances(StartInstancesRequest {
            instance_ids: vec![instance_id],
            ..Default::default()
        })
        .sync()
        .expect("failed to start instance");
}

fn ec2_stop_instance(instance_id: String) {
    let client = Ec2Client::new(Region::default());
    client
        .stop_instances(StopInstancesRequest {
            instance_ids: vec![instance_id],
            ..Default::default()
        })
        .sync()
        .expect("failed to stop instance");
}

fn s3_list_buckets() {
    let client = S3Client::new(Region::default());
    let output = client
        .list_buckets()
        .sync()
        .expect("failed to list buckets");
    let buckets = output.buckets.expect("missing buckets field");
    for bucket in buckets {
        let name = bucket.name.expect("missing bucket name");
        println!("{}", name);
    }
}

#[derive(Debug, StructOpt)]
enum Ec2 {
    /// List instances.
    Instances,
    /// Show an instance's IP address(es)
    Addr { instance_id: String },
    /// Start an instance.
    StartInstance { instance_id: String },
    /// Stop an instance.
    StopInstance { instance_id: String },
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
    S3(S3),
}

fn main() {
    match Command::from_args() {
        Command::Ec2(Ec2::Instances) => ec2_list_instances(),
        Command::Ec2(Ec2::Addr { instance_id }) => {
            ec2_show_addresses(instance_id)
        }
        Command::Ec2(Ec2::StartInstance { instance_id }) => {
            ec2_start_instance(instance_id)
        }
        Command::Ec2(Ec2::StopInstance { instance_id }) => {
            ec2_stop_instance(instance_id)
        }
        Command::S3(S3::Buckets) => s3_list_buckets(),
    }
}
