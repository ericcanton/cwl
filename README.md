# cwl: CloudWatch Logs CLI

Colorful printing of AWS CloudWatch Logs.

Check out `./cwl ls --help` for more info.


## Basic usage

You need to have the AWS access and secret keys you want to use stored under the `[default]` profile in `$HOME/.aws/credientials`, which is where the AWS CLI stores them.  
(Future: add `--profile` parameter to select from other profiles in that file.)

To compile, you'll need Rust. Compile with `cargo build` (or `cargo build --release` for optimizations) and find the executable under `./target/debug/cwl` (or `./target/release/cwl`, resp).

* To list log groups, call with:  
    ```shell
    ./cwl ls

    ```  
* You can view the log streams associated with a group by calling with `-g <group_name>`. 
* You can view events in a log stream by calling with `-g <group_name> -s '<stream_name>'`. Note single quotes `'...'` around `<stream_name>`: stream names contain `$LATEST`, so your shell will try to parse this otherwise.
