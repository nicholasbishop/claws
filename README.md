# claws

AWS command-line tool. The purpose of this tool is not to be a
complete replacement for [awscli](https://aws.amazon.com/cli), but
rather to provide a more convenient interface for some common
commands.

## Installation

    cargo install claws

## Usage

Currently just a couple commands are implemented.

### EC2

List instances:

    claws ec2 instances
    
Start or stop an instance:

    claws ec2 start <instance-id>
    claws ec2 stop <instance-id>
    
### S3
    
List buckets:

    claws s3 buckets
