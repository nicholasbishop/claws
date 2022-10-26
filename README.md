**This tool is no longer under active development. If you are interested in taking over or repurposing the name on crates.io, feel free to contact me: nbishop@nbishop.net**

# claws

AWS command-line tool. The purpose of this tool is not to be a
complete replacement for [awscli](https://aws.amazon.com/cli), but
rather to provide a more convenient interface for some common
commands.

## Installation

    cargo install claws

## Usage

Currently just a few commands are implemented.

### EC2

List instances:

    claws ec2 instances
    
Get instance IP addresses:

    claws ec2 addr <instance-id>
    
Start, stop, or reboot an instance:

    claws ec2 start <instance-id>
    claws ec2 stop <instance-id>
    claws ec2 reboot <instance-id>
    
### CloudWatch Logs

List log groups:

    claws logs groups [<prefix>]

List recent streams in a group:

    claws logs recent-streams [--limit <n>] <log-group-name>
    
### S3
    
List buckets:

    claws s3 buckets
