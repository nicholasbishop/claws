# claws

AWS command-line tool. The purpose of this tool is not to be a
complete replacement for [awscli](https://aws.amazon.com/cli), but
rather to provide a more convenient interface for some common
commands.

## Installation

    cargo install claws

## Usage

Currently just a couple commands are implemented.

List EC2 instances:

    claws ec2 instances
    
List S3 buckets:

    claws s3 buckets
